use rocket::form::Form;
use rocket::fs::FileServer;
use rocket_dyn_templates::{context, Template};

use database::{create_message, get_messages, run_migrations};

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
fn create_message_view(message_data: Form<CreateMessageRequest>) -> Template {
    // Add the new message to the list of messages
    let message = create_message(&message_data.message).expect("Could not create message");

    // Get the message's text
    let text = message.text;

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

    rocket::build()
        .mount("/", routes![index, get_messages_view, create_message_view])
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
}
