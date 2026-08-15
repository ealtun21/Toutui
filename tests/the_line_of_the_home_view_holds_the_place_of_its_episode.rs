//! The line of the Home view of a library of podcasts holds the place of its
//! own episode. See T-228.
//!
//! **A line of that view is one episode**, and the identity of the item names
//! every episode of one podcast (T-223). The Home view read that identity alone:
//! the place of the user reached no line, the mark of the media that plays stood
//! on every line of the podcast, and a position of a live message reached none
//! of them.
//!
//! The measurement of the real program v0.8.56 inside tmux, against the sandbox
//! (podman on :13399), of the podcast `Arthur Gordon Pym` of the library
//! `Podcasts`. The two episodes stood at 80 percent (`Chapter 00`, 4:04 of 5:05)
//! and at 10 percent (`Chapter 01`, 2:12 of 21:59) of the server, and the shelf
//! Continue Listening held the two of them:
//!
//! - The two lines said `Chapter 01` and `Chapter 00` with **no percent at
//!   all**, and the panel of the line said
//!   `[Arthur Gordon Pym] - Author: LibriVox - Episode: 1 - Duration: 22m` and
//!   nothing of the place.
//! - The user pressed the key `l` on `Chapter 01`. The row of the player said
//!   `Arthur Gordon Pym — Chapter 01` with `⏸ 2:37 / 21:59`, and **the two lines
//!   each took the mark `▶`**.
//!
//! **The control of the same run** (the trap 206): the Home view of the library
//! `Books` gave `5%  A Book Of Many Hours` and `50% A Long Test Book`, and the
//! panel said `Progress: 5%, 7h38m left, Not finished`. **A library of books
//! held every value that a library of podcasts held of none.**
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The three functions are pure, therefore this test needs no server and no
//! screen. **Three builds of the fault each fail it**: a read of the place that
//! drops the episode, a key of a line that drops the episode, and a list of the
//! positions of a live message that drops the rows of the episodes.

use serde_json::json;
use toutui::api::live::progress_of_the_user;
use toutui::api::me::get_media_progress::Root;
use toutui::logic::home_view::the_key_of_the_line;
use toutui::logic::the_positions::the_place_of_a_media;

/// The podcast and its two episodes of the measurement. The two episodes hold
/// the identity of the podcast, and that identity names no one of them alone.
const THE_PODCAST: &str = "the-podcast-of-arthur-gordon-pym";
const THE_EPISODE: &str = "the-episode-of-chapter-01";
const THE_SECOND_EPISODE: &str = "the-episode-of-chapter-00";

/// The book of the control of the measurement. Its row names no episode.
const THE_BOOK: &str = "the-book-of-many-hours";

/// The answer of `GET /api/me` of the measurement: the two episodes of one
/// podcast at two different places, and the book of the control.
fn the_places_of_the_account() -> serde_json::Value {
    json!({
        "mediaProgress": [
            {
                "id": "one",
                "userId": "a-user",
                "libraryItemId": THE_BOOK,
                "episodeId": null,
                "mediaItemId": "a-media",
                "mediaItemType": "book",
                "duration": 27480.0,
                "progress": 0.05,
                "currentTime": 1374.0,
                "isFinished": false,
                "hideFromContinueListening": false,
                "ebookLocation": null,
                "ebookProgress": 0,
                "lastUpdate": 1,
                "startedAt": 1,
                "finishedAt": null
            },
            {
                "id": "two",
                "userId": "a-user",
                "libraryItemId": THE_PODCAST,
                "episodeId": THE_SECOND_EPISODE,
                "mediaItemId": THE_SECOND_EPISODE,
                "mediaItemType": "podcastEpisode",
                "duration": 305.71102,
                "progress": 0.8,
                "currentTime": 244.0,
                "isFinished": false,
                "hideFromContinueListening": false,
                "ebookLocation": null,
                "ebookProgress": 0,
                "lastUpdate": 1,
                "startedAt": 1,
                "finishedAt": null
            },
            {
                "id": "three",
                "userId": "a-user",
                "libraryItemId": THE_PODCAST,
                "episodeId": THE_EPISODE,
                "mediaItemId": THE_EPISODE,
                "mediaItemType": "podcastEpisode",
                "duration": 1319.601633,
                "progress": 0.1,
                "currentTime": 132.0,
                "isFinished": false,
                "hideFromContinueListening": false,
                "ebookLocation": null,
                "ebookProgress": 0,
                "lastUpdate": 1,
                "startedAt": 1,
                "finishedAt": null
            }
        ]
    })
}

