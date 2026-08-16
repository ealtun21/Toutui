//! The panel of a line of the view of the episodes of a podcast holds the value
//! of that line. See T-288.
//!
//! **The list of the lengths held one value for each episode of an audio
//! file**, and the list of the numbers of the episodes held one value for each
//! episode: an episode with no audio file therefore moved every length after it
//! up one line, and the last line of the view held no length at all.
//! `the_lengths_of_the_episodes` of that same file obeys the rule of T-24
//! already (T-236), and `collect_durations_pod_ep` did not.
//!
//! **The panel of that last line then said the words of a program.** The road
//! of the view of the library held three branches of the length of its lists,
//! and each of them drew `Error: Podcast metadata missing.`,
//! `Error: Episode info rendering mismatch.`, or
//! `Error: Episode data unavailable or index out of bounds.` in the place of
//! the panel. The road of the view of a search reads every list with `at`, and
//! it holds no such branch.
//!
//! The measurement of the real program v0.8.116 inside tmux, against the
//! sandbox (podman on :13399) with
//! `docs/harness/a_field_of_one_row_goes_away.py 13506 13399 requests.log
//! /api/items/b793354b-9841-480a-bd09-41923596517e media.episodes 0 audioFile`.
//! The podcast `Arthur Gordon Pym` of the library `Podcasts` holds 11 episodes,
//! and the first of them lost its `audioFile`. The keys `Tab`, `j`, and `l`
//! gave the view of its episodes:
//!
//! ```text
//! ➤ 22% Chapter 00
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 0 - Duration: 22m
//! ```
//!
//! `Chapter 00` is 305 seconds long, and `Chapter 01` is 1319 seconds long:
//! the panel of the first line said the length of the second episode. Ten keys
//! `j` gave the last line:
//!
//! ```text
//! ➤     Chapter 10
//! Error: Episode data unavailable or index out of bounds.
//! ```
//!
//! **That road writes one line of the log at every frame**: the log held 35
//! lines of `render_info_pod_ep: Index 10 out of bounds for episode/duration
//! vectors (ep_len=11, dur_len=10)!` in nine seconds.
//!
//! The same measurement of the corrected program said `Duration: N/A` for the
//! first line and
//! `[Arthur Gordon Pym] - Author: LibriVox - Episode: 10 - Duration: 12m` for
//! the last one, and the log held no line of that render at all. **The control
//! of the same run** (the trap 206): the same program against the sandbox with
//! no proxy said `Duration: 5m` for the first line and `Duration: 12m` for the
//! last one.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The functions are pure, therefore this test needs no server and no screen.

use toutui::api::library_items::get_pod_ep::Root as Podcast;
use toutui::api::utils::collect_get_pod_ep::{collect_durations_pod_ep, collect_episodes_pod_ep};
use toutui::logic::the_panel_of_a_line::{the_panel_of_an_episode, ThePlaceOfThePanel};

#[tokio::test]
async fn the_panel_of_an_episode_holds_the_value_of_its_line() {
    // One podcast of four episodes, in the shape of the sandbox: a length, an
    // episode with no audio file, a length of 0, and a length again.
    let podcast: Podcast = serde_json::from_value(serde_json::json!({
        "id": "pod-1",
        "media": {
            "metadata": { "title": "Arthur Gordon Pym", "author": "LibriVox" },
            "episodes": [
                { "id": "ep-0", "episode": "0", "title": "Chapter 00",
                  "audioFile": { "duration": 305.71102 } },
                { "id": "ep-1", "episode": "1", "title": "Chapter 01" },
                { "id": "ep-2", "episode": "2", "title": "Chapter 02",
                  "audioFile": { "duration": 0.0 } },
                { "id": "ep-3", "episode": "3", "title": "Chapter 03",
                  "audioFile": { "duration": 699.506939 } }
            ]
        }
    }))
    .expect("a podcast of four episodes");

    let numbers = collect_episodes_pod_ep(&podcast).await;
    let lengths = collect_durations_pod_ep(&podcast).await;

    assert_eq!(
        lengths.len(),
        numbers.len(),
        "the lists of the view of the episodes stand one against the other by \
         the number of the line (T-24): a list of the lengths that skips an \
         episode with no audio file gives the length of the episode after it \
         to that line, and it leaves the last line with no value at all"
    );

    assert_eq!(
        lengths,
        vec![
            "5m".to_string(),
            "N/A".to_string(),
            "N/A".to_string(),
            "12m".to_string()
        ],
        "an episode with no audio file and a length of 0 are each a length that \
         the server did not give (T-180), and the words of such a value beside \
         the label Duration: are N/A (T-249)"
    );

    // The panel of the last line of that view. The lists of the podcast hold
    // four lines, therefore the line 3 is the last one.
    let place = ThePlaceOfThePanel {
        percent: "22".to_string(),
        the_time_that_is_left: "5m left,".to_string(),
        the_end: "Not finished".to_string(),
    };

    assert_eq!(
        the_panel_of_an_episode(
            "Arthur Gordon Pym",
            "LibriVox",
            &numbers,
            &lengths,
            3,
            "",
            &place,
        ),
        "[Arthur Gordon Pym] - Author: LibriVox - Episode: 3 - Duration: 12m\n\
         Progress: 22%, 5m left, Not finished",
    );

    // **A line that the lists of the view do not hold says the words of a value
    // that the program does not have**, and it says no word of a program at
    // all. This is the condition that the measurement above made: 11 numbers of
    // episodes and 10 lengths.
    let panel = the_panel_of_an_episode(
        "Arthur Gordon Pym",
        "LibriVox",
        &numbers,
        &lengths[..3],
        3,
        "",
        &place,
    );

    assert_eq!(
        panel,
        "[Arthur Gordon Pym] - Author: LibriVox - Episode: 3 - Duration: N/A\n\
         Progress: 22%, 5m left, Not finished",
        "a line that a list of this view does not hold keeps the panel of its \
         media, and the value of that list says N/A (T-249)"
    );

    // A line that no list of the view holds. The name of the podcast and the
    // name of the author belong to the podcast, therefore they stay.
    assert_eq!(
        the_panel_of_an_episode("Arthur Gordon Pym", "LibriVox", &[], &[], 10, "", &place),
        "[Arthur Gordon Pym] - Author: LibriVox - Episode: N/A - Duration: N/A\n\
         Progress: 22%, 5m left, Not finished",
    );

    assert!(
        !panel.contains("Error"),
        "the panel of a line says no word of a program to the user (T-91), and \
         the words of this one were `Error: Episode data unavailable or index \
         out of bounds.`: {panel}"
    );
}
