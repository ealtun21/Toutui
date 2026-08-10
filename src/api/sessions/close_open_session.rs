//! The request that closes an open listening session.

use crate::api::client::error::ApiError;
use crate::api::client::{ApiClient, Idempotent};
use reqwest::Method;

/// Closes an open listening session.
///
/// The request has no body. Therefore the server keeps the position that the
/// last sync request gave.
///
/// The client never sends this request a second time.
///
/// See <https://api.audiobookshelf.org/#close-an-open-session>.
pub async fn close_session_without_send_prg_data(
    client: &ApiClient,
    session_id: &str,
) -> Result<(), ApiError> {
    client
        .send(
            Method::POST,
            &format!("/api/session/{}/close", session_id),
            None,
            Idempotent::No,
        )
        .await?;

    Ok(())
}
