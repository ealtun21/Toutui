//! The devices of an e-reader, and the book that the server sends to one.
//! See T-119.
//!
//! This is the last row of section 4 of `docs/T-24-coverage.md` that said `No`
//! for a function that a user of a terminal can use.
//!
//! **The list of the devices does not come from the settings of the e-mail.**
//! The road of the handover named `GET /api/emails/settings`, and a measurement
//! against an Audiobookshelf 2.36.0 on 2026-08-12 shows that this endpoint
//! cannot give the list to a user:
//!
//! | The request | The account `root` | The account `user` |
//! |---|---|---|
//! | `GET /api/emails/settings` | `200`, and every device | **`404`** |
//! | `POST /api/emails/send-ebook-to-device` | `200` | `200` |
//! | `GET /api/me` | no device at all | no device at all |
//!
//! Every endpoint of `/api/emails/` holds an `adminMiddleware` of the server.
//! **The user can send a book, and no such endpoint names the device that they
//! may use.**
//!
//! **`POST /api/authorize` is the answer.** The server makes one payload for
//! the login and for that endpoint, and it holds `ereaderDevices`: **the server
//! filters that list for the account itself**. One request, a bearer token, and
//! no permission of an administrator.
//!
//! The server holds four values of `availabilityOption` (`adminOrUp`,
//! `userOrUp`, `guestOrUp`, and `specificUsers`). **This program reads none of
//! them**: the server gave the list of that account already, and a rule of the
//! program would be a second authority that can disagree with the first one.
//!
//! **The send needs its own time limit.** The measurement of the same day, with
//! the books of the sandbox:
//!
//! | The book | The size | The time of the request |
//! |---|---|---|
//! | A Book That No Reader Reads | 0.1 MB | 0.007 s |
//! | A Big Book Of A Scan | 45.2 MB | 3.6 s |
//! | A Huge Book Of A Scan | **479.5 MB** | **36.2 s** |
//!
//! That is about 13 megabytes each second, and every part of the work stands on
//! the server. `REQUEST_TIMEOUT` of the client is **15 seconds**, therefore a
//! book of more than about 200 megabytes stopped at that limit **while the
//! server sent it**, and the user read a fault of a work that succeeded.
//! `MAX_BOOK_BYTES` of this program is 502 megabytes: the condition is not a
//! condition of an imagined book.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use std::time::Duration;

/// The time limit of one send.
///
/// The measurement above gives about 13 megabytes each second. Fifteen minutes
/// carry `MAX_BOOK_BYTES` (502 megabytes) at 0.56 megabytes each second, and
/// that is twenty-three times slower than the measurement.
/// `download_to_file` holds the same shape for the same reason: a request that
/// carries a file needs no limit of 15 seconds.
pub const THE_TIME_OF_A_SEND: Duration = Duration::from_secs(60 * 15);

/// One device of an e-reader, as the server names it.
///
/// The name is the value that `POST /api/emails/send-ebook-to-device` takes,
/// and the address is for the user: two devices of one user can hold the same
/// name of a make.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Device {
    pub name: String,
    #[serde(default)]
    pub email: String,
}

