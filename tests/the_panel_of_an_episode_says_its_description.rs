//! The panel of the view of the episodes of a podcast says the description of
//! that episode. See T-251.
//!
//! **The render of that view read the subtitle of the episode alone**, and the
//! program asked the server for no description of an episode at all:
//! `collect_descs_pod_ep` gave one value for a view of many lines — the
//! description of the podcast — and no render of the screen read it. That is
//! the shape of T-250, which the Home view of a library of podcasts held.
//!
//! The measurement of 2026-08-15, of the real program inside tmux against the
//! sandbox. The podcast `Arthur Gordon Pym` holds a description of the server,
//! and `Chapter 01` of it holds the show notes of that episode:
//!
//! ```text
//! ──────────────────────────Episodes [11 items]──────────────────────────
//!   22% Chapter 00
//! ➤ 74% Chapter 01
//!   89% Chapter 02
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 1 - Duration: 22m
//! Progress: 74%, 5m left, Not finished
//! No description available
//! ```
//!
//! The Library view of that same run, of that same program, said the
//! description of the podcast of its line. The panel of the episodes said
//! nothing of the two.

use toutui::api::library_items::get_pod_ep::Root as ItemRoot;
use toutui::api::utils::collect_get_pod_ep::{collect_descs_pod_ep, collect_subtitles_pod_ep};
use toutui::logic::the_panel_of_a_line::the_description_of_a_podcast;
use toutui::utils::values_of_the_server::NO_DESCRIPTION;

/// The three roads of the panel of one episode.
///
/// **The parts of this test stay in one function.** See T-144 and T-157.
#[tokio::test]
async fn the_panel_of_an_episode_says_the_description_of_that_episode() {
    // `Arthur Gordon Pym` of the sandbox on 2026-08-15, with the show notes of
    // `Chapter 01` of the measurement.
    let podcast: ItemRoot = serde_json::from_value(serde_json::json!({
        "id": "b793354b-9841-480a-bd09-41923596517e",
        "media": {
            "metadata": {
                "title": "Arthur Gordon Pym",
                "description": "<p>The one complete novel of Edgar Allan Poe.</p>"
            },
            "episodes": [
                { "id": "episode-0", "title": "Chapter 00", "subtitle": "", "description": "" },
                {
                    "id": "episode-1",
                    "title": "Chapter 01",
                    "subtitle": "",
                    "description": "<p>The show notes of Chapter 01.</p>"
                },
                {
                    "id": "episode-2",
                    "title": "Chapter 02",
                    "subtitle": "The second chapter.",
                    "description": "<p>The show notes of Chapter 02.</p>"
                }
            ]
        }
    }))
    .expect("the answer of the server must read");

    let subtitles = collect_subtitles_pod_ep(&podcast).await;
    let descriptions = collect_descs_pod_ep(&podcast).await;

    // **The list holds one value for each episode** (the rule of T-24): the box
    // held one value of the podcast for the whole view before this item,
    // therefore every line after the first one stood against no value at all.
    assert_eq!(
        descriptions.len(),
        subtitles.len(),
        "the lists of this view stand one against the other by the number of \
         the line: {:?}",
        descriptions
    );

    // **The two boxes hold the value of the server alone** (T-249 and T-250): a
    // box that a fallback reads must hold no word of the program.
    assert_eq!(subtitles, vec!["", "", "The second chapter."]);
    assert_eq!(
        descriptions,
        vec![
            "The one complete novel of Edgar Allan Poe.",
            "The show notes of Chapter 01.",
            "The show notes of Chapter 02."
        ]
    );

    let panel = |line: usize| the_description_of_a_podcast(&subtitles[line], &descriptions[line]);

    // An episode of no description of its own says what the podcast is.
    assert_eq!(panel(0), "The one complete novel of Edgar Allan Poe.");

    // **The description of an episode is not the description of its podcast.**
    // The panel said `No description available` for this line, and the server
    // gave the show notes of it in that same answer.
    assert_eq!(panel(1), "The show notes of Chapter 01.");

    // The subtitle of the episode stands first, as it did before this item: a
    // podcast whose feed gives a subtitle loses nothing.
    assert_eq!(panel(2), "The second chapter.");

    // A podcast that holds none of the three says why the panel holds no text
    // (T-249).
    let nothing: ItemRoot = serde_json::from_value(serde_json::json!({
        "id": "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806",
        "media": {
            "metadata": { "title": "Letters of Two Brides", "description": "" },
            "episodes": [
                { "id": "episode-1", "title": "Letter 1", "subtitle": "", "description": null },
                { "id": "episode-2", "title": "Letter 2" }
            ]
        }
    }))
    .expect("the answer of the server must read");

    let subtitles = collect_subtitles_pod_ep(&nothing).await;
    let descriptions = collect_descs_pod_ep(&nothing).await;

    assert_eq!(subtitles, vec!["", ""]);
    assert_eq!(descriptions, vec!["", ""]);
    assert_eq!(
        the_description_of_a_podcast(&subtitles[0], &descriptions[0]),
        NO_DESCRIPTION
    );
    assert_eq!(
        the_description_of_a_podcast(&subtitles[1], &descriptions[1]),
        NO_DESCRIPTION
    );

    // A description of no letter is no description (T-249): a podcast whose
    // description is `<p> </p>` gives the words of the program, and not a line
    // of one space.
    let of_no_letter: ItemRoot = serde_json::from_value(serde_json::json!({
        "id": "a-podcast",
        "media": {
            "metadata": { "title": "A podcast", "description": "<p> </p>" },
            "episodes": [{ "id": "episode-1", "title": "An episode" }]
        }
    }))
    .expect("the answer of the server must read");

    let descriptions = collect_descs_pod_ep(&of_no_letter).await;
    assert_eq!(
        the_description_of_a_podcast("", &descriptions[0]),
        NO_DESCRIPTION
    );

    // A podcast of no episode gives no line, therefore it gives no value.
    let no_episode: ItemRoot = serde_json::from_value(serde_json::json!({
        "id": "a-podcast",
        "media": { "metadata": { "title": "A podcast", "description": "A description." } }
    }))
    .expect("the answer of the server must read");

    assert!(collect_descs_pod_ep(&no_episode).await.is_empty());
    assert!(collect_subtitles_pod_ep(&no_episode).await.is_empty());
}
