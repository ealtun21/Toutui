//! The line of the view of the episodes of a podcast holds the place of that
//! episode. See T-229.
//!
//! **A line of that view is one episode**, and no line of it said one word of
//! that episode. The view took the titles of the server and it put them on the
//! screen: no mark of the media that plays, no mark of the end, and no percent
//! of the user.
//!
//! The measurement of the real program v0.8.57 inside tmux, against the sandbox
//! (podman on :13399), of the podcast `Arthur Gordon Pym` of the library
//! `Podcasts`. The server held `Chapter 00` at 30 percent, `Chapter 01`
//! finished, and `Chapter 02` at a place of 700 seconds. The user opened the
//! episodes of that podcast and played `Chapter 02`:
//!
//! - the eleven lines of that view each held the title alone, and the panel of
//!   the line said `[Arthur Gordon Pym] - Author: LibriVox - Episode: 2 -
//!   Duration: 39m` and nothing of the place of the user;
//! - the row of the player of that same frame said
//!   `Arthur Gordon Pym — Chapter 02 by LibriVox` with `⏸ 12:21 / 38:56`.
//!
//! **The control of the same run** (the trap 206): the Library view of that
//! same program gave `▶   Arthur Gordon Pym`, and the Home view gave
//! `▶   Chapter 02`, `✓   Letter 57`, and `Progress: 0%, Not finished` in its
//! panel. **The one view that holds a line for each episode held every mark of
//! none.**
//!
//! The same measurement of the corrected program gave `30% Chapter 00`,
//! `✓   Chapter 01`, and `▶   Chapter 02`, and the panel said
//! `Progress: 0%, Not finished`. A second client moved `Chapter 00` to 55
//! percent with `curl` while the program stood, and the line said
//! `55% Chapter 00` **269 milliseconds** later with no key of the user.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The function is pure, therefore this test needs no server and no screen.
//! **Three builds of the fault each fail it**: a line that reads no place at
//! all, a key of a line that drops the episode, and a line that takes the row of
//! its neighbour.

use toutui::logic::the_episodes::the_lines_of_the_episodes;

/// The podcast and its episodes of the measurement. Every episode of one
/// podcast holds the identity of that podcast, and that identity names no one
/// of them alone (T-223).
const THE_PODCAST: &str = "the-podcast-of-arthur-gordon-pym";

fn texts(values: &[&str]) -> Vec<String> {
    values.iter().map(|one| one.to_string()).collect()
}

/// The place of the user of each line, in the form of
/// `App::book_progress_cnt_list`: the percent and the mark of the end.
fn the_places() -> Vec<Vec<String>> {
    vec![
        texts(&["30", "Not finished"]),
        texts(&["100", "Finished"]),
        texts(&["0", "Not finished"]),
        texts(&[" N/A", " N/A"]),
    ]
}

#[test]
fn the_line_of_the_view_of_the_episodes_holds_its_place() {
    let titles = texts(&["Chapter 00", "Chapter 01", "Chapter 02", "Chapter 03"]);
    let ids = texts(&[
        "the-episode-of-chapter-00",
        "the-episode-of-chapter-01",
        "the-episode-of-chapter-02",
        "the-episode-of-chapter-03",
    ]);

    // `Chapter 02` plays. The key of a media names the episode after the item.
    let plays = format!("{}/{}", THE_PODCAST, "the-episode-of-chapter-02");

    let lines = the_lines_of_the_episodes(THE_PODCAST, &titles, &ids, &the_places(), Some(&plays));

    // **The place of the user stands on the line of its own episode.** The
    // program gave these four lines the title alone.
    assert_eq!(
        lines,
        texts(&[
            "30% Chapter 00",
            "✓   Chapter 01",
            "▶   Chapter 02",
            "    Chapter 03",
        ]),
        "each line of the view of the episodes holds the place of its episode"
    );

    // **The mark of the media that plays stands on one line.** The identity of
    // the item names every episode of that podcast (T-223), therefore a key that
    // drops the episode gives the mark `▶` to the whole podcast.
    assert_eq!(
        lines.iter().filter(|line| line.contains('▶')).count(),
        1,
        "one episode of a podcast plays, and one line holds the mark of it"
    );

    // **An episode that plays and that stands at no percent keeps the mark of
    // the media that plays**: the mark of the place of `Chapter 02` is 0
    // percent, and the line of it says that it plays.
    assert!(
        lines[2].starts_with('▶'),
        "the line of the episode that plays holds the mark of the media that plays"
    );

    // **A media that no place of the account names takes no mark at all**, as a
    // media that never played takes none in the Home view.
    assert_eq!(
        lines[3], "    Chapter 03",
        "an episode of no place of the account holds its title alone"
    );

    // **A podcast whose episodes no media plays holds no mark of a playback.**
    let no_playback = the_lines_of_the_episodes(THE_PODCAST, &titles, &ids, &the_places(), None);

    assert!(
        !no_playback.iter().any(|line| line.contains('▶')),
        "no media plays, therefore no line holds the mark of the media that plays"
    );

    // **A playback of a second podcast reaches no line of this one.**
    let a_second_podcast = the_lines_of_the_episodes(
        THE_PODCAST,
        &titles,
        &ids,
        &the_places(),
        Some("a-second-podcast/the-episode-of-chapter-02"),
    );

    assert_eq!(
        a_second_podcast, no_playback,
        "an episode of a different podcast plays, and no line of this podcast holds its mark"
    );

    // **An episode whose identity the server did not give keeps its line**
    // (T-226): it holds no key, therefore no media that plays stands on it.
    let no_identity = the_lines_of_the_episodes(
        THE_PODCAST,
        &titles,
        &texts(&["", "", "", ""]),
        &the_places(),
        Some(&plays),
    );

    assert_eq!(
        no_identity,
        texts(&[
            "30% Chapter 00",
            "✓   Chapter 01",
            "    Chapter 02",
            "    Chapter 03",
        ]),
        "an episode of no identity keeps its line and its place, and it takes no mark of a playback"
    );

    // **A list of the places that the server did not give leaves every title.**
    // The request of the places that failed takes a line of the log, and the
    // view stays the view that the user reads.
    let no_place = the_lines_of_the_episodes(THE_PODCAST, &titles, &ids, &[], Some(&plays));

    assert_eq!(
        no_place,
        texts(&[
            "    Chapter 00",
            "    Chapter 01",
            "▶   Chapter 02",
            "    Chapter 03",
        ]),
        "a view of no place of the user keeps its titles and the mark of the media that plays"
    );
}
