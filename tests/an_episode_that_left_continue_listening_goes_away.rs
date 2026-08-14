//! An episode that leaves the shelf Continue Listening leaves the Home view.
//! See T-226.
//!
//! **A line of the Home view of a library of podcasts is one episode**, and
//! `_ids_cnt_list` holds the identity of the **podcast** for each of those
//! lines. Two episodes of one podcast therefore hold one value there, and the
//! identity of the item names no line alone (T-223).
//!
//! The measurement of 2026-08-15, against the sandbox, of the podcast
//! `Arthur Gordon Pym` of the library `Podcasts`: `Chapter 00` and `Chapter 01`
//! stood together on the shelf Continue Listening, and the user pressed the key
//! `N` on `Chapter 01`. The server took the value, the shelf of the server then
//! held `Chapter 00` alone, the live message came (`user_updated: the position
//! of 20 media`), and **the screen kept `Home [13 items]` with both lines**.
//! The program said `The media is away from Continue Listening now.` and it
//! showed the media on that shelf. A book of the same key in the same run gave
//! `Home [34 items]` and then `Home [33 items]`.
//!
//! The cause stood in two places: `the_media_away_from_continue_listening` of
//! `src/api/live.rs` dropped **every** row of an episode, and the loop of
//! `take_the_media_that_left_away` of `src/app.rs` compared the identity of the
//! item. A correction of the first one alone would take **every** episode of
//! that podcast away from the shelf.
//!
//! The three functions are pure, therefore this test needs no server and no
//! screen. **The parts of this test stay in one function** (T-144 and T-157).

use std::collections::BTreeSet;
use toutui::api::live::the_media_away_from_continue_listening;
use toutui::logic::home_view::the_media_that_left_the_shelf;
use toutui::logic::live::the_key_of_the_media;

/// The rule of T-226, in its three parts.
#[test]
fn an_episode_that_left_the_shelf_takes_its_line_and_no_other_line() {
    // --- The key of a media names the episode. ---
    //
    // Two episodes of one podcast hold the identity of that podcast (T-223).
    assert_eq!(the_key_of_the_media("pod", Some("ep1")), "pod/ep1");
    assert_ne!(
        the_key_of_the_media("pod", Some("ep1")),
        the_key_of_the_media("pod", Some("ep2")),
    );

    // A book gives the identity of its item, and a library of books gives no
    // episode at all.
    assert_eq!(the_key_of_the_media("book", None), "book");
    assert_eq!(the_key_of_the_media("book", Some("")), "book");

    // --- A row of an episode of the message belongs to the list. ---
    //
    // The body is the shape of `mediaProgress` of the message `user_updated`.
    let body = serde_json::json!({
        "mediaProgress": [
            // The user hid this episode with the key `N`.
            { "libraryItemId": "pod", "episodeId": "ep1",
              "progress": 0.1, "isFinished": false,
              "hideFromContinueListening": true },
            // The second episode of that same podcast stays on the shelf.
            { "libraryItemId": "pod", "episodeId": "ep2",
              "progress": 0.2, "isFinished": false,
              "hideFromContinueListening": false },
            // A third episode of it that the user finished.
            { "libraryItemId": "pod", "episodeId": "ep3",
              "progress": 1.0, "isFinished": true,
              "hideFromContinueListening": false },
            // A book of the account. The rule of T-66 stays.
            { "libraryItemId": "book", "progress": 1.0, "isFinished": true,
              "hideFromContinueListening": false },
            // A book that the user hears now.
            { "libraryItemId": "book-2", "progress": 0.5, "isFinished": false,
              "hideFromContinueListening": false },
        ]
    });

    let away: BTreeSet<String> = the_media_away_from_continue_listening(&body)
        .into_iter()
        .collect();

    assert!(
        away.contains("pod/ep1"),
        "an episode that the user hid must leave the shelf: {:?}",
        away
    );
    assert!(
        away.contains("pod/ep3"),
        "an episode that the user finished must leave the shelf: {:?}",
        away
    );
    assert!(
        !away.contains("pod/ep2"),
        "an episode of that podcast that stays must not leave it: {:?}",
        away
    );
    assert!(
        away.contains("book"),
        "the rule of a book stays: {:?}",
        away
    );
    assert!(!away.contains("book-2"), "{:?}", away);

    // **The identity of the podcast alone must not stand in the list.** A line
    // of the Home view holds that identity for every episode, therefore a
    // value of `pod` there takes every episode of the podcast away.
    assert!(
        !away.contains("pod"),
        "the identity of the item names no episode: {:?}",
        away
    );

    // --- The lines of the Home view read that list by their own key. ---
    //
    // The four lines of a library of podcasts: three episodes of one podcast on
    // the shelf of Continue Listening, and one episode of another shelf. A
    // media stands on two shelves, therefore each shelf gives its own number
    // (T-66).
    let ids = vec![
        "pod".to_string(),
        "pod".to_string(),
        "pod".to_string(),
        "pod".to_string(),
    ];
    let episode_ids = vec![
        "ep1".to_string(),
        "ep2".to_string(),
        "ep3".to_string(),
        "ep1".to_string(),
    ];
    let on_the_shelf = vec![true, true, true, false];

    let left = the_media_that_left_the_shelf(&ids, &episode_ids, &on_the_shelf, &away);

    assert_eq!(
        left,
        BTreeSet::from([0, 2]),
        "the line of ep1 and the line of ep3 leave the shelf, and no other line"
    );

    // The line 3 holds the same episode as the line 0, and it stands on another
    // shelf: the shelf of the line decides, and not the media (T-66).
    assert!(
        !left.contains(&3),
        "a line of another shelf stays: {:?}",
        left
    );

    // --- A library of books holds no episode at all. ---
    let ids = vec!["book".to_string(), "book-2".to_string()];
    let left = the_media_that_left_the_shelf(&ids, &[], &[true, true], &away);

    assert_eq!(left, BTreeSet::from([0]), "the rule of a book stays");

    // --- An episode whose identity the server did not give keeps its line. ---
    //
    // No key can name it, therefore the safe road is the line that stays
    // (T-203).
    let ids = vec!["pod".to_string()];
    let episode_ids = vec!["N/A".to_string()];
    let left = the_media_that_left_the_shelf(&ids, &episode_ids, &[true], &away);

    assert!(
        left.is_empty(),
        "an episode with no identity keeps its line: {:?}",
        left
    );
}
