pub struct Message {
    pub text: String,
}

impl Message {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

pub struct MessageDatabase {
    messages: Vec<Message>,
}

impl MessageDatabase {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

// TODO: Allow the message database to be retrieved within its mutex via a request guard

// use rocket::request::{FromRequest, Outcome, Request};
// use rocket::State;

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for MessageDatabase {
//     type Error = ();
//
//     async fn from_request(request: &'r Request<'_>) -> Outcome<&'r Self, ()> {
//         request
//             .guard::<&State<MessageDatabase>>()
//             .await
//             .map(|database| database.inner())
//     }
// }
