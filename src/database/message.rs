use macros::load_query;
use rusqlite::{named_params, params, Connection, Error, Result};

use super::constants::DB_PATH;

#[derive(Clone)]
pub struct Message {
    pub text: String,
}

impl Message {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

///
/// Creates a new message in the database
///     
/// # Arguments
/// * `message` - The message to be created
///
pub fn create_message(message: &str) -> Result<Message, Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        load_query!("insert_message.sql"),
        named_params! { ":message": message },
    )?;

    // Get the last inserted row's message
    let mut statement = conn.prepare(load_query!("select_last_message_text.sql"))?;
    let messages = statement
        .query_map(params![], |row| Ok(Message::new(row.get(0)?)))?
        .map(|result| result.unwrap())
        .collect::<Vec<Message>>();

    let message = messages.get(0).unwrap().clone();

    Ok(message)
}

pub fn get_messages() -> Result<Vec<Message>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare(load_query!("select_all_messages_text.sql"))?;
    let messages = statement
        .query_map(params![], |row| Ok(Message::new(row.get(0)?)))?
        .map(|result| result.unwrap())
        .collect::<Vec<Message>>();

    Ok(messages)
}

pub fn get_message_by_id(id: i32) -> Result<Option<Message>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare(load_query!("select_single_message_text.sql"))?;

    let messages = statement
        .query_map(named_params! { ":id": id}, |row| {
            Ok(Message::new(row.get(0)?))
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<Message>>();

    let message = messages.get(0).map(|message| message.clone());

    Ok(message)
}
