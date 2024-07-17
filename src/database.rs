use rusqlite::{params, Connection, Error, Result};

#[derive(Clone)]
pub struct Message {
    pub text: String,
}

impl Message {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

// Embed migrations into code here
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("database");
}

const DB_PATH: &'static str = "database/jdp-db.db";

///
/// Loads migrations from the database/migrations directory and runs them
///
pub fn run_migrations() -> Result<(), Error> {
    let mut conn = Connection::open(DB_PATH)?;

    embedded::migrations::runner().run(&mut conn).unwrap();

    Ok(())
}

///
/// Creates a new message in the database
///     
/// # Arguments
/// * `message` - The message to be created
///
pub fn create_message(message: &str) -> Result<(), Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute("INSERT INTO message (text) VALUES (?1)", params![message])?;

    Ok(())
}

pub fn get_messages() -> Result<Vec<Message>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare("SELECT text FROM message")?;
    let messages = statement
        .query_map(params![], |row| Ok(Message::new(row.get(0)?)))?
        .map(|result| result.unwrap())
        .collect::<Vec<Message>>();

    Ok(messages)
}

pub fn get_message_by_id(id: i32) -> Result<Option<Message>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare("SELECT text FROM message WHERE id = ?1")?;
    let messages = statement
        .query_map(params![id], |row| Ok(Message::new(row.get(0)?)))?
        .map(|result| result.unwrap())
        .collect::<Vec<Message>>();

    let message = messages.get(0).map(|message| message.clone());

    Ok(message)
}
