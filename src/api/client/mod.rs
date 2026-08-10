//! The HTTP client of the application.
//!
//! The client owns one `reqwest::Client`. Therefore the program uses
//! connection pooling. The client asks the endpoint pool for an address
//! before each request. If the address does not answer, and if the request
//! is idempotent, the client sends the request one time to the next address.

pub mod endpoint;
pub mod error;
pub mod probe;

use endpoint::EndpointPool;
use error::{classify_status, classify_transport, ApiError};
use reqwest::header::CONTENT_DISPOSITION;
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// The time to wait for a connection. A short time makes the program detect
/// an address that does not answer quickly.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// The time to wait for a full answer. Downloads do not use this value.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// The time to wait for a full download. An audiobook can be some
/// gigabytes. Therefore the download needs much more time.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60 * 60 * 6);

/// Tells if the client can send a request a second time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Idempotent {
    /// The request does not change data, or it sets an absolute value.
    Yes,
    /// A second request makes a duplicate. An example is a session request.
    No,
}

/// The HTTP client of the application.
pub struct ApiClient {
    http: reqwest::Client,
    pool: Arc<EndpointPool>,
    token: String,
}

// The token is a secret. Therefore the debug output does not show it.
impl fmt::Debug for ApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiClient")
            .field("pool", &self.pool)
            .field("token", &"<hidden>")
            .finish()
    }
}

impl ApiClient {
    /// Makes a client.
    ///
    /// The token must be the decrypted token. Give the token one time. The
    /// client does not decrypt a token for each request.
    pub fn new(pool: Arc<EndpointPool>, token: String) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| classify_transport(&error))?;

        Ok(ApiClient { http, pool, token })
    }

    /// Gives the endpoint pool. The probe task and the tests use it.
    pub fn pool(&self) -> Arc<EndpointPool> {
        Arc::clone(&self.pool)
    }

    /// Gives the HTTP client. The probe task uses it.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Sends a request. This function has all the failover logic.
    ///
    /// The `path` value must start with a slash.
    pub async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
        idempotent: Idempotent,
    ) -> Result<Response, ApiError> {
        let first = self.pool.active().ok_or(ApiError::Unreachable)?;

        let first_error = match self
            .attempt(&first, method.clone(), path, body.clone())
            .await
        {
            Ok(response) => return Ok(response),
            Err(error) => error,
        };

        // A fault of the request gives the same answer on each address.
        if !first_error.is_endpoint_fault() {
            return Err(first_error);
        }

        self.pool.mark_down(&first);

        if idempotent == Idempotent::No {
            return Err(first_error);
        }

        let second = match self.pool.next_after(&first) {
            Some(url) => url,
            None => return Err(ApiError::Unreachable),
        };

        match self.attempt(&second, method, path, body).await {
            Ok(response) => Ok(response),
            Err(error) => {
                if error.is_endpoint_fault() {
                    self.pool.mark_down(&second);
                }
                Err(error)
            }
        }
    }

    /// Sends one request to one address.
    async fn attempt(
        &self,
        base_url: &str,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response, ApiError> {
        let mut request = self
            .http
            .request(method, format!("{}{}", base_url, path))
            .bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|e| classify_transport(&e))?;

        match classify_status(response.status()) {
            Some(error) => Err(error),
            None => Ok(response),
        }
    }

    /// Reads JSON with a `GET` request. A `GET` request is idempotent.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = self.send(Method::GET, path, None, Idempotent::Yes).await?;

        response
            .json::<T>()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))
    }

    /// Sends a `PATCH` request. The Audiobookshelf progress endpoint sets an
    /// absolute position. Therefore this request is idempotent.
    pub async fn patch_json<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        self.send(Method::PATCH, path, Some(value), Idempotent::Yes)
            .await?;

        Ok(())
    }

    /// Sends a `POST` request and reads the answer.
    ///
    /// The client never sends this request a second time. A second request
    /// makes a duplicate listening session on the server.
    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        let response = self
            .send(Method::POST, path, Some(value), Idempotent::No)
            .await?;

        response
            .json::<T>()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))
    }

    /// Sends a `POST` request that has no answer body.
    pub async fn post_no_content<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        self.send(Method::POST, path, Some(value), Idempotent::No)
            .await?;

        Ok(())
    }

    /// Downloads a file to a directory.
    ///
    /// This method does not use the normal request timeout. An audiobook can
    /// be some gigabytes, and the download can take many minutes. The
    /// connect timeout is still active. Therefore an address that does not
    /// answer still fails quickly.
    ///
    /// The method takes the file name from the `Content-Disposition` header.
    /// If the header has no name, the method uses `fallback_filename`.
    ///
    /// The method does not change to a different address, because a second
    /// download costs much time and much data. It marks the address `Down`
    /// and gives the error. The user starts the download again.
    pub async fn download_to_file(
        &self,
        path: &str,
        dest_dir: &Path,
        fallback_filename: &str,
    ) -> Result<PathBuf, ApiError> {
        let base_url = self.pool.active().ok_or(ApiError::Unreachable)?;

        let response = self
            .http
            .get(format!("{}{}", base_url, path))
            .bearer_auth(&self.token)
            .timeout(DOWNLOAD_TIMEOUT)
            .send()
            .await
            .map_err(|error| {
                let error = classify_transport(&error);
                if error.is_endpoint_fault() {
                    self.pool.mark_down(&base_url);
                }
                error
            })?;

        if let Some(error) = classify_status(response.status()) {
            return Err(error);
        }

        let filename = response
            .headers()
            .get(CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split("filename=").nth(1))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| fallback_filename.to_string());

        tokio::fs::create_dir_all(dest_dir)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        let dest_path = dest_dir.join(&filename);

        let bytes = response
            .bytes()
            .await
            .map_err(|error| classify_transport(&error))?;

        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        file.write_all(&bytes)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        // A `tokio::fs::File` keeps the bytes in a buffer. The flush sends
        // the bytes to the disk. Without the flush the file can be empty.
        file.flush()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        Ok(dest_path)
    }
}
