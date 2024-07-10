use super::rocket;
use rocket::form::validate::Contains;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;

#[test]
fn test_index() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    // TODO: Use the uri!() macro
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string();
    assert_eq!(body.contains("JDP"), true);
}

#[test]
fn test_get_messages() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    // Create a new message
    let response = client
        .post("/create-message")
        .header(ContentType::Form)
        .body("message=A%20test%20string!")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string();
    assert_eq!(body.contains("A test string!"), true);

    // Check the new message appears upon GET
    let response = client.get("/message").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string();
    assert_eq!(body.contains("A test string!"), true);
}
