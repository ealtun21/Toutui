//! The answer of a session that gave no place and no identity. See T-182.
//!
//! **The road of T-179, of T-180, and of T-181 named this answer**: a field
//! that the program reads with a **default** gives no fault of a decode, and
//! the program then **uses** that default. The three items before this one took
//! `metadata.size`, `duration`, and `ino` and `index` of
//! `GET /api/items/:id`. **This item takes `POST /api/items/:id/play`**, and
//! two fields of that answer held a fault of the user.
//!
//! Two measurements of 2026-08-14 against the sandbox with
//! `docs/harness/a_field_of_the_answer_goes_away.py` on the port 13503 and the
//! path `/api/items/6ba57b9a-acb5-44f9-b2b6-39ad9107b420/play` — the book of
//! eight hours, one file. The place of the account came of
//! `PATCH /api/me/progress/:id` with `currentTime: 12000` (the section 15 of
//! `docs/TEST-SERVER.md`).
//!
//! **The field `currentTime`:**
//!
//! | The measurement | Before | After |
//! |---|---|---|
//! | The Home view | `41% A Book Of Many Hours` | the same |
//! | The log of the key `l` | **`[play] the item … starts at 0 seconds with 1 tracks`** | `starts at 12000 seconds` |
//! | The row of the player, after six seconds | **`▶ 5:04 / 8:00:00 … (1%)`** | the place of the user |
//! | The place of the account, after twenty seconds | **`currentTime 1096`** | the place of the user |
//! | The words of the program | **none** | none, and the place stays |
//!
//! **The place of the user went away on the server too**: the loop of the
//! playback sent the place of the first second to the session, and 12000
//! seconds became 1096 seconds. No word of the program said it.
//!
//! **The field `id`:**
//!
//! | The measurement | Before | After |
//! |---|---|---|
//! | The row of the database | **`''\|6ba57b9a…\|2565\|A Book Of Many Hours`** | no row, and no playback |
//! | The log, at each sync | **`the server did not accept the sync: The server does not have this item.`** | no request of a session with no name |
//! | The log, at the close | **`the server did not close the session: The server does not have this item.`** | the same |
//! | The words of the program | **none** | `The session of the server has no identity.` |
//!
//! **Two programs of one account then held one row.** `id_session` is the key
//! of the table `listening_session`, therefore the second program wrote no row
//! of its own: the position of the second program stood in the row of the first
//! one (`''|6ba57b9a…|4596|…`, and the owner of that row was the first
//! program).
//!
//! **This test needs no server.** The bodies below are the answers of the two
//! proxies of the measurement, with the fields that this road reads.

use serde_json::json;
use toutui::api::client::error::ApiError;
use toutui::api::library_items::play_lib_item_or_pod::collect_info_item;
use toutui::api::me::get_media_progress::the_path_of_the_place;
use toutui::logic::the_playback::{
    the_place_of_a_media_that_never_played, the_place_of_the_session, the_start_of_a_playback,
    the_words_of_a_playback_that_did_not_start, TheStartOfAPlayback, WhyNot,
};

/// The media of the measurement.
const THE_MEDIA: &str = "6ba57b9a-acb5-44f9-b2b6-39ad9107b420";

/// The identity of the session, of the answer of the sandbox.
const THE_SESSION: &str = "ec843f64-f487-4fb9-9358-2d2d9737e8d0";

/// The place of the account, in seconds.
const THE_PLACE: f64 = 12000.0;

/// The answer of `POST /api/items/:id/play` of the sandbox, with the fields
/// that `collect_info_item` reads.
fn the_answer_of_the_session() -> serde_json::Value {
    json!({
        "id": THE_SESSION,
        "libraryItemId": THE_MEDIA,
        "mediaType": "book",
        "mediaMetadata": {"title": "A Book Of Many Hours"},
        "displayTitle": "A Book Of Many Hours",
        "displayAuthor": "Many Hours Author",
        "duration": 28800.0,
        "playMethod": 0,
        "currentTime": THE_PLACE,
        "audioTracks": [{
            "index": 1,
            "duration": 28800.0,
            "contentUrl": "/api/items/6ba57b9a/file/9103848",
            "mimeType": "audio/mpeg"
        }]
    })
}

