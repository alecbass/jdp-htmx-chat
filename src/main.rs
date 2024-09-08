use std::sync::Mutex;

use askama::Template;
use axum::extract::State;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Form;
use axum::Router;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use database::message::{create_message, get_messages};
use database::run_migrations;
use extractors::ExtractSession;
use middleware::session_middleware;
use template::HtmlTemplate;
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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    has_session_id: bool,
    session_id: String,
}
///
/// GET request to load the index page
///
async fn index_view(ExtractSession(session): ExtractSession) -> impl IntoResponse {
    if session.is_none() {
        return HtmlTemplate(IndexTemplate {
            has_session_id: false,
            session_id: "".to_string(),
        });
    }

    let session = session.unwrap();

    let template = IndexTemplate {
        has_session_id: true,
        session_id: session.id,
    };
    HtmlTemplate(template)
}

#[derive(Deserialize)]
struct CreateMessageRequest {
    message: String,
}

#[derive(Template)]
#[template(path = "messages.html")]
struct GetMessagesTemplate {
    success: bool,
    messages: Vec<String>,
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

    // Format the current state's messages into a list of message texts
    let messages = messages
        .iter()
        .map(|message| message.text.clone())
        .collect::<Vec<String>>();

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
    message: String,
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

    // Add the new message to the list of messages
    let message = create_message(&message_data.message, 1).expect("Could not create message");

    // Get the message's text
    let text = message.text;

    // Broadcast to all active clients that a new message was created
    let mut websocket_handler = state.websocket_handler.lock().unwrap();

    let broadcast_template = NewMessageTemplate { message: text };

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
        std::net::TcpListener::bind("0.0.0.0:8001").expect("Could not start websocket server.");

    let websocket_handler = Box::new(Mutex::new(WebSocketHandler::new()));

    // This lives in state but is meant to be static for the entire runtime of the program, so
    // having it leaked doesn't seem like a big deal
    // Having it static satisfies the state's Clone derivation requirement
    let websocket_handler: &'static Mutex<WebSocketHandler> = Box::leak(websocket_handler);

    let state = AppState { websocket_handler };

    std::thread::spawn(move || {
        for stream in server.incoming() {
            let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();

            websocket
                .send(tungstenite::Message::Text("Hello from server!".to_string()))
                .unwrap();

            let lock = websocket_handler.lock();

            if let Err(e) = &lock {
                eprintln!("Failed to lock websocket handler: {}", e);
            }

            let mut lock = lock.unwrap();

            lock.add_websocket(websocket);
        }
    });

    let static_dir = ServeDir::new("static");

    println!("WebSocket server listening...");
    // build our application with a route
    let app = Router::new()
        .route("/", get(index_view))
        .route("/message/", get(get_messages_view))
        .route("/create-message/", post(create_message_view))
        .nest_service("/static", static_dir)
        .with_state(state)
        .layer(from_fn(session_middleware));

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
