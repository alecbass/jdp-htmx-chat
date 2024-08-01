use rusqlite::{Connection, Error, Result};

use constants::DB_PATH;

mod constants;
pub mod message;

// Embed migrations into code here
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("database");
}

///
/// Loads migrations from the database/migrations directory and runs them
///
pub fn run_migrations() -> Result<(), Error> {
    let mut conn = Connection::open(DB_PATH)?;

    embedded::migrations::runner().run(&mut conn).unwrap();

    Ok(())
}
