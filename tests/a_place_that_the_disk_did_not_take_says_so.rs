//! A place of a playback that the disk did not take says so. See T-210.
//!
//! **The parts of this test stay in one function**: the box of the process holds
//! one value for the whole binary, and two test functions of one module fight
//! for that slot (T-144 and T-157).
//!
//! The measurement of 2026-08-14, of the real program of the sandbox: the server
//! was away, the database of the account took no write (`chmod 444`), and the
//! program played a book of eight hours from the disk. The row of the player said
//! `▶ 53:42 / 8:00:00`, the row of the message said nothing, and the log held
//! **one line for each second** of the playback — 25 lines in 25 seconds, and
//! 28800 lines for the whole book. The place of the user reached neither the
//! server nor the disk, and no word of the screen said it.

use toutui::logic::playback::the_place_of_the_disk::{
    the_disk_says, the_notice, the_notice_of_the_player, THE_DISK_TAKES_NO_PLACE,
};

#[test]
fn the_row_of_the_player_says_that_the_disk_keeps_no_place() {
    // The words of the engine stand alone while the disk takes the place.
    assert_eq!(the_notice(None, false), None);
    assert_eq!(
        the_notice(Some("Reconnected".to_string()), false),
        Some("Reconnected".to_string()),
    );

    // **A word that names work that stands must live while that work stands.**
    // A message of the program lives six seconds, and this condition stands for
    // the whole playback: the row of the player therefore holds the word.
    assert_eq!(
        the_notice(None, true),
        Some(THE_DISK_TAKES_NO_PLACE.to_string()),
    );

    // The engine holds its own word, and the two of them stand together.
    let both = the_notice(Some("Reconnected".to_string()), true).expect("the two words");

    assert!(both.contains("Reconnected"), "the word of the engine stays");
    assert!(
        both.contains(THE_DISK_TAKES_NO_PLACE),
        "the word of the disk comes beside it"
    );

    // The box of the process: the loop of the playback writes it, and the render
    // reads it. The render reads no disk (T-204).
    assert_eq!(the_notice_of_the_player(None), None);

    the_disk_says(true);
    assert_eq!(
        the_notice_of_the_player(None),
        Some(THE_DISK_TAKES_NO_PLACE.to_string()),
    );

    // A write that the disk took again takes the word away.
    the_disk_says(false);
    assert_eq!(the_notice_of_the_player(None), None);

    // **A write of a loop of a playback says one line of the log** (T-207).
    // `keep_progress` wrote a line of its own for each second, therefore the
    // rule of the loop reached that write no more.
    let of_the_offline = include_str!("../src/logic/offline/mod.rs");
    let start = of_the_offline
        .find("pub fn keep_progress(")
        .expect("the function that keeps the place for the server");
    let body = &of_the_offline[start..start + 700];

    assert!(
        !body.contains("warn!"),
        "keep_progress must write no line of the log: the loop of an offline \
         playback calls it at each second, and the caller says the fault one \
         time. See T-210."
    );

    // The loop of the offline playback reads the answer of that write.
    let of_the_playback = include_str!("../src/logic/playback/mod.rs");
    let start = of_the_playback
        .find("pub async fn follow_playback_offline(")
        .expect("the loop of the offline playback");
    let loop_of_the_disk = &of_the_playback[start..];

    assert!(
        loop_of_the_disk.contains("the_disk_said_nothing_of_the_place_that_waits"),
        "the loop must read the answer of the write of the place that waits for \
         the server: that row is the one copy of an offline playback (T-152), \
         and the old code threw the answer away. See T-210."
    );

    assert!(
        loop_of_the_disk.contains("the_place_of_the_disk::the_disk_says("),
        "the loop must give the word of the disk to the row of the player. \
         See T-210."
    );
}
