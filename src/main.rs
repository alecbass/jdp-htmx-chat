use std::sync::Mutex;

use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use database::{Message, MessageDatabase};

#[macro_use]
extern crate rocket;

mod database;

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
fn get_messages(database: &State<Mutex<MessageDatabase>>) -> Template {
    // Lock the database to get the list of messages
    let database = database.lock().unwrap();

    // Format the current state's messages into a list of message texts
    let messages = database
        .get_messages()
        .iter()
        .map(|message| message.text.clone())
        .collect::<Vec<String>>();

    Template::render(
        "messages",
        context! {
            messages: messages
        },
    )
}

///
/// POST request to create a new message, and return the current list of messages as HTML
///
#[post("/create-message", data = "<message_data>")]
fn create_message(
    message_data: Form<CreateMessageRequest>,
    database: &State<Mutex<MessageDatabase>>,
) -> Template {
    let database = database.lock();

    if let Err(e) = database {
        return Template::render(
            "messages",
            context! {
                error: format!("Failed to lock the database: {}", e)
            },
        );
    }

    let mut database = database.unwrap();

    // Add the new message to the list of messages
    let new_message = Message::new(message_data.message.clone());

    database.add_message(new_message);

    // Format the current state's messages into a list of message texts
    let messages = database
        .get_messages()
        .iter()
        .map(|message| message.text.clone())
        .collect::<Vec<String>>();

    Template::render(
        "messages",
        context! {
            messages: messages
        },
    )
}

#[launch]
fn rocket() -> _ {
    // Initialise the "database" of messages
    let database = Mutex::new(MessageDatabase::new());

    rocket::build()
        .mount("/", routes![index, get_messages, create_message])
        .mount("/static", FileServer::from("static"))
        .manage(database)
        .attach(Template::fairing())
}
