//! A timer for sleep. See T-24.
//!
//! A person who listens in bed asks for this, and every other client of
//! Audiobookshelf has it. The server holds no timer: the work is in the
//! client.
//!
//! The key `t` moves through the choices: 5, 10, 15, 30, 45, and 60 minutes,
//! the end of the chapter, and then off. The volume falls slowly in the last
//! 30 seconds, and the playback then pauses.
//!
//! **The timer measures the time of the clock, and not the time of the book.**
//! A speed of 2.0 plays 60 minutes of a book in 30 minutes of the clock, and
//! the user who says "in 30 minutes" means the clock. The choice "the end of
//! the chapter" reads the book, therefore it divides by the speed.
//!
//! The functions that calculate are pure, and a test gives them the time.

use crate::player::engine::PlaybackStatus;
use std::time::{Duration, Instant};

/// The choices of the key, in the sequence of the key.
///
/// The number is a number of minutes. `0` means the end of the chapter.
pub const CHOICES: [u64; 7] = [5, 10, 15, 30, 45, 60, 0];

/// The time of the fall of the volume, before the pause.
pub const FADE: Duration = Duration::from_secs(30);

/// The state of the timer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timer {
    /// The moment of the pause.
    pub ends_at: Instant,
    /// The volume of the user, before the fall.
    pub volume: f32,
    /// The playback that the timer belongs to. A different media stops the
    /// timer: the user asked for sleep during that book, and not during the
    /// next one.
    pub playback_id: u64,
    /// The name of the choice, for the screen.
    pub label: &'static str,
}

/// What the tick asks of the caller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// The timer does nothing now.
    Nothing,
    /// The volume must take this value. The value is between 0 and the volume
    /// of the user.
    Volume(f32),
    /// The time came. The caller pauses the playback and it puts the volume
    /// of the user back.
    Sleep(f32),
    /// The timer has no meaning now: the playback stopped, or a different
    /// media plays. The caller forgets the timer and it puts this volume
    /// back.
    Off(f32),
}

/// Gives the choice after this one.
///
/// `None` is "off". The sequence ends at "off" again, therefore the key can
/// stop the timer with no second key.
pub fn next_choice(now: Option<u64>) -> Option<u64> {
    match now {
        None => Some(CHOICES[0]),
        Some(value) => {
            let place = CHOICES.iter().position(|one| *one == value)?;
            CHOICES.get(place + 1).copied()
        }
    }
}

/// Gives the name of a choice for the screen.
pub fn label_of(minutes: u64) -> &'static str {
    match minutes {
        5 => "5 minutes",
        10 => "10 minutes",
        15 => "15 minutes",
        30 => "30 minutes",
        45 => "45 minutes",
        60 => "60 minutes",
        _ => "the end of the chapter",
    }
}

/// Gives what the caller must do now, for a state of the engine.
///
/// The user asked for sleep during one media. A playback that stopped, and a
/// media that is not that media, therefore stop the timer.
///
/// `now` is the time of the clock. The caller gives it, therefore a test needs
/// no wait.
pub fn action_for(timer: &Timer, status: PlaybackStatus, playback_id: u64, now: Instant) -> Action {
    if status == PlaybackStatus::Stopped || playback_id != timer.playback_id {
        return Action::Off(timer.volume);
    }

    action_at(timer, now)
}

/// Gives what the caller must do now, for a playback that continues.
///
/// `now` is the time of the clock. The caller gives it, therefore a test needs
/// no wait.
pub fn action_at(timer: &Timer, now: Instant) -> Action {
    if now >= timer.ends_at {
        return Action::Sleep(timer.volume);
    }

    let left = timer.ends_at - now;

    if left > FADE {
        return Action::Nothing;
    }

    // The volume falls in a straight line to zero.
    let part = left.as_secs_f32() / FADE.as_secs_f32();

    Action::Volume((timer.volume * part).clamp(0.0, timer.volume))
}

/// Gives the time that is left, for the screen.
pub fn left(timer: &Timer, now: Instant) -> Duration {
    timer.ends_at.saturating_duration_since(now)
}

/// Writes the time that is left, for the screen.
pub fn text_of(timer: &Timer, now: Instant) -> String {
    let left = left(timer, now).as_secs();

    format!("💤 {}:{:02}", left / 60, left % 60)
}

