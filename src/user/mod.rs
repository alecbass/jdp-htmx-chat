use rusqlite::Error;

use crate::database::session::Session;
use crate::database::user::{retrieve_user, User};

pub fn get_user_from_session(session: &Session) -> Result<Option<User>, Error> {
    let user_id = session.user_id;

    if user_id.is_none() {
        return Ok(None);
    }

    let user_id = user_id.unwrap();

    let user_lookup = retrieve_user(user_id);

    if let Err(e) = user_lookup {
        return Err(e);
    }

    Ok(user_lookup.unwrap())
}
