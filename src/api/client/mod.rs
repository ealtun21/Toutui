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
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// The time to wait for a full answer. Downloads do not use this value.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

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

/// Tells why an address goes to the state `Down`. See T-171.
///
/// **A server that answers `500` is not a server that is away.** The pool marks
/// that address down, because a different address of the same server can answer
/// it (T-87), and the words for the user are not the words of a server that
/// gives no answer at all: the header of the program said "the server does not
/// answer" for a server that answered `curl` in 1.4 milliseconds.
fn why_the_address_goes_down(error: &ApiError) -> endpoint::WhyDown {
    match error {
        ApiError::Server(_) => endpoint::WhyDown::ItAnsweredWithAFault,
        _ => endpoint::WhyDown::ItGaveNoAnswer,
    }
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
        // **The request tries an address before the program says that no
        // address answered.** See T-128 and `EndpointPool::an_address`.
        let first = self.pool.an_address().ok_or(ApiError::Unreachable)?;

        let first_error = match self
            .attempt(&first, method.clone(), path, body.clone())
            .await
        {
            Ok(response) => {
                // The address answered, therefore the requests that stopped at
                // their time limit before it say nothing about it. See T-97.
                self.pool.the_address_answered(&first);
                return Ok(response);
            }
            Err(error) => error,
        };

        // A fault of the request gives the same answer on each address.
        if !first_error.is_endpoint_fault() {
            return Err(first_error);
        }

        // **A request that stopped at its time limit is not evidence that the
        // address is down.** The server does slow work for some requests of a
        // user, and a pool of one address then has no address at all. See T-97
        // and T-87.
        if self.the_address_must_go_down(&first, &first_error) {
            self.pool.mark_down(
                &first,
                &format!("{}", first_error),
                why_the_address_goes_down(&first_error),
            );
        }

        if idempotent == Idempotent::No {
            return Err(first_error);
        }

        let second = match self.pool.next_after(&first) {
            // **The fault of the first address is the answer of this request.**
            // The old code gave `Unreachable` here, therefore a pool of one
            // address said "No server address answered" for a request that
            // stopped at its time limit, and the user did not read the reason.
            // See T-97.
            None => return Err(first_error),
            Some(url) => url,
        };

        match self.attempt(&second, method, path, body).await {
            Ok(response) => {
                self.pool.the_address_answered(&second);
                Ok(response)
            }
            Err(error) => {
                if error.is_endpoint_fault() && self.the_address_must_go_down(&second, &error) {
                    self.pool.mark_down(
                        &second,
                        &format!("{}", error),
                        why_the_address_goes_down(&error),
                    );
                }
                Err(error)
            }
        }
    }

    /// Tells if this fault of this address gives it the state `Down`.
    ///
    /// A connection that no machine takes is evidence at once. A request that
    /// stopped at its time limit needs a second one of its kind: the server
    /// does slow work for some requests of a user. See T-97.
    fn the_address_must_go_down(&self, url: &str, error: &ApiError) -> bool {
        if !matches!(error, ApiError::Timeout) {
            return true;
        }

        self.pool.a_request_stopped_at_its_time_limit(url)
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

        the_body_of_the_answer(response).await
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

        the_body_of_the_answer(response).await
    }

    /// Sends a `DELETE` request that has no answer body.
    ///
    /// A `DELETE` is idempotent: the same request a second time gives the
    /// same state of the server. Therefore the client may try a second
    /// address when the first one does not answer.
    pub async fn delete_no_content(&self, path: &str) -> Result<(), ApiError> {
        self.send(Method::DELETE, path, None, Idempotent::Yes)
            .await?;

        Ok(())
    }

    /// Sends a `POST` request, and it gives the status and the body that the
    /// server answered.
    ///
    /// Every other method of this client classifies the status before the
    /// caller reads the answer, and the body of a fault then goes away.
    /// **`POST /api/emails/send-ebook-to-device` answers `404` for three
    /// different conditions**, and the body is the one place that tells them
    /// apart: "Ereader device not found", "Library item not found", and "Ebook
    /// file not found". See T-119.
    ///
    /// `time_limit` holds for this request only, and it replaces
    /// `REQUEST_TIMEOUT`. The connect timeout does not change, therefore an
    /// address that no machine takes still fails at once.
    ///
    /// A fault of the transport still gives an `ApiError`, and it still marks
    /// the address down. The request is not idempotent: a second one sends a
    /// second e-mail.
    pub async fn post_and_read_the_answer<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        time_limit: Duration,
    ) -> Result<(u16, String), ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        let base_url = self.pool.an_address().ok_or(ApiError::Unreachable)?;

        let answer = self
            .http
            .post(format!("{}{}", base_url, path))
            .bearer_auth(&self.token)
            .timeout(time_limit)
            .json(&value)
            .send()
            .await
            .map_err(|error| classify_transport(&error));

        let answer = match answer {
            Ok(answer) => answer,
            Err(error) => {
                if error.is_endpoint_fault() && self.the_address_must_go_down(&base_url, &error) {
                    self.pool.mark_down(
                        &base_url,
                        &format!("{}", error),
                        why_the_address_goes_down(&error),
                    );
                }
                return Err(error);
            }
        };

        // The address answered. A status of the answer is a fault of the
        // request, and never a fault of the address. See T-87.
        self.pool.the_address_answered(&base_url);

        let status = answer.status().as_u16();

        let words = answer
            .text()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        Ok((status, words))
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
        let base_url = self.pool.an_address().ok_or(ApiError::Unreachable)?;

        let response = self
            .http
            .get(format!("{}{}", base_url, path))
            .bearer_auth(&self.token)
            .timeout(DOWNLOAD_TIMEOUT)
            .send()
            .await
            .map_err(|error| {
                let error = classify_transport(&error);
                // A download holds a time limit of its own, and a download that
                // reaches it says no more of the address than a request does.
                // See T-97.
                if error.is_endpoint_fault() && self.the_address_must_go_down(&base_url, &error) {
                    self.pool.mark_down(
                        &base_url,
                        &format!("{}", error),
                        why_the_address_goes_down(&error),
                    );
                }
                error
            })?;

        // The address answered. A status of the answer is a fault of the
        // request, and never a fault of the address (T-87), therefore this line
        // stands before the status. See T-128.
        self.pool.the_address_answered(&base_url);

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

        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        // **The part that comes goes to the disk, and the program holds no whole
        // book in its memory.** `response.bytes()` held the whole answer: a book
        // of a scan of 502 megabytes gave the program of the user a peak of 1007
        // megabytes, because the buffer grows by a copy of itself. The
        // measurement of 2026-08-12 gave 8 megabytes with this loop.
        //
        // `logic::download::fetch` of a media of the disk holds the same shape.
        // See T-116.
        let mut response = response;

        while let Some(part) = response
            .chunk()
            .await
            .map_err(|error| classify_transport(&error))?
        {
            file.write_all(&part)
                .await
                .map_err(|error| ApiError::Decode(error.to_string()))?;
        }

        // A `tokio::fs::File` keeps the bytes in a buffer. The flush sends
        // the bytes to the disk. Without the flush the file can be empty.
        file.flush()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        Ok(dest_path)
    }
}

/// Reads the body of an answer as the structure that the caller asked for, and
/// it names the fault of a body that does not agree with that structure.
///
/// **`reqwest::Response::json` hides the cause.** A measurement of 2026-08-14
/// with `docs/harness/another_body_of_the_libraries.py`, which answered
/// `GET /api/libraries` with the body of the sandbox and one field fewer: the
/// program said `The answer of the server is not valid: error decoding response
/// body` for a field that it never reads, for a field that it reads, and for a
/// body of no JSON at all. **Those four words name no field and no place**, and
/// the user, the maintainer, and the log of the program each read the same
/// sentence for three different faults.
///
/// `serde_json` names the field and the place of the fault:
/// `missing field `name` at line 1 column 3971`. The body of the answer stays
/// in the memory of this function alone, therefore no line of it reaches the
/// screen or the log: a body can hold a token. See T-176.
async fn the_body_of_the_answer<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, ApiError> {
    let text = response
        .text()
        .await
        .map_err(|error| ApiError::Decode(error.to_string()))?;

    serde_json::from_str(&text).map_err(|error| ApiError::Decode(error.to_string()))
}
