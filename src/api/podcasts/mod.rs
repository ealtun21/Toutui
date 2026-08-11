//! Add a podcast to a library. See T-24.
//!
//! The README names this function. Three requests do the work, and every one
//! of them is measured against an Audiobookshelf 2.36.0 on 2026-08-11:
//!
//! | Request | Answer |
//! |---|---|
//! | `GET /api/search/podcast?term=balzac` | a list of 48, with `title`, `artistName`, `description`, `feedUrl`, `trackCount`, and `cover`. **`limit` changes nothing** |
//! | `POST /api/podcasts/feed` with `{"rssFeed":"..."}` | `200`, and the key `podcast` with `metadata` and `episodes` |
//! | `POST /api/podcasts` with the body of `body_for` | `200`, and the new item of the library |
//!
//! **The server asks iTunes for the search.** Therefore the search needs the
//! network of the server, and not the network of the user.
//!
//! **The client gives the path of the directory.** The server makes that
//! directory in the folder of the library. A title comes from the network, and
//! a title can hold `/` and `..`; `directory_of` removes them. See the tests.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;

/// The largest number of answers of the search.
///
/// The server does not take a limit: a measurement with `limit=3` gave 48
/// answers. Therefore the program cuts the list itself.
pub const LIMIT: usize = 50;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Found {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist_name: String,
    #[serde(default)]
    pub description_plain: String,
    #[serde(default)]
    pub feed_url: String,
    #[serde(default)]
    pub track_count: i64,
}

/// Asks the server for the podcasts that agree with the words of the user.
pub async fn search_podcast(client: &ApiClient, words: &str) -> Result<Vec<Found>, ApiError> {
    // A space and every other character must not break the path.
    let query = crate::api::libraries::search_library::encode_the_query(words);

    let mut all: Vec<Found> = client
        .get_json(&format!("/api/search/podcast?term={}", query))
        .await?;

    all.truncate(LIMIT);
    Ok(all)
}

