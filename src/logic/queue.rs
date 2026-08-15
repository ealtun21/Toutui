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
//!   change the media that plays. **A media that waits already moves to the
//!   end**, because the disk holds one row for one media (T-231), and **the
//!   sentence of that key says that the media moved** (T-232).
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
use std::collections::BTreeMap;
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

/// The place that the key `n` gave a media, and the place that it held before.
/// See T-232.
///
/// **A media that came in and a media that moved are two conditions**, and the
/// key said one sentence for the two of them: `"A Long Test Book" is number 2
/// of the queue.` came of a queue that grew from 1 media to 2 **and** of a
/// queue of 2 media where that book went from the place 1 to the place 2. A
/// user who does not press the key `q` cannot tell the two.
///
/// The program holds the reason of each of them (T-91), therefore
/// `the_words_of_the_key_that_adds` says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThePlaceOfTheMedia {
    /// The place of the media now. The first place is 1.
    pub place: usize,
    /// The place that the media held before this key, and nothing for a media
    /// that the queue did not hold.
    pub the_place_before: Option<usize>,
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

    /// Takes every media of this identity out of the queue.
    ///
    /// **The queue of the disk holds one row for one media** (T-231): the
    /// primary key of the table is the account, the server, the item, and the
    /// episode. Therefore the queue of the process holds one entry for one
    /// media too, and the two agree.
    ///
    /// **The answer gives the place that the media held** (T-232), and the
    /// first place is 1. A caller that says a word for the user needs it: a
    /// media that came in and a media that moved are two conditions, and the
    /// program must not say the first one for the second one (T-91).
    fn take_the_key_out(&mut self, key: &str) -> Option<usize> {
        let place = self.entries.iter().position(|entry| entry.key() == key);
        self.entries.retain(|entry| entry.key() != key);

        place.map(|place| place + 1)
    }

    /// Puts a media at the end of the queue.
    ///
    /// The function gives the place of the media, and the first place is 1.
    /// The message for the user reads that number.
    ///
    /// **A media that stands in the queue already moves to the end** (T-231),
    /// and the answer is the place that it takes there. The old shape put it in
    /// a second time: the message then said `is number 3 of the queue` for a
    /// view of two lines, because `save_the_queue` writes one row for one media
    /// and the row of the second place went away with the row of the third one.
    /// **The disk is the truth of the queue** (T-147), and the disk holds that
    /// media at the end.
    ///
    /// **The answer says the place of before too** (T-232): the key `n` on a
    /// media that waits already said the same sentence as the key `n` on a
    /// media that came in, and the program held the reason of the other one.
    pub fn add(&mut self, entry: Entry) -> ThePlaceOfTheMedia {
        let the_place_before = self.take_the_key_out(&entry.key());
        self.entries.push(entry);

        ThePlaceOfTheMedia {
            place: self.entries.len(),
            the_place_before,
        }
    }

    /// Puts a media at the front of the queue.
    ///
    /// The queue gives this media to the next playback. A playback that did not
    /// start calls this function with the media that it did not play, therefore
    /// the media stands where it stood before that playback. See T-146.
    ///
    /// **A media that stands in the queue already moves to the front** (T-231):
    /// the user pressed the key `n` on the media of a playback that then did not
    /// start, and a queue of two entries of one media writes one row on the disk.
    pub fn put_at_the_front(&mut self, entry: Entry) {
        self.take_the_key_out(&entry.key());
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

    /// Makes the text of each line of the view, with no place of the user.
    ///
    /// A caller that holds no place of the user gives none: the message of a
    /// test names the media of the queue and no percent. The view of the user
    /// calls `the_lines_of_the_queue`. See T-230.
    pub fn lines(&self) -> Vec<String> {
        the_lines_of_the_queue(&self.entries, &BTreeMap::new(), None)
    }
}

