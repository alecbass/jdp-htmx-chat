use rocket_dyn_templates::{context, Template};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Template {
    Template::render(
        "index",
        context! {
            message: "Page"
        },
    )
}

#[post("/message")]
fn create_message() -> Template {
    Template::render(
        "index",
        context! {
            message: "Hello, world!"
        },
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, create_message])
        .attach(Template::fairing())
}
