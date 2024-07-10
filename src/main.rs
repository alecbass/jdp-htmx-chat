use rocket::form::Form;
use rocket::fs::FileServer;
use rocket_dyn_templates::{context, Template};

#[macro_use]
extern crate rocket;

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
fn get_messages() -> Template {
    let messages = vec!["hellooo", "epic"];

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
#[post("/create-message", data = "<message>")]
fn create_message(message: Form<CreateMessageRequest>) -> Template {
    let messages = vec!["hellooo", "epic", &message.message];

    Template::render(
        "messages",
        context! {
            messages: messages
        },
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, get_messages, create_message])
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
}
