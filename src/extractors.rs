use axum::extract::FromRequestParts;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::{async_trait, http::HeaderValue};
use axum_extra::extract::CookieJar;

use crate::database::session::{create_session, retrieve_session, Session};

/// Pulls the current session out of the custom session_id HTTP header
pub struct ExtractSession(pub Session);

pub fn get_session_from_headers(headers: &HeaderMap<HeaderValue>) -> Option<Session> {
    // Read the session ID header
    let session_id = headers.get(COOKIE).map(|id| id.to_str());

    if let Some(session_id) = session_id {
        if let Ok(session_id) = session_id {
            if let Ok(db_session) = retrieve_session(session_id) {
                return db_session;
            }
        }
    }

    None
}

#[async_trait]
impl<S> FromRequestParts<S> for ExtractSession
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await;

        if jar.is_err() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse cookies from headers",
            ));
        }

        let jar = jar.unwrap();

        let mut session = None;

        if let Some(session_cookie) = jar.get("session_id") {
            println!("{:?}", session_cookie);
            // Try and load an existing session
            let session_id = session_cookie.value();
            println!("{session_id}");

            if let Ok(session_lookup) = retrieve_session(session_id) {
                // The database lookup succeeded, set the result of an existing session or not
                session = session_lookup;
            }
        }

        println!("Sessssion exists: {}", session.is_some());

        if session.is_none() {
            // If the request has no session, generate one
            if let Ok(new_session) = create_session() {
                println!("Created new session: {}", new_session.id);
                return Ok(ExtractSession(new_session));
            } else {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create session",
                ));
            }
        }

        Ok(ExtractSession(session.unwrap()))
    }
}