/// The place of each line, the key of each line, and the positions of a live
/// message. See T-228.
#[test]
fn the_line_of_an_episode_holds_the_place_of_that_episode() {
    let body = the_places_of_the_account();
    let rows: Vec<Root> =
        serde_json::from_value(body["mediaProgress"].clone()).expect("the answer of the server");

    // **The place of the user of one episode belongs to the line of that
    // episode.** The Home view held no place at all: the row of the podcast
    // names an episode, therefore a read of the item alone found no row.
    let of_the_first = the_place_of_a_media(&rows, THE_PODCAST, Some(THE_EPISODE))
        .expect("the row of the episode that the user opened");
    let of_the_second = the_place_of_a_media(&rows, THE_PODCAST, Some(THE_SECOND_EPISODE))
        .expect("the row of the second episode");

    assert_eq!(of_the_first.current_time, 132.0);
    assert_eq!(of_the_second.current_time, 244.0);

    // **The two lines hold two different places**, and the identity of the item
    // gives one of them to both: `GET /api/me/progress/:item` of a podcast
    // answers with the place of **one** episode of it (T-188).
    assert_ne!(of_the_first.current_time, of_the_second.current_time);

    // A book names no episode, and its row stands at the identity of its item.
    assert_eq!(
        the_place_of_a_media(&rows, THE_BOOK, None)
            .expect("the row of the book of the control")
            .current_time,
        1374.0
    );

    // **A podcast holds no place of its own** (T-219). A caller of the item of a
    // podcast takes no row of an episode of it.
    assert!(the_place_of_a_media(&rows, THE_PODCAST, None).is_none());

    // The key of each line of the Home view. The two lines of one podcast hold
    // one identity of the item, and the key of each of them holds its episode.
    let ids = vec![THE_PODCAST.to_string(), THE_PODCAST.to_string()];
    let episode_ids = vec![THE_EPISODE.to_string(), THE_SECOND_EPISODE.to_string()];

    let key_of_the_first =
        the_key_of_the_line(&ids, &episode_ids, 0).expect("the key of that line");
    let key_of_the_second =
        the_key_of_the_line(&ids, &episode_ids, 1).expect("the key of that line");

    assert_ne!(key_of_the_first, key_of_the_second);
    assert!(key_of_the_first.contains(THE_EPISODE));
    assert!(key_of_the_second.contains(THE_SECOND_EPISODE));

    // **A library of books gives no list of the episodes**, and the key of such
    // a line is the identity of its item.
    assert_eq!(
        the_key_of_the_line(&[THE_BOOK.to_string()], &[], 0),
        Some(THE_BOOK.to_string())
    );

    // A line that no list names holds no key at all.
    assert_eq!(the_key_of_the_line(&ids, &episode_ids, 9), None);

    // **The positions of a live message reach the line of an episode.** The old
    // list held the rows of the books alone, therefore the mark of every episode
    // stayed at the value of the request of the start for ever.
    let of_the_message = progress_of_the_user(&body);

    let percent_of = |key: &str| {
        of_the_message
            .iter()
            .find(|(one, _)| one == key)
            .map(|(_, progress)| progress.percent.clone())
    };

    assert_eq!(percent_of(&key_of_the_first), Some("10".to_string()));
    assert_eq!(percent_of(&key_of_the_second), Some("80".to_string()));
    assert_eq!(percent_of(THE_BOOK), Some("5".to_string()));

    // The key of a row of an episode names that episode, therefore no line takes
    // the identity of the podcast alone.
    assert_eq!(percent_of(THE_PODCAST), None);
}