/// Makes the text of each line of the list of the answers.
pub fn lines(all: &[Found]) -> Vec<String> {
    all.iter()
        .map(|one| {
            format!(
                "{} — {} [{} episode(s)]",
                one.title, one.artist_name, one.track_count
            )
        })
        .collect()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FeedAnswer {
    #[serde(default)]
    pub podcast: Feed,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Feed {
    #[serde(default)]
    pub metadata: FeedMetadata,
    #[serde(default)]
    pub episodes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedMetadata {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub feed_url: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    /// The server writes this value as a text, and not as a yes or a no.
    #[serde(default)]
    pub explicit: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Reads a feed. The server gets the file, and not the program.
pub async fn get_feed(client: &ApiClient, feed_url: &str) -> Result<Feed, ApiError> {
    let body = serde_json::json!({ "rssFeed": feed_url });

    let answer: FeedAnswer = client.post_json("/api/podcasts/feed", &body).await?;

    Ok(answer.podcast)
}

/// The largest length of the name of a directory, in characters.
///
/// A file system gives an error above 255 bytes. A character outside ASCII
/// takes more than one byte, therefore this number is smaller.
const NAME_LIMIT: usize = 60;

/// Makes the name of the directory of a podcast.
///
/// **The title comes from the network.** A title can hold `/`, and a title can
/// be `..`; the server would then write outside the folder of the library.
/// This function keeps a letter, a number, a space, a dash, and an underscore,
/// and it removes every other character.
///
/// A name that becomes empty gives `A podcast`.
pub fn directory_of(title: &str) -> String {
    let mut name = String::with_capacity(title.len());
    let mut space = false;

    for one in title.chars() {
        // A letter of a different writing is a letter, therefore the name of
        // a podcast in Greek or in Japanese keeps its name.
        let keep = one.is_alphanumeric() || matches!(one, ' ' | '-' | '_');

        if !keep {
            space = true;
            continue;
        }

        if one == ' ' || space {
            // Two spaces together give one space.
            if !name.ends_with(' ') && !name.is_empty() {
                name.push(' ');
            }

            space = false;

            if one == ' ' {
                continue;
            }
        }

        name.push(one);
    }

    let name: String = name.trim().chars().take(NAME_LIMIT).collect();
    let name = name.trim().to_string();

    // A full stop cannot start the name: a name of full stops only would give
    // `.` or `..`, and those names name a directory that exists.
    if name.is_empty() {
        return "A podcast".to_string();
    }

    name
}

/// Makes the body of `POST /api/podcasts`.
///
/// `folder` is the path of the folder of the library, for example
/// `/podcasts`. The server makes the directory of the podcast inside it.
pub fn body_for(
    feed: &Feed,
    library_id: &str,
    folder_id: &str,
    folder_path: &str,
) -> serde_json::Value {
    let metadata = &feed.metadata;
    let title = metadata
        .title
        .clone()
        .unwrap_or_else(|| "A podcast".to_string());

    serde_json::json!({
        "path": format!("{}/{}", folder_path.trim_end_matches('/'), directory_of(&title)),
        "folderId": folder_id,
        "libraryId": library_id,
        "media": {
            "metadata": {
                "title": title,
                "author": metadata.author,
                "description": metadata.description,
                "releaseDate": "",
                "genres": metadata.categories,
                "feedUrl": metadata.feed_url,
                "imageUrl": metadata.image,
                "itunesPageUrl": metadata.link,
                "language": metadata.language,
                "explicit": metadata.explicit.as_deref() == Some("true"),
            },
            // The server gets an episode when the program asks for it. A new
            // podcast must not start a download of every episode by itself.
            "autoDownloadEpisodes": false,
        },
    })
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Made {
    #[serde(default)]
    pub id: String,
}

/// Writes a new podcast in the library of the server.
pub async fn create_podcast(
    client: &ApiClient,
    body: &serde_json::Value,
) -> Result<Made, ApiError> {
    client.post_json("/api/podcasts", body).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first answer of `term=balzac`, measured on 2026-08-11.
    fn the_answer_of_the_search() -> Vec<Found> {
        serde_json::from_value(serde_json::json!([
            {
                "id": 1795341128i64,
                "title": "Another Study of Woman by Honoré de Balzac (1799 - 1850)",
                "artistName": "ciesse",
                "description": "",
                "descriptionPlain": "",
                "genres": ["Fiction", "Podcasts"],
                "trackCount": 3,
                "feedUrl": "https://www.spreaker.com/show/6199792/episodes/feed",
                "explicit": false
            },
            { "title": "A Second Podcast", "artistName": "A Name", "trackCount": 12,
              "feedUrl": "https://example.test/feed" }
        ]))
        .expect("the answer of the server must read")
    }

    #[test]
    fn the_answer_of_a_real_server_reads() {
        let all = the_answer_of_the_search();

        assert_eq!(all.len(), 2);
        assert_eq!(all[0].artist_name, "ciesse");
        assert_eq!(all[0].track_count, 3);
        assert!(all[0].feed_url.starts_with("https://"));
    }

    #[test]
    fn an_answer_with_no_field_gives_no_fault() {
        let all: Vec<Found> =
            serde_json::from_value(serde_json::json!([{}])).expect("the answer must read");

        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "");
        assert_eq!(all[0].track_count, 0);
    }

    #[test]
    fn every_answer_gives_one_line() {
        let text = lines(&the_answer_of_the_search());

        assert_eq!(text.len(), 2);
        assert!(text[0].contains("Another Study of Woman"));
        assert!(text[0].contains("ciesse"));
        assert!(text[0].contains("3 episode(s)"));
    }

    /// The answer of `POST /api/podcasts/feed`, measured on 2026-08-11.
    fn the_answer_of_the_feed() -> Feed {
        let answer: FeedAnswer = serde_json::from_value(serde_json::json!({
            "podcast": {
                "metadata": {
                    "image": "https://example.test/cover.jpg",
                    "categories": ["Fiction"],
                    "feedUrl": "https://www.spreaker.com/show/6199792/episodes/feed",
                    "type": "serial",
                    "title": "Another Study of Woman",
                    "language": "en",
                    "explicit": "false",
                    "author": "ciesse",
                    "pubDate": null,
                    "link": "https://example.test/page",
                    "description": "A series of tales."
                },
                "episodes": [ {}, {}, {} ]
            }
        }))
        .expect("the answer of the server must read");

        answer.podcast
    }

    #[test]
    fn the_answer_of_the_feed_reads() {
        let feed = the_answer_of_the_feed();

        assert_eq!(
            feed.metadata.title.as_deref(),
            Some("Another Study of Woman")
        );
        assert_eq!(feed.metadata.author.as_deref(), Some("ciesse"));
        assert_eq!(feed.episodes.len(), 3);
        // The server writes `explicit` as a text.
        assert_eq!(feed.metadata.explicit.as_deref(), Some("false"));
    }

    #[test]
    fn the_body_holds_what_the_server_needs() {
        let body = body_for(&the_answer_of_the_feed(), "lib-1", "folder-1", "/podcasts");

        assert_eq!(body["libraryId"], "lib-1");
        assert_eq!(body["folderId"], "folder-1");
        assert_eq!(body["path"], "/podcasts/Another Study of Woman");
        assert_eq!(body["media"]["metadata"]["title"], "Another Study of Woman");
        assert_eq!(
            body["media"]["metadata"]["feedUrl"],
            the_answer_of_the_feed().metadata.feed_url.unwrap()
        );
        assert_eq!(body["media"]["metadata"]["explicit"], false);
        // A new podcast must not start a download of every episode.
        assert_eq!(body["media"]["autoDownloadEpisodes"], false);
    }

    #[test]
    fn a_feed_that_says_explicit_gives_a_yes() {
        let mut feed = the_answer_of_the_feed();
        feed.metadata.explicit = Some("true".to_string());

        let body = body_for(&feed, "lib-1", "folder-1", "/podcasts");
        assert_eq!(body["media"]["metadata"]["explicit"], true);
    }

    #[test]
    fn a_folder_that_ends_with_a_line_gives_one_line_only() {
        let body = body_for(&the_answer_of_the_feed(), "lib-1", "folder-1", "/podcasts/");

        assert_eq!(body["path"], "/podcasts/Another Study of Woman");
    }

    /// **The title comes from the network.** A title that holds a path would
    /// write outside the folder of the library.
    #[test]
    fn a_title_cannot_leave_the_folder() {
        for bad in [
            "../../etc/passwd",
            "..",
            ".",
            "/etc/passwd",
            "a/b/c",
            "a\\b",
            "....//....//x",
            "~/.ssh/id_rsa",
            "a\0b",
            "$(rm -rf /)",
            "`whoami`",
        ] {
            let name = directory_of(bad);

            assert!(!name.contains('/'), "{:?} gives {:?}", bad, name);
            assert!(!name.contains('\\'), "{:?} gives {:?}", bad, name);
            assert!(!name.contains('\0'), "{:?} gives {:?}", bad, name);
            assert!(!name.contains(".."), "{:?} gives {:?}", bad, name);
            assert_ne!(name, ".");
            assert_ne!(name, "..");
            assert!(!name.is_empty(), "{:?} gives an empty name", bad);
        }
    }

    #[test]
    fn a_title_of_a_podcast_keeps_its_name() {
        assert_eq!(
            directory_of("Another Study of Woman by Honoré de Balzac (1799 - 1850)"),
            "Another Study of Woman by Honoré de Balzac 1799 - 1850"
        );
        assert_eq!(directory_of("The Daily"), "The Daily");
        assert_eq!(directory_of("99% Invisible"), "99 Invisible");
    }

    /// A name of a different writing must keep its letters.
    #[test]
    fn a_name_of_a_different_writing_keeps_its_letters() {
        assert_eq!(
            directory_of("日本語のポッドキャスト"),
            "日本語のポッドキャスト"
        );
        assert_eq!(directory_of("Ελληνικά"), "Ελληνικά");
    }

    #[test]
    fn a_title_that_gives_nothing_gives_a_name() {
        assert_eq!(directory_of(""), "A podcast");
        assert_eq!(directory_of("///"), "A podcast");
        assert_eq!(directory_of("..."), "A podcast");
        assert_eq!(directory_of("   "), "A podcast");
    }

    /// A file system gives an error for a name that is too long.
    #[test]
    fn a_very_long_title_gives_a_short_name() {
        let name = directory_of(&"a".repeat(500));

        assert_eq!(name.chars().count(), NAME_LIMIT);
    }

    #[test]
    fn two_spaces_together_give_one_space() {
        assert_eq!(directory_of("The   Daily"), "The Daily");
        // A dash is a character of a name, therefore two dashes stay.
        assert_eq!(directory_of("A -- B"), "A -- B");
        assert_eq!(directory_of("A  /  B"), "A B");
    }

    #[test]
    fn a_feed_with_no_title_still_gives_a_body() {
        let feed = Feed::default();
        let body = body_for(&feed, "lib-1", "folder-1", "/podcasts");

        assert_eq!(body["path"], "/podcasts/A podcast");
        assert_eq!(body["media"]["metadata"]["title"], "A podcast");
    }
}
