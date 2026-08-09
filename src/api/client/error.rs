//! The failure categories of an API request.
//!
//! The category tells the caller what to do. `Unreachable` starts the
//! offline mode. `Unauthorized` asks the user to log in again. `Forbidden`
//! tells the user that the account has no permission.

// No caller uses these items now. Task 10 connects the application to the
// client. Remove this attribute in Task 10.
#![allow(dead_code)]

use std::fmt;

/// The cause of a failed API request.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    /// No endpoint answered.
    Unreachable,
    /// The endpoint did not answer in the permitted time.
    Timeout,
    /// The server refused the token. The token is not valid.
    Unauthorized,
    /// The account does not have the necessary permission.
    Forbidden,
    /// The server does not have the item.
    NotFound,
    /// The server reported an internal fault. The value is the HTTP status.
    Server(u16),
    /// The answer of the server does not agree with the expected format.
    Decode(String),
}

impl ApiError {
    /// Tells if a different endpoint can give a better answer.
    ///
    /// A fault of the endpoint permits a second attempt. A fault of the
    /// request does not, because each endpoint gives the same answer.
    pub fn is_endpoint_fault(&self) -> bool {
        matches!(
            self,
            ApiError::Unreachable | ApiError::Timeout | ApiError::Server(_)
        )
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unreachable => write!(f, "No server address answered."),
            ApiError::Timeout => write!(f, "The server did not answer in time."),
            ApiError::Unauthorized => write!(f, "The token is not valid. Log in again."),
            ApiError::Forbidden => write!(f, "Your account does not have this permission."),
            ApiError::NotFound => write!(f, "The server does not have this item."),
            ApiError::Server(code) => write!(f, "The server reported a fault. Status {}.", code),
            ApiError::Decode(detail) => {
                write!(f, "The answer of the server is not valid: {}", detail)
            }
        }
    }
}

impl std::error::Error for ApiError {}

/// Puts an HTTP status into a category.
///
/// Gives `None` if the status shows success.
pub fn classify_status(status: reqwest::StatusCode) -> Option<ApiError> {
    if status.is_success() {
        return None;
    }

    match status.as_u16() {
        401 => Some(ApiError::Unauthorized),
        403 => Some(ApiError::Forbidden),
        404 => Some(ApiError::NotFound),
        code => Some(ApiError::Server(code)),
    }
}

/// Puts a transport fault into a category.
pub fn classify_transport(error: &reqwest::Error) -> ApiError {
    if error.is_timeout() {
        ApiError::Timeout
    } else if error.is_decode() {
        ApiError::Decode(error.to_string())
    } else {
        ApiError::Unreachable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn a_success_status_is_not_an_error() {
        assert!(classify_status(StatusCode::OK).is_none());
        assert!(classify_status(StatusCode::NO_CONTENT).is_none());
    }

    #[test]
    fn client_statuses_get_their_own_category() {
        assert!(matches!(
            classify_status(StatusCode::UNAUTHORIZED),
            Some(ApiError::Unauthorized)
        ));
        assert!(matches!(
            classify_status(StatusCode::FORBIDDEN),
            Some(ApiError::Forbidden)
        ));
        assert!(matches!(
            classify_status(StatusCode::NOT_FOUND),
            Some(ApiError::NotFound)
        ));
    }

    #[test]
    fn a_server_status_keeps_its_code() {
        assert!(matches!(
            classify_status(StatusCode::BAD_GATEWAY),
            Some(ApiError::Server(502))
        ));
    }

    /// Only these categories permit a change to a different endpoint. A 404
    /// comes from the server, and a different endpoint gives the same
    /// answer.
    #[test]
    fn only_endpoint_faults_permit_a_change_of_endpoint() {
        assert!(ApiError::Unreachable.is_endpoint_fault());
        assert!(ApiError::Timeout.is_endpoint_fault());
        assert!(ApiError::Server(503).is_endpoint_fault());

        assert!(!ApiError::NotFound.is_endpoint_fault());
        assert!(!ApiError::Unauthorized.is_endpoint_fault());
        assert!(!ApiError::Forbidden.is_endpoint_fault());
        assert!(!ApiError::Decode("bad".to_string()).is_endpoint_fault());
    }
}
