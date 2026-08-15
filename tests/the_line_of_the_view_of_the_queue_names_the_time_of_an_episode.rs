//! The line of the view of the queue names the time of an episode. See T-236.
//!
//! **An episode of a podcast went into the queue with no length at all.** The
//! two views that hold an episode — the Home view of a library of podcasts and
//! the view of the episodes of a podcast — hold the length of each episode as a
//! **text**, and a text gives no number: `selected_length` therefore gave
//! `None` for each of them, the key `n` wrote a row of the disk with no length,
//! and the line of that media of the view of the queue said no time while the
//! line of every book of that same view said the time that is left (T-234).
//!
//! The measurement of the real program v0.8.64 inside tmux, against the sandbox
//! (podman on :13399). The user pressed the key `n` on `A Long Test Book` of the
//! library `Books`, and then the key `n` on `Chapter 02` of `Arthur Gordon Pym`
//! of the library `Podcasts`, and then the key `q`:
//!
//! ```text
//! ➤ 50% 1. 📕 A Long Test Book — Long Author  (15m left)
//!   32% 2. 🎙 Chapter 02 — Arthur Gordon Pym
//! ```
//!
//! **The control of the same run** (the trap 206): the line of the book of that
//! same view said `(15m left)`, therefore the row of the time works; and the
//! panel of the Home view of that same program, of that same episode, said
//! `[Arthur Gordon Pym] - Author: LibriVox - Episode: 2 - Duration: 39m`.
//! The program held the length of that episode already, in the very view where
//! the user pressed the key.
//!
//! The same measurement of the corrected program:
//!
//! ```text
//! ➤ 50% 1. 📕 A Long Test Book — Long Author  (15m left)
//!   32% 2. 🎙 Chapter 02 — Arthur Gordon Pym  (27m left)
//!   ✓   3. 🎙 Chapter 01 — Arthur Gordon Pym  (22m)
//! ```
//!
//! **The line 3 is the second fault of that same measurement.** The server
//! writes the place of a media that the user finished below the length of it:
//! the row of `Chapter 01` stood at 1319 seconds of 1319.601633 with
//! `isFinished`, therefore the line of it said `(0m left)`. T-234 said already
//! that a media that came to its end keeps the length, and it read the number of
//! seconds for that decision. The mark of the row is the truth of the end.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The functions are pure, therefore this test needs no server and no screen.
//! **Seven builds of the fault each fail it**: a collector of the shelves that
//! gives no number, a collector of the episodes of a podcast that gives no
//! number, a line of a media that the user finished that says `0m left`, and a
//! length of 0 that gives `0m`, and the three arms of `selected_length` of the
//! two views that hold an episode and of a length of 0.

use std::collections::BTreeMap;
use toutui::api::libraries::get_library_perso_view_pod::Root as Shelf;
use toutui::api::library_items::get_pod_ep::Root as Podcast;
use toutui::api::utils::collect_get_pod_ep::the_lengths_of_the_episodes;
use toutui::api::utils::collect_personalized_view_pod::the_lengths_of_the_episodes_of_the_shelves;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::queue::{the_lines_of_the_queue, Entry};

fn episode(id: &str, title: &str, length: Option<f64>) -> Entry {
    Entry {
        target: PlaybackTarget::Episode {
            item_id: "a-podcast".to_string(),
            episode_id: id.to_string(),
        },
        title: title.to_string(),
        author: "Arthur Gordon Pym".to_string(),
        duration: length,
    }
}

fn place(percent: &str, finished: &str, seconds: &str) -> Vec<String> {
    vec![
        percent.to_string(),
        finished.to_string(),
        seconds.to_string(),
    ]
}

