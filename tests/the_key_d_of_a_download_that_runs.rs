//! The key `D` on a media that this program downloads already. See T-154.
//!
//! **The map of the progress is global and its key is the media.** A second
//! press of the key `D` on one media therefore wrote over the row of the
//! download that runs: `fetch_item` found the lock of T-148 in the hand of the
//! first task, it wrote `Failed` on that row, and `render_downloads` — which
//! draws a bar for each row of the state `Running` — drew no bar at all. The
//! measurement of 2026-08-14 gave **58 seconds of a download of 10041092 bytes
//! with no sign of it on the screen**, and a book of some hundred megabytes
//! gives an hour of it.
//!
//! The words were wrong beside it: the program said "A different program of
//! this account downloads …" for a download of this program. The key `X` holds
//! the two sentences since T-150, and the key `D` held the sentence of the
//! other window alone.
//!
//! The test needs no server and no file: the claim is a rule of the map.

use toutui::logic::download::progress::{
    claim_the_download, release_the_download, DownloadProgress, DownloadState,
    TheClaimOfTheDownload,
};
use toutui::logic::download::{new_progress_map, text_of_the_key_that_downloads};

const THE_MEDIA: &str = "an episode of a podcast";

/// Gives the row of a download that runs, with the bytes of its way.
fn the_row_of_a_download_that_runs() -> DownloadProgress {
    DownloadProgress {
        key: THE_MEDIA.to_string(),
        title: "Letter 15".to_string(),
        file_index: 1,
        file_count: 1,
        bytes_done: 3_931_735,
        bytes_total: 10_041_092,
        state: DownloadState::Running,
    }
}

/// The bar of the screen: `render_downloads` draws one bar for each row of the
/// state `Running`.
fn the_number_of_the_bars(map: &toutui::logic::download::progress::ProgressMap) -> usize {
    map.read()
        .expect("the map must give its rows")
        .values()
        .filter(|row| row.state == DownloadState::Running)
        .count()
}

#[test]
fn a_second_press_keeps_the_bar_and_the_bytes_of_the_download_that_runs() {
    let map = new_progress_map();

    map.write()
        .expect("the map must take a row")
        .insert(THE_MEDIA.to_string(), the_row_of_a_download_that_runs());

    let claim = claim_the_download(&map, THE_MEDIA, "Letter 15");

    assert_eq!(
        claim,
        TheClaimOfTheDownload::ThisProgramDownloadsIt,
        "the map of this program holds that download"
    );

    // **This is the harm of T-154**: the row said `Failed`, therefore the user
    // read nothing of a download that runs.
    assert_eq!(
        the_number_of_the_bars(&map),
        1,
        "the bar of the download that runs must stay on the screen"
    );

    let rows = map.read().expect("the map must give its rows");
    let row = rows.get(THE_MEDIA).expect("the row must stay");

    assert_eq!(
        row.bytes_done, 3_931_735,
        "the second press must write no byte of that row"
    );
    assert_eq!(row.bytes_total, 10_041_092);
    assert_eq!(row.state, DownloadState::Running);
}

#[test]
fn the_first_press_takes_the_place_of_the_media() {
    let map = new_progress_map();

    assert_eq!(
        claim_the_download(&map, THE_MEDIA, "Letter 15"),
        TheClaimOfTheDownload::ThePlaceIsTaken
    );

    assert_eq!(
        the_number_of_the_bars(&map),
        1,
        "the bar comes with the key, and not with the first byte"
    );
}

/// A download that came to its end, and a download that stopped, hold the place
/// of no media: the key `D` takes that media again.
#[test]
fn a_download_that_is_not_running_gives_the_place_back() {
    for state in [
        DownloadState::Finished,
        DownloadState::Failed("the server answered 404".to_string()),
    ] {
        let map = new_progress_map();

        let mut row = the_row_of_a_download_that_runs();
        row.state = state.clone();

        map.write()
            .expect("the map must take a row")
            .insert(THE_MEDIA.to_string(), row);

        assert_eq!(
            claim_the_download(&map, THE_MEDIA, "Letter 15"),
            TheClaimOfTheDownload::ThePlaceIsTaken,
            "the state {:?} holds no place",
            state
        );

        let rows = map.read().expect("the map must give its rows");
        let row = rows.get(THE_MEDIA).expect("the row must stay");
        assert_eq!(row.bytes_done, 0, "a new download starts at no byte");
        assert_eq!(row.state, DownloadState::Running);
    }
}

/// A claim that stays `Running` for ever holds the key `D` of that media for
/// ever. Every road out of the download therefore gives the place back.
#[test]
fn a_download_that_did_not_start_gives_the_place_back() {
    let map = new_progress_map();

    assert_eq!(
        claim_the_download(&map, THE_MEDIA, "Letter 15"),
        TheClaimOfTheDownload::ThePlaceIsTaken
    );

    release_the_download(&map, THE_MEDIA, "no authentication token");

    assert_eq!(
        the_number_of_the_bars(&map),
        0,
        "a download that did not start draws no bar"
    );

    assert_eq!(
        claim_the_download(&map, THE_MEDIA, "Letter 15"),
        TheClaimOfTheDownload::ThePlaceIsTaken,
        "the key D takes that media again"
    );
}

/// The sentence names this program, and it names no different program. See
/// T-150 for the two sentences of the key `X`.
#[test]
fn the_sentence_names_this_program() {
    let text = text_of_the_key_that_downloads("Letter 15");

    assert!(text.contains("This program downloads"), "{}", text);
    assert!(text.contains("\"Letter 15\""), "{}", text);
    assert!(
        !text.contains("different program"),
        "no different program exists: {}",
        text
    );
}

/// **No unit test reaches the key handler of `src/app.rs`**, therefore this
/// part reads the source: the claim must stand before the first request, and
/// every road out of the download must give the place back.
#[test]
fn the_download_claims_the_media_before_it_asks_the_server() {
    let source = include_str!("../src/logic/download/mod.rs");

    let claim = source
        .find("claim_the_download(")
        .expect("the download must claim the media");

    let request = source
        .find("get_item(&client")
        .expect("the download asks the server for the item");

    assert!(
        claim < request,
        "the claim must stand before the request of the item"
    );

    assert!(
        source.matches("release_the_download(").count() >= 3,
        "every road out of the download must give the place back"
    );
}
