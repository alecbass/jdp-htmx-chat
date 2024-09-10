use std::sync::Mutex;

use askama::Template;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Form;
use axum::Router;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use database::message::{create_message, get_messages, Message};
use database::run_migrations;
use database::session::set_session_user;
use database::user::create_user;
use extractors::ExtractSession;
use template::HtmlTemplate;
use user::get_user_from_session;
use validators::validate_message;
use websocket::WebSocketHandler;

mod database;
mod extractors;
mod middleware;
mod template;
mod user;
mod validators;
mod websocket;

#[cfg(test)]
mod tests;

const API_ADDRESS: &'static str = env!("API_ADDRESS");
const WEBSOCKET_ADDRESS: &'static str = env!("WEBSOCKET_ADDRESS");
const WEBSOCKET_CONNECT_URL: &'static str = env!("WEBSOCKET_CONNECT_URL");

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    is_logged_in: bool,
    user_name: String,
    websocket_url: &'static str,
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

    if jar.get("session_id").is_none() {
        let cookie = Cookie::build(("session_id", session.id.clone()))
            .secure(true)
            .build();
        jar = jar.add(cookie);
    }

    let template = IndexTemplate {
        is_logged_in,
        user_name,
        websocket_url: WEBSOCKET_CONNECT_URL,
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

#[derive(Template)]
#[template(path = "messages.html")]
struct GetMessagesTemplate {
    success: bool,
    messages: Vec<Message>,
    error: String,
}

///
/// GET request to load all messages
///
async fn get_messages_view() -> impl IntoResponse {
    let messages = get_messages();

    if let Err(e) = messages {
        let template = GetMessagesTemplate {
            success: false,
            messages: vec![],
            error: format!("Error: {}", e),
        };

        return HtmlTemplate(template);
    }

    let messages = messages.unwrap();

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
    message: Message,
}

#[derive(Template)]
#[template(path = "new_message_success.html")]
struct NewMessageSuccessTemplate {
    success: bool,
    error: String,
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
        let template = NewMessageSuccessTemplate {
            success: false,
            error: "Could not create message".to_string(),
        };
        return HtmlTemplate(template);
    }

    let user_id = session.user_id;

    if user_id.is_none() {
        return HtmlTemplate(NewMessageSuccessTemplate {
            success: false,
            error: "Not logged in".to_string(),
        });
    }

    // Add the new message to the list of messages
    let message =
        create_message(&message_data.message, user_id.unwrap()).expect("Could not create message");

    // Broadcast to all active clients that a new message was created
    let mut websocket_handler = state.websocket_handler.lock().unwrap();

    let broadcast_template = NewMessageTemplate { message };

    if let Ok(html) = broadcast_template.render() {
        if let Err(e) = websocket_handler.broadcast(&html) {
            eprintln!("Websocket broadcasting error: {}", e);
        }
    };

    HtmlTemplate(NewMessageSuccessTemplate {
        success: true,
        error: "".to_string(),
    })
}

#[tokio::main]
async fn main() {
    run_migrations().expect("Could not run migrations");
    let server =
        std::net::TcpListener::bind(WEBSOCKET_ADDRESS).expect("Could not start websocket server.");

    let websocket_handler = Box::new(Mutex::new(WebSocketHandler::new()));

    // This lives in state but is meant to be static for the entire runtime of the program, so
    // having it leaked doesn't seem like a big deal
    // Having it static satisfies the state's Clone derivation requirement
    let websocket_handler: &'static Mutex<WebSocketHandler> = Box::leak(websocket_handler);

    let state = AppState { websocket_handler };

    std::thread::spawn(move || {
        for stream in server.incoming() {
            if let Err(e) = stream {
                eprintln!("Error with incoming stream: {}", e);
                continue;
            }

            let stream = stream.unwrap();
            let peer_addr = stream.peer_addr();

            let websocket_accept = tungstenite::accept(stream);

            if let Err(ref e) = websocket_accept {
                eprintln!("Error accepting websocket: {}", e);
                continue;
            }

            let mut websocket = websocket_accept.unwrap();

            if let Err(e) =
                websocket.send(tungstenite::Message::Text("Hello from server!".to_string()))
            {
                eprintln!("Failed to send message to websocket: {}", e);
            }

            let lock = websocket_handler.lock();

            if let Err(ref e) = lock {
                eprintln!("Failed to lock websocket handler: {}", e);
                continue;
            }

            let mut lock = lock.unwrap();

            println!("Accepted websocket from {:?}", peer_addr);
            lock.add_websocket(websocket);
        }
    });

    let static_dir = ServeDir::new("static");

    println!("WebSocket server listening at {}...", WEBSOCKET_ADDRESS);
    // build our application with a route
    let app = Router::new()
        .route("/", get(index_view))
        .route("/login/", post(login_view))
        .route("/message/", get(get_messages_view))
        .route("/create-message/", post(create_message_view))
        .nest_service("/static", static_dir)
        .with_state(state);

    let listener = TcpListener::bind(API_ADDRESS).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
