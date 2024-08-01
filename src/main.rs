use std::net::TcpListener;
use std::sync::Mutex;

use rocket::fs::FileServer;
use rocket::{form::Form, State};
use rocket_dyn_templates::{context, Template};

use database::message::{create_message, get_messages};
use database::run_migrations;
use validators::validate_message;
use websocket::WebSocketHandler;

#[macro_use]
extern crate rocket;

mod database;
mod validators;
mod websocket;

#[cfg(test)]
mod tests;

///
/// GET request to load the index page
///
#[get("/")]
fn index() -> Template {
    let messages = vec!["index", "messages"];

    Template::render(
        "index",
        context! {
            messages: messages
        },
    )
}

#[derive(FromForm)]
struct CreateMessageRequest {
    message: String,
}

///
/// GET request to load all messages
///
#[get("/message")]
fn get_messages_view() -> Template {
    let messages = get_messages();

    if let Err(e) = messages {
        return Template::render(
            "messages",
            context! {
                messages: Vec::<String>::new(),
                error: format!("Error: {}", e)
            },
        );
    }

    let messages = messages.unwrap();

    // Format the current state's messages into a list of message texts
    let messages = messages
        .iter()
        .map(|message| message.text.clone())
        .collect::<Vec<String>>();

    Template::render(
        "messages",
        context! {
            messages: messages,
        },
    )
}

///
/// POST request to create a new message, and return the newly created message as HTML
///
#[post("/create-message", data = "<message_data>")]
fn create_message_view(
    message_data: Form<CreateMessageRequest>,
    websocket_handler: &State<&'static Mutex<WebSocketHandler>>,
) -> Template {
    let text = &message_data.message;

    let validation = validate_message(text);

    if let Err(_e) = validation {
        return Template::render(
            "new_message",
            context! {
                error: "Invalid message"
            },
        );
    }

    // Add the new message to the list of messages
    let message = create_message(&message_data.message).expect("Could not create message");

    // Get the message's text
    let text = message.text;

    // Broadcast to all active clients that a new message was created
    let mut websocket_handler = websocket_handler.lock().unwrap();

    // Broadcast a new message to swap into the messages section
    let broadcast_text = format!(
        "<section id=\"messages\" hx-swap-oob=\"beforeend\">
            <div class=\"message message-new\">
                {}
            </div>
        </section>",
        text
    );

    if let Err(e) = websocket_handler.broadcast(&broadcast_text) {
        eprintln!("Error broadcasting: {}", e);
    } else {
        println!("Broadcasted!");
    }

    Template::render(
        "new_message",
        context! {
            message: text
        },
    )
}

#[launch]
fn rocket() -> _ {
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

    rocket::build()
        .mount("/", routes![index, get_messages_view, create_message_view,])
        .mount("/static", FileServer::from("static"))
        .manage(websocket_handler)
        .attach(Template::fairing())
}
