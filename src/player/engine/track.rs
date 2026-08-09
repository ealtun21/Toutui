//! The audio files of a book, and the calculation of the position.
//!
//! A book can have more than one audio file. The test library has a book with
//! 209 audio files. The player reports the position in the file that plays
//! now. This module changes that value to the position in the whole book.

/// One audio file of a book.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// The sequence of the file. The first file has the number 1.
    pub index: u32,
    /// The identity of the file on the server.
    pub ino: String,
    /// The name of the file. The decoder uses the extension as a hint.
    pub filename: String,
    /// The length of the file in seconds.
    pub duration: f64,
    /// The start of this file in the book, in seconds.
    pub start_offset: f64,
}

/// One chapter of a book. The values come from the field `media.chapters` of
/// the API.
#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    /// The start in the book, in seconds.
    pub start: f64,
    /// The end in the book, in seconds.
    pub end: f64,
    /// The name of the chapter.
    pub title: String,
}

/// A movement to the previous chapter goes to the chapter before the current
/// chapter if the position is less than this number of seconds after the
/// start. This behaviour agrees with the usual behaviour of an audio player.
const PREVIOUS_CHAPTER_LIMIT: f64 = 3.0;

/// The audio files of one book, and the chapters of that book.
#[derive(Debug, Clone, Default)]
pub struct TrackList {
    tracks: Vec<Track>,
    chapters: Vec<Chapter>,
    total: f64,
}

impl TrackList {
    /// Makes a track list.
    ///
    /// The function calculates `start_offset` for each track. The caller gives
    /// the tracks in the correct sequence.
    pub fn new(mut tracks: Vec<Track>, chapters: Vec<Chapter>) -> Self {
        let mut total = 0.0;

        for track in tracks.iter_mut() {
            track.start_offset = total;
            total += track.duration.max(0.0);
        }

        TrackList {
            tracks,
            chapters,
            total,
        }
    }

    /// Makes tracks from a list of lengths. The tests use this function.
    pub fn from_durations(durations: &[f64]) -> Vec<Track> {
        durations
            .iter()
            .enumerate()
            .map(|(position, duration)| Track {
                index: position as u32 + 1,
                ino: format!("ino-{}", position),
                filename: format!("{:03}.mp3", position + 1),
                duration: *duration,
                start_offset: 0.0,
            })
            .collect()
    }

    /// Gives the length of the whole book in seconds.
    pub fn total_duration(&self) -> f64 {
        self.total
    }

    /// Gives the track and the offset in that track for a position in the
    /// book.
    ///
    /// A position before the start gives the first track. A position at the
    /// end or after the end gives the last track. Gives `None` only if the
    /// book has no track.
    pub fn locate(&self, position: f64) -> Option<(usize, f64)> {
        if self.tracks.is_empty() {
            return None;
        }

        if position <= 0.0 || position.is_nan() {
            return Some((0, 0.0));
        }

        for (number, track) in self.tracks.iter().enumerate() {
            let end = track.start_offset + track.duration;

            if position < end {
                return Some((number, position - track.start_offset));
            }
        }

        let last = self.tracks.len() - 1;
        let offset = (position - self.tracks[last].start_offset).min(self.tracks[last].duration);

        Some((last, offset))
    }

    /// Gives the position in the book for a track and an offset.
    pub fn position_of(&self, track_index: usize, offset: f64) -> f64 {
        match self.tracks.get(track_index) {
            Some(track) => track.start_offset + offset,
            None => offset,
        }
    }

    /// Gives the chapter that holds a position.
    ///
    /// Gives `None` if the book has no chapter. 118 books of the test library
    /// have no chapter.
    pub fn chapter_at(&self, position: f64) -> Option<&Chapter> {
        self.chapters
            .iter()
            .find(|chapter| position >= chapter.start && position < chapter.end)
    }

    /// Gives the start of the chapter after the current position.
    pub fn next_chapter_start(&self, position: f64) -> Option<f64> {
        self.chapters
            .iter()
            .map(|chapter| chapter.start)
            .find(|start| *start > position)
    }

    /// Gives the position for a movement to the previous chapter.
    ///
    /// The movement goes to the start of the current chapter. If the position
    /// is less than three seconds after that start, the movement goes to the
    /// chapter before it.
    pub fn previous_chapter_start(&self, position: f64) -> Option<f64> {
        let current = self.chapter_at(position)?;

        if position - current.start > PREVIOUS_CHAPTER_LIMIT {
            return Some(current.start);
        }

        let earlier = self
            .chapters
            .iter()
            .map(|chapter| chapter.start)
            .rfind(|start| *start < current.start);

        Some(earlier.unwrap_or(current.start))
    }

    /// Gives one track.
    pub fn get(&self, track_index: usize) -> Option<&Track> {
        self.tracks.get(track_index)
    }

