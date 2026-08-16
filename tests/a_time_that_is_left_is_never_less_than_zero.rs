//! **A time that is left of less than zero is no time at all.** See T-289.
//!
//! `convert_seconds_for_prg` writes the time that is left of every panel of a
//! line of this program, and it took the difference of the two numbers of its
//! caller with no guard at all. Two roads of the real program gave a negative
//! difference:
//!
//! - **a length that the server did not give is not a length of 0** (T-180):
//!   the three callers of `src/app.rs` read the length with
//!   `length.unwrap_or(0.0)`, and a media of no length therefore gave
//!   `0 - the place of the user`;
//! - **the place of the user can stand past the length of the media**: the
//!   `duration` of the server and the `duration` of the audio file do not
//!   agree.
//!
//! The measurement of 2026-08-16, of the real program v0.8.117 inside tmux
//! against the sandbox, of the view of the episodes of `Arthur Gordon Pym`:
//!
//! ```text
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 0 - Duration: N/A
//! Progress: 22%, -1m left, Not finished
//!
//! [Arthur Gordon Pym] - Author: LibriVox - Episode: 2 - Duration: 39m
//! Progress: 100%, -1h-1m left, Finished
//! ```
//!
//! The first panel names a length that the program does not have, and it then
//! says a time made of that same length. The second one holds the fault of the
//! form too: `/` and `%` of Rust go toward zero, therefore -61 minutes gave
//! `-1h` and `-1m` together, and **the program says a time in one form**.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module (the shape of T-144 and of T-157).
//! This test needs no such slot, and the rule of the file stays.

use toutui::utils::convert_seconds::convert_seconds_for_prg;

#[test]
fn a_time_that_is_left_is_never_less_than_zero() {
    // **A length that the program does not have says no time at all.** The
    // callers of `src/app.rs` give 0 for a media of no length (T-180), and the
    // panel of that line says `Duration: N/A` beside it.
    assert_eq!(
        convert_seconds_for_prg(0.0, 66.0),
        "",
        "a length of 0 is a length that the server did not give, and it holds no time that is left"
    );
    assert_eq!(convert_seconds_for_prg(-1.0, 66.0), "");
    assert_eq!(convert_seconds_for_prg(f64::NAN, 66.0), "");

    // **A place past the length of the media says that no time is left.** The
    // two neighbours of this function hold that rule already:
    // `the_left_of_the_row` of the row of the player takes `saturating_sub`,
    // and `the_time_of_the_line` of the view of the queue writes the difference
    // inside a guard of `place < length`.
    assert_eq!(
        convert_seconds_for_prg(2336.731429, 6000.0),
        "0m left,",
        "a place past the length of the media holds no time that is left"
    );
    assert_eq!(convert_seconds_for_prg(305.71102, 336.0), "0m left,");

    // No text of this function holds the character of a number that is less
    // than zero, on any road of the two numbers of a caller.
    for length in [0.0, 1.0, 60.0, 305.71102, 2336.731429, 30000.0] {
        for place in [0.0, 1.0, 66.0, 2070.0, 6000.0, 100000.0] {
            let text = convert_seconds_for_prg(length, place);
            assert!(
                !text.contains('-'),
                "the time that is left of a length of {length} and of a place of {place} says {text:?}"
            );
        }
    }

    // The control of the same run: the two numbers of the panel of `Chapter 02`
    // of that same measurement, with the place of the user of the sandbox. The
    // real program said `Progress: 89%, 4m left, Not finished`.
    assert_eq!(convert_seconds_for_prg(2336.731429, 2070.0), "4m left,");
    assert_eq!(convert_seconds_for_prg(1319.601633, 1000.0), "5m left,");

    // A place of 0 is a playback that did not begin, and the panel of it says
    // no time. That rule stood before this one, and it stays.
    assert_eq!(convert_seconds_for_prg(2336.731429, 0.0), "");

    // A time of more than one hour keeps the two parts of its form.
    assert_eq!(convert_seconds_for_prg(7200.0, 60.0), "1h59m left,");
    assert_eq!(convert_seconds_for_prg(7260.0, 60.0), "2h left,");
}