/// The same answer, and one named field of it goes away. This is the work of
/// `docs/harness/a_field_of_the_answer_goes_away.py`.
fn without_the_field(name: &str) -> serde_json::Value {
    let mut answer = the_answer_of_the_session();
    answer
        .as_object_mut()
        .expect("the answer of the session is an object")
        .remove(name);
    answer
}

/// **The parts of this test stay in one function**: two test functions of one
/// module fight for the box of the process (T-144 and T-157).
#[test]
fn the_answer_of_a_session_says_what_the_server_did_not_give() {
    let subtitle = json!("A Book Of Many Hours");

    // The answer of the sandbox holds both fields, and the program reads them.
    let whole = collect_info_item(&the_answer_of_the_session(), &subtitle);

    assert_eq!(the_place_of_the_session(&whole[0]), Some(THE_PLACE));
    assert_eq!(whole[3], THE_SESSION);
    assert_eq!(
        the_start_of_a_playback(&whole),
        TheStartOfAPlayback::ItStartsAt(THE_PLACE)
    );

    // **The place of a server that gave no place is not the place 0.** The
    // program does not have that place, therefore it asks the server for it.
    let no_place = collect_info_item(&without_the_field("currentTime"), &subtitle);

    assert_eq!(
        the_place_of_the_session(&no_place[0]),
        None,
        "a place that the server did not give is not the place 0: {:?}",
        no_place[0]
    );
    assert_eq!(
        the_start_of_a_playback(&no_place),
        TheStartOfAPlayback::TheProgramAsksForThePlace,
        "the program must ask the server for a place that the answer did not hold"
    );

    // A text of no number says the same thing, and it never says 0.
    assert_eq!(the_place_of_the_session(""), None);
    assert_eq!(the_place_of_the_session("   "), None);
    assert_eq!(the_place_of_the_session("N/A"), None);
    assert_eq!(the_place_of_the_session("0"), Some(0.0));

    // **A session that the server did not name is no session.** The program
    // cannot sync it, and it cannot close it.
    let no_name = collect_info_item(&without_the_field("id"), &subtitle);

    assert!(
        no_name[3].is_empty(),
        "a session with no identity must hold no name: {:?}",
        no_name[3]
    );
    assert_eq!(
        the_start_of_a_playback(&no_name),
        TheStartOfAPlayback::TheSessionHasNoIdentity,
        "a playback of a session with no name syncs nothing and closes nothing"
    );
    // The place of that answer says nothing: a session that the program cannot
    // name gives no playback at all.
    assert_eq!(
        the_start_of_a_playback(&collect_info_item(&without_the_field("id"), &subtitle)),
        TheStartOfAPlayback::TheSessionHasNoIdentity
    );

    // The place of the second read: the status 404 is the answer of a media
    // that never played, and every other fault stops the playback.
    assert_eq!(
        the_place_of_a_media_that_never_played(&ApiError::NotFound),
        Some(0.0)
    );
    assert_eq!(
        the_place_of_a_media_that_never_played(&ApiError::Unauthorized),
        None
    );

    // The path of that second read names the episode of a podcast.
    assert_eq!(
        the_path_of_the_place(THE_MEDIA, None),
        format!("/api/me/progress/{}", THE_MEDIA)
    );
    assert_eq!(
        the_path_of_the_place(THE_MEDIA, Some("an-episode")),
        format!("/api/me/progress/{}/an-episode", THE_MEDIA)
    );

    // Each of the two faults says one sentence to the user. The measurement
    // read no word at all.
    assert_eq!(
        the_words_of_a_playback_that_did_not_start(WhyNot::TheSessionHasNoIdentity),
        "The session of the server has no identity."
    );
    assert_eq!(
        the_words_of_a_playback_that_did_not_start(WhyNot::ThePlaceDidNotCome(
            "The server does not have this item."
        )),
        "The server did not give the place of this media: The server does not have this item."
    );
}
