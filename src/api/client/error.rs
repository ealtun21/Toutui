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

/// Tells if a report of a fault holds a token that the server refused.
///
/// **The startup makes many requests, and each of them can carry this fault.**
/// The old code let the report leave `main`: the user then read
/// `Error: The token is not valid. Log in again.` with a line of the source of
/// the program, and no screen of the program came. The program reads the
/// category here, and it opens the login screen. See T-123.
///
/// The function looks at every cause of the report, because a caller can put
/// the fault of the API inside a report of its own.
pub fn the_token_is_not_valid(report: &color_eyre::eyre::Report) -> bool {
    report.chain().any(|cause| {
        matches!(
            cause.downcast_ref::<ApiError>(),
            Some(ApiError::Unauthorized)
        )
    })
}

/// Tells if a report of a fault holds a read of the accounts that failed.
///
/// **A refresh is not a start** (T-205). T-199 gave the read of the accounts a
/// fault of its own, and `main` stops the program with it: at the start that is
/// right, because the program holds no account at all. A refresh of the key `R`
/// holds the account, the token, every list, and the playback of the user
/// already, therefore a database that a second program of this account writes
/// (T-140) must take none of them away. The refresh reads the category here,
/// and it keeps the application that stands.
///
/// The function looks at every cause of the report, because a caller can put the
/// fault of the database inside a report of its own.
pub fn the_accounts_did_not_come(report: &color_eyre::eyre::Report) -> bool {
    report
        .chain()
        .any(|cause| cause.is::<crate::db::TheAccountsDidNotCome>())
}

