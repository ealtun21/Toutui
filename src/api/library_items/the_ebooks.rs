//! The ebooks of one item. See T-76.
//!
//! **An item can hold more than one ebook.** `media.ebookFile` names one of
//! them, and the key `e` opens that one. `GET /api/items/:id` gives every file
//! of the item in `libraryFiles`, and a file of the type `ebook` is a book that
//! the user can read.
//!
//! A measurement of 2026-08-11 against the sandbox put an EPUB book beside the
//! PDF book of one item:
//!
//! ```text
//! ebookFile: A Book Of The Test.pdf
//!  file: 01 - Part 1.mp3        audio  ino 33169532
//!  file: 02 - Part 2.wma        audio  ino 33161684
//!  file: A Book Of The Test.pdf ebook  ino 6121534
//!  file: A Second Book.epub     ebook  ino 94488
//! ```
//!
//! The same measurement gave the file of each ebook:
//!
//! ```text
//! GET /api/items/:id/ebook           200  53688 bytes  application/pdf
//! GET /api/items/:id/ebook/6121534   200  53688 bytes  application/pdf
//! GET /api/items/:id/ebook/94488     200 136761 bytes  application/epub+zip
//! ```
//!
//! Therefore the address of one ebook is the address of the ebook of the item
//! with the `ino` of the file after it.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::Value;

/// One ebook file of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ebook {
    /// The identity of the file inside the item. The address of the file needs
    /// it.
    pub ino: String,
    /// The name of the file, for the user.
    pub name: String,
    /// The number of the bytes of the file, of `metadata.size`.
    ///
    /// **A size of 0 is a size that the server did not give** (T-179). The
    /// download of the ebook then counts nothing, because it holds no truth of
    /// the length. See T-196.
    pub size: u64,
    /// The server opens this ebook for the address that carries no `ino`. It is
    /// the book of `media.ebookFile`.
    pub is_the_book_of_the_server: bool,
}

impl Ebook {
    /// The line of this ebook, in the list of the screen.
    pub fn line(&self) -> String {
        if self.is_the_book_of_the_server {
            format!("{} (the book of the server)", self.name)
        } else {
            self.name.clone()
        }
    }
}

/// Gives every ebook of the answer of `GET /api/items/:id`.
///
/// The function is pure, therefore a test needs no server. The sequence of the
/// list is the sequence of the server, and the book of the server stands first:
/// the key `e` opens that book, and the user finds it at the top.
pub fn the_ebooks_of_the_answer(answer: &Value) -> Vec<Ebook> {
    let of_the_server = answer
        .pointer("/media/ebookFile/ino")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let mut all: Vec<Ebook> = answer
        .get("libraryFiles")
        .and_then(Value::as_array)
        .map(|files| {
            files
                .iter()
                .filter(|file| file.get("fileType").and_then(Value::as_str) == Some("ebook"))
                .filter_map(|file| {
                    let ino = file.get("ino").and_then(Value::as_str)?.to_string();

                    let name = file
                        .pointer("/metadata/filename")
                        .and_then(Value::as_str)
                        .unwrap_or(ino.as_str())
                        .to_string();

                    // The one truth of the length of a body that names none.
                    // See T-196.
                    let size = file
                        .pointer("/metadata/size")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);

                    Some(Ebook {
                        is_the_book_of_the_server: ino == of_the_server,
                        ino,
                        name,
                        size,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    all.sort_by_key(|one| !one.is_the_book_of_the_server);

    all
}

/// Asks the server for every ebook of one item.
pub async fn the_ebooks_of_the_item(
    client: &ApiClient,
    item_id: &str,
) -> Result<Vec<Ebook>, ApiError> {
    let answer: Value = client.get_json(&format!("/api/items/{}", item_id)).await?;

    Ok(the_ebooks_of_the_answer(&answer))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the measurement of 2026-08-11, with the fields that this
    /// module reads.
    fn an_item_of_two_ebooks() -> Value {
        serde_json::json!({
            "media": { "ebookFile": { "ino": "6121534" } },
            "libraryFiles": [
                { "ino": "33169532", "fileType": "audio",
                  "metadata": { "filename": "01 - Part 1.mp3" } },
                { "ino": "6121534", "fileType": "ebook",
                  "metadata": { "filename": "A Book Of The Test.pdf" } },
                { "ino": "94488", "fileType": "ebook",
                  "metadata": { "filename": "A Second Book.epub" } }
            ]
        })
    }

    #[test]
    fn the_list_holds_the_ebooks_and_no_audio_file() {
        let all = the_ebooks_of_the_answer(&an_item_of_two_ebooks());

        assert_eq!(all.len(), 2, "an audio file is not an ebook");
        assert_eq!(all[0].name, "A Book Of The Test.pdf");
        assert_eq!(all[1].name, "A Second Book.epub");
    }

    #[test]
    fn the_book_of_the_server_stands_first() {
        let answer = serde_json::json!({
            "media": { "ebookFile": { "ino": "94488" } },
            "libraryFiles": [
                { "ino": "6121534", "fileType": "ebook",
                  "metadata": { "filename": "A Book Of The Test.pdf" } },
                { "ino": "94488", "fileType": "ebook",
                  "metadata": { "filename": "A Second Book.epub" } }
            ]
        });

        let all = the_ebooks_of_the_answer(&answer);

        assert_eq!(all[0].name, "A Second Book.epub");
        assert!(all[0].is_the_book_of_the_server);
        assert!(!all[1].is_the_book_of_the_server);
    }

    #[test]
    fn the_line_names_the_book_of_the_server() {
        let all = the_ebooks_of_the_answer(&an_item_of_two_ebooks());

        assert_eq!(
            all[0].line(),
            "A Book Of The Test.pdf (the book of the server)"
        );
        assert_eq!(all[1].line(), "A Second Book.epub");
    }

    #[test]
    fn an_item_with_no_ebook_gives_no_line() {
        let answer = serde_json::json!({
            "media": {},
            "libraryFiles": [
                { "ino": "1", "fileType": "audio",
                  "metadata": { "filename": "01.mp3" } }
            ]
        });

        assert!(the_ebooks_of_the_answer(&answer).is_empty());
    }

    /// A server that gives no `libraryFiles` must not stop the program.
    #[test]
    fn an_answer_with_no_field_gives_no_line() {
        assert!(the_ebooks_of_the_answer(&serde_json::json!({})).is_empty());
    }

    /// A file with no name of the file takes its identity as the name. The user
    /// then reads a line that is not empty.
    #[test]
    fn a_file_with_no_name_takes_its_identity() {
        let answer = serde_json::json!({
            "libraryFiles": [ { "ino": "94488", "fileType": "ebook" } ]
        });

        assert_eq!(the_ebooks_of_the_answer(&answer)[0].name, "94488");
    }
}
