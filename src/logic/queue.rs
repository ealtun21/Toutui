//! The queue of media. See T-24.
//!
//! The client played one media, and it then stopped. A user who wanted to hear
//! the next book of a series pressed a key for each book. The server gives no
//! endpoint for a queue: Audiobookshelf holds the queue in the web page and it
//! sends nothing. Therefore the queue belongs to this client.
//!
//! # The rules of the queue
//!
//! - The key `n` puts the selected media at the end of the queue. It does not
//!   change the media that plays.
//! - The queue starts the next media when a media comes to **its end** only. A
//!   media that the user stopped, and a media that a different playback took
//!   away, leave the queue where it is.
//! - The key `q` shows the queue. Inside that view, `l` starts a media now,
//!   and `X` takes a media out of the queue.
//!
//! # Why the list is a structure, and not a global list only
//!
//! `Queue` holds no lock and no global value. Therefore a test calls every
//! function directly, and the tests do not need a sequence. The global value
//! below is a thin box around that structure.
//!
//! The queue lives in the memory of the process. A user who stops the
//! application loses it. A queue on the disk needs a table of the database and
//! a rule for a media that the server does not hold now, and the value of that
//! work is small.

use crate::logic::playback::PlaybackTarget;
use crate::utils::convert_seconds::convert_seconds;
use std::sync::{Mutex, OnceLock};

/// One media that waits in the queue.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The book or the episode that the queue plays.
    pub target: PlaybackTarget,
    pub title: String,
    pub author: String,
    /// The length of the media, in seconds. A view that holds the length as a
    /// text only gives nothing here, and the line of the queue then shows no
    /// length. The view of the episodes of a podcast is such a view.
    pub duration: Option<f64>,
}

impl Entry {
    /// Gives the identity of the media. A book gives the identity of the item,
    /// and an episode gives the identity of the item and of the episode.
    ///
    /// Two episodes of one podcast have the same item. Therefore the identity
    /// of the item alone does not name a media.
    pub fn key(&self) -> String {
        match self.target.episode_id() {
            Some(episode_id) => format!("{}/{}", self.target.item_id(), episode_id),
            None => self.target.item_id().to_string(),
        }
    }
}

/// What happened at the end of a playback.
///
/// The loop of the playback gives this value, and the queue reads it. See
/// `crate::logic::playback::follow_playback`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The media came to its end.
    Finished,
    /// The user stopped the media, or a different playback took the engine.
    Stopped,
    /// The playback did not start.
    Fault,
}

/// Tells if the queue must start the next media.
///
/// **The queue goes on after an end, and after nothing else.** A user who
/// presses the key of a different book asks for that book, and not for the
/// queue. A user who stops the playback asks for silence. The old code had no
/// queue, therefore this rule is new, and a test holds it.
pub fn the_queue_goes_on(outcome: Outcome) -> bool {
    matches!(outcome, Outcome::Finished)
}

/// The media that wait.
#[derive(Debug, Default, Clone)]
pub struct Queue {
    entries: Vec<Entry>,
}

impl Queue {
    /// Gives the media that wait, in their sequence.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Tells if the queue holds this media already.
    pub fn holds(&self, key: &str) -> bool {
        self.entries.iter().any(|entry| entry.key() == key)
    }

    /// Puts a media at the end of the queue.
    ///
    /// The function gives the place of the media, and the first place is 1.
    /// The message for the user reads that number.
    ///
    /// A media that stands in the queue already goes in a second time. The
    /// user can want to hear one episode two times, and a rule that refuses it
    /// gives no value.
    pub fn add(&mut self, entry: Entry) -> usize {
        self.entries.push(entry);
        self.entries.len()
    }

    /// Takes the first media out of the queue.
    pub fn take_next(&mut self) -> Option<Entry> {
        if self.entries.is_empty() {
            return None;
        }

        Some(self.entries.remove(0))
    }

