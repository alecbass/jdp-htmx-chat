use macros::load_query;
use rusqlite::{named_params, params, Connection, Error, OptionalExtension, Result};

use super::constants::DB_PATH;

pub struct User {
    pub id: i32,
    pub name: String,
}

impl User {
    pub fn new(id: i32, name: String) -> Self {
        Self { id, name }
    }
}

pub fn create_user(name: &str) -> Result<User, Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        load_query!("insert_user.sql"),
        named_params! { ":user_name": name},
    )?;

    // Get the created user
    let mut statement = conn.prepare(load_query!("select_last_user.sql"))?;

    let user = statement.query_row(params![], |row| Ok(User::new(row.get(0)?, row.get(1)?)));

    user
}

pub fn retrieve_user(id: i32) -> Result<Option<User>, Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut statement = conn.prepare(load_query!("select_user.sql"))?;
    let user = statement
        .query_row(named_params! {":id": id}, |row| {
            Ok(User::new(row.get(0)?, row.get(1)?))
        })
        .optional();

    user
}