/// Gives the text of each line of the view of the queue. See T-230.
///
/// **A line of this view is one media, and it held no place of that media at
/// all**: no percent of the user, no mark of the media that the user finished,
/// and no mark of the media that plays. Every other list of a media of the
/// program wraps its title with `crate::ui::marks::line`: the Home view with
/// `marks::of_progress` (T-44 and T-228), the Library view with
/// `marks::of_library`, and the view of the episodes of a podcast with
/// `marks::of_progress` too (T-229). This list wrapped nothing.
///
/// The measurement of 2026-08-15: `A Second Book Of Many Hours` played and
/// `A Big Book Of A Scan` stood at 42 percent. The Home view of that same
/// program said `▶   A Second Book Of Many Hours` and
/// `42% A Big Book Of A Scan`, and the two lines of the queue of that same
/// second said `1. 📕 A Big Book Of A Scan — Big Author  (0m)` and
/// `2. 📕 A Second Book Of Many Hours — Many Hours Author  (8h)`. A book that
/// the user finished stood in that queue before it, and its line said the same
/// nothing.
///
/// `places` holds one row for each media, keyed by `Entry::key`, in the form of
/// `App::book_progress_cnt_list`: the percent of the user and the mark of the
/// end. **The key names the episode after the item** (T-223, T-228, and
/// T-229): two episodes of one podcast hold the identity of that podcast, and
/// a key of the item alone would give one mark to every episode of it. A media
/// of no row takes no mark, as a media that never played takes none.
///
/// The number of the place stands after the mark, therefore the user reads the
/// sequence. The length stands at the end.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_lines_of_the_queue(
    entries: &[Entry],
    places: &BTreeMap<String, Vec<String>>,
    playing: Option<&str>,
) -> Vec<String> {
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let kind = if entry.target.episode_id().is_some() {
                "🎙"
            } else {
                "📕"
            };

            let mut line = format!("{}. {} {}", index + 1, kind, entry.title);

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

            let key = entry.key();
            let plays_now = playing.is_some_and(|playing| playing == key);
            let row = places.get(&key);
            let percent = row
                .and_then(|row| row.first())
                .map(String::as_str)
                .unwrap_or("");
            let finished = row
                .and_then(|row| row.get(1))
                .map(String::as_str)
                .unwrap_or("");

            crate::ui::marks::line(
                &crate::ui::marks::of_progress(percent, finished, plays_now),
                &line,
            )
        })
        .collect()
}

/// The place of the user of each media of the queue, keyed by `Entry::key`.
///
/// The task of `show_the_queue` writes this box, and the render reads it. The
/// shape is the shape of `crate::logic::the_episodes`: a task asks, the box
/// holds the answer, and the render takes it at the next frame. See T-230.
fn box_of_the_places() -> &'static Mutex<BTreeMap<String, Vec<String>>> {
    static PLACES: OnceLock<Mutex<BTreeMap<String, Vec<String>>>> = OnceLock::new();
    PLACES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Writes the place of the user of each media of the queue. See T-230.
///
/// The list **takes the place** of the list that came before it: the key `q`
/// asks the server again, and a media that left the queue must take no line of
/// a later frame.
pub fn keep_the_places(places: BTreeMap<String, Vec<String>>) {
    if let Ok(mut slot) = box_of_the_places().lock() {
        *slot = places;
    }
}

