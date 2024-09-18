use std::sync::Mutex;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Form;
use axum::Router;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use macros::query;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use database::message::{
    can_user_delete, create_message, delete_message, get_message_by_id, get_messages, Message,
};
use database::run_migrations;
use database::session::set_session_user;
use database::user::{create_user, retrieve_user};
use extractors::ExtractSession;
use template::HtmlTemplate;
use user::get_user_from_session;
use validators::validate_message;
use websocket::WebSocketHandler;

mod database;
mod extractors;
mod template;
mod user;
mod validators;
mod websocket;

#[cfg(test)]
mod tests;

const API_ADDRESS: &'static str = env!("API_ADDRESS");
const WEBSOCKET_ADDRESS: &'static str = env!("WEBSOCKET_ADDRESS");
const WEBSOCKET_CONNECT_URL: Option<&'static str> = option_env!("WEBSOCKET_CONNECT_URL");

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    is_logged_in: bool,
    user_name: String,
    websocket_url: Option<&'static str>,
    enable_websockets: bool,
}

///
/// GET request to load the index page
///
async fn index_view(
    ExtractSession(session): ExtractSession,
    mut jar: CookieJar,
) -> impl IntoResponse {
    let user = get_user_from_session(&session);

    let mut is_logged_in = false;
    let mut user_name = "".to_string();

    if let Ok(ref user) = user {
        if let Some(user) = user {
            is_logged_in = true;
            user_name = user.name.clone();
        }
    }

    let cookie = Cookie::build(("session_id", session.id.clone()))
        .secure(true)
        .build();
    jar = jar.add(cookie);

    let websocket_url = WEBSOCKET_CONNECT_URL;
    let enable_websockets = websocket_url.is_some();

    let template = IndexTemplate {
        is_logged_in,
        user_name,
        websocket_url,
        enable_websockets,
    };

    (jar, HtmlTemplate(template))
}

#[derive(Template)]
#[template(path = "login_result.html")]
struct LoginResultTemplate {
    user_name: String,
    is_logged_in: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    name: String,
}

async fn login_view(
    ExtractSession(session): ExtractSession,
    Form(request): Form<LoginRequest>,
) -> impl IntoResponse {
    let user = create_user(&request.name);

    if user.is_err() {
        return HtmlTemplate(LoginResultTemplate {
            user_name: "".to_string(),
            is_logged_in: false,
        });
    }

    let user = user.unwrap();

    if set_session_user(&session.id, user.id).is_err() {
        return HtmlTemplate(LoginResultTemplate {
            user_name: "".to_string(),
            is_logged_in: false,
        });
    }

    HtmlTemplate(LoginResultTemplate {
        user_name: user.name,
        is_logged_in: true,
    })
}

#[derive(Deserialize)]
struct CreateMessageRequest {
    message: String,
}

struct MessageDetail {
    message: Message,
    can_delete: bool,
}

#[derive(Template)]
#[template(path = "messages.html")]
struct GetMessagesTemplate {
    success: bool,
    messages: Vec<MessageDetail>,
    error: String,
}

///
/// GET request to load all messages
///
async fn get_messages_view(ExtractSession(session): ExtractSession) -> impl IntoResponse {
    let messages = get_messages();

    if let Err(e) = messages {
        let template = GetMessagesTemplate {
            success: false,
            messages: vec![],
            error: format!("Error: {}", e),
        };

        return HtmlTemplate(template);
    }

    // Get the logged in user
    let mut user = None;

    if let Some(user_id) = session.user_id {
        let user_lookup = retrieve_user(user_id);

        if let Ok(user_lookup) = user_lookup {
            user = user_lookup;
        }
    }

    let messages = messages
        .unwrap()
        .into_iter()
        .map(|message| {
            let can_delete = match &user {
                Some(user) => can_user_delete(&message, user),
                None => false,
            };

            MessageDetail {
                message,
                can_delete,
            }
        })
        .collect();

    let template = GetMessagesTemplate {
        success: true,
        messages,
        error: "".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "new_message.html")]
struct NewMessageTemplate {
    message_detail: Option<MessageDetail>,
    error: Option<&'static str>,
}

#[derive(Clone)]
struct AppState {
    websocket_handler: &'static Mutex<WebSocketHandler>,
}

///
/// POST request to create a new message, and return the newly created message as HTML
///
async fn create_message_view(
    State(state): State<AppState>,
    ExtractSession(session): ExtractSession,
    Form(message_data): Form<CreateMessageRequest>,
) -> impl IntoResponse {
    let text = &message_data.message;

    let validation = validate_message(text);

    if let Err(_e) = validation {
        return (
            StatusCode::BAD_REQUEST,
            HtmlTemplate(NewMessageTemplate {
                message_detail: None,
                error: Some("Invalid message"),
            }),
        );
    }

    let user_id = session.user_id;

    if user_id.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            HtmlTemplate(NewMessageTemplate {
                message_detail: None,
                error: Some("Not logged in"),
            }),
        );
    }

    // Get the logged in user
    let user = match retrieve_user(user_id.unwrap()) {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                HtmlTemplate(NewMessageTemplate {
                    message_detail: None,
                    error: Some("Not logged in"),
                }),
            );
        }
    };

    if user.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            HtmlTemplate(NewMessageTemplate {
                message_detail: None,
                error: Some("Not logged in"),
            }),
        );
    }

    let user = user.unwrap();

    // Add the new message to the list of messages
    let message = create_message(&message_data.message, user.id);

    if message.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            HtmlTemplate(NewMessageTemplate {
                message_detail: None,
                error: Some("Error creating message"),
            }),
        );
    }

    let message = message.unwrap();

    // Broadcast to all active clients that a new message was created
    let mut websocket_handler = state.websocket_handler.lock().unwrap();

    let can_delete = can_user_delete(&message, &user);

    let template = NewMessageTemplate {
        message_detail: Some(MessageDetail {
            message,
            can_delete,
        }),
        error: None,
    };

    if let Ok(html) = template.render() {
        if let Err(e) = websocket_handler.broadcast(&html) {
            eprintln!("Websocket broadcasting error: {}", e);
        }
    };

    (StatusCode::CREATED, HtmlTemplate(template))
}

