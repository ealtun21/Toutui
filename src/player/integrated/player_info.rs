use log::info;
use crate::db::crud::*;

pub fn player_info(username: &str) -> Vec<String> {
    let mut player_info = Vec::new();

    match get_listening_session() {
        Ok(Some(session)) => {
            player_info.push(session.title);
            player_info.push(session.author);

            if let Ok(num) = session.chapter.trim().parse::<u32>() {
                let new_chapter = format!("Chapter {}", num + 1);
                player_info.push(new_chapter);
            } else {
                player_info.push(session.chapter.clone()); 
            }

            player_info.push(session.is_playback.to_string());
            player_info.push(format_time(session.current_time));

            let speed_rate_str = get_speed_rate(username);
            let speed_rate: f32 = speed_rate_str.parse().unwrap_or(1.0);
            let original_duration = session.duration.parse::<u32>().unwrap_or(0);
            let adjusted_duration = (original_duration as f32 / speed_rate) as u32;
            player_info.push(format_time(adjusted_duration)); 

            let remaining_time = adjusted_duration.saturating_sub(session.current_time);
            player_info.push(format_time(session.elapsed_time));
            player_info.push(format_time(remaining_time)); 

            player_info.push(format!("{}", progress_percent(session.current_time, adjusted_duration)));
        }
        Ok(None) => {
            player_info.push("N/A".to_string());
        }
        Err(e) => {
            player_info.push("Error".to_string());
            info!("[player_info] Error retrieving data: {}", e);
        }
    }

    player_info.push(get_speed_rate(username));

    player_info
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