#[tokio::test]
async fn the_line_of_the_view_of_the_queue_names_the_time_of_an_episode() {
    // The shelves of a library of podcasts, in the shape of the sandbox: one
    // entity of an episode of a length, one entity of a podcast with no episode
    // (it takes no line at all), one entity of an episode of an audio file with
    // no length, and one entity of an episode with no audio file.
    let shelves: Vec<Shelf> = serde_json::from_value(serde_json::json!([
        {
            "id": "newest-episodes",
            "label": "Newest Episodes",
            "type": "episode",
            "entities": [
                { "id": "pod-1",
                  "media": { "metadata": { "title": "Arthur Gordon Pym" } },
                  "recentEpisode": { "id": "ep-1", "libraryItemId": "pod-1",
                                     "title": "Chapter 02",
                                     "audioFile": { "duration": 2336.731429 } } },
                { "id": "pod-2",
                  "media": { "metadata": { "title": "A Podcast Of No Episode" } } },
                { "id": "pod-1",
                  "media": { "metadata": { "title": "Arthur Gordon Pym" } },
                  "recentEpisode": { "id": "ep-2", "libraryItemId": "pod-1",
                                     "title": "Chapter 03",
                                     "audioFile": { "duration": 0.0 } } },
                { "id": "pod-1",
                  "media": { "metadata": { "title": "Arthur Gordon Pym" } },
                  "recentEpisode": { "id": "ep-3", "libraryItemId": "pod-1",
                                     "title": "Chapter 04" } }
            ]
        }
    ]))
    .expect("the shelves of a library of podcasts");

    assert_eq!(
        the_lengths_of_the_episodes_of_the_shelves(&shelves).await,
        vec![Some(2336.731429), None, None],
        "the Home view of a library of podcasts must hold the length of each \
         episode as a number: the key n of that view puts an episode in the \
         queue, and the line of that media says the time that is left (T-236). \
         A length of 0 and an episode of no audio file are a length that the \
         server did not give (T-180)"
    );

    // One podcast of four episodes: a length, a length of 0, no audio file, and
    // a length again. **The list holds one value for each episode**: a list
    // that skips the episode of no audio file gives the length of the episode
    // after it to that line (T-24).
    let podcast: Podcast = serde_json::from_value(serde_json::json!({
        "id": "pod-1",
        "media": {
            "episodes": [
                { "id": "ep-1", "audioFile": { "duration": 1319.601633 } },
                { "id": "ep-2", "audioFile": { "duration": 0.0 } },
                { "id": "ep-3" },
                { "id": "ep-4", "audioFile": { "duration": 305.71102 } }
            ]
        }
    }))
    .expect("a podcast of four episodes");

    assert_eq!(
        the_lengths_of_the_episodes(&podcast).await,
        vec![Some(1319.601633), None, None, Some(305.71102)],
        "the view of the episodes of a podcast must hold the length of each \
         episode as a number, and it must hold one value for each episode: the \
         lists of that view stand one against the other by the number of the \
         line (T-24 and T-236)"
    );

    // The three lines of the measurement above, of the corrected program.
    let entries = vec![
        episode("ep-1", "Chapter 02", Some(2336.731429)),
        episode("ep-2", "Chapter 01", Some(1319.601633)),
        episode("ep-3", "Chapter 04", None),
    ];

    let mut places = BTreeMap::new();
    places.insert(
        "a-podcast/ep-1".to_string(),
        place(" 32", " Not finished", "740"),
    );
    places.insert(
        "a-podcast/ep-2".to_string(),
        place(" 62", " Finished", "1319"),
    );

    let lines = the_lines_of_the_queue(&entries, &places, None);

    assert!(
        lines[0].contains("(27m left)"),
        "the line of an episode must say the time that is left of it: the \
         episode stood at 740 seconds of 2336.7, and 1596.7 seconds stay (T-236). The line says: {}",
        lines[0]
    );

    assert!(
        lines[1].contains("(22m)") && !lines[1].contains("left"),
        "a media that the mark of its row says is finished must say the length \
         of it: the server writes the place of such a media below the length, \
         and the line then said `0m left` (T-234 and T-236). The line says: {}",
        lines[1]
    );

    assert!(
        !lines[2].contains('(') && lines[2].contains("Chapter 04"),
        "a media of no length must say no time at all, and it must keep its \
         title (T-180 and T-236). The line says: {}",
        lines[2]
    );

    // A row of the disk of a version before T-236 holds a length of 0 for an
    // episode. That length is no length, therefore the line says no time and
    // it does not say `0m`.
    let of_an_old_version = vec![episode("ep-4", "Chapter 05", Some(0.0))];
    let lines = the_lines_of_the_queue(&of_an_old_version, &BTreeMap::new(), None);

    assert!(
        !lines[0].contains('('),
        "a length of 0 of a row of the disk is a length that the server did \
         not give, and the line of it must not say `0m` (T-180 and T-236). \
         The line says: {}",
        lines[0]
    );

    // **The key `n` reads `selected_length`**, and that function gave `None`
    // for the two views that hold an episode. The lengths of the collectors
    // above reach the queue through it alone, therefore the test reads it.
    //
    // **The block ends at the function after this one** (the trap 209): a
    // window of a number of characters is a test of the comments of the
    // function, and the words of a correction then take a line out of it.
    let source = include_str!("../src/app.rs");

    let start = source
        .find("    fn selected_length(&self) -> Option<f64> {")
        .expect("the program reads the length of the selected media");
    let end = start
        + source[start..]
            .find("\n    /// ")
            .expect("a function comes after this one");
    let block = &source[start..end];

    assert!(
        block.contains("self.the_lengths_of_the_episodes_of_the_home_view"),
        "the Home view of a library of podcasts must give the length of the \
         episode of its line: that view gave nothing at all, and the key n of \
         it put an episode in the queue with no length (T-236)"
    );

    assert!(
        block.contains("self.the_lengths_of_the_episodes_search")
            && block.contains("self.the_lengths_of_the_episodes"),
        "the view of the episodes of a podcast must give the length of the \
         episode of its line, and the two ways into that view hold the \
         episodes in two different lists (T-236)"
    );

    assert!(
        block.contains("length.filter(|length| *length > 0.0)"),
        "a length of 0 is a length that the server did not give, therefore no \
         key of the program may take it for a length (T-180 and T-236)"
    );
}
