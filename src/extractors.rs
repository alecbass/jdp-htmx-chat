use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::database::session::{create_session, retrieve_session, Session};

pub struct ExtractSession(pub Option<Session>);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractSession
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Read the session ID header
        let session_id = parts.headers.get("session_id").map(|id| id.to_str());

        let mut session = None;

        if let Some(session_id) = session_id {
            if let Ok(session_id) = session_id {
                if let Ok(db_session) = retrieve_session(session_id) {
                    session = db_session;
                }
            }
        }

        if session.is_none() {
            // If the request has no session, generate one
            if let Ok(new_session) = create_session() {
                session = Some(new_session);
            }
        }

        Ok(ExtractSession(session))
    }
}