impl Device {
    /// The line of this device on the screen.
    pub fn line(&self) -> String {
        match self.email.trim() {
            "" => self.name.clone(),
            address => format!("{} - {}", self.name, address),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TheAnswerOfTheLogin {
    #[serde(default, rename = "ereaderDevices")]
    ereader_devices: Vec<Device>,
}

/// Gives the devices that this account may use.
///
/// **The server filters the list for the account of the token**, therefore this
/// program adds no rule of its own.
pub async fn the_devices_of_the_account(client: &ApiClient) -> Result<Vec<Device>, ApiError> {
    let answer: TheAnswerOfTheLogin = client
        .post_json("/api/authorize", &serde_json::json!({}))
        .await?;

    Ok(answer.ereader_devices)
}

/// What the server did with the book.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheEnd {
    /// The server sent the book.
    TheServerSentIt,
    /// The server refused the request, and this text says why.
    TheServerRefused(String),
}

/// Asks the server to send the ebook of one item to one device.
///
/// **The server sends the book of `media.ebookFile`**, and never a file that
/// the caller names: an item can hold more than one ebook (T-76), and the
/// endpoint takes the item only.
pub async fn send_the_ebook(
    client: &ApiClient,
    item_id: &str,
    device_name: &str,
) -> Result<TheEnd, ApiError> {
    let body = serde_json::json!({
        "libraryItemId": item_id,
        "deviceName": device_name,
    });

    let (status, words) = client
        .post_and_read_the_answer(
            "/api/emails/send-ebook-to-device",
            &body,
            THE_TIME_OF_A_SEND,
        )
        .await?;

    Ok(the_end_of_the_send(status, &words))
}

/// Reads the status and the body of the answer. The function is pure,
/// therefore a test needs no server.
///
/// **The status alone cannot make the sentence.** The server answers `404` for
/// three different conditions, and the body is the one place that tells them
/// apart:
///
/// | The condition | The status | The body |
/// |---|---|---|
/// | The server sent the book | `200` | `OK` |
/// | The server has no settings of the e-mail | `400` | `Failed to verify SMTP connection configuration` |
/// | The e-mail did not go | `400` | the words of nodemailer |
/// | No device holds that name | `404` | `Ereader device not found` |
/// | The server does not hold that item | `404` | `Library item not found` |
/// | The item holds no ebook | `404` | `Ebook file not found` |
/// | The account may not do it | `403` | `Forbidden` |
pub fn the_end_of_the_send(status: u16, words: &str) -> TheEnd {
    if (200..300).contains(&status) {
        return TheEnd::TheServerSentIt;
    }

    let words = words.trim();

    let reason = match (status, words) {
        (404, w) if w.eq_ignore_ascii_case("Ebook file not found") => {
            "This item holds no ebook. The server sends a book, and not an audio file.".to_string()
        }
        (404, w) if w.eq_ignore_ascii_case("Ereader device not found") => {
            "The server does not hold that device now. Press the key again for the new list."
                .to_string()
        }
        (404, w) if w.eq_ignore_ascii_case("Library item not found") => {
            "The server does not hold this item now.".to_string()
        }
        (403, _) => "This account may not send this book to that device.".to_string(),
        (400, w) if w.to_ascii_lowercase().contains("verify smtp") => {
            "The server cannot send an e-mail. An administrator of the server gives the \
             settings of the e-mail."
                .to_string()
        }
        (_, "") => format!("The server answered {}.", status),
        (_, w) => format!("The server answered {}: {}", status, the_short_words(w)),
    };

    TheEnd::TheServerRefused(reason)
}

/// Holds the words of the server at a length that one row can draw.
///
/// **The row of the message holds one line** (the trap 11 of the program), and
/// a message of 200 letters loses its end in a terminal of 160 columns.
fn the_short_words(words: &str) -> String {
    let words: String = words.split_whitespace().collect::<Vec<_>>().join(" ");

    if words.chars().count() <= 80 {
        return words;
    }

    let short: String = words.chars().take(77).collect();

    format!("{}...", short.trim_end())
}

/// The sentence that the user reads after the key that sends.
///
/// **The send stands on the server**, and the program cannot measure it: the
/// server reads the file, it makes the e-mail, and it gives the bytes to an
/// SMTP server. A book of 480 megabytes took 36 seconds of the sandbox, and a
/// server of the internet is slower.
pub fn the_sentence_of_the_send(title: &str, device: &str, end: &TheEnd) -> String {
    match end {
        TheEnd::TheServerSentIt => {
            format!("The server sent \"{}\" to {}.", title, device)
        }
        TheEnd::TheServerRefused(reason) => {
            format!("The server did not send \"{}\": {}", title, reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_of_two_hundred_is_the_book_that_went() {
        assert_eq!(the_end_of_the_send(200, "OK"), TheEnd::TheServerSentIt);
    }

    #[test]
    fn the_three_answers_of_four_hundred_and_four_say_three_things() {
        let no_book = the_end_of_the_send(404, "Ebook file not found");
        let no_device = the_end_of_the_send(404, "Ereader device not found");
        let no_item = the_end_of_the_send(404, "Library item not found");

        assert_ne!(no_book, no_device);
        assert_ne!(no_book, no_item);
        assert_ne!(no_device, no_item);

        let TheEnd::TheServerRefused(words) = no_book else {
            panic!("the server refused the request");
        };

        assert!(
            words.contains("holds no ebook"),
            "the sentence names the condition: {}",
            words
        );
    }

    #[test]
    fn the_settings_of_the_email_name_an_administrator() {
        let TheEnd::TheServerRefused(words) =
            the_end_of_the_send(400, "Failed to verify SMTP connection configuration")
        else {
            panic!("the server refused the request");
        };

        assert!(
            words.contains("administrator"),
            "the user learns who corrects it: {}",
            words
        );
    }

    #[test]
    fn a_body_that_no_rule_holds_keeps_the_words_of_the_server() {
        let TheEnd::TheServerRefused(words) = the_end_of_the_send(400, "Message failed: 550 no")
        else {
            panic!("the server refused the request");
        };

        assert!(
            words.contains("550"),
            "the words of the server stay: {}",
            words
        );
    }

    #[test]
    fn the_words_of_the_server_hold_one_row() {
        let long = "a".repeat(300);
        let TheEnd::TheServerRefused(words) = the_end_of_the_send(400, &long) else {
            panic!("the server refused the request");
        };

        assert!(
            words.chars().count() <= 150,
            "the row of the message holds one line: {} letters",
            words.chars().count()
        );
    }

    #[test]
    fn the_line_of_a_device_holds_the_name_and_the_address() {
        let device = Device {
            name: "The Kindle".to_string(),
            email: "k@example.invalid".to_string(),
        };

        assert_eq!(device.line(), "The Kindle - k@example.invalid");

        let no_address = Device {
            name: "The Kindle".to_string(),
            email: String::new(),
        };

        assert_eq!(no_address.line(), "The Kindle");
    }

    #[test]
    fn the_devices_come_from_the_payload_of_the_login() {
        let answer: TheAnswerOfTheLogin = serde_json::from_value(serde_json::json!({
            "user": { "username": "toutuitest" },
            "ereaderDevices": [
                { "name": "Kobo", "email": "kobo@example.invalid",
                  "availabilityOption": "adminOrUp", "users": [] }
            ]
        }))
        .expect("the payload of the login");

        assert_eq!(answer.ereader_devices.len(), 1);
        assert_eq!(answer.ereader_devices[0].name, "Kobo");
    }

    #[test]
    fn a_payload_with_no_device_gives_an_empty_list() {
        let answer: TheAnswerOfTheLogin =
            serde_json::from_value(serde_json::json!({ "user": {} })).expect("the payload");

        assert!(answer.ereader_devices.is_empty());
    }

    #[test]
    fn the_sentence_names_the_book_and_the_device() {
        let words = the_sentence_of_the_send("Alice", "Kobo", &TheEnd::TheServerSentIt);

        assert!(words.contains("Alice"), "{}", words);
        assert!(words.contains("Kobo"), "{}", words);
    }
}