    /// Gives the number of tracks.
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Tells if the book has no track.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three tracks of 10, 20, and 30 seconds. The book lasts 60 seconds.
    fn list() -> TrackList {
        TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), Vec::new())
    }

    /// Chapters at 0 to 25, 25 to 45, and 45 to 60.
    fn list_with_chapters() -> TrackList {
        let chapters = vec![
            Chapter {
                start: 0.0,
                end: 25.0,
                title: "One".to_string(),
            },
            Chapter {
                start: 25.0,
                end: 45.0,
                title: "Two".to_string(),
            },
            Chapter {
                start: 45.0,
                end: 60.0,
                title: "Three".to_string(),
            },
        ];
        TrackList::new(TrackList::from_durations(&[10.0, 20.0, 30.0]), chapters)
    }

    #[test]
    fn the_start_offset_is_the_sum_of_the_tracks_before_it() {
        let list = list();
        assert_eq!(list.get(0).unwrap().start_offset, 0.0);
        assert_eq!(list.get(1).unwrap().start_offset, 10.0);
        assert_eq!(list.get(2).unwrap().start_offset, 30.0);
    }

    #[test]
    fn the_total_duration_is_the_sum_of_the_tracks() {
        assert_eq!(list().total_duration(), 60.0);
    }

    #[test]
    fn a_position_gives_the_correct_track_and_offset() {
        let list = list();
        assert_eq!(list.locate(0.0).unwrap(), (0, 0.0));
        assert_eq!(list.locate(5.0).unwrap(), (0, 5.0));
        assert_eq!(list.locate(15.0).unwrap(), (1, 5.0));
        assert_eq!(list.locate(45.0).unwrap(), (2, 15.0));
    }

    /// The first second of a track belongs to that track, and not to the
    /// track before it.
    #[test]
    fn the_boundary_belongs_to_the_track_that_starts() {
        let list = list();
        assert_eq!(list.locate(10.0).unwrap(), (1, 0.0));
        assert_eq!(list.locate(30.0).unwrap(), (2, 0.0));
    }

    #[test]
    fn a_position_before_the_start_gives_the_first_track() {
        assert_eq!(list().locate(-5.0).unwrap(), (0, 0.0));
    }

    /// A position at the end or after the end gives the last track. The
    /// engine must not fail when the book comes to the end.
    #[test]
    fn a_position_at_the_end_gives_the_last_track() {
        let list = list();
        assert_eq!(list.locate(60.0).unwrap().0, 2);
        assert_eq!(list.locate(1000.0).unwrap().0, 2);
    }

    #[test]
    fn an_empty_list_gives_no_position() {
        let list = TrackList::new(Vec::new(), Vec::new());
        assert!(list.locate(0.0).is_none());
        assert!(list.is_empty());
    }

    #[test]
    fn the_position_of_a_track_and_an_offset_is_correct() {
        let list = list();
        assert_eq!(list.position_of(0, 5.0), 5.0);
        assert_eq!(list.position_of(1, 5.0), 15.0);
        assert_eq!(list.position_of(2, 15.0), 45.0);
    }

    /// This is the test for a book with many files. The test library has a
    /// book with 209 audio files.
    #[test]
    fn a_book_with_209_tracks_gives_the_correct_position() {
        let durations = vec![300.0; 209];
        let list = TrackList::new(TrackList::from_durations(&durations), Vec::new());

        assert_eq!(list.len(), 209);
        assert_eq!(list.total_duration(), 62700.0);
        assert_eq!(list.get(208).unwrap().start_offset, 62400.0);
        assert_eq!(list.locate(62500.0).unwrap(), (208, 100.0));
    }

    #[test]
    fn the_chapter_at_a_position_is_correct() {
        let list = list_with_chapters();
        assert_eq!(list.chapter_at(0.0).unwrap().title, "One");
        assert_eq!(list.chapter_at(24.9).unwrap().title, "One");
        assert_eq!(list.chapter_at(25.0).unwrap().title, "Two");
        assert_eq!(list.chapter_at(50.0).unwrap().title, "Three");
    }

    #[test]
    fn the_next_chapter_gives_the_start_of_the_next_chapter() {
        let list = list_with_chapters();
        assert_eq!(list.next_chapter_start(0.0).unwrap(), 25.0);
        assert_eq!(list.next_chapter_start(30.0).unwrap(), 45.0);
    }

    #[test]
    fn the_last_chapter_has_no_next_chapter() {
        assert!(list_with_chapters().next_chapter_start(50.0).is_none());
    }

    /// The rule agrees with the usual behaviour of an audio player. The first
    /// operation goes to the start of the current chapter.
    #[test]
    fn the_previous_chapter_goes_to_the_start_of_the_current_chapter() {
        let list = list_with_chapters();
        assert_eq!(list.previous_chapter_start(30.0).unwrap(), 25.0);
    }

    /// If the position is less than 3 seconds after the start of the chapter,
    /// the operation goes to the chapter before it.
    #[test]
    fn the_previous_chapter_goes_back_near_the_start_of_a_chapter() {
        let list = list_with_chapters();
        assert_eq!(list.previous_chapter_start(26.0).unwrap(), 0.0);
    }

    #[test]
    fn the_first_chapter_gives_the_start_of_the_book() {
        let list = list_with_chapters();
        assert_eq!(list.previous_chapter_start(1.0).unwrap(), 0.0);
    }

    /// 118 books of the test library have no chapter. The two commands must
    /// do nothing for those books.
    #[test]
    fn a_book_with_no_chapter_gives_no_chapter_movement() {
        let list = list();
        assert!(list.chapter_at(10.0).is_none());
        assert!(list.next_chapter_start(10.0).is_none());
        assert!(list.previous_chapter_start(10.0).is_none());
    }
}