/// The words for a user whose program cannot read the lists of the server.
///
/// **T-123 closed this road for a token that the server refused, and every
/// other fault of the first request kept it.** A measurement of 2026-08-14 with
/// `docs/harness/one_path_fails.py`, which answered `500` to
/// `GET /api/libraries`: the program stopped, and the terminal of the user held
///
/// ```text
/// Error: The server reported a fault. Status 500.
///
/// Location:
///     src/app.rs:644:44
/// ```
///
/// A line of the source of this program says nothing to a user, and it names no
/// road. See T-172.
///
/// **The offline mode of T-25 is not the road of this fault.** That mode is made
/// for a server that gives no answer, and its words say that the server does not
/// answer: a server that reports a fault answers, and those words are a reason
/// that the program does not have (T-91 and T-171). The program says what the
/// server said, and it stops.
///
/// The function looks at every cause of the report, because a caller can put the
/// fault of the API inside a report of its own. A report with no fault of the
/// API gives its own text.
pub fn the_words_of_a_program_that_stops(
    report: &color_eyre::eyre::Report,
    username: &str,
    server: &str,
) -> String {
    // **A fault of the database is not a fault of the server.** The words below
    // say that the program cannot read the lists of the server, and a program
    // that did not read the accounts of its own database must not say that: a
    // view never says a reason that the program does not have (T-91). See
    // T-199.
    if let Some(fault) = report
        .chain()
        .find_map(|cause| cause.downcast_ref::<crate::db::TheAccountsDidNotCome>())
    {
        return format!(
            "Toutui stops: it cannot read the accounts of its database.\n\
             {}\n\
             The account is {}.\n\
             Toutui changed nothing. Stop a second Toutui of this account, and start this one \
             again.",
            fault, username
        );
    }

    // **A fault of the configuration file is not a fault of the server.** The
    // file belongs to the user, and the user can correct it: therefore the
    // words name that file and the road back, and they do not say that the
    // server did anything at all (T-91). See T-265.
    if let Some(fault) = report
        .chain()
        .find_map(|cause| cause.downcast_ref::<crate::config::TheConfigurationFileDidNotCome>())
    {
        return format!(
            "Toutui stops: it cannot read its configuration file.\n\
             {}\n\
             Correct that file, or give it a different name: Toutui then makes a new file.\n\
             Toutui changed nothing.",
            fault
        );
    }

    let what_the_server_said = report
        .chain()
        .find_map(|cause| cause.downcast_ref::<ApiError>())
        .map(|error| error.to_string())
        .unwrap_or_else(|| report.to_string());

    format!(
        "Toutui stops: it cannot read the lists of the server.\n\
         {}\n\
         The account is {}, and the server is {}.\n\
         Toutui changed nothing. Try again later, or speak to an administrator \
         of the server.",
        what_the_server_said, username, server
    )
}

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

    /// The startup gives one report, and the program must read the category of
    /// the token inside it. See T-123.
    #[test]
    fn a_report_of_a_token_that_is_not_valid_says_so() {
        let report = color_eyre::eyre::Report::new(ApiError::Unauthorized);

        assert!(the_token_is_not_valid(&report));
    }

    /// A caller can put the fault inside a report of its own, and the sentence
    /// of the caller must not hide the category.
    #[test]
    fn a_report_that_holds_the_fault_deeper_says_so() {
        let report = color_eyre::eyre::Report::new(ApiError::Unauthorized)
            .wrap_err("the libraries of the server");

        assert!(the_token_is_not_valid(&report));
    }

    /// Every other fault must not send the user to the login screen. A server
    /// that does not answer starts the offline mode.
    #[test]
    fn every_other_fault_keeps_the_user_out_of_the_login_screen() {
        for fault in [
            ApiError::Unreachable,
            ApiError::Timeout,
            ApiError::Forbidden,
            ApiError::NotFound,
            ApiError::Server(500),
            ApiError::Decode("bad".to_string()),
        ] {
            let report = color_eyre::eyre::Report::new(fault.clone());

            assert!(
                !the_token_is_not_valid(&report),
                "{:?} must not open the login screen",
                fault
            );
        }

        let of_a_string = color_eyre::eyre::eyre!("The token is not valid. Log in again.");
        assert!(
            !the_token_is_not_valid(&of_a_string),
            "the words of a sentence are not the category"
        );
    }

    /// **A user must not read a line of the source of this program.** The
    /// measurement of T-172 gave `Error: The server reported a fault. Status
    /// 500.` with `Location: src/app.rs:644:44`, and no screen of the program
    /// came.
    #[test]
    fn the_words_of_a_program_that_stops_name_the_server_and_no_line_of_the_source() {
        let report = color_eyre::eyre::Report::new(ApiError::Server(500));

        let words = the_words_of_a_program_that_stops(&report, "toutuitest", "127.0.0.1:13500");

        assert!(
            words.contains("The server reported a fault. Status 500."),
            "the words must say what the server said: {}",
            words
        );
        assert!(words.contains("toutuitest"), "{}", words);
        assert!(words.contains("127.0.0.1:13500"), "{}", words);
        assert!(
            words.contains("Toutui changed nothing"),
            "the user must know that the program wrote nothing: {}",
            words
        );

        // **No line of the source, and no name of a file of this program.**
        assert!(!words.contains("Location"), "{}", words);
        assert!(!words.contains("src/"), "{}", words);
        assert!(!words.contains(".rs"), "{}", words);

        // The offline mode of T-25 is not the road of this fault: the server
        // answers. See T-171.
        assert!(!words.contains("does not answer"), "{}", words);

        // A report that holds no fault of the API gives its own text.
        let other = color_eyre::eyre::Report::msg("the disk is full");
        let words = the_words_of_a_program_that_stops(&other, "toutuitest", "one:1");

        assert!(words.contains("the disk is full"), "{}", words);
    }

    /// A fault of the configuration file names that file and the road back, and
    /// it says nothing of the server: the server answered nothing at all at
    /// that moment. See T-265.
    #[test]
    fn the_words_of_a_configuration_file_that_did_not_come_name_the_file_and_the_road_back() {
        let report = color_eyre::eyre::Report::new(crate::config::TheConfigurationFileDidNotCome {
            path: "/home/one/.config/toutui/config.toml".to_string(),
            reason: "TOML parse error at line 64, column 31\nunclosed array, expected `]`"
                .to_string(),
        });

        let words = the_words_of_a_program_that_stops(&report, "toutuitest", "127.0.0.1:13399");

        assert!(
            words.contains("/home/one/.config/toutui/config.toml"),
            "the words must name the file of the user: {}",
            words
        );
        assert!(
            words.contains("unclosed array"),
            "the words must hold what the crate said, because it names the line: {}",
            words
        );
        assert!(
            words.contains("Correct that file"),
            "the words must name the road back: {}",
            words
        );
        assert!(words.contains("Toutui changed nothing"), "{}", words);

        // **The server is not the reason.** A view never says a reason that the
        // program does not have (T-91).
        assert!(!words.contains("lists of the server"), "{}", words);
        assert!(!words.contains("127.0.0.1:13399"), "{}", words);

        // **No line of the source of this program.**
        assert!(!words.contains("Location"), "{}", words);
        assert!(!words.contains("src/"), "{}", words);
    }
}
