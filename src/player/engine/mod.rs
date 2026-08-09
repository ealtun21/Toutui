//! The audio engine of the application.
//!
//! The engine decodes the audio in the process. The application does not start
//! a different program. Therefore the token stays in the memory of the
//! process.

pub mod http_file;
pub mod pos_probe;
pub mod track;

use std::time::Duration;

/// Changes the position that `rodio` reports to the position in the media.
///
/// `rodio::Player::get_pos` gives the position of the sound that plays now.
/// The test module `pos_probe` measures the behaviour of that function when
/// the speed is not 1.0. The engine must report the position in the media to
/// the server, because the server counts the seconds of the recording and not
/// the seconds of the listener.
pub fn media_position(reported: Duration, speed: f32) -> f64 {
    let seconds = reported.as_secs_f64();

    if speed <= 0.0 || !speed.is_finite() {
        return seconds;
    }

    // A measurement on 2026-08-10 shows that `get_pos` gives the time of the
    // listener. A sound of 1.0 second at the speed 2.0 gives 0.5 seconds.
    // The server counts the seconds of the recording. Therefore the function
    // multiplies by the speed. The test module `pos_probe` holds the
    // measurement.
    seconds * speed as f64
}

#[cfg(test)]
mod tests {
    use super::media_position;
    use std::time::Duration;

    #[test]
    fn a_normal_speed_does_not_change_the_position() {
        let position = media_position(Duration::from_secs(30), 1.0);
        assert!((position - 30.0).abs() < 0.001);
    }

    /// `get_pos` gives the time of the listener. A listener that uses the
    /// speed 2.0 for 30 seconds hears 60 seconds of the recording. The server
    /// must get the value 60.
    #[test]
    fn a_double_speed_gives_the_position_in_the_media() {
        let position = media_position(Duration::from_secs(30), 2.0);
        assert!((position - 60.0).abs() < 0.001);
    }

    /// A speed that is less than 1.0 gives a position that is smaller than
    /// the time of the listener.
    #[test]
    fn a_slow_speed_gives_a_smaller_position() {
        let position = media_position(Duration::from_secs(30), 0.5);
        assert!((position - 15.0).abs() < 0.001);
    }

    /// A speed of zero, or a speed that is not a number, must not give a
    /// position of zero and must not give an infinite value.
    #[test]
    fn a_speed_that_is_not_valid_gives_the_reported_position() {
        assert!((media_position(Duration::from_secs(30), 0.0) - 30.0).abs() < 0.001);
        assert!((media_position(Duration::from_secs(30), -1.0) - 30.0).abs() < 0.001);
        assert!((media_position(Duration::from_secs(30), f32::NAN) - 30.0).abs() < 0.001);
    }
}
