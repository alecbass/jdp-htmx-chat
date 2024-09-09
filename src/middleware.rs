use std::time::Duration;

use axum::extract::Request;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::AppendHeaders;
use axum::response::Response;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::Expiration;
use axum_extra::extract::CookieJar;

use crate::database::session::retrieve_session;
use crate::extractors::ExtractSession;

pub async fn session_middleware(
    ExtractSession(session): ExtractSession,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {
    // Pre-view
    if let Some(session_cookie) = jar.get("session_id") {
        println!("Session ID: {:?}", session_cookie);
        let session_id = session_cookie.value();

        if let Ok(session) = retrieve_session(session_id) {
            if let Some(session) = session {
                println!(
                    "Found session {} for user {:?}",
                    session.id, session.user_id
                );
            }
        }
    }

    let response = next.run(request).await;

    // // Post-view
    // let cookie = Cookie::build(("session_id", session.id))
    //     .domain("localhost")
    //     .secure(true)
    //     .build();
    //
    // let jar = jar.add(cookie);

    response
}
