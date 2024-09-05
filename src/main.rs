use std::net::TcpListener;
use std::sync::Mutex;

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use axum::{extract, Json, Router};

use database::message::{create_message, get_messages};
use database::run_migrations;
use template::HtmlTemplate;
use validators::validate_message;
use websocket::WebSocketHandler;

mod database;
mod template;
mod validators;
mod websocket;

#[cfg(test)]
mod tests;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}
///
/// GET request to load the index page
///
fn index() -> impl IntoResponse {
    let template = IndexTemplate {};
    HtmlTemplate(template)
}

struct CreateMessageRequest {
    message: String,
}

#[derive(Template)]
#[template(path = "messages.html")]
struct GetMessagesTemplate {
    messages: Vec<String>,
    error: String,
}

///
/// GET request to load all messages
///
fn get_messages_view() -> impl IntoResponse {
    let messages = get_messages();

    if let Err(e) = messages {
        let template = GetMessagesTemplate {
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
        messages,
        error: "".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "new_message.html")]
struct NewMessageTemplate {
    error: String,
}

#[derive(Template)]
#[template(path = "new_message_success.html")]
struct NewMessageSuccessTemplate {}

///
/// POST request to create a new message, and return the newly created message as HTML
///
fn create_message_view(
    message_data: Form<CreateMessageRequest>,
    websocket_handler: &State<&'static Mutex<WebSocketHandler>>,
    metadata: Metadata,
) -> impl IntoResponse {
    let text = &message_data.message;

    let validation = validate_message(text);

    if let Err(_e) = validation {
        let template = NewMessageTemplate {
            error: "Invalid message".to_string(),
        };
        return HtmlTemplate(template);
    }

    // Add the new message to the list of messages
    let message = create_message(&message_data.message).expect("Could not create message");

    // Get the message's text
    let text = message.text;

    // Broadcast to all active clients that a new message was created
    let mut websocket_handler = websocket_handler.lock().unwrap();

    // Broadcast a new message to swap into the messages section
    let broadcast_html = metadata.render("new_message", context! { message: &text });

    if let Some((_, broadcast_html)) = broadcast_html {
        if let Err(e) = websocket_handler.broadcast(&broadcast_html) {
            eprintln!("Error broadcasting: {}", e);
        }
    }

    let template = NewMessageSuccessTemplate {};
    HtmlTemplate(template)
}

#[tokio::main]
async fn main() {
    run_migrations().expect("Could not run migrations");
    let server = TcpListener::bind("0.0.0.0:8001").expect("Could not start websocket server.");

    // Create a central repository of all active websockets
    let websocket_handler = Box::new(Mutex::new(WebSocketHandler::new()));

    // This lives in state but is meant to be static for the entire runtime of the program, so
    // having it leaked doesn't seem like a big deal
    let websocket_handler: &'static Mutex<WebSocketHandler> = Box::leak(websocket_handler);

    std::thread::spawn(move || {
        for stream in server.incoming() {
            println!("ACCEPTING STREAM");
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

    println!("WebSocket server listening...");
    // build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .route("/upload/:file_id/:auth_code/", post(upload))
        .route("/:file_id/", delete(delete_file))
        .layer(
            // Add CORS
            cors,
        );

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
