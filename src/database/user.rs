use macros::load_query;
use rusqlite::{named_params, params, Connection, Error, Result};

use super::constants::DB_PATH;

pub struct User {
    id: i32,
    name: String,
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
        named_params! { ":name": name},
    )?;

    // Get the created user
    let mut statement = conn.prepare(load_query!("select_last_user.sql"))?;
    let user = statement.query_row(params![], |row| Ok(User::new(row.get(0)?, row.get(1)?)));

    user
}
