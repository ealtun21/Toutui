//! The audio files of the answer of `GET /api/items/:id`.
//!
//! **A field that the server did not give is not a value.** T-179 read that rule
//! of `metadata.size`, T-180 read it of `duration`, and this module holds it for
//! the two fields that name a file and that put it in its place: `ino` and
//! `index`. See T-181.
//!
//! The download and the playback both read `media.audioFiles`, therefore the two
//! roads take the same numbers of the same answer.
//!
//! The module is pure, therefore a test examines it with no server.

/// Gives the identity of a file of the server, of the field `ino`.
///
/// **A file with no identity has no address**: every request of a file of the
/// server stands on `/api/items/:id/file/:ino`. Therefore the function gives
/// `None`, and the caller says so. A value of no character is the same as no
/// field at all.
///
/// Audiobookshelf writes the inode of the file system in this field. The number
/// of a file system of another form does not fit a string of JSON in every
/// server, therefore the function reads a number too.
pub fn the_identity_of_a_file(file: &serde_json::Value) -> Option<String> {
    let value = file.get("ino")?;

    let identity = match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        _ => return None,
    };

    if identity.is_empty() {
        return None;
    }

    Some(identity)
}

/// Gives the number of each audio file of the answer, in the sequence of that
/// answer.
///
/// **The field `index` puts the files of a book in their sequence** (T-2), and a
/// file that holds no such field took the number 1 of `unwrap_or(1)` before
/// T-181. Two files of one book then held the number 1: the sort put the last
/// file of the book in the middle of it, two files took the same name on the
/// disk, and the row of the second file replaced the row of the first one in the
/// table `download_files`. **The user lost a part of the book, and the program
/// said nothing.**
///
/// The rule: a book whose files each hold an `index` keeps that sequence, and a
/// book of one file or more with no `index` takes the sequence of the answer.
/// The server gave that sequence, and no other information about the sequence
/// exists.
pub fn the_numbers_of_the_files(files: &[serde_json::Value]) -> Vec<u32> {
    let of_the_server: Option<Vec<u32>> = files
        .iter()
        .map(|file| {
            file.get("index")
                .and_then(|value| value.as_u64())
                .map(|value| value as u32)
        })
        .collect();

    match of_the_server {
        Some(numbers) => numbers,
        None => (1..=files.len() as u32).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_identity_comes_of_a_string_or_of_a_number() {
        assert_eq!(
            the_identity_of_a_file(&json!({ "ino": "30853118" })).as_deref(),
            Some("30853118")
        );
        assert_eq!(
            the_identity_of_a_file(&json!({ "ino": 30853118 })).as_deref(),
            Some("30853118")
        );
    }

    #[test]
    fn a_file_of_a_server_that_gave_no_ino_holds_no_identity() {
        assert_eq!(the_identity_of_a_file(&json!({})), None);
        assert_eq!(the_identity_of_a_file(&json!({ "ino": "" })), None);
        assert_eq!(the_identity_of_a_file(&json!({ "ino": null })), None);
    }

    #[test]
    fn the_numbers_of_a_book_of_the_server_stay() {
        let files = vec![json!({ "index": 3 }), json!({ "index": 1 })];

        assert_eq!(the_numbers_of_the_files(&files), vec![3, 1]);
    }

    /// **The measurement of T-181.** The server gave no `index` of the last file
    /// of a book of three files. The old rule gave that file the number 1, and
    /// the first file of the book held the number 1 too.
    #[test]
    fn a_file_with_no_index_gives_every_file_the_number_of_the_answer() {
        let files = vec![
            json!({ "index": 1 }),
            json!({ "index": 2 }),
            json!({ "ino": "3" }),
        ];

        assert_eq!(the_numbers_of_the_files(&files), vec![1, 2, 3]);
    }

    #[test]
    fn a_book_with_no_index_at_all_takes_the_sequence_of_the_answer() {
        let files = vec![json!({}), json!({}), json!({})];

        assert_eq!(the_numbers_of_the_files(&files), vec![1, 2, 3]);
    }

    #[test]
    fn a_list_of_no_file_gives_no_number() {
        assert!(the_numbers_of_the_files(&[]).is_empty());
    }
}
