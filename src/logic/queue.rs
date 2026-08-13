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
//! # The queue on the disk
//!
//! The queue stood in the memory of the process, and a user who stopped the
//! program lost it. **The table `queue` of the database holds it now.** See
//! T-56.
//!
//! - The program names the account one time at the start, and it then reads the
//!   queue of that account. A user with an account on two servers keeps one
//!   queue for each of them.
//! - Every change of the queue writes every row again. A queue holds some media,
//!   therefore that write costs almost nothing and it needs no rule for a row
//!   that changed.
//!
//! - **A media that the server does not hold now stays in the queue.** The row
//!   holds the identity of the item, and the server answers the playback.
//!
//! # The disk is the truth of the queue (T-147)
//!
//! The queue of the process stood beside the queue of the disk, and the write
//! above holds **every** row: a second program of the account therefore wrote
//! its own memory over the media of the first one. Two windows each put one book
//! in the queue, each screen said "the queue holds 1 item", and the disk held one
//! book of the two.
//!
//! **Every change of the queue reads the disk first**, and the view of the queue
//! reads it again when it opens. This is the rule of T-142 for a second state of
//! the program: the file of the disk is the truth, and the program reads it at
//! the moment that it uses it.
//!
//! # A playback that did not start keeps its media (T-146)
//!
//! The queue took a media out **before** the playback of that media started, and
//! a playback that did not start therefore took the media of the user away for
//! ever. A server that goes away in the middle of a queue met that rule at the
//! end of every media.
//!
//! **The media goes back to the front of the queue, and the queue stops there.**
//! The queue must not go on to the media after it: a server that does not answer
//! gives the same fault to every media of the queue, therefore a queue that goes
//! on empties itself in one second. `the_media_goes_back_to_the_queue` holds this
//! rule, and `crate::logic::playback::play` reads it.

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

