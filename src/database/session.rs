use std::time::{SystemTime, UNIX_EPOCH};

use macros::load_query;
use rusqlite::{named_params, params, Connection, Error, OptionalExtension, Result};
use uuid::Uuid;

use super::constants::DB_PATH;

pub struct Session {
    pub id: String,
    pub user_id: Option<i32>,
    pub expires_at: i64,
}

impl Session {
    pub fn new(id: String, user_id: Option<i32>, expires_at: i64) -> Session {
        Self {
            id,
            user_id,
            expires_at,
        }
    }
}

fn generate_session_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn create_session() -> Result<Session, Error> {
    let session_id = generate_session_id();
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards somehow")
        .as_millis() as u64;

    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        load_query!("insert_session.sql"),
        named_params! { ":id": session_id, ":expires_at": expires_at },
    )?;

    // Get the created user
    let mut statement = conn.prepare(load_query!("select_last_session.sql"))?;
    let session = statement.query_row(params![], |row| {
        Ok(Session::new(row.get(0)?, row.get(1)?, row.get(2)?))
    });

    session
}

pub fn retrieve_session(id: &str) -> Result<Option<Session>, Error> {
    let conn = Connection::open(DB_PATH)?;

    // Get the created user
    let mut statement = conn.prepare(load_query!("select_last_session.sql"))?;
    let session = statement
        .query_row(named_params! {":id": id}, |row| {
            Ok(Session::new(row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .optional();

    session
}

pub fn set_session_user(session_id: &str, user_id: i32) -> Result<Session, Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        load_query!("update_session_user.sql"),
        named_params! { ":session_id": session_id, ":user_id": user_id},
    )?;

    let mut statement = conn.prepare(load_query!("select_session.sql"))?;
    let session = statement
        .query_row(named_params! {":id": session_id}, |row| {
            Ok(Session::new(row.get(0)?, row.get(1)?, row.get(2)?))
        });

    session
}
