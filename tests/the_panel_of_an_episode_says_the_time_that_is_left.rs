//! The panel of a line of an episode of a podcast says the time that is left.
//! See T-244.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The measurement of 2026-08-15 against the sandbox. The panel of a book says
//! the percent, the time that is left, and the mark of the end, and the panel of
//! an episode said the percent and the mark of the end alone. The Home view of
//! the library `Podcasts` held `Chapter 00` of `Arthur Gordon Pym` at 66 seconds
//! of 306:
//!
//! ```text
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 0 - Duration: 5m
//! Progress: 22%, Not finished
//! ```
//!
//! The view of the episodes of that same podcast of that same run held
//! `Chapter 02` at 1168 seconds of 2337:
//!
//! ```text
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 2 - Duration: 39m
//! Progress: 50%, Not finished
//! ```
//!
//! **The control of that same run** (the trap 206): the view of the media of
//! `A Podcast Playlist` named `Chapter 00` of that same podcast, and the panel
//! of that line said `Progress: 100%, 0m left, Finished`. One program of one
//! account therefore said the time that is left of one episode in one view and
//! nothing of it in two others.
//!
//! The two roads of the fault: the format of the panel named no time that is
//! left, and neither function of the place of the panel made one. The Home view
//! read `duration_cnt_list`, which holds the length of a book and which a
//! library of podcasts leaves empty, and the view of the episodes gave the
//! empty text.

/// Gives the block of a function of a file of the source. See the trap 209.
///
/// A window of a number of characters is a window of the comments of the
/// function after it: the words of a correction take a line out of that window,
/// and the gate then says that the program lost a rule that it holds. The block
/// ends at the comment or at the head of the function that comes after this one.
fn the_block_of(source: &str, head: &str) -> String {
    let start = source
        .find(head)
        .unwrap_or_else(|| panic!("the source holds no function `{}`", head));
    let body = &source[start + head.len()..];

    let end = body
        .find("\n    /// ")
        .into_iter()
        .chain(body.find("\n    pub fn "))
        .chain(body.find("\n    fn "))
        .min()
        .unwrap_or(body.len());

    body[..end].to_string()
}

/// The same block for a function that stands outside an `impl`.
fn the_block_of_a_free_function(source: &str, head: &str) -> String {
    let start = source
        .find(head)
        .unwrap_or_else(|| panic!("the source holds no function `{}`", head));
    let body = &source[start + head.len()..];

    let end = body
        .find("\n/// ")
        .into_iter()
        .chain(body.find("\nasync fn "))
        .chain(body.find("\nfn "))
        .chain(body.find("\npub fn "))
        .min()
        .unwrap_or(body.len());

    body[..end].to_string()
}

#[test]
fn the_panel_of_an_episode_says_the_time_that_is_left() {
    // ---------------------------------------------------------------------
    // The format of the three panels that name an episode.
    // ---------------------------------------------------------------------

    let screen = std::fs::read_to_string("src/ui/tui.rs").expect("the source of the screen");

    for head in [
        "fn render_info_home(",
        "fn render_info_pod_ep(",
        "fn render_info_pod_ep_search(",
    ] {
        let block = the_block_of(&screen, head);

        assert!(
            block.contains("Progress: {}%, {} {}"),
            "the panel of an episode of `{}` names no time that is left",
            head
        );
        assert!(
            block.contains("place.the_time_that_is_left"),
            "the panel of an episode of `{}` says no time that is left",
            head
        );
    }

    // ---------------------------------------------------------------------
    // The two functions that make the place of the panel of an episode.
    // ---------------------------------------------------------------------

    let source = std::fs::read_to_string("src/app.rs").expect("the source of the application");

    // **The length of the media of a line of the Home view is the length of an
    // episode for a library of podcasts**: this function read
    // `duration_cnt_list`, and a library of podcasts fills no row of it.
    let home = the_block_of(&source, "fn the_place_of_the_panel_of_the_home_view(");

    assert!(
        home.contains("convert_seconds_for_prg(\n            length.unwrap_or(0.0),"),
        "the time of the row of the Home view takes no length of an episode"
    );

    // **The row of the place of an episode holds the place of the user in
    // seconds** (T-244), therefore this function makes the time of the row of
    // that place and of the length of the episode.
    let episodes = the_block_of(&source, "fn the_place_of_the_panel_of_this_podcast(");

    assert!(
        episodes.contains("convert_seconds_for_prg("),
        "the panel of an episode of a podcast makes no time that is left"
    );
    assert!(
        episodes.contains("the_part_of_the_row(places, selected, 2)"),
        "the panel of an episode reads no place of the user in seconds"
    );
    assert!(
        !episodes.contains("\n            \"\",\n"),
        "the panel of an episode still gives the empty time of the row"
    );

    // ---------------------------------------------------------------------
    // The row of the place of an episode holds three parts.
    // ---------------------------------------------------------------------

    let places = the_block_of_a_free_function(&source, "async fn the_places_of_the_episodes(");

    assert!(
        places.contains("collect_current_time_prg(row).await.to_string()"),
        "the row of the place of an episode holds no place in seconds"
    );

    // A row of three parts, for an episode that the answer of the account does
    // not name too: the reader of the third part takes no row of two parts.
    assert!(
        places.contains("let no_row = ||") && places.matches("\" N/A\".to_string()").count() >= 3,
        "the row of an episode that never played holds no third part"
    );

    // ---------------------------------------------------------------------
    // The pure function of the place of the panel.
    // ---------------------------------------------------------------------

    // The road of the row: no playback of this program, and no message of the
    // server. The panel keeps the three values of the row.
    let of_the_row = toutui::logic::the_panel_of_a_line::the_place_of_the_panel(
        false,
        None,
        None,
        Some(2336.7),
        " 50",
        "19m left,",
        " Not finished",
    );

    assert_eq!(of_the_row.the_time_that_is_left, "19m left,");

    // The road of the engine of this program (T-239). The time that is left
    // comes of the place of the engine and of the length of the episode.
    let of_the_engine = toutui::logic::the_panel_of_a_line::the_place_of_the_panel(
        true,
        Some(1520.0),
        None,
        Some(2336.7),
        " 50",
        "19m left,",
        " Not finished",
    );

    assert_eq!(of_the_engine.percent, "65");
    assert_eq!(of_the_engine.the_time_that_is_left, "14m left,");

    // **A media that the user did not begin names no time that is left**
    // (`convert_seconds_for_prg`): the server held `Chapter 01` at the percent
    // 81 with no `currentTime` at all, and the panel of that line said
    // `Progress: 81%,  Not finished`.
    assert!(
        toutui::utils::convert_seconds::convert_seconds_for_prg(1320.0, 0.0).is_empty(),
        "a media of no place of the user names a time that is left"
    );
}