/// Gives the time of the clock that the end of a chapter needs.
///
/// The book plays at a speed, therefore 600 seconds of a book at the speed
/// 2.0 take 300 seconds of the clock. A speed that is not a number, or a
/// speed of zero, gives the time of the book.
pub fn clock_time_of(seconds_of_the_book: f64, speed: f32) -> Duration {
    let speed = if speed.is_finite() && speed > 0.0 {
        f64::from(speed)
    } else {
        1.0
    };

    let seconds = (seconds_of_the_book / speed).max(0.0);

    // A book of many hours still gives a value that a `Duration` holds.
    Duration::from_secs_f64(seconds.min(24.0 * 3600.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_timer(ends_in: Duration, now: Instant) -> Timer {
        Timer {
            ends_at: now + ends_in,
            volume: 0.8,
            playback_id: 1,
            label: "30 minutes",
        }
    }

    #[test]
    fn the_key_moves_through_every_choice_and_it_stops() {
        let mut choice = next_choice(None);
        let mut seen = Vec::new();

        for _ in 0..20 {
            match choice {
                Some(value) => {
                    seen.push(value);
                    choice = next_choice(Some(value));
                }
                None => break,
            }
        }

        assert_eq!(seen, CHOICES.to_vec());
        assert_eq!(choice, None, "the sequence must end at off");
    }

    /// A value that the program does not know must not hold the key.
    #[test]
    fn a_choice_that_does_not_exist_gives_off() {
        assert_eq!(next_choice(Some(999)), None);
    }

    #[test]
    fn every_choice_has_a_name() {
        for one in CHOICES {
            assert!(!label_of(one).is_empty());
        }

        assert_eq!(label_of(0), "the end of the chapter");
        assert_eq!(label_of(30), "30 minutes");
    }

    #[test]
    fn a_timer_of_many_minutes_does_nothing_now() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(600), now);

        assert_eq!(action_at(&timer, now), Action::Nothing);
    }

    /// The volume falls in the last 30 seconds, and it does not fall before.
    #[test]
    fn the_volume_falls_in_the_last_thirty_seconds() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(30), now);

        // A number of 32 bits does not hold every value exactly, therefore
        // the test accepts a small difference.
        let volume_at = |seconds| match action_at(&timer, now + Duration::from_secs(seconds)) {
            Action::Volume(value) => value,
            other => panic!("the volume must fall: {:?}", other),
        };

        assert!((volume_at(1) - 0.8 * 29.0 / 30.0).abs() < 0.001);
        assert!((volume_at(15) - 0.4).abs() < 0.01);
        assert!(volume_at(29) < volume_at(15));
    }

    #[test]
    fn the_time_gives_the_sleep_and_the_volume_of_the_user() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(30), now);

        assert_eq!(
            action_at(&timer, now + Duration::from_secs(30)),
            Action::Sleep(0.8)
        );
        assert_eq!(
            action_at(&timer, now + Duration::from_secs(300)),
            Action::Sleep(0.8)
        );
    }

    /// The volume must never go above the volume of the user, and never below
    /// zero.
    #[test]
    fn the_volume_stays_between_zero_and_the_volume_of_the_user() {
        let now = Instant::now();
        let timer = a_timer(FADE, now);

        for second in 0..30 {
            if let Action::Volume(value) = action_at(&timer, now + Duration::from_secs(second)) {
                assert!(
                    (0.0..=0.8).contains(&value),
                    "the volume {} is wrong",
                    value
                );
            }
        }
    }

    /// The user asked for sleep during one media. The timer must go away
    /// when that media stops, and when a different media plays.
    #[test]
    fn a_playback_that_stopped_stops_the_timer() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(600), now);

        assert_eq!(
            action_for(&timer, PlaybackStatus::Stopped, 1, now),
            Action::Off(0.8)
        );
        assert_eq!(
            action_for(&timer, PlaybackStatus::Playing, 2, now),
            Action::Off(0.8),
            "a different media must stop the timer"
        );
    }

    /// A pause is not a stop. A user who pauses the book and comes back must
    /// keep the timer.
    #[test]
    fn a_pause_keeps_the_timer() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(600), now);

        assert_eq!(
            action_for(&timer, PlaybackStatus::Paused, 1, now),
            Action::Nothing
        );
        assert_eq!(
            action_for(&timer, PlaybackStatus::Stalled, 1, now),
            Action::Nothing
        );
    }

    #[test]
    fn the_whole_life_of_a_timer() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(300), now);
        let of = |seconds| {
            action_for(
                &timer,
                PlaybackStatus::Playing,
                1,
                now + Duration::from_secs(seconds),
            )
        };

        assert_eq!(of(0), Action::Nothing);
        assert_eq!(of(269), Action::Nothing);
        assert!(matches!(of(280), Action::Volume(_)));
        assert!(matches!(of(299), Action::Volume(_)));
        assert_eq!(of(300), Action::Sleep(0.8));
        assert_eq!(of(400), Action::Sleep(0.8));
    }

    #[test]
    fn the_screen_shows_the_time_that_is_left() {
        let now = Instant::now();
        let timer = a_timer(Duration::from_secs(754), now);

        assert_eq!(text_of(&timer, now), "💤 12:34");
        assert_eq!(
            text_of(&timer, now + Duration::from_secs(1000)),
            "💤 0:00",
            "a timer that passed must show no time below zero"
        );
    }

    /// A speed of 2.0 plays 600 seconds of a book in 300 seconds of the
    /// clock.
    #[test]
    fn the_speed_changes_the_time_of_the_clock() {
        assert_eq!(clock_time_of(600.0, 1.0), Duration::from_secs(600));
        assert_eq!(clock_time_of(600.0, 2.0), Duration::from_secs(300));
        assert_eq!(clock_time_of(600.0, 0.5), Duration::from_secs(1200));
    }

    #[test]
    fn a_speed_that_is_not_valid_gives_the_time_of_the_book() {
        assert_eq!(clock_time_of(600.0, 0.0), Duration::from_secs(600));
        assert_eq!(clock_time_of(600.0, -1.0), Duration::from_secs(600));
        assert_eq!(clock_time_of(600.0, f32::NAN), Duration::from_secs(600));
    }

    #[test]
    fn a_chapter_that_ended_gives_no_time() {
        assert_eq!(clock_time_of(-30.0, 1.0), Duration::from_secs(0));
    }

    /// A book of many days must not make a `Duration` that the program cannot
    /// hold.
    #[test]
    fn a_very_long_time_stays_inside_one_day() {
        assert!(clock_time_of(f64::MAX, 1.0) <= Duration::from_secs(24 * 3600));
    }
}