    /// Takes one media out of the queue, by its place.
    ///
    /// The view of the queue calls this function for the key `l` and for the
    /// key `X`. An index that is too large gives nothing, and it does not stop
    /// the program. See T-41.
    pub fn take_at(&mut self, index: usize) -> Option<Entry> {
        if index >= self.entries.len() {
            return None;
        }

        Some(self.entries.remove(index))
    }

    /// Empties the queue.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Gives the line that the view selects after a remove.
    ///
    /// The list becomes shorter, therefore the old place can stand outside the
    /// list. The selection then goes to the last line. An empty queue gives no
    /// selection.
    pub fn selection_after_a_remove(&self, selected: usize) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }

        Some(selected.min(self.entries.len() - 1))
    }

    /// Makes the text of each line of the view.
    ///
    /// The number of the place stands at the start, therefore the user reads
    /// the sequence. The length stands at the end.
    pub fn lines(&self) -> Vec<String> {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let mark = if entry.target.episode_id().is_some() {
                    "🎙"
                } else {
                    "📕"
                };

                let mut line = format!("{}. {} {}", index + 1, mark, entry.title);

                if !entry.author.trim().is_empty() {
                    line.push_str(" — ");
                    line.push_str(entry.author.trim());
                }

                // A view that holds the length as a text only gives no number.
                // The line then shows the name and no length.
                if let Some(length) = entry.duration {
                    let text = convert_seconds(vec![length])
                        .first()
                        .cloned()
                        .unwrap_or_default();

                    line.push_str("  (");
                    line.push_str(&text);
                    line.push(')');
                }

                line
            })
            .collect()
    }
}

/// The queue of the process.
fn box_of_the_queue() -> &'static Mutex<Queue> {
    static QUEUE: OnceLock<Mutex<Queue>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(Queue::default()))
}

/// Runs a function on the queue of the process.
///
/// A lock that a thread broke must not stop the application. The application
/// takes the value that stands inside the lock.
fn with_the_queue<T>(work: impl FnOnce(&mut Queue) -> T) -> T {
    let mut queue = box_of_the_queue()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    work(&mut queue)
}

/// Puts a media at the end of the queue of the process.
pub fn add(entry: Entry) -> usize {
    with_the_queue(|queue| queue.add(entry))
}

/// Takes the first media out of the queue of the process.
pub fn take_next() -> Option<Entry> {
    with_the_queue(|queue| queue.take_next())
}

/// Takes one media out of the queue of the process, by its place.
pub fn take_at(index: usize) -> Option<Entry> {
    with_the_queue(|queue| queue.take_at(index))
}

/// Empties the queue of the process.
pub fn clear() {
    with_the_queue(|queue| queue.clear())
}

/// Gives a copy of the queue of the process.
///
/// The screen reads a copy, therefore the render holds no lock. This is the
/// same shape as the other views. See the head of `crate::logic::stats`.
pub fn snapshot() -> Queue {
    with_the_queue(|queue| queue.clone())
}

