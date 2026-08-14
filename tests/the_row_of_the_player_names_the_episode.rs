//! The row of the player names the episode of a podcast. See T-225.
//!
//! **The title of a playback of a podcast is the name of the podcast**, and
//! every episode of that podcast holds it (T-223):
//! `post_start_playback_session_pod` reads `mediaMetadata.title` for the title
//! of the row, and the name of the episode stands in `displayTitle` of the same
//! answer. No part of the program read that second field.
//!
//! The measurement of v0.8.53 against the sandbox, inside tmux, of the podcast
//! `Arthur Gordon Pym` of the library `Podcasts`. `Chapter 02` stood in the
//! queue, and the user played `Chapter 00`:
//!
//! | The moment | The row of the player |
//! |---|---|
//! | `Chapter 00` plays | `Arthur Gordon Pym by LibriVox \| No chapter`, and `▶ 1:28 / 5:05` |
//! | the queue starts `Chapter 02` **with no key of the user** | `Arthur Gordon Pym by LibriVox \| No chapter`, and `⏸ 0:9 / 38:56` |
//!
//! **The two rows hold the same name.** The length of the media was the one
//! value that moved, and a length names no episode: the user could not tell
//! which episode plays, and the row of the player is the one part of the screen
//! that follows a media that the queue changes. That is the condition that
//! T-224 left open, and the key `b` of T-224 stands on it.
//!
//! The same measurement of the corrected program said
//! `Arthur Gordon Pym — Chapter 00 by LibriVox` and then
//! `Arthur Gordon Pym — Chapter 02 by LibriVox`. A book of the same run kept
//! its row of before: `A Long Test Book by Long Author | The second part`.
//!
//! **This test needs no server and no engine**: the two functions of the
//! correction are pure, and `player_info` reads a `PlaybackState` of the
//! memory.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at
//! one run of six (T-144 and T-157).

use serde_json::json;
use toutui::api::library_items::play_lib_item_or_pod::{
    collect_info_item, the_name_of_the_episode,
};
use toutui::player::engine::{PlaybackState, PlaybackStatus};
use toutui::player::integrated::player_info::{player_info, the_title_of_the_row};

/// The podcast of the sandbox, and one episode of it.
const THE_NAME_OF_THE_PODCAST: &str = "Arthur Gordon Pym";
const THE_NAME_OF_THE_EPISODE: &str = "Chapter 02";

/// The answer of `POST /api/items/:id/play/:episode` of the sandbox, of the
/// measurement above. The fields that `collect_info_item` reads stand in it.
fn the_answer_of_the_session_of_an_episode() -> serde_json::Value {
    json!({
        "id": "play_a1b2c3",
        "currentTime": 0.0,
        "displayTitle": THE_NAME_OF_THE_EPISODE,
        "displayAuthor": "LibriVox",
        "mediaMetadata": { "title": THE_NAME_OF_THE_PODCAST },
        "audioTracks": [{ "contentUrl": "/hls/play_a1b2c3/output.m3u8", "duration": 2336.7 }]
    })
}

#[test]
fn the_row_of_the_player_names_the_episode_of_a_podcast() {
    // The answer of the session of an episode holds the name of the podcast in
    // its title, and the name of the episode in `displayTitle`.
    let answer = the_answer_of_the_session_of_an_episode();
    let of_the_episode = collect_info_item(&answer, &answer["displayTitle"]);
    assert_eq!(of_the_episode[4], THE_NAME_OF_THE_PODCAST);

    assert_eq!(
        the_name_of_the_episode(&of_the_episode, true).as_deref(),
        Some(THE_NAME_OF_THE_EPISODE),
        "the program reads the name of the episode of the answer of the session"
    );

    // **A book has no such name.** `post_start_playback_session_book` gives the
    // title of the book for the subtitle, therefore a row that took it would
    // say the name of the book two times.
    let of_a_book = collect_info_item(&answer, &answer["mediaMetadata"]["title"]);
    assert_eq!(
        the_name_of_the_episode(&of_a_book, false),
        None,
        "a book gives no name of an episode"
    );

    // **A name that the server did not give is no name** (T-91 and T-182). An
    // answer with no `displayTitle` gives the default `N/A` of
    // `collect_info_item`, and an answer of a name of no character gives a text
    // of no character.
    let mut with_no_name = the_answer_of_the_session_of_an_episode();
    with_no_name["displayTitle"] = json!(null);
    let of_no_name = collect_info_item(&with_no_name, &with_no_name["displayTitle"]);
    assert_eq!(of_no_name[5], "N/A");
    assert_eq!(
        the_name_of_the_episode(&of_no_name, true),
        None,
        "the row says nothing that the server did not say"
    );

    let mut with_no_character = the_answer_of_the_session_of_an_episode();
    with_no_character["displayTitle"] = json!("   ");
    let of_no_character = collect_info_item(&with_no_character, &with_no_character["displayTitle"]);
    assert_eq!(
        the_name_of_the_episode(&of_no_character, true),
        None,
        "a name of no character is a name that the server did not give"
    );

    // The name of the media of the row.
    assert_eq!(
        the_title_of_the_row(THE_NAME_OF_THE_PODCAST, Some(THE_NAME_OF_THE_EPISODE)),
        "Arthur Gordon Pym — Chapter 02",
        "the row of an episode names the podcast and the episode"
    );
    assert_eq!(
        the_title_of_the_row("A Long Test Book", None),
        "A Long Test Book",
        "a media with no name of an episode keeps its own name alone"
    );

    // The row of the player reads that name. **Two episodes of one podcast gave
    // one row before this item**, and the length was the one value that moved.
    let of_the_first = PlaybackState {
        item_id: "b793354b-9841-480a-bd09-41923596517e".to_string(),
        episode_id: Some("845f9d16-2121-40b1-a3ed-682cab9ed178".to_string()),
        title: THE_NAME_OF_THE_PODCAST.to_string(),
        episode_title: Some("Chapter 00".to_string()),
        author: "LibriVox".to_string(),
        position: 88.0,
        duration: 305.7,
        status: PlaybackStatus::Playing,
        ..PlaybackState::default()
    };

    let of_the_second = PlaybackState {
        episode_id: Some("ff28a3b0-4ade-4a41-a3c3-864d264354a7".to_string()),
        episode_title: Some(THE_NAME_OF_THE_EPISODE.to_string()),
        position: 9.0,
        duration: 2336.7,
        ..of_the_first.clone()
    };

    let the_first_row = player_info(1.0, &of_the_first);
    let the_second_row = player_info(1.0, &of_the_second);

    assert_eq!(the_first_row[0], "Arthur Gordon Pym — Chapter 00");
    assert_eq!(the_second_row[0], "Arthur Gordon Pym — Chapter 02");
    assert_ne!(
        the_first_row[0], the_second_row[0],
        "the queue changes the episode with no key of the user, and the row of \
         the player is the one part of the screen that says it"
    );

    // The author of the row stays the author of the podcast.
    assert_eq!(the_second_row[1], "LibriVox");

    // A book keeps its row of before.
    let of_a_book = PlaybackState {
        item_id: "a book".to_string(),
        title: "A Long Test Book".to_string(),
        author: "Long Author".to_string(),
        status: PlaybackStatus::Playing,
        ..PlaybackState::default()
    };
    assert_eq!(player_info(1.0, &of_a_book)[0], "A Long Test Book");
}
