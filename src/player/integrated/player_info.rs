use crate::db::crud::get_speed_rate;
use crate::player::engine::{PlaybackState, PlaybackStatus};

/// Gives the values that the player panel shows.
///
/// The engine gives the position and the length. Both values belong to the
/// whole book, and not to one audio file. See T-2.
///
/// The list always holds ten values. Therefore the panel can read each
/// position with no examination.
pub fn player_info(username: &str, state: &PlaybackState) -> Vec<String> {
    let position = state.position.max(0.0) as u32;
    let duration = state.duration.max(0.0) as u32;

    let chapter = state
        .chapter_title
        .clone()
        .unwrap_or_else(|| "No chapter".to_string());

    // A stall is not a pause. The user did not stop the playback, thus the
    // panel shows the sign of the playback.
    let is_playing = matches!(
        state.status,
        PlaybackStatus::Playing | PlaybackStatus::Stalled
    );

    let speed = if state.speed > 0.0 {
        state.speed
    } else {
        get_speed_rate(username).parse::<f32>().unwrap_or(1.0)
    };

    vec![
        state.title.clone(),
        state.author.clone(),
        chapter,
        is_playing.to_string(),
        format_time(position),
        format_time(duration),
        format_time(position),
        format_time(duration.saturating_sub(position)),
        format!("{}", progress_percent(position, duration)),
        format!("{:.2}", speed),
        the_volume_of_the_row(state.volume),
    ]
}

/// Gives the volume for the row of the player.
///
/// **The row says nothing at the volume of the file.** That volume is the
/// volume of almost every playback, and a row of 80 columns holds little. A
/// user who changed the volume must read it, because a media that plays and
/// gives no sound looks like a fault of the program. See T-80.
///
/// The function is pure, therefore a test needs no engine.
pub fn the_volume_of_the_row(volume: f32) -> String {
    let percent = (volume * 100.0).round() as i64;

    if percent == 100 {
        return String::new();
    }

    format!(" | Vol: {}%", percent)
}

/// Calculate the progress of the playback in percent.
/// `total_duration` is the duration of the whole book, not the duration of one
/// audio file. A book with many audio files gives a position that is larger
/// than the first file. Thus the function keeps the result between 0 and 100.
/// A total duration of zero gives 0. The function does not divide by zero.
fn progress_percent(current_time: u32, total_duration: u32) -> u32 {
    if total_duration == 0 {
        return 0;
    }
    let percent = (current_time as f64 / total_duration as f64) * 100.0;
    percent.round().clamp(0.0, 100.0) as u32
}

fn format_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}:{:02}", minutes, secs)
    } else {
        format!("0:{}", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::progress_percent;
    use super::the_volume_of_the_row;

    /// The keys `o` and `i` changed the volume, and no row of the screen moved.
    /// A sweep of 2026-08-11 found it. See T-80.
    #[test]
    fn the_row_names_the_volume_when_it_is_not_the_volume_of_the_file() {
        assert_eq!(the_volume_of_the_row(1.0), "");
        assert_eq!(the_volume_of_the_row(0.8), " | Vol: 80%");
        assert_eq!(the_volume_of_the_row(0.0), " | Vol: 0%");
        assert_eq!(the_volume_of_the_row(1.5), " | Vol: 150%");
    }

    // Real values from upstream issue #33. The book has a total duration of
    // 53764 seconds. The first audio file is 841 seconds long.
    const TOTAL_DURATION: u32 = 53764;
    const FIRST_FILE_DURATION: u32 = 841;

    #[test]
    fn whole_book_duration_gives_100_percent_at_the_end() {
        assert_eq!(progress_percent(TOTAL_DURATION, TOTAL_DURATION), 100);
    }

    #[test]
    fn first_file_duration_no_longer_gives_6393_percent() {
        // The old code divided by the duration of the first audio file. That
        // gave 6393 percent. The clamp keeps the result at 100 percent.
        assert_eq!(progress_percent(TOTAL_DURATION, FIRST_FILE_DURATION), 100);
    }

    #[test]
    fn position_zero_gives_0_percent() {
        assert_eq!(progress_percent(0, TOTAL_DURATION), 0);
    }

    #[test]
    fn zero_total_duration_gives_0_percent() {
        assert_eq!(progress_percent(TOTAL_DURATION, 0), 0);
        assert_eq!(progress_percent(0, 0), 0);
    }

    #[test]
    fn position_larger_than_total_clamps_to_100_percent() {
        assert_eq!(progress_percent(TOTAL_DURATION + 1000, TOTAL_DURATION), 100);
    }

    #[test]
    fn half_of_the_book_gives_50_percent() {
        assert_eq!(progress_percent(TOTAL_DURATION / 2, TOTAL_DURATION), 50);
    }
}