/// Gives the place of the user of each media of the queue. See T-230.
///
/// A request that did not come back gives an empty list, and every line then
/// holds its title alone, as it did before T-230.
pub fn the_places() -> BTreeMap<String, Vec<String>> {
    match box_of_the_places().lock() {
        Ok(places) => places.clone(),
        Err(_) => BTreeMap::new(),
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

    match crate::db::crud::read_the_queue(username, server) {
        Ok(rows) if rows.is_empty() => {}
        Ok(rows) => {
            log::info!("[queue] the disk holds {} media of the queue", rows.len());

            read_the_disk();
        }
        Err(error) => {
            log::error!(
                "[queue] the program did not read the queue of the disk: {}. The queue of this \
                 program holds no media of the disk yet.",
                error
            );
        }
    }
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
fn read_the_disk() -> bool {
    let Some((username, server)) = the_account().lock().ok().and_then(|place| place.clone()) else {
        return true;
    };

    // **A read that failed is not a queue with no media** (T-202). The old code
    // took `Vec::new()` of such a read and it emptied the queue of the process:
    // the view then said "The queue is empty. Press n on a media to put it in the
    // queue." while the disk held the media of the user, and a change of the
    // queue after it wrote that emptiness on the disk.
    //
    // **The queue of the process stays**, therefore the view says what this
    // program last read of the disk, and every caller that changes the queue
    // changes nothing at all: the disk is the truth of the queue (T-147), and a
    // program that did not read that truth must not write over it.
    let rows = match crate::db::crud::read_the_queue(&username, &server) {
        Ok(rows) => rows,
        Err(error) => {
            log::error!(
                "[queue] the program did not read the queue of the disk: {}. The queue of this \
                 program stays, and no key changes it.",
                error
            );

            return false;
        }
    };

    let entries: Vec<Entry> = rows.iter().map(entry_of_the_row).collect();

    with_the_queue(|queue| {
        queue.clear();

        for entry in entries {
            queue.add(entry);
        }
    });

    true
}

/// Takes the queue of the disk, for a view that opens. See T-147.
///
/// The view of the queue reads the media of the process at every frame. A second
/// program of the account can have changed the queue, therefore the view takes
/// the queue of the disk at the moment that it opens.
pub fn read_the_queue_again() {
    read_the_disk();
}

/// The words of a key of the queue that the disk did not permit. See T-202.
///
/// **The disk is the truth of the queue** (T-147), therefore a program that did
/// not read that truth writes nothing over it: the key of the user changes
/// nothing at all, and a key that does nothing says why (T-79).
///
/// The function is pure, therefore a test needs no queue and no database.
pub fn the_words_of_a_queue_that_the_disk_did_not_give() -> String {
    "The program did not read the queue of its disk. Stop a second Toutui, and press the key again."
        .to_string()
}

/// The words of a key of the queue that the disk did not hold. See T-206.
///
/// **A read and a write are two conditions** (T-206). The program read the queue
/// of the disk, and the disk then did not take the change of the user: a disk
/// that is full, a file of a database with no permission of a write, and a
/// different program of the account each give that condition. The queue of the
/// disk holds the media of before, therefore the key changed nothing at all, and
/// a key that does nothing says why (T-79).
///
/// **The sentence names the key of the view that the user sees at that moment**
/// (T-183): the key `n` stands in a view of the media, and the key `X` stands in
/// the view of the queue.
///
/// The function is pure, therefore a test needs no queue and no database.
pub fn the_words_of_a_queue_that_the_disk_did_not_hold(
    fault: TheDiskDidNotAnswer,
    key: &str,
) -> String {
    match fault {
        TheDiskDidNotAnswer::TheRead => the_words_of_a_queue_that_the_disk_did_not_give(),
        TheDiskDidNotAnswer::TheWrite => format!(
            "The program did not write the queue of this account: the database did not answer. The \
             queue does not change. Press {} again.",
            key
        ),
    }
}

/// Gives the word of the work of the disk that failed, for a line of the log.
/// See T-206.
pub fn the_word_of_the_work_of_the_disk(fault: TheDiskDidNotAnswer) -> &'static str {
    match fault {
        TheDiskDidNotAnswer::TheRead => "read",
        TheDiskDidNotAnswer::TheWrite => "write",
    }
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

/// What the line of the view of the queue holds now. See T-161.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheLineOfTheUser {
    /// The media that the user chose stands in the queue, at this place.
    ItStandsAt(usize),
    /// The media that the user chose is not in the queue now.
    ItWentAway,
    /// The line of the user is not the line that the program holds. The
    /// program reads the media of the line of the user again.
    TheUserChoseAnother,
}

/// Tells what happened to the media that the user chose in the view of the
/// queue.
///
/// **The queue changes while the view stands open, and no key of the user does
/// it.** A media that comes to its end takes the media of the front of the
/// queue away, and a second program of the account takes a media out with the
/// key `X`. The lines keep the number of the line, therefore a media that the
/// user did not choose moves under the cursor with no word at all: the key `X`
/// then took that media out, and the key `l` played it and stopped the media
/// that plays. See T-161, and T-160 for the same rule of the Home view.
///
/// `of_the_program` is the line and the media of that line at the frame before,
/// and `of_the_user` is the line of the user now.
///
/// The function is pure, therefore a test needs no queue and no database.
pub fn what_the_line_of_the_user_holds(
    entries: &[Entry],
    of_the_program: Option<(usize, &str)>,
    of_the_user: Option<usize>,
) -> TheLineOfTheUser {
    let Some((line, key)) = of_the_program else {
        return TheLineOfTheUser::TheUserChoseAnother;
    };

    // The user moved the cursor after that frame. The line of the user is the
    // truth of the choice, therefore the program reads the media of it again.
    if of_the_user != Some(line) {
        return TheLineOfTheUser::TheUserChoseAnother;
    }

    match entries.iter().position(|entry| entry.key() == key) {
        Some(place) => TheLineOfTheUser::ItStandsAt(place),
        None => TheLineOfTheUser::ItWentAway,
    }
}

/// The text for the user when the media of their line leaves the queue.
///
/// **The program cannot know which media the user wants now**, therefore it
/// takes the line away and it says what happened. A key of the selection then
/// changes no media at all, and the user chooses the next one.
///
/// The sentence names no cause: this program cannot tell a media that came to
/// the front of the queue from a media that a second program took out (T-91).
/// It names the two keys of the view that give a line again, and it promises no
/// other key (T-118 and T-143). See T-161.
pub fn the_text_of_the_media_that_went_away(title: &str) -> String {
    format!(
        "The media \"{}\" is not in the queue now. \
         No line is selected: the keys j and k select one.",
        title
    )
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
///
/// **The answer says whether the disk took the queue** (T-206). The old shape
/// gave nothing at all, therefore every caller told the user that the queue of
/// the user changed while the disk of the user held the queue of before.
fn write_the_queue() -> bool {
    let Some((username, server)) = the_account().lock().ok().and_then(|place| place.clone()) else {
        return true;
    };

    let rows: Vec<crate::db::crud::QueueRow> =
        with_the_queue(|queue| queue.entries().iter().map(row_of_the_entry).collect());

    if let Err(error) = crate::db::crud::save_the_queue(&username, &server, &rows) {
        log::warn!(
            "[queue] the program did not write the queue: {}. The queue of this program goes back \
             to the queue of the disk.",
            error
        );

        return false;
    }

    true
}

/// Changes the queue of the process, and it puts that change on the disk.
///
/// **The queue comes of the disk first** (T-147 and T-202): a second program of
/// the account can hold media that this program never saw, and a program that
/// did not read that truth writes nothing over it.
///
/// **A change that the disk did not take is no change** (T-206). The disk is the
/// truth of the queue, therefore a change that stands in this program alone is a
/// change that no program of the account reads: the key `n` said
/// `"…" is number 1 of the queue. Press q to see the queue.` and the key `q` of
/// that same sentence then said that the queue is empty. The queue of the
/// process goes back to the queue of before, and the caller says why.
fn the_queue_changes<T>(work: impl FnOnce(&mut Queue) -> T) -> Result<T, TheDiskDidNotAnswer> {
    if !read_the_disk() {
        return Err(TheDiskDidNotAnswer::TheRead);
    }

    let before = with_the_queue(|queue| queue.clone());
    let answer = with_the_queue(work);

    if !write_the_queue() {
        with_the_queue(|queue| *queue = before);

        return Err(TheDiskDidNotAnswer::TheWrite);
    }

    Ok(answer)
}

/// Puts a media at the end of the queue of the process.
///
/// The answer gives the place of the media and the place that it held before
/// (T-232), and it says why for a disk that did not answer (T-202 and T-206).
pub fn add(entry: Entry) -> Result<ThePlaceOfTheMedia, TheDiskDidNotAnswer> {
    the_queue_changes(|queue| queue.add(entry))
}

/// Gives the sentence of the key `n`. See T-232.
///
/// **Three conditions, and three sentences.** The key said one of them for the
/// three: `"…" is number N of the queue. Press q to see the queue.`
///
/// - The queue did not hold the media: it came in, and the number is its line.
/// - The queue held the media at another place: it **moved**, and the queue
///   holds the same number of media as before. The sentence names the two
///   places, because the program has them (T-91).
/// - The queue held the media at the last place: the key changed nothing at
///   all, and **a key that does nothing must say why** (T-79).
///
/// The function is pure, therefore a test needs no queue, no database, and no
/// screen.
pub fn the_words_of_the_key_that_adds(title: &str, place: ThePlaceOfTheMedia) -> String {
    match place.the_place_before {
        None => format!(
            "\"{}\" is number {} of the queue. Press q to see the queue.",
            title, place.place
        ),
        Some(before) if before == place.place => format!(
            "\"{}\" waits at number {} of the queue already. Press q to see the queue.",
            title, place.place
        ),
        Some(before) => format!(
            "\"{}\" waits in the queue already. It moves from number {} to number {}. Press q to \
             see the queue.",
            title, before, place.place
        ),
    }
}

/// Puts a media at the front of the queue of the process. See T-146.
pub fn put_at_the_front(entry: Entry) -> Result<(), TheDiskDidNotAnswer> {
    the_queue_changes(|queue| queue.put_at_the_front(entry))
}

/// Takes the first media out of the queue of the process.
///
/// **A disk that did not answer gives no media** (T-202 and T-206), and the queue
/// then stops with every media of the user on the disk: that is the road of a
/// queue with no media, and no media of the user goes away.
pub fn take_next() -> Result<Option<Entry>, TheDiskDidNotAnswer> {
    the_queue_changes(|queue| queue.take_next())
}

/// Takes one media out of the queue of the process, by its place and by its
/// identity.
///
/// A key of the view of the queue calls this: the place comes of the line that
/// the user selected, and the identity holds that line when the disk moved under
/// it. See `the_place_of_the_media` and T-147.
///
/// **The answer holds the place that the media held** (T-233), and the first
/// place is 1. The sentence of the key `X` reads that number: a media that goes
/// out of a queue of ten changes the number of every media after it, and the
/// title alone does not say which number went away. **The place of the answer is
/// the place of the disk, and not the number of the line of the view**: a second
/// program of the account moves the media under that view, and this function then
/// takes the media of the line at another place.
pub fn take_the_media(
    index: usize,
    key: &str,
) -> Result<Option<TheMediaThatWentOut>, TheDiskDidNotAnswer> {
    the_queue_changes(|queue| {
        let place = the_place_of_the_media(queue.entries(), index, key)?;
        let entry = queue.take_at(place)?;

        Some(TheMediaThatWentOut {
            place: place + 1,
            entry,
        })
    })
}

/// The media that a key took out of the queue, and the place that it held. See
/// T-233.
#[derive(Debug, Clone, PartialEq)]
pub struct TheMediaThatWentOut {
    /// The place of the media at the moment that it went out. The first place
    /// is 1, therefore this is the number that the line of the view holds.
    pub place: usize,
    /// The media itself.
    pub entry: Entry,
}

/// The disk of the queue did not answer, therefore the key changed nothing. See
/// T-202 and T-206.
///
/// **A media that a second program took out and a disk that says nothing are two
/// conditions**, and the sentence of the key must not name the first one for the
/// second one (T-91): `text_of_the_key_that_takes` says that the media of the
/// line waits no more, and the media of a disk that says nothing waits still.
///
/// **A read and a write are two conditions too** (T-206): a disk that the program
/// reads and cannot write gives the second one alone, and the two sentences of
/// `the_words_of_a_queue_that_the_disk_did_not_hold` name what the program did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheDiskDidNotAnswer {
    /// The program did not read the queue of the disk.
    TheRead,
    /// The program read the queue of the disk, and the disk did not take the
    /// change of the user.
    TheWrite,
}

/// Gives the sentence of the key `X` of the view of the queue. See T-151.
///
/// `title_of_the_line` is the title that the view drew, and `what_went_out` is
/// the title of the media that this key took out. **A different program of the
/// account can take that media out before this key** (T-147), and the key then
/// takes nothing at all: it said nothing at all before this work, and **a key
/// that does nothing must say why** (T-79).
///
/// The two roads give one sentence, and that sentence says the truth of both:
/// the media of that line waits no more. The program cannot say which program
/// took it out, therefore the sentence names no program (T-91).
///
/// **The sentence names the place of that media** (T-233). `place` is the place
/// of the disk when this key took the media out, and it is the number of the
/// line of the view when a second program took it out first: the two roads give
/// the number that the user saw. The media that goes out changes the number of
/// every media after it, therefore the title alone leaves the user with no way
/// to read the view again.
///
/// The function is pure, therefore a test needs no queue and no database.
pub fn text_of_the_key_that_takes(
    place: usize,
    title_of_the_line: Option<&str>,
    what_went_out: Option<&str>,
) -> Option<String> {
    let title = what_went_out.or(title_of_the_line)?;

    Some(format!(
        "\"{}\" was number {} of the queue. It is not in the queue now.",
        title, place
    ))
}

/// Empties the queue of the process.
///
/// This takes every media of the account away, therefore it needs no read of the
/// disk: the queue of every program of that account is empty after it.
///
/// **A change that the disk did not take is no change** (T-206), therefore the
/// queue of the process comes back for a disk that said nothing, and the answer
/// is `false`.
pub fn clear() -> bool {
    let before = with_the_queue(|queue| queue.clone());
    with_the_queue(|queue| queue.clear());

    if !write_the_queue() {
        with_the_queue(|queue| *queue = before);

        return false;
    }

    true
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

    /// **The media that plays comes to its end, and the queue takes the media
    /// of the front away.** The media that the user chose then stands one line
    /// higher, and the cursor of the user must go with it: the key `X` took a
    /// media that the user did not choose out before this rule, and the key `l`
    /// played it. See T-161.
    #[test]
    fn the_cursor_of_the_user_goes_with_the_media_of_its_line() {
        let entries = vec![book("b", "Second"), book("c", "Third")];

        assert_eq!(
            what_the_line_of_the_user_holds(&entries, Some((1, "b")), Some(1)),
            TheLineOfTheUser::ItStandsAt(0),
            "the media of the line of the user stands one line higher now"
        );

        assert_eq!(
            what_the_line_of_the_user_holds(&entries, Some((1, "c")), Some(1)),
            TheLineOfTheUser::ItStandsAt(1),
            "a queue that did not move keeps the line of the user"
        );
    }

    /// The media of the line of the user leaves the queue: it came to the front
    /// and it plays now, or a second program of the account took it out. **No
    /// key of the selection may then reach a media that the user did not
    /// choose.** See T-161.
    #[test]
    fn the_media_of_the_line_of_the_user_can_go_away() {
        let entries = vec![book("b", "Second"), book("c", "Third")];

        assert_eq!(
            what_the_line_of_the_user_holds(&entries, Some((0, "a")), Some(0)),
            TheLineOfTheUser::ItWentAway,
            "the media that the user chose is not in the queue now"
        );

        assert_eq!(
            what_the_line_of_the_user_holds(&[], Some((0, "b")), Some(0)),
            TheLineOfTheUser::ItWentAway,
            "an empty queue holds no media of any line"
        );
    }

    /// The user moves the cursor, and that key is a choice: the program reads
    /// the media of the new line, and it says nothing at all. See T-161.
    #[test]
    fn a_key_of_the_user_gives_the_media_of_the_new_line() {
        let entries = vec![book("a", "First"), book("b", "Second")];

        assert_eq!(
            what_the_line_of_the_user_holds(&entries, Some((0, "a")), Some(1)),
            TheLineOfTheUser::TheUserChoseAnother,
            "the user pressed j after that frame"
        );

        assert_eq!(
            what_the_line_of_the_user_holds(&entries, Some((0, "a")), None),
            TheLineOfTheUser::TheUserChoseAnother,
            "no line of the user, therefore no media of a line"
        );

        assert_eq!(
            what_the_line_of_the_user_holds(&entries, None, Some(0)),
            TheLineOfTheUser::TheUserChoseAnother,
            "the program holds no media of a line yet"
        );
    }

    /// The text names the media, and it promises the two keys of the view
    /// only. See T-118, T-143, and T-161.
    #[test]
    fn the_text_names_the_media_that_left_the_queue() {
        let text = the_text_of_the_media_that_went_away("A Second Book Of Many Hours");

        assert!(
            text.contains("A Second Book Of Many Hours"),
            "the user must read which media went away: {}",
            text
        );

        assert!(
            text.contains("the keys j and k"),
            "the text must say how the user selects a media again: {}",
            text
        );

        for key in ["l:", "X:", "press Enter"] {
            assert!(
                !text.contains(key),
                "the text must promise no other key: {}",
                text
            );
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

        assert_eq!(queue.add(book("a", "First")).place, 1);
        assert_eq!(queue.add(book("b", "Second")).place, 2);
        assert_eq!(queue.add(book("c", "Third")).place, 3);

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

    /// **The disk holds one row for one media**, therefore the queue of the
    /// process holds one entry for it and the media moves to the end. The old
    /// rule of this test said that the media goes in a second time, and the key
    /// `n` then named a number that no line of the view held. See T-231.
    #[test]
    fn the_same_media_takes_one_place() {
        let mut queue = Queue::default();

        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));

        assert_eq!(queue.add(book("a", "First")).place, 2);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.entries()[1].key(), "a");
    }

    #[test]
    fn every_media_gives_one_line_with_its_place() {
        let mut queue = Queue::default();
        queue.add(book("a", "First"));
        queue.add(book("b", "Second"));

        let lines = queue.lines();

        assert_eq!(lines.len(), 2);
        // The mark of the place of the user stands at the start of the line,
        // and the number of the place stands after it. A media of no place
        // takes a mark of spaces alone. See T-230.
        assert!(
            lines[0].starts_with(&format!(
                "{}1. ",
                crate::ui::marks::of_progress("", "", false)
            )),
            "the line is {:?}",
            lines[0]
        );
        assert!(
            lines[1].starts_with(&format!(
                "{}2. ",
                crate::ui::marks::of_progress("", "", false)
            )),
            "the line is {:?}",
            lines[1]
        );
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