#[derive(Template)]
#[template(path = "delete_message.html")]
struct DeleteMessageTemplate {
    success: bool,
    message_id: Option<i32>,
    error: String,
}

/// View to delete a message
///
/// The requesting user must be logged on and have created the message
async fn delete_message_view(
    ExtractSession(session): ExtractSession,
    Path(message_id): Path<i32>,
) -> impl IntoResponse {
    let user_id = session.user_id;

    if user_id.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            HtmlTemplate(DeleteMessageTemplate {
                success: false,
                message_id: None,
                error: "Not logged in".to_string(),
            }),
        );
    }

    // Get the logged in user
    let user = match retrieve_user(user_id.unwrap()) {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                HtmlTemplate(DeleteMessageTemplate {
                    success: false,
                    message_id: None,
                    error: "Not logged in".to_string(),
                }),
            )
        }
    };

    if user.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            HtmlTemplate(DeleteMessageTemplate {
                success: false,
                message_id: None,
                error: "Not logged in".to_string(),
            }),
        );
    }

    let user = user.unwrap();

    let message = match get_message_by_id(message_id) {
        Ok(message) => message,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                HtmlTemplate(DeleteMessageTemplate {
                    success: false,
                    message_id: None,
                    error: format!("Failed to retrieve message: {e}"),
                }),
            )
        }
    };

    if message.is_none() {
        return (
            StatusCode::NOT_FOUND,
            HtmlTemplate(DeleteMessageTemplate {
                success: false,
                message_id: None,
                error: format!("Message {message_id} does not exist"),
            }),
        );
    }

    let message = message.unwrap();

    let can_delete = can_user_delete(&message, &user);

    if !can_delete {
        return (
            StatusCode::FORBIDDEN,
            HtmlTemplate(DeleteMessageTemplate {
                success: false,
                message_id: None,
                error: "Permission denied".to_string(),
            }),
        );
    }

    if let Err(e) = delete_message(message_id) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            HtmlTemplate(DeleteMessageTemplate {
                success: true,
                message_id: None,
                error: format!("Error deleting message: {e}"),
            }),
        );
    };

    (
        StatusCode::OK,
        HtmlTemplate(DeleteMessageTemplate {
            success: true,
            message_id: Some(message.id),
            error: "".to_string(),
        }),
    )
}

#[query]
fn test_macro() {}

#[tokio::main]
async fn main() {
    run_migrations().expect("Could not run migrations");

    // let server =
    //     std::net::TcpListener::bind(WEBSOCKET_ADDRESS).expect("Could not start websocket server.");
    //
    //
    // let websocket_handler = Box::new(Mutex::new(WebSocketHandler::new()));
    //
    // // This lives in state but is meant to be static for the entire runtime of the program, so
    // // having it leaked doesn't seem like a big deal
    // // Having it static satisfies the state's Clone derivation requirement
    // let websocket_handler: &'static Mutex<WebSocketHandler> = Box::leak(websocket_handler);
    //
    // let state = AppState { websocket_handler };
    //
    // std::thread::spawn(move || {
    //     for stream in server.incoming() {
    //         if let Err(e) = stream {
    //             eprintln!("Error with incoming stream: {}", e);
    //             continue;
    //         }
    //
    //         let stream = stream.unwrap();
    //         let peer_addr = stream.peer_addr();
    //
    //         // let callback = |_request: &WebsocketRequest, mut response: WebsocketResponse| {
    //         //     let headers = response.headers_mut();
    //         //
    //         //     headers.append("Access-Control-Allow-Methods", "GET, POST".parse().unwrap());
    //         //
    //         //     Ok(response)
    //         // };
    //
    //         let websocket_accept = tungstenite::accept(stream);
    //
    //         if let Err(ref e) = websocket_accept {
    //             eprintln!("Error accepting websocket stream: {}", e);
    //             continue;
    //         }
    //
    //         let mut websocket = websocket_accept.unwrap();
    //
    //         if let Err(e) =
    //             websocket.send(tungstenite::Message::Text("Hello from server!".to_string()))
    //         {
    //             eprintln!("Failed to send message to websocket: {}", e);
    //         }
    //
    //         let lock = websocket_handler.lock();
    //
    //         if let Err(ref e) = lock {
    //             eprintln!("Failed to lock websocket handler: {}", e);
    //             continue;
    //         }
    //
    //         let mut lock = lock.unwrap();
    //
    //         println!("Accepted websocket from {:?}", peer_addr);
    //         lock.add_websocket(websocket);
    //     }
    // });
    //
    // let static_dir = ServeDir::new("static");
    //
    // println!("WebSocket server listening at {}...", WEBSOCKET_ADDRESS);
    //
    // // build our application with a route
    // let app = Router::new()
    //     .route("/", get(index_view))
    //     .route("/login/", post(login_view))
    //     .route("/message/", get(get_messages_view))
    //     .route("/create-message/", post(create_message_view))
    //     .route("/delete/:message_id/", delete(delete_message_view))
    //     .nest_service("/static", static_dir)
    //     .with_state(state);
    //
    // let listener = TcpListener::bind(API_ADDRESS).await.unwrap();
    // axum::serve(listener, app).await.unwrap();
}
