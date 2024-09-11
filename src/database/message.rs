use macros::load_query;
use rusqlite::{named_params, params, Connection, Error, OptionalExtension, Result, Row};

use super::{constants::DB_PATH, user::User};

#[derive(Clone)]
pub struct Message {
    pub id: i32,
    pub text: String,
    pub author_id: i32,
    pub author_name: String,
}

impl Message {
    pub fn new(id: i32, text: String, author_id: i32, author_name: String) -> Self {
        Self {
            id,
            text,
            author_id,
            author_name,
        }
    }
}

impl<'stmt> TryFrom<&'stmt Row<'stmt>> for Message {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.get(0)?;
        let text = row.get(1)?;
        let author_id = row.get(2)?;
        let author_name = row.get(3)?;

        Ok(Self {
            id,
            text,
            author_id,
            author_name,
        })
    }
}

/// Creates a new message in the database
///     
/// # Arguments
/// * `message` - The message to be created
///
pub fn create_message(message: &str, user_id: i32) -> Result<Message, Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        load_query!("insert_message.sql"),
        named_params! {
            ":message": message,
            ":user_id": user_id
        },
    )?;

    // Get the last inserted row's message
    let mut statement = conn.prepare(load_query!("select_last_message.sql"))?;
    let message = statement.query_row(params![], |row| row.try_into())?;

    Ok(message)
}

/// Retrieves all messages
pub fn get_messages() -> Result<Vec<Message>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare(load_query!("select_all_messages.sql"))?;
    let messages = statement
        .query_map(params![], |row| row.try_into())?
        .map(|row| row.unwrap())
        .collect::<Vec<Message>>();

    Ok(messages)
}

/// Retrieves a specific message with a given ID
pub fn get_message_by_id(id: i32) -> Result<Option<Message>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare(load_query!("select_message.sql"))?;

    statement
        .query_row(named_params! { ":message_id": id}, |row| row.try_into())
        .optional()
}

pub fn delete_message(message_id: i32) -> Result<usize, Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        load_query!("delete_message.sql"),
        named_params! { ":message_id": message_id },
    )
}

/// Returns whether a mesage can be deleted by a given user
///
/// A message can only be deleted by its own author
pub fn can_user_delete(message: &Message, user: &User) -> bool {
    message.author_id == user.id
}
