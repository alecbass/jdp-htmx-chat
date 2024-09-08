use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

use crate::extractors::ExtractSession;

pub async fn session_middleware(
    ExtractSession(session): ExtractSession,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    if let Some(session) = session {
        // Set the session ID header
        let headers = response.headers_mut();

        let session_id_header = HeaderValue::from_str(&session.id);

        if let Ok(header_value) = session_id_header {
            headers.insert("session_id", header_value);
        }
    }

    response
}
