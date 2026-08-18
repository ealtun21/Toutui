//! What a filter of a library can hold. See T-24.
//!
//! `GET /api/libraries/:id/filterdata` gives the values, and
//! `?filter=<type>.<base64>` uses one of them. A measurement against an
//! Audiobookshelf 2.36.0 on 2026-08-11 gives, for the library of books of the
//! sandbox: 4 authors, 2 series, 0 genres, 0 tags, 0 narrators, 0 languages,
//! and 0 publishers.
//!
//! An author and a series come with an identity and a name, and the filter
//! takes the identity. A genre, a tag, a narrator, a language, and a publisher
//! come as a text, and the filter takes that text.
//!
//! **The answer of that endpoint holds no tag.** A measurement on 2026-08-11 gave
//! a media with the tag `a-test-tag`: `GET /api/items/:id` holds that tag,
//! `GET /api/tags` gives `{"tags":["a-test-tag"]}`, and `filterdata` gives
//! `tags: []` — **after a scan of the library as well**. A filter of
//! `tags.<base64>` works: it gave the one media of that tag.
//!
//! Therefore the program asks `GET /api/tags` for the tags, and it puts them in
//! the same list. That endpoint holds the tags of the **whole server**, and the
//! filter of a library then gives no media for a tag of a different library. A
//! group of an empty answer says so. See T-60.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use crate::logic::sort_filter::{filter_value, FilterChoice};
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterData {
    #[serde(default)]
    pub authors: Vec<Named>,
    #[serde(default)]
    pub series: Vec<Named>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub narrators: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub publishers: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Named {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// Asks the server what a filter of this library can hold.
pub async fn get_filter_data(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<FilterData, ApiError> {
    client
        .get_json(&format!("/api/libraries/{}/filterdata", id_selected_lib))
        .await
}

/// The answer of `GET /api/tags`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Tags {
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Asks the server for the tags. See T-60.
///
/// `filterdata` holds no tag, therefore the program asks this endpoint. A server
/// that gives an error gives no tag, and the view of the filter then holds every
/// other group: a tag is one group of eight.
pub async fn get_the_tags(client: &ApiClient) -> Vec<String> {
    match client.get_json::<Tags>("/api/tags").await {
        Ok(answer) => answer.tags,
        Err(error) => {
            log::warn!("[filter] the server gave no tag: {}", error);
            Vec::new()
        }
    }
}

/// Puts the tags of the server in the data of the filter.
///
/// The function keeps a tag that `filterdata` gave already, and it adds no tag
/// two times. The sequence of the answer of the server is the sequence of the
/// list.
pub fn with_the_tags(mut data: FilterData, tags: Vec<String>) -> FilterData {
    for tag in tags {
        let tag = tag.trim().to_string();

        if tag.is_empty() || data.tags.iter().any(|one| one == &tag) {
            continue;
        }

        data.tags.push(tag);
    }

    data
}

/// The largest number of values of one group.
///
/// A library of 2056 items can hold many authors. The screen shows a list, and
/// a list of a thousand names is not a list that a person reads. The user
/// looks for a book of that author with the key `/` instead.
pub const LIMIT: usize = 100;

/// Makes the choices of the filter for the screen.
///
/// A group with no value gives no choice. The screen then shows the groups
/// that the library holds, and no empty title.
pub fn choices(data: &FilterData) -> Vec<FilterChoice> {
    let mut out: Vec<FilterChoice> = Vec::new();

    // An author and a series carry an identity, and the filter takes it. The
    // name of an author can change, and the identity does not.
    for (group, kind, values) in [
        ("The authors", "authors", &data.authors),
        ("The series", "series", &data.series),
    ] {
        for one in values.iter().take(LIMIT) {
            let (Some(id), Some(name)) = (one.id.as_ref(), one.name.as_ref()) else {
                continue;
            };

            // A value of no character gives a filter of no value, which the
            // program does not apply (T-386): such a row is a row that does
            // nothing. A name of no character gives a row of no words. Each
            // of them takes a line of the log and no choice. See T-387.
            if id.trim().is_empty() || name.trim().is_empty() {
                log::warn!(
                    "[filter] a row of {} of the server has no identity or no \
                     name: id {:?}, name {:?}. It takes no choice.",
                    kind,
                    id,
                    name
                );
                continue;
            }

            out.push(FilterChoice {
                label: name.clone(),
                group,
                value: filter_value(kind, id),
            });
        }
    }

    // A genre, a tag, a narrator, a language, and a publisher are a text, and
    // the filter takes that text.
    for (group, kind, values) in [
        ("The genres", "genres", &data.genres),
        ("The tags", "tags", &data.tags),
        ("The narrators", "narrators", &data.narrators),
        ("The languages", "languages", &data.languages),
        ("The publishers", "publishers", &data.publishers),
    ] {
        for one in values.iter().take(LIMIT) {
            // A text of no character gives a blank row whose filter has no
            // value (T-386): it takes a line of the log and no choice. The
            // text of a real choice stays as the server gave it, because the
            // filter of the server compares that text. See T-387.
            if one.trim().is_empty() {
                log::warn!(
                    "[filter] a text of {} of the server has no character. \
                     It takes no choice.",
                    kind
                );
                continue;
            }

            out.push(FilterChoice {
                label: one.clone(),
                group,
                value: filter_value(kind, one),
            });
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `filterdata` holds no tag, therefore the program asks `GET /api/tags` and
    /// it puts those tags in the same list. See T-60.
    #[test]
    fn the_tags_of_the_server_come_in_the_list() {
        let data = FilterData::default();

        let with = with_the_tags(
            data,
            vec!["a-test-tag".to_string(), "a second tag".to_string()],
        );

        assert_eq!(with.tags, vec!["a-test-tag", "a second tag"]);

        // The sequence of the answer of the server is the sequence of the list,
        // and the choices of the view hold every tag.
        let choices = choices(&with);
        let of_the_tags: Vec<&FilterChoice> = choices
            .iter()
            .filter(|one| one.group == "The tags")
            .collect();

        assert_eq!(of_the_tags.len(), 2);
        assert_eq!(of_the_tags[0].label, "a-test-tag");
        // The filter of the server takes the text in the form of base64.
        assert_eq!(of_the_tags[0].value, filter_value("tags", "a-test-tag"));
    }

    /// A tag that `filterdata` gave already must not stand two times, and a tag
    /// of no letters is no tag.
    #[test]
    fn a_tag_does_not_stand_two_times() {
        let data = FilterData {
            tags: vec!["a-test-tag".to_string()],
            ..Default::default()
        };

        let with = with_the_tags(
            data,
            vec![
                "a-test-tag".to_string(),
                "  ".to_string(),
                "  a-test-tag  ".to_string(),
                "a new tag".to_string(),
            ],
        );

        assert_eq!(with.tags, vec!["a-test-tag", "a new tag"]);
    }

    /// The answer of the sandbox, measured on 2026-08-11.
    fn the_answer_of_the_server() -> FilterData {
        serde_json::from_value(serde_json::json!({
            "authors": [
                { "id": "312c42ff", "name": "Lewis Carroll" },
                { "id": "cc5891d3", "name": "Long Author" }
            ],
            "genres": [],
            "tags": [],
            "series": [ { "id": "e23c87a9", "name": "Second Series" } ],
            "narrators": [],
            "languages": [],
            "publishers": [],
            "publishedDecades": [],
            "bookCount": 9
        }))
        .expect("the answer of the server must read")
    }

    #[test]
    fn the_answer_of_a_real_server_reads() {
        let data = the_answer_of_the_server();

        assert_eq!(data.authors.len(), 2);
        assert_eq!(data.series.len(), 1);
        assert!(data.genres.is_empty());
    }

    #[test]
    fn every_value_gives_a_choice_of_the_filter() {
        let choices = choices(&the_answer_of_the_server());

        assert_eq!(choices.len(), 3);
        assert_eq!(choices[0].label, "Lewis Carroll");
        assert_eq!(choices[0].group, "The authors");
        assert_eq!(choices[0].value, "authors.MzEyYzQyZmY=");
        assert_eq!(choices[2].group, "The series");
        assert_eq!(choices[2].value, "series.ZTIzYzg3YTk=");
    }

    #[test]
    fn a_text_of_a_group_gives_its_own_value() {
        let data: FilterData = serde_json::from_value(serde_json::json!({
            "genres": ["Science Fiction"],
            "tags": ["one"]
        }))
        .expect("the answer must read");

        let choices = choices(&data);
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].group, "The genres");
        assert_eq!(choices[0].value, "genres.U2NpZW5jZSBGaWN0aW9u");
        assert_eq!(choices[1].value, "tags.b25l");
    }

    #[test]
    fn an_answer_with_no_value_gives_no_choice() {
        let data: FilterData =
            serde_json::from_value(serde_json::json!({})).expect("an answer must read");

        assert!(choices(&data).is_empty());
    }

    #[test]
    fn an_author_with_no_identity_gives_no_choice() {
        let data: FilterData = serde_json::from_value(serde_json::json!({
            "authors": [ { "name": "A Name With No Identity" }, { "id": "a" } ]
        }))
        .expect("an answer must read");

        assert!(choices(&data).is_empty());
    }

    /// An identity of no character gives a filter of no value, which the
    /// program does not apply (T-386): the row then does nothing at all. The
    /// measurement of T-387 took such a row, the disk held `authors.`, and
    /// the refresh discarded it with no word.
    #[test]
    fn an_author_of_an_empty_identity_gives_no_choice() {
        let data: FilterData = serde_json::from_value(serde_json::json!({
            "authors": [
                { "id": "", "name": "A Ghost Author" },
                { "id": "a", "name": "A Real Author" }
            ],
            "series": [ { "id": "  ", "name": "A Ghost Series" } ]
        }))
        .expect("an answer must read");

        let choices = choices(&data);
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].label, "A Real Author");
    }

    /// A name of no character gives a row of no words: the user cannot read
    /// what the filter takes. See T-387.
    #[test]
    fn an_author_of_an_empty_name_gives_no_choice() {
        let data: FilterData = serde_json::from_value(serde_json::json!({
            "authors": [ { "id": "a", "name": "" } ]
        }))
        .expect("an answer must read");

        assert!(choices(&data).is_empty());
    }

    /// A text of no character gives a blank row whose filter has no value.
    /// The measurement of T-387 took such a genre, the disk held `genres.`,
    /// and the refresh discarded it with no word. A text with characters
    /// stays as the server gave it, because the filter of the server
    /// compares that text.
    #[test]
    fn a_text_of_no_character_gives_no_choice() {
        let data: FilterData = serde_json::from_value(serde_json::json!({
            "genres": ["", "  ", "Adventure"],
            "narrators": [""],
            "languages": ["  "],
            "publishers": [""]
        }))
        .expect("an answer must read");

        let choices = choices(&data);
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].label, "Adventure");
        assert_eq!(choices[0].value, filter_value("genres", "Adventure"));
    }

    /// A library of many authors must not give a list that no person reads.
    #[test]
    fn a_very_long_group_stops_at_the_limit() {
        let authors: Vec<serde_json::Value> = (0..LIMIT + 50)
            .map(|number| serde_json::json!({ "id": number.to_string(), "name": "An Author" }))
            .collect();

        let data: FilterData = serde_json::from_value(serde_json::json!({ "authors": authors }))
            .expect("an answer must read");

        assert_eq!(choices(&data).len(), LIMIT);
    }
}
