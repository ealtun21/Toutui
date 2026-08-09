//! The speed of the playback, with no change of the pitch.
//!
//! `rodio::Player::set_speed` does not stretch the time. It increases the
//! sample rate that the source reports, thus the pitch increases with the
//! speed. A voice then sounds too high. An audiobook player must not do this,
//! because a user listens for many hours.
//!
//! This module uses WSOLA (Waveform Similarity Overlap-Add). WSOLA stretches
//! the time and keeps the pitch.
//!
//! The queue of `rodio` owns the source, and `Wsola::set_speed` needs a
//! mutable reference. Therefore the source reads a value that the engine and
//! the source share. The engine writes the value, and the source reads it.

use rodio::source::SeekError;
use rodio::Source;
use rodio_wsola::Wsola;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// The speed that the engine and the source share.
///
/// The value is an `f32` in the form of its bits, because an atomic `f32`
/// does not exist in the standard library.
#[derive(Debug, Clone)]
pub struct SharedSpeed(Arc<AtomicU32>);

impl SharedSpeed {
    /// Makes a shared speed.
    pub fn new(speed: f32) -> Self {
        SharedSpeed(Arc::new(AtomicU32::new(safe(speed).to_bits())))
    }

    /// Reads the speed.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    /// Writes the speed. The source reads the new value on the next sample.
    pub fn set(&self, speed: f32) {
        self.0.store(safe(speed).to_bits(), Ordering::Relaxed);
    }
}

impl Default for SharedSpeed {
    fn default() -> Self {
        SharedSpeed::new(1.0)
    }
}

/// Keeps a speed in the permitted limits.
///
/// A speed of zero or a value that is not a number stops the sound. Therefore
/// the function gives 1.0 for such a value.
fn safe(speed: f32) -> f32 {
    if !speed.is_finite() || speed <= 0.0 {
        return 1.0;
    }

    speed.clamp(0.1, 5.0)
}

/// A source that stretches the time and reads its speed from a shared value.
///
/// `Wsola` gives the same behaviour as the speed of `rodio` for the position:
/// `try_seek` multiplies by the speed, and the position that the player
/// reports is the time of the listener. Therefore the calculation of the
/// position in `crate::player::engine` does not change.
pub struct SpeedSource<I>
where
    I: Source,
{
    inner: Wsola<I>,
    shared: SharedSpeed,
    current: f32,
}

impl<I> SpeedSource<I>
where
    I: Source,
{
    /// Wraps a source. The source then obeys the shared speed.
    pub fn new(input: I, shared: SharedSpeed) -> Self {
        let speed = shared.get();

        SpeedSource {
            inner: Wsola::new(input, speed),
            shared,
            current: speed,
        }
    }

    /// Reads the shared value, and gives it to WSOLA if it changed.
    ///
    /// The function runs for each sample. An atomic read costs almost
    /// nothing, and a change of the speed must operate immediately.
    fn follow_shared_speed(&mut self) {
        let wanted = self.shared.get();

        if (wanted - self.current).abs() > f32::EPSILON {
            self.inner.set_speed(wanted);
            self.current = wanted;
        }
    }
}

impl<I> Iterator for SpeedSource<I>
where
    I: Source,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.follow_shared_speed();
        self.inner.next()
    }
}

impl<I> Source for SpeedSource<I>
where
    I: Source,
{
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        // Give WSOLA the speed first. `Wsola::try_seek` multiplies the
        // position by its own speed, thus a wrong speed gives a wrong
        // position.
        self.follow_shared_speed();
        self.inner.try_seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shared_speed_keeps_its_value() {
        let speed = SharedSpeed::new(1.5);
        assert!((speed.get() - 1.5).abs() < 0.001);

        speed.set(2.0);
        assert!((speed.get() - 2.0).abs() < 0.001);
    }

    /// Two copies of the shared speed show the same value. The engine writes,
    /// and the source reads.
    #[test]
    fn a_copy_of_the_shared_speed_reads_the_new_value() {
        let speed = SharedSpeed::new(1.0);
        let copy = speed.clone();

        speed.set(1.8);
        assert!((copy.get() - 1.8).abs() < 0.001);
    }

    /// A speed of zero stops the sound. A value that is not a number does the
    /// same. Therefore the value becomes 1.0.
    #[test]
    fn a_speed_that_is_not_valid_becomes_one() {
        assert!((safe(0.0) - 1.0).abs() < 0.001);
        assert!((safe(-2.0) - 1.0).abs() < 0.001);
        assert!((safe(f32::NAN) - 1.0).abs() < 0.001);
        assert!((safe(f32::INFINITY) - 1.0).abs() < 0.001);
    }

    #[test]
    fn a_speed_stays_in_the_limits() {
        assert!((safe(10.0) - 5.0).abs() < 0.001);
        assert!((safe(0.01) - 0.1).abs() < 0.001);
    }

    #[test]
    fn the_default_speed_is_one() {
        assert!((SharedSpeed::default().get() - 1.0).abs() < 0.001);
    }
}