/// Tells if the media that the queue started must go back into the queue.
///
/// **A playback that did not start takes no media of the user away.** The
/// outcome `Fault` says that no audio played at all: the server did not answer,
/// or it gave no stream, or the disk holds no copy. The media of that playback
/// therefore goes back to the front of the queue. See T-146.
///
/// An end and a stop keep the queue as it is: the user heard that media.
pub fn the_media_goes_back_to_the_queue(outcome: Outcome) -> bool {
    matches!(outcome, Outcome::Fault)
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

    /// Puts a media at the front of the queue.
    ///
    /// The queue gives this media to the next playback. A playback that did not
    /// start calls this function with the media that it did not play, therefore
    /// the media stands where it stood before that playback. See T-146.
    pub fn put_at_the_front(&mut self, entry: Entry) {
        self.entries.insert(0, entry);
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

/// The account and the server of the queue on the disk. See T-56.
///
/// The functions of the queue take no account, because the keys of the program
/// call them. Therefore the program names the account one time, at the start.
fn the_account() -> &'static Mutex<Option<(String, String)>> {
    static ACCOUNT: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    ACCOUNT.get_or_init(|| Mutex::new(None))
}

/// Names the account of the queue, and it reads the queue of the disk.
///
/// The start of the program calls this one time. A user of a second account on a
/// second server gets the queue of that account: the two queues stand apart in
/// the database.
pub fn read_the_queue_of_the_account(username: &str, server: &str) {
    if let Ok(mut place) = the_account().lock() {
        *place = Some((username.to_string(), server.to_string()));
    }

    let rows = crate::db::crud::read_the_queue(username, server);

    if rows.is_empty() {
        return;
    }

    log::info!("[queue] the disk holds {} media of the queue", rows.len());

    read_the_disk();
}

/// Takes the queue of the disk into the queue of this program. See T-147.
///
/// **The disk is the truth of the queue.** Every program of the account writes
/// every row of the queue again, therefore a program that changes the queue of
/// its own memory alone takes the media of the other program away: two windows
/// each put one book in the queue, and the disk held one of them.
///
/// A program that has no account named yet reads nothing: the queue then belongs
/// to a test, and a test must not touch the database of a user.
fn read_the_disk() {
    let Some((username, server)) = the_account().lock().ok().and_then(|place| place.clone()) else {
        return;
    };

    let entries: Vec<Entry> = crate::db::crud::read_the_queue(&username, &server)
        .iter()
        .map(entry_of_the_row)
        .collect();

    with_the_queue(|queue| {
        queue.clear();

        for entry in entries {
            queue.add(entry);
        }
    });
}

/// Takes the queue of the disk, for a view that opens. See T-147.
///
/// The view of the queue reads the media of the process at every frame. A second
/// program of the account can have changed the queue, therefore the view takes
/// the queue of the disk at the moment that it opens.
pub fn read_the_queue_again() {
    read_the_disk();
}

/// Gives the place of the media that a view named.
///
/// The view of the queue holds the lines of a moment, and the disk can hold
/// other media at the moment of the key (T-147). The place of the line is
/// therefore not enough: the function takes the place when the media of that
/// place is the media of the line, and it takes the first media of that identity
/// otherwise.
///
/// A media that stands in the queue no more gives nothing, and the key then does
/// nothing at all. The function is pure, therefore a test needs no database.
pub fn the_place_of_the_media(entries: &[Entry], index: usize, key: &str) -> Option<usize> {
    if entries.get(index).is_some_and(|entry| entry.key() == key) {
        return Some(index);
    }

    entries.iter().position(|entry| entry.key() == key)
}

/// Makes an entry of the queue of one row of the database.
///
/// A row with an episode gives an episode, and a row with no episode gives a
/// book. The length of a book of the database is the length of the whole book,
/// therefore the playback needs no request for it.
fn entry_of_the_row(row: &crate::db::crud::QueueRow) -> Entry {
    let target = if row.id_pod.trim().is_empty() {
        PlaybackTarget::Book {
            item_id: row.id_item.clone(),
            whole_book_duration: row.duration,
        }
    } else {
        PlaybackTarget::Episode {
            item_id: row.id_item.clone(),
            episode_id: row.id_pod.clone(),
        }
    };

    Entry {
        target,
        title: row.title.clone(),
        author: row.author.clone(),
        duration: row.duration,
    }
}

/// Makes a row of the database of one entry of the queue.
fn row_of_the_entry(entry: &Entry) -> crate::db::crud::QueueRow {
    crate::db::crud::QueueRow {
        id_item: entry.target.item_id().to_string(),
        id_pod: entry.target.episode_id().unwrap_or_default().to_string(),
        title: entry.title.clone(),
        author: entry.author.clone(),
        duration: entry.duration,
    }
}

/// Writes the queue of the process on the disk.
///
/// The queue holds some media, therefore the write of every row costs almost
/// nothing. A program that has no account named yet writes nothing: the queue
/// then belongs to a test, and a test must not touch the database of a user.
fn write_the_queue() {
    let Some((username, server)) = the_account().lock().ok().and_then(|place| place.clone()) else {
        return;
    };

    let rows: Vec<crate::db::crud::QueueRow> =
        with_the_queue(|queue| queue.entries().iter().map(row_of_the_entry).collect());

    if let Err(error) = crate::db::crud::save_the_queue(&username, &server, &rows) {
        log::warn!("[queue] the program did not write the queue: {}", error);
    }
}

/// Puts a media at the end of the queue of the process.
///
/// **The queue comes of the disk first** (T-147): a second program of the
/// account can hold media that this program never saw, and the write below holds
/// every row.
pub fn add(entry: Entry) -> usize {
    read_the_disk();
    let place = with_the_queue(|queue| queue.add(entry));
    write_the_queue();
    place
}

/// Puts a media at the front of the queue of the process. See T-146.
pub fn put_at_the_front(entry: Entry) {
    read_the_disk();
    with_the_queue(|queue| queue.put_at_the_front(entry));
    write_the_queue();
}

/// Takes the first media out of the queue of the process.
pub fn take_next() -> Option<Entry> {
    read_the_disk();
    let entry = with_the_queue(|queue| queue.take_next());
    write_the_queue();
    entry
}

/// Takes one media out of the queue of the process, by its place and by its
/// identity.
///
/// A key of the view of the queue calls this: the place comes of the line that
/// the user selected, and the identity holds that line when the disk moved under
/// it. See `the_place_of_the_media` and T-147.
pub fn take_the_media(index: usize, key: &str) -> Option<Entry> {
    read_the_disk();

    let entry = with_the_queue(|queue| {
        let place = the_place_of_the_media(queue.entries(), index, key)?;
        queue.take_at(place)
    });

    write_the_queue();
    entry
}

/// Empties the queue of the process.
///
/// This takes every media of the account away, therefore it needs no read of the
/// disk: the queue of every program of that account is empty after it.
pub fn clear() {
    with_the_queue(|queue| queue.clear());
    write_the_queue();
}

/// Forgets the account of the queue. A test calls this, and the queue then
/// touches no database.
pub fn forget_the_account() {
    if let Ok(mut place) = the_account().lock() {
        *place = None;
    }
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

    /// The view holds the lines of a moment, and a second program can change
    /// the queue under it. The identity of the line holds the media. See T-147.
    #[test]
    fn the_key_takes_the_media_of_its_own_line() {
        let mut queue = Queue::default();
        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));
        queue.add(book("c", "Third"));

        let entries = queue.entries();

        // The place and the media agree: the place answers.
        assert_eq!(the_place_of_the_media(entries, 1, "b"), Some(1));

        // A second program put a media before it. The identity answers, and the
        // place does not.
        assert_eq!(the_place_of_the_media(entries, 0, "c"), Some(2));

        // A place outside the queue gives the media of the identity.
        assert_eq!(the_place_of_the_media(entries, 99, "a"), Some(0));

        // A media that stands in the queue no more gives nothing.
        assert_eq!(
            the_place_of_the_media(entries, 0, "a-media-of-no-queue"),
            None
        );
        assert_eq!(the_place_of_the_media(&[], 0, "a"), None);
    }

    /// The rule of the queue: an end starts the next media, and nothing else
    /// does.
    /// The media of a playback that did not start goes back to the front, and
    /// the media after it does not move. See T-146.
    #[test]
    fn a_media_goes_back_to_the_front_of_the_queue() {
        let mut queue = Queue::default();
        queue.add(book("b", "Second"));

        let first = book("a", "First");
        queue.put_at_the_front(first);

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.entries()[0].title, "First");
        assert_eq!(queue.entries()[1].title, "Second");
    }

    /// A media that the queue gave, and that the playback took and gave back,
    /// stands where it stood.
    #[test]
    fn the_queue_of_a_media_that_did_not_play_does_not_change() {
        let mut queue = Queue::default();
        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));

        let entry = queue.take_next().unwrap();
        queue.put_at_the_front(entry);

        assert_eq!(
            queue.lines(),
            {
                let mut same = Queue::default();
                same.add(book("a", "First"));
                same.add(book("b", "Second"));
                same.lines()
            },
            "the queue must hold the same media in the same sequence"
        );
    }

    /// **The rule of T-146.** A playback that gave no audio at all keeps the
    /// media of the user, and an end and a stop do not.
    #[test]
    fn a_playback_that_did_not_start_keeps_its_media() {
        assert!(the_media_goes_back_to_the_queue(Outcome::Fault));
        assert!(!the_media_goes_back_to_the_queue(Outcome::Finished));
        assert!(!the_media_goes_back_to_the_queue(Outcome::Stopped));
    }

    /// **The queue must not go on after a fault.** A server that does not answer
    /// gives the same fault to every media of the queue, therefore a queue that
    /// goes on empties itself in one second.
    #[test]
    fn a_fault_stops_the_queue() {
        assert!(!the_queue_goes_on(Outcome::Fault));
    }

    #[test]
    fn only_an_end_starts_the_next_media() {
        assert!(the_queue_goes_on(Outcome::Finished));
        assert!(!the_queue_goes_on(Outcome::Stopped));
        assert!(!the_queue_goes_on(Outcome::Fault));
    }
}
