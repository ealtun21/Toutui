//! The authors of a library. See T-24.
//!
//! `GET /api/libraries/:id/authors` gives every author with the number of
//! their books. A measurement against an Audiobookshelf 2.36.0 on 2026-08-11
//! gives four authors for the library of books of the sandbox:
//!
//! ```json
//! { "authors": [ { "id": "f49b0437", "name": "Test Author",
//!                  "description": null, "numBooks": 1,
//!                  "lastFirst": "Author, Test" } ] }
//! ```
//!
//! `GET /api/authors/:id` gives one author, and it gives no `numBooks`.
//! Therefore the list is the answer that the view needs, and the view sends
//! one request.
//!
//! The key `l` on an author gives the filter of that author. The program holds
//! that work already, in `crate::logic::sort_filter`.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
struct Answer {
    #[serde(default)]
    authors: Vec<Author>,
}

/// The answer of the narrators. See T-73.
#[derive(Debug, Clone, Default, Deserialize)]
struct AnswerOfTheNarrators {
    #[serde(default)]
    narrators: Vec<Author>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub num_books: i64,
}

/// Asks the server for the authors of a library.
///
/// The answer comes in the sequence of the server. This function puts the
/// authors in the sequence of the alphabet, because a person reads a list of
/// names in that sequence.
pub async fn get_authors(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Author>, ApiError> {
    let answer: Answer = client
        .get_json(&format!("/api/libraries/{}/authors", id_selected_lib))
        .await?;

    let mut all = answer.authors;

    // **An author with no identity belongs to no line** (T-386, and the rule
    // of T-183 and of T-192): the one key of the line is the filter of that
    // identity, and a filter of an identity of no character asks the server
    // for nothing. The log holds the one word of that fault, because the
    // view of a list that lost a line has no view of it (T-177).
    all.retain(|one| {
        if one.id.is_empty() {
            log::warn!(
                "[authors] The answer of the server holds the author \"{}\" with no \
                 identity. The program cannot ask the server for the books of that \
                 author, therefore the line goes away.",
                one.name
            );
            return false;
        }
        true
    });

    all.sort_by_key(|one| one.name.to_lowercase());

    Ok(all)
}

/// Asks the server for the narrators of a library. See T-73.
///
/// A narrator of the server holds the shape of an author: an identity, a name,
/// and a number of books. A measurement against an Audiobookshelf 2.36.0 on
/// 2026-08-11:
///
/// ```json
/// { "narrators": [ { "id": "QSBUZXN0IE5hcnJhdG9y",
///                    "name": "A Test Narrator", "numBooks": 2 } ] }
/// ```
///
/// **A narrator holds no row of its own on the server.** The identity is the name
/// in base64, and the server holds the narrator inside the metadata of a file.
/// Therefore the answer gives no description, and the filter of the library takes
/// the name. See `logic::authors::Kind::filter_of`.
pub async fn get_narrators(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Author>, ApiError> {
    let answer: AnswerOfTheNarrators = client
        .get_json(&format!("/api/libraries/{}/narrators", id_selected_lib))
        .await?;

    let mut all = answer.narrators;

    // **A narrator with no name belongs to no line** (T-386): a narrator
    // holds no row of its own on the server, and the filter of a narrator
    // takes the name. A name of no character asks the server for nothing.
    all.retain(|one| {
        if one.name.is_empty() {
            log::warn!(
                "[authors] The answer of the server holds a narrator with no name. The \
                 program cannot ask the server for the books of that narrator, therefore \
                 the line goes away."
            );
            return false;
        }
        true
    });

    all.sort_by_key(|one| one.name.to_lowercase());

    Ok(all)
}

/// Makes the text of each line of the list of the authors.
pub fn lines(all: &[Author]) -> Vec<String> {
    all.iter()
        .map(|one| {
            let name = if one.name.is_empty() {
                "An author with no name"
            } else {
                &one.name
            };

            format!("{} [{} book(s)]", name, one.num_books)
        })
        .collect()
}

/// Gives the description of an author for the screen.
pub fn description_of(author: &Author) -> String {
    match author.description.as_deref() {
        Some(text) if !text.trim().is_empty() => crate::utils::html_text::to_plain_text(text),
        _ => "No description available".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the sandbox, measured on 2026-08-11.
    fn the_answer_of_the_server() -> Answer {
        serde_json::from_value(serde_json::json!({
            "authors": [
                { "id": "f49b0437", "asin": null, "name": "Test Author",
                  "description": null, "imagePath": null, "numBooks": 1,
                  "lastFirst": "Author, Test" },
                { "id": "312c42ff", "name": "Lewis Carroll", "numBooks": 1,
                  "description": "<p>An author of England.</p>" },
                { "id": "cc5891d3", "name": "long author", "numBooks": 2 }
            ]
        }))
        .expect("the answer of the server must read")
    }

    #[test]
    fn the_answer_of_a_real_server_reads() {
        let all = the_answer_of_the_server().authors;

        assert_eq!(all.len(), 3);
        assert_eq!(all[0].name, "Test Author");
        assert_eq!(all[0].num_books, 1);
        assert_eq!(all[0].description, None);
    }

    #[test]
    fn an_answer_with_no_author_gives_no_fault() {
        let answer: Answer =
            serde_json::from_value(serde_json::json!({})).expect("the answer must read");

        assert!(answer.authors.is_empty());
        assert!(lines(&answer.authors).is_empty());
    }

    #[test]
    fn every_author_gives_one_line() {
        let text = lines(&the_answer_of_the_server().authors);

        assert_eq!(text.len(), 3);
        assert_eq!(text[0], "Test Author [1 book(s)]");
        assert_eq!(text[2], "long author [2 book(s)]");
    }

    #[test]
    fn an_author_with_no_name_gives_a_line() {
        let all = vec![Author {
            id: "a".to_string(),
            name: String::new(),
            description: None,
            num_books: 0,
        }];

        assert_eq!(lines(&all), vec!["An author with no name [0 book(s)]"]);
    }

    /// The description holds HTML. The screen shows no tag.
    #[test]
    fn the_description_holds_no_tag() {
        let all = the_answer_of_the_server().authors;

        assert_eq!(description_of(&all[1]), "An author of England.");
        assert_eq!(description_of(&all[0]), "No description available");
    }

    #[test]
    fn a_description_of_spaces_only_gives_the_sentence() {
        let author = Author {
            id: "a".to_string(),
            name: "A Name".to_string(),
            description: Some("   ".to_string()),
            num_books: 0,
        };

        assert_eq!(description_of(&author), "No description available");
    }
}