/// Gives the number of media that wait.
pub fn len() -> usize {
    with_the_queue(|queue| queue.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book(id: &str, title: &str) -> Entry {
        Entry {
            target: PlaybackTarget::Book {
                item_id: id.to_string(),
                whole_book_duration: Some(60.0),
            },
            title: title.to_string(),
            author: "An Author".to_string(),
            duration: Some(60.0),
        }
    }

    fn episode(item: &str, id: &str) -> Entry {
        Entry {
            target: PlaybackTarget::Episode {
                item_id: item.to_string(),
                episode_id: id.to_string(),
            },
            title: "An Episode".to_string(),
            author: String::new(),
            duration: None,
        }
    }

    #[test]
    fn a_new_queue_is_empty() {
        let queue = Queue::default();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(queue.lines().is_empty());
    }

    #[test]
    fn the_queue_keeps_the_sequence_of_the_user() {
        let mut queue = Queue::default();

        assert_eq!(queue.add(book("a", "First")), 1);
        assert_eq!(queue.add(book("b", "Second")), 2);
        assert_eq!(queue.add(book("c", "Third")), 3);

        assert_eq!(queue.take_next().unwrap().title, "First");
        assert_eq!(queue.take_next().unwrap().title, "Second");
        assert_eq!(queue.take_next().unwrap().title, "Third");
        assert!(queue.take_next().is_none());
    }

    #[test]
    fn a_media_that_the_queue_gave_is_not_in_the_queue() {
        let mut queue = Queue::default();

        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));
        queue.take_next();

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.entries()[0].title, "Second");
    }

    #[test]
    fn the_view_takes_a_media_by_its_place() {
        let mut queue = Queue::default();

        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));
        queue.add(book("c", "Third"));

        assert_eq!(queue.take_at(1).unwrap().title, "Second");
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.entries()[1].title, "Third");
    }

    /// The list can become shorter between the key and the work. An index of a
    /// vector stops the program, and this function must not. See T-41.
    #[test]
    fn a_place_outside_the_queue_gives_nothing() {
        let mut queue = Queue::default();
        queue.add(book("a", "First"));

        assert!(queue.take_at(1).is_none());
        assert!(queue.take_at(99).is_none());
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn the_queue_becomes_empty() {
        let mut queue = Queue::default();

        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));
        queue.clear();

        assert!(queue.is_empty());
    }

    /// Two episodes of one podcast have the same item. The identity must
    /// separate them.
    #[test]
    fn an_episode_has_its_own_identity() {
        assert_eq!(episode("pod", "ep1").key(), "pod/ep1");
        assert_ne!(episode("pod", "ep1").key(), episode("pod", "ep2").key());
        assert_eq!(book("a", "First").key(), "a");
    }

    #[test]
    fn the_queue_finds_a_media_that_it_holds() {
        let mut queue = Queue::default();
        queue.add(episode("pod", "ep1"));

        assert!(queue.holds("pod/ep1"));
        assert!(!queue.holds("pod/ep2"));
        assert!(!queue.holds("pod"));
    }

    /// A user can want one episode two times. The queue must take it.
    #[test]
    fn the_same_media_goes_in_two_times() {
        let mut queue = Queue::default();

        queue.add(book("a", "First"));
        queue.add(book("a", "First"));

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn every_media_gives_one_line_with_its_place() {
        let mut queue = Queue::default();
        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));

        let lines = queue.lines();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("1. "), "the line is {:?}", lines[0]);
        assert!(lines[1].starts_with("2. "), "the line is {:?}", lines[1]);
        assert!(lines[0].contains("First"));
        assert!(lines[0].contains("An Author"));
        assert!(lines[0].contains("1m"), "the line is {:?}", lines[0]);
    }

    /// A podcast gives no author in every view. A line with an empty name must
    /// not hold the mark of the separation.
    #[test]
    fn a_media_with_no_author_gives_a_line_with_no_separation() {
        let mut queue = Queue::default();
        queue.add(episode("pod", "ep1"));

        assert!(!queue.lines()[0].contains('—'));
    }

    /// The view of the episodes of a podcast holds the length as a text only.
    /// A line with no length must not show the length zero.
    #[test]
    fn a_media_with_no_length_gives_a_line_with_no_length() {
        let mut queue = Queue::default();
        queue.add(episode("pod", "ep1"));

        let line = queue.lines()[0].clone();

        assert!(!line.contains('('), "the line is {:?}", line);
        assert!(line.contains("An Episode"));
    }

    /// The selection must stay inside the list after a remove.
    #[test]
    fn the_selection_follows_a_remove() {
        let mut queue = Queue::default();
        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));

        // The user removed the last line of a list of three.
        assert_eq!(queue.selection_after_a_remove(2), Some(1));
        assert_eq!(queue.selection_after_a_remove(0), Some(0));

        queue.clear();
        assert_eq!(queue.selection_after_a_remove(0), None);
    }

    /// The rule of the queue: an end starts the next media, and nothing else
    /// does.
    #[test]
    fn only_an_end_starts_the_next_media() {
        assert!(the_queue_goes_on(Outcome::Finished));
        assert!(!the_queue_goes_on(Outcome::Stopped));
        assert!(!the_queue_goes_on(Outcome::Fault));
    }
}
