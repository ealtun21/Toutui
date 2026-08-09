//! A measurement of the behaviour of `rodio::Player::get_pos`.
//!
//! This module holds tests only. The tests record how `get_pos` behaves when
//! the speed is not 1.0. The calculation of the position depends on this
//! behaviour.

#[cfg(test)]
mod tests {
    use rodio::buffer::SamplesBuffer;
    use rodio::Player;
    use std::num::NonZero;

    /// The value of each sample of the test sound. The value is not zero.
    /// Therefore a test can find the end of the sound, because the queue
    /// gives silence after the end.
    const LEVEL: f32 = 0.5;

    /// The sample rate of the test sound. 8000 samples give 1 second.
    const RATE: u32 = 8000;

    /// The largest number of samples that a test reads.
    ///
    /// `Player::new` makes a queue that stays alive when it is empty.
    /// Therefore the output gives silence for ever, and a test must never
    /// read to the end.
    const LIMIT: usize = 40_000;

    /// Makes a sound of one second. The sound has one channel.
    ///
    /// `rodio` gives the channel count and the sample rate as `NonZero`
    /// values. Therefore the test cannot give a plain integer.
    fn one_second() -> SamplesBuffer {
        SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(RATE).unwrap(),
            vec![LEVEL; RATE as usize],
        )
    }

    /// Reads the output and gives the number of samples of the sound.
    ///
    /// The function counts the samples before the first silence. The queue
    /// gives silence after the sound, thus the first silence is the end.
    fn count_sound(output: &mut dyn Iterator<Item = f32>) -> usize {
        let mut count = 0;

        for sample in output.take(LIMIT) {
            if sample == 0.0 {
                break;
            }
            count += 1;
        }

        count
    }

    /// A sound of 1 second at the normal speed gives 8000 samples.
    #[test]
    fn a_normal_speed_gives_the_full_number_of_samples() {
        let (player, mut output) = Player::new();
        player.append(one_second());
        player.play();

        let samples = count_sound(&mut output);
        assert!(
            (7900..=8100).contains(&samples),
            "the normal speed gives approximately 8000 samples, but it gave {}",
            samples
        );
    }

    /// A speed of 2.0 keeps the number of samples.
    ///
    /// A measurement on 2026-08-10 gives 8000 samples at the speed 2.0, and
    /// not 4000. `rodio` does not remove samples for a higher speed. It
    /// increases the sample rate that the source reports. The sound card then
    /// plays the same samples in half the time.
    #[test]
    fn a_double_speed_keeps_the_number_of_samples() {
        let (player, mut output) = Player::new();
        player.set_speed(2.0);
        player.append(one_second());
        player.play();

        let samples = count_sound(&mut output);
        assert!(
            (7900..=8100).contains(&samples),
            "a speed of 2.0 keeps approximately 8000 samples, but it gave {}",
            samples
        );
    }

    /// This is the measurement that the engine needs.
    ///
    /// A measurement on 2026-08-10 gives 0.5 seconds for a sound of 1.0
    /// second at the speed 2.0. Therefore `get_pos` gives the time of the
    /// listener, and it does not give the position in the media. The function
    /// `media_position` multiplies by the speed for this reason.
    ///
    /// If a later version of `rodio` changes this behaviour, this test fails.
    /// Then `media_position` must change too.
    #[test]
    fn get_pos_gives_the_time_of_the_listener() {
        let (player, mut output) = Player::new();
        player.set_speed(2.0);
        player.append(one_second());
        player.play();

        let _ = count_sound(&mut output);
        let position = player.get_pos().as_secs_f64();

        assert!(
            (0.4..=0.6).contains(&position),
            "get_pos must give approximately 0.5 seconds for 1.0 second of \
             media at the speed 2.0, but it gave {}",
            position
        );
    }

    /// The position and the speed together give the position in the media.
    #[test]
    fn the_measured_position_gives_the_correct_media_position() {
        let (player, mut output) = Player::new();
        player.set_speed(2.0);
        player.append(one_second());
        player.play();

        let _ = count_sound(&mut output);
        let media = crate::player::engine::media_position(player.get_pos(), player.speed());

        assert!(
            (0.9..=1.1).contains(&media),
            "the media position must be approximately 1.0 second, but it is {}",
            media
        );
    }
}
