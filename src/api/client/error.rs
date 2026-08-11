//! The failure categories of an API request.
//!
//! The category tells the caller what to do. `Unreachable` starts the
//! offline mode. `Unauthorized` asks the user to log in again. `Forbidden`
//! tells the user that the account has no permission.

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
    /// A fault of the endpoint permits a second attempt, and the client marks
    /// that address as down. A fault of the request does not: every address
    /// gives the same answer, and the address is well.
    ///
    /// **A status of 400 is a fault of the request.** The old code held every
    /// status of `Server` as a fault of the endpoint, therefore one answer of
    /// 400 marked the address down and **every request after it said "No server
    /// address answered"**. The server of Audiobookshelf answers 400 for work
    /// that a user does every day: a book that stands in a collection already,
    /// an episode that stands in a playlist already, and a podcast whose
    /// directory exists. A measurement of 2026-08-11 put a book in a playlist
    /// two times, and the program then had no server until the examination of
    /// the address came again. See T-87.
    pub fn is_endpoint_fault(&self) -> bool {
        match self {
            ApiError::Unreachable | ApiError::Timeout => true,
            // The server answered, and it understood the request. A status of
            // 500 or more is the fault of that machine, and a different address
            // of the same server can answer it.
            ApiError::Server(status) => *status >= 500,
            _ => false,
        }
    }

    /// Tells if the application must use the offline mode.
    ///
    /// The server does not answer, thus the application reads the local copy.
    /// A token that is not valid is a different condition: the server answers,
    /// and the user must log in again. The offline mode does not help there.
    pub fn is_offline(&self) -> bool {
        matches!(self, ApiError::Unreachable | ApiError::Timeout)
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
        assert!(ApiError::Server(500).is_endpoint_fault());

        // **A status of 400 is a fault of the request.** The server answers
        // 400 for a book that stands in a collection already, and the program
        // must not lose its address for that. See T-87.
        assert!(!ApiError::Server(400).is_endpoint_fault());
        assert!(!ApiError::Server(409).is_endpoint_fault());
        assert!(!ApiError::Server(429).is_endpoint_fault());

        assert!(!ApiError::NotFound.is_endpoint_fault());
        assert!(!ApiError::Unauthorized.is_endpoint_fault());
        assert!(!ApiError::Forbidden.is_endpoint_fault());
        assert!(!ApiError::Decode("bad".to_string()).is_endpoint_fault());
    }

    /// The offline mode starts when no address answers. A server that answers
    /// with a fault of the request does not start it: the local copy gives no
    /// help for a token that is not valid.
    #[test]
    fn only_an_address_that_does_not_answer_starts_the_offline_mode() {
        assert!(ApiError::Unreachable.is_offline());
        assert!(ApiError::Timeout.is_offline());

        assert!(!ApiError::Unauthorized.is_offline());
        assert!(!ApiError::Forbidden.is_offline());
        assert!(!ApiError::NotFound.is_offline());
        assert!(!ApiError::Server(500).is_offline());
        assert!(!ApiError::Decode("bad".to_string()).is_offline());
    }
}
