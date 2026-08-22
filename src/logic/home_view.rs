//! The lines of the Home view. See T-24.
//!
//! `GET /api/libraries/:id/personalized` gives six shelves for a library of
//! books, and the program kept one of them. This module makes the lines of
//! every shelf: a line for the name of the shelf, and a line for each media of
//! that shelf.
//!
//! The functions here are pure, therefore a test needs no server and no
//! screen.

use crate::api::libraries::get_library_perso_view::{Entity, Root};
use crate::api::libraries::get_library_perso_view_pod::{Entity as EntityPod, Root as RootPod};
use crate::api::utils::collect_series::SeriesView;

/// A text of no letter is no identity. See T-114.
fn some_identity(id: Option<&String>) -> bool {
    id.is_some_and(|id| !id.trim().is_empty())
}

/// Tells if an entity of a library of books gives the program a road to a
/// playback.
///
/// **An entity with no identity gives no such road**: the key `Enter` sends a
/// path built of that identity, and a path with no identity asks the server
/// for an item that does not exist (T-389, the same fault as T-388). Every
/// function of this module that counts a media of the Home view must agree
/// with `crate::api::utils::collect_personalized_view::media_entities` on
/// which entity gives a line (the rule of T-24), and this is that one rule.
fn media_entity_has_an_identity(entity: &Entity) -> bool {
    some_identity(entity.id.as_ref())
}

/// The same rule for a library of podcasts: an episode needs the identity of
/// the episode itself and the identity of the item that holds it, because the
/// key `Enter` builds a path of the two of them.
fn episode_entity_has_an_identity(entity: &EntityPod) -> bool {
    let Some(episode) = entity.recent_episode.as_ref() else {
        return false;
    };

    some_identity(episode.id.as_ref()) && some_identity(episode.library_item_id.as_ref())
}

/// One line of the Home view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeRow {
    /// The name of a shelf. The user cannot select this line.
    Shelf { label: String },
    /// One media. The number is the position of the media in
    /// `_ids_cnt_list` and in every other list of the Home view.
    Media { item: usize },
    /// One series of the shelf `recent-series`. The number is the position of
    /// the series in `App::series`.
    Series { series: usize },
}

impl HomeRow {
    /// Tells if the user can select this line.
    ///
    /// A line that names a shelf is a title, and not a media. The keys `j` and
    /// `k` go over it.
    pub fn is_a_line_of_the_user(&self) -> bool {
        !matches!(self, HomeRow::Shelf { .. })
    }

    /// Gives the position in the lists of the Home view, if the line holds a
    /// media.
    pub fn item(&self) -> Option<usize> {
        match self {
            HomeRow::Media { item } => Some(*item),
            _ => None,
        }
    }

    /// Gives the position of the series, if the line holds a series.
    pub fn series(&self) -> Option<usize> {
        match self {
            HomeRow::Series { series } => Some(*series),
            _ => None,
        }
    }
}

/// Makes the lines of the Home view of a library of books.
///
/// The sequence of the shelves is the sequence of the server. The number of a
/// line of a media counts the media of every shelf before it, and it counts
/// them in the same way as `crate::api::utils::collect_personalized_view`: an
/// entity that holds no media gives no number.
///
/// **A shelf that gives no line gives no name.** The shelf `newest-authors`
/// holds an author, an author holds no media and no book, therefore that shelf
/// would give a name with nothing below it.
///
/// A shelf of series gives a line for each series that `series` holds. The
/// user opens that line with the key `l`, and the books of the series come in
/// the sequence of the series, as they do in the Library view. See T-22.
pub fn group_home(shelves: &[Root], series: &[SeriesView]) -> Vec<HomeRow> {
    let mut rows: Vec<HomeRow> = Vec::new();
    let mut item = 0;

    for shelf in shelves {
        let mut of_this_shelf: Vec<HomeRow> = Vec::new();

        for entity in shelf.entities.iter().flatten() {
            // **An entity with no identity gives no line here** (T-389): the
            // sequence of this function must be the sequence of
            // `crate::api::utils::collect_personalized_view::media_entities`
            // (the rule of T-24), and that function now drops such an entity
            // too, because the program has no road to a playback of it.
            if entity.media.is_some() {
                if media_entity_has_an_identity(entity) {
                    of_this_shelf.push(HomeRow::Media { item });
                    item += 1;
                }
                continue;
            }

            // A series of the shelf `recent-series`. The program holds the
            // series of the library already, therefore the line points at one
            // of them and the view of the books needs no new request.
            if entity.books.is_some() {
                if let Some(index) = position_of_the_series(entity.id.as_deref(), series) {
                    of_this_shelf.push(HomeRow::Series { series: index });
                }
            }
        }

        if of_this_shelf.is_empty() {
            continue;
        }

        rows.push(HomeRow::Shelf {
            label: the_name_of_the_shelf(shelf.id.as_deref(), &shelf.label),
        });
        rows.append(&mut of_this_shelf);
    }

    rows
}

/// Makes the lines of the Home view of a library of podcasts.
///
/// A library of podcasts gives no series. The number of a line counts the
/// entities that hold an episode and a media, in the same way as
/// `crate::api::utils::collect_personalized_view_pod`.
pub fn group_home_pod(shelves: &[RootPod]) -> Vec<HomeRow> {
    let mut rows: Vec<HomeRow> = Vec::new();
    let mut item = 0;

    for shelf in shelves {
        let mut of_this_shelf: Vec<HomeRow> = Vec::new();

        for entity in shelf.entities.iter().flatten() {
            // **An episode or a podcast with no identity gives no line here**
            // (T-389): the sequence of this function must be the sequence of
            // `crate::api::utils::collect_personalized_view_pod::episode_entities`
            // (the rule of T-24), and that function now drops such an entity
            // too, because the program has no road to a playback of it.
            if entity.recent_episode.is_none() || entity.media.is_none() {
                continue;
            }

            if episode_entity_has_an_identity(entity) {
                of_this_shelf.push(HomeRow::Media { item });
                item += 1;
            }
        }

        if of_this_shelf.is_empty() {
            continue;
        }

        rows.push(HomeRow::Shelf {
            label: the_name_of_the_shelf(shelf.id.as_deref(), &shelf.label),
        });
        rows.append(&mut of_this_shelf);
    }

    rows
}

/// Tells, for each media of the lists of the Home view, if that media stands on
/// the shelf of Continue Listening.
///
/// The number of the value is the number of `HomeRow::Media`, therefore this
/// function counts the entities in the same way as `group_home`, and it must
/// keep the same rule of an identity (T-389).
pub fn the_media_of_continue_listening(shelves: &[Root]) -> Vec<bool> {
    let mut of_the_shelf: Vec<bool> = Vec::new();

    for shelf in shelves {
        let is_the_shelf = the_shelf_of_continue_listening(shelf.id.as_deref(), &shelf.label);

        for entity in shelf.entities.iter().flatten() {
            if entity.media.is_some() && media_entity_has_an_identity(entity) {
                of_the_shelf.push(is_the_shelf);
            }
        }
    }

    of_the_shelf
}

/// The same for a library of podcasts. See `group_home_pod` for the count.
pub fn the_media_of_continue_listening_pod(shelves: &[RootPod]) -> Vec<bool> {
    let mut of_the_shelf: Vec<bool> = Vec::new();

    for shelf in shelves {
        let is_the_shelf = the_shelf_of_continue_listening(shelf.id.as_deref(), &shelf.label);

        for entity in shelf.entities.iter().flatten() {
            if entity.recent_episode.is_none() || entity.media.is_none() {
                continue;
            }

            if episode_entity_has_an_identity(entity) {
                of_the_shelf.push(is_the_shelf);
            }
        }
    }

    of_the_shelf
}

/// Gives the name of the line of a shelf.
///
/// **The label of a shelf is a name for the user, and it is no address**: a
/// shelf with no label keeps its media, because the id of each of those media
/// reaches every request of the program. The line of that shelf needs a name
/// all the same, and the program has two roads to it:
///
/// 1. The label of the server, when it holds a character.
/// 2. The identity of the shelf, which is the same on every server (T-24):
///    `continue-listening`, `recently-added`, and the others.
///
/// A shelf that holds neither takes the name of this program. That name says
/// what the program has, and it promises nothing (T-91 and T-118). See T-190.
pub fn the_name_of_the_shelf(id: Option<&str>, label: &str) -> String {
    if !label.trim().is_empty() {
        return label.to_string();
    }

    match id {
        Some(id) if !id.trim().is_empty() => id.to_string(),
        _ => "A shelf with no name".to_string(),
    }
}

/// Tells if a shelf is the shelf of Continue Listening.
///
/// The identity comes first, because it is the same on every server. The label
/// is the answer for a server that gives no identity. See T-24.
fn the_shelf_of_continue_listening(id: Option<&str>, label: &str) -> bool {
    match id {
        Some(id) => id == "continue-listening",
        None => label == "Continue Listening",
    }
}

/// Takes the media that left the shelf of Continue Listening away from the
/// lines.
///
/// `has_left` reads the number of the media, and that number does **not**
/// change: the lists of the Home view stand beside the lines, therefore a line
/// that goes away must not move the number of the line after it.
///
/// **A shelf that holds no line any more gives no name.** This is the rule of
/// `group_home`, and a name with nothing below it says nothing to the user.
/// See T-66.
pub fn without_the_media_that_left(
    rows: &[HomeRow],
    has_left: impl Fn(usize) -> bool,
) -> Vec<HomeRow> {
    let kept: Vec<&HomeRow> = rows
        .iter()
        .filter(|row| match row {
            HomeRow::Media { item } => !has_left(*item),
            _ => true,
        })
        .collect();

    let mut answer: Vec<HomeRow> = Vec::new();

    for (place, row) in kept.iter().enumerate() {
        if matches!(row, HomeRow::Shelf { .. }) {
            let holds_a_line = matches!(
                kept.get(place + 1),
                Some(HomeRow::Media { .. }) | Some(HomeRow::Series { .. })
            );

            if !holds_a_line {
                continue;
            }
        }

        answer.push((*row).clone());
    }

    answer
}

/// Gives the key of one media of the Home view.
///
/// **A line of a library of podcasts is one episode**, and the identity of the
/// item names every episode of one podcast (T-223). The mark of the media that
/// plays stood on every line of that podcast, and the position of a live message
/// reached no line of it: the user played `Chapter 01` of `Arthur Gordon Pym` of
/// the sandbox, and `Chapter 00` took the mark of the media that plays with it
/// (T-228).
///
/// `episode_ids` holds the second half of the key, and it holds no value at all
/// for a library of books: the key of a book is the identity of its item. The
/// value is the value of `crate::logic::live::the_key_of_the_media`, therefore
/// the key of a line, the key of the media that plays, the key of a position of
/// a live message, and the key of `the_media_that_left_the_shelf` are one value.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_key_of_the_line(ids: &[String], episode_ids: &[String], item: usize) -> Option<String> {
    ids.get(item).map(|id| {
        crate::logic::live::the_key_of_the_media(id, episode_ids.get(item).map(String::as_str))
    })
}

/// Gives the number of each media of the Home view that left the shelf of
/// Continue Listening.
///
/// **The list holds the number of the line, and not the identity of the
/// media.** One media stands on two shelves: a measurement of 2026-08-11 showed
/// a book on Continue Listening and on Recently Added together. A list of the
/// identities took both lines away, and the server gives the second one.
/// Therefore each shelf gives its own number, and `on_the_shelf` says which
/// number belongs to Continue Listening. See T-66.
///
/// **A line of a library of podcasts is one episode** (T-226). `ids` holds the
/// identity of the **podcast** for each of those lines, therefore two episodes
/// of one podcast hold one value there and the identity of the item names no
/// line alone (T-223). `episode_ids` gives the second half of the key, and it
/// holds no value at all for a library of books: the key of a book is the
/// identity of its item.
///
/// `away` holds the keys of `crate::logic::live::the_key_of_the_media`. An
/// episode whose identity the server did not give takes a key that no message
/// carries, therefore its line stays: a line that stays is the safe road, and a
/// line that goes away with no reason takes the media of the user with it
/// (T-203).
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_media_that_left_the_shelf(
    ids: &[String],
    episode_ids: &[String],
    on_the_shelf: &[bool],
    away: &std::collections::BTreeSet<String>,
) -> std::collections::BTreeSet<usize> {
    let mut answer = std::collections::BTreeSet::new();

    for (item, stands) in on_the_shelf.iter().enumerate() {
        if !stands {
            continue;
        }

        let Some(id) = ids.get(item) else {
            continue;
        };

        let key =
            crate::logic::live::the_key_of_the_media(id, episode_ids.get(item).map(String::as_str));

        if away.contains(&key) {
            answer.insert(item);
        }
    }

    answer
}

/// Tells which media went away from under the line of the user, if one did.
///
/// `rows` and `selected` are the lines and the line of the user **before** the
/// change, and the answer is the number of the media of the lists of the Home
/// view.
///
/// **A media that goes away moves the media below it under the line of the
/// user.** The lines keep the number of the line, therefore the media that
/// stood below takes that number with no word at all: the key `M` of two
/// presses marked two media, and a second window of the account made the same
/// thing with no key of this user (T-160).
pub fn the_media_of_the_line_that_went_away(
    rows: &[HomeRow],
    selected: Option<usize>,
    has_left: impl Fn(usize) -> bool,
) -> Option<usize> {
    let item = rows.get(selected?)?.item()?;

    has_left(item).then_some(item)
}

/// The text for the user when the media of their line goes away from the shelf
/// of Continue Listening.
///
/// **The program cannot know which media the user wants now**, therefore it
/// takes the line away and it says what happened. A key of the selection then
/// changes no media at all, and the user chooses the next one. The text names
/// the two keys that the Home view holds, and it promises no other key (T-118
/// and T-143). See T-160.
pub fn the_text_of_the_media_that_went_away(title: &str) -> String {
    format!(
        "The media \"{}\" is not on the shelf Continue Listening now. \
         No line is selected: the keys j and k select one.",
        title
    )
}

/// Gives the position of a series of the list of the series.
fn position_of_the_series(id: Option<&str>, series: &[SeriesView]) -> Option<usize> {
    let id = id?;
    series.iter().position(|one| one.id == id)
}

/// Gives one value for each line: `true` for a line that the user can
/// select, and `false` for the name of a shelf.
fn lines_of_the_user(rows: &[HomeRow]) -> Vec<bool> {
    rows.iter().map(HomeRow::is_a_line_of_the_user).collect()
}

/// Gives the first line that the user can select.
pub fn first_line(rows: &[HomeRow]) -> Option<usize> {
    crate::logic::list_moves::first(&lines_of_the_user(rows))
}

/// Gives the last line that the user can select.
pub fn last_line(rows: &[HomeRow]) -> Option<usize> {
    crate::logic::list_moves::last(&lines_of_the_user(rows))
}

/// Gives the line after this one. The move goes to the first line at the end.
pub fn next_line(rows: &[HomeRow], from: usize) -> Option<usize> {
    crate::logic::list_moves::next(&lines_of_the_user(rows), from)
}

/// Gives the line before this one. The move stops at the first line.
pub fn previous_line(rows: &[HomeRow], from: usize) -> Option<usize> {
    crate::logic::list_moves::previous(&lines_of_the_user(rows), from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(id: &str, name: &str) -> SeriesView {
        SeriesView {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            books: Vec::new(),
        }
    }

    /// The answer of the sandbox for a library of books, measured on
    /// 2026-08-11. The numbers of the media are smaller here, and the shape is
    /// the same.
    fn the_shelves() -> Vec<Root> {
        serde_json::from_value(serde_json::json!([
            { "id": "continue-listening", "label": "Continue Listening",
              "entities": [ { "id": "a", "media": {} }, { "id": "b", "media": {} } ] },
            { "id": "recently-added", "label": "Recently Added",
              "entities": [ { "id": "c", "media": {} } ] },
            { "id": "recent-series", "label": "Recent Series",
              "entities": [ { "id": "series-1", "name": "A Series", "books": [] },
                            { "id": "series-9", "name": "A Series of No List", "books": [] } ] },
            { "id": "discover", "label": "Discover",
              "entities": [ { "id": "d", "media": {} } ] },
            { "id": "newest-authors", "label": "Newest Authors",
              "entities": [ { "id": "author-1", "name": "An Author" } ] }
        ]))
        .expect("the answer of the server must read")
    }

    #[test]
    fn every_shelf_gives_a_name_and_its_media() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        assert_eq!(
            rows,
            vec![
                HomeRow::Shelf {
                    label: "Continue Listening".to_string()
                },
                HomeRow::Media { item: 0 },
                HomeRow::Media { item: 1 },
                HomeRow::Shelf {
                    label: "Recently Added".to_string()
                },
                HomeRow::Media { item: 2 },
                HomeRow::Shelf {
                    label: "Recent Series".to_string()
                },
                HomeRow::Series { series: 0 },
                HomeRow::Shelf {
                    label: "Discover".to_string()
                },
                HomeRow::Media { item: 3 },
            ]
        );
    }

    /// The shelf of the authors holds no media and no book. A name with
    /// nothing below it says nothing to the user.
    #[test]
    fn a_shelf_that_gives_no_line_gives_no_name() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        assert!(!rows.contains(&HomeRow::Shelf {
            label: "Newest Authors".to_string()
        }));
    }

    /// A series that the program does not hold gives no line. The view of the
    /// books reads `App::series`, therefore a line that points at nothing
    /// would open an empty view.
    #[test]
    fn a_series_that_the_program_does_not_hold_gives_no_line() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        assert_eq!(
            rows.iter().filter(|row| row.series().is_some()).count(),
            1,
            "the shelf holds two series, and the program holds one of them"
        );
    }

    /// A library of podcasts gives no series. The shelf of the series is then
    /// absent, and the shelf of the media stays.
    #[test]
    fn a_library_with_no_series_gives_the_shelves_of_the_media() {
        let rows = group_home(&the_shelves(), &[]);

        assert!(!rows.contains(&HomeRow::Shelf {
            label: "Recent Series".to_string()
        }));
        assert_eq!(rows.iter().filter(|row| row.item().is_some()).count(), 4);
    }

    /// The number of a media counts the media of the shelves before it. It
    /// must not count a series and it must not count an author, because the
    /// lists of the Home view hold no value for them.
    #[test]
    fn the_number_of_a_media_counts_the_media_only() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);
        let numbers: Vec<usize> = rows.iter().filter_map(HomeRow::item).collect();

        assert_eq!(numbers, vec![0, 1, 2, 3]);
    }

    #[test]
    fn an_answer_with_no_shelf_gives_no_line() {
        assert!(group_home(&[], &[]).is_empty());
        assert!(group_home_pod(&[]).is_empty());
        assert_eq!(first_line(&[]), None);
        assert_eq!(last_line(&[]), None);
    }

    #[test]
    fn a_shelf_of_podcasts_gives_the_episodes_only() {
        let shelves: Vec<RootPod> = serde_json::from_value(serde_json::json!([
            { "id": "newest-episodes", "label": "Newest Episodes",
              "entities": [ { "id": "p1", "media": {},
                              "recentEpisode": { "id": "e1", "libraryItemId": "p1" } } ] },
            { "id": "recently-added", "label": "Recently Added",
              "entities": [ { "id": "p2", "media": {} } ] },
            { "id": "listen-again", "label": "Listen Again",
              "entities": [ { "id": "p1", "media": {},
                              "recentEpisode": { "id": "e2", "libraryItemId": "p1" } } ] }
        ]))
        .expect("the answer must read");

        assert_eq!(
            group_home_pod(&shelves),
            vec![
                HomeRow::Shelf {
                    label: "Newest Episodes".to_string()
                },
                HomeRow::Media { item: 0 },
                HomeRow::Shelf {
                    label: "Listen Again".to_string()
                },
                HomeRow::Media { item: 1 },
            ]
        );
    }

    /// A media, or an episode, with no identity gives no line: the program has
    /// no road to a playback of it, and the number of a line must still count
    /// the same media as `crate::api::utils::collect_personalized_view` and
    /// `crate::api::utils::collect_personalized_view_pod` (the rule of T-24).
    /// See T-389.
    #[test]
    fn a_media_with_no_identity_gives_no_line() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "id": "continue-listening", "label": "Continue Listening",
              "entities": [ { "media": {} },
                            { "id": "   ", "media": {} },
                            { "id": "b", "media": {} } ] }
        ]))
        .expect("the answer must read");

        assert_eq!(
            group_home(&shelves, &[]),
            vec![
                HomeRow::Shelf {
                    label: "Continue Listening".to_string()
                },
                HomeRow::Media { item: 0 },
            ]
        );
    }

    #[test]
    fn an_episode_with_no_identity_gives_no_line() {
        let shelves: Vec<RootPod> = serde_json::from_value(serde_json::json!([
            { "id": "newest-episodes", "label": "Newest Episodes",
              "entities": [
                  { "id": "p1", "media": {}, "recentEpisode": { "libraryItemId": "p1" } },
                  { "id": "p1", "media": {}, "recentEpisode": { "id": "e1" } },
                  { "id": "p1", "media": {},
                    "recentEpisode": { "id": "e2", "libraryItemId": "p1" } }
              ] }
        ]))
        .expect("the answer must read");

        assert_eq!(
            group_home_pod(&shelves),
            vec![
                HomeRow::Shelf {
                    label: "Newest Episodes".to_string()
                },
                HomeRow::Media { item: 0 },
            ]
        );
    }

    #[test]
    fn the_move_goes_over_the_name_of_a_shelf() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        // 0 is a name, 1 and 2 are media, 3 is a name, 4 is a media.
        assert_eq!(first_line(&rows), Some(1));
        assert_eq!(next_line(&rows, 1), Some(2));
        assert_eq!(
            next_line(&rows, 2),
            Some(4),
            "the name of a shelf is not a line"
        );
        assert_eq!(previous_line(&rows, 4), Some(2));
        assert_eq!(
            previous_line(&rows, 1),
            Some(1),
            "the move stops at the first line"
        );
        assert_eq!(last_line(&rows), Some(8));
    }

    #[test]
    fn the_move_goes_back_to_the_first_line_at_the_end() {
        let rows = group_home(&the_shelves(), &[]);
        let last = last_line(&rows).expect("the view holds a line");

        assert_eq!(next_line(&rows, last), first_line(&rows));
    }

    /// The value belongs to the number of the media, and not to the number of
    /// the line. The shelf of Continue Listening holds the media 0 and 1 of
    /// the answer of the sandbox.
    #[test]
    fn the_shelf_of_each_media_of_the_home_view() {
        assert_eq!(
            the_media_of_continue_listening(&the_shelves()),
            vec![true, true, false, false],
            "the two media of Continue Listening, the media of Recently Added, and the media of Discover"
        );
    }

    /// A server that gives no identity of a shelf gives the label only.
    #[test]
    fn the_label_names_the_shelf_when_the_identity_is_absent() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "label": "Continue Listening", "entities": [ { "id": "a", "media": {} } ] },
            { "label": "Recently Added", "entities": [ { "id": "b", "media": {} } ] }
        ]))
        .expect("the answer must read");

        assert_eq!(the_media_of_continue_listening(&shelves), vec![true, false]);
    }

    /// A media that the user finished leaves the shelf of Continue Listening,
    /// and it stays on every other shelf. See T-66.
    #[test]
    fn a_media_that_left_goes_away_from_its_line() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        // The media 1 is the second media of Continue Listening.
        let after = without_the_media_that_left(&rows, |item| item == 1);

        assert_eq!(
            after,
            vec![
                HomeRow::Shelf {
                    label: "Continue Listening".to_string()
                },
                HomeRow::Media { item: 0 },
                HomeRow::Shelf {
                    label: "Recently Added".to_string()
                },
                HomeRow::Media { item: 2 },
                HomeRow::Shelf {
                    label: "Recent Series".to_string()
                },
                HomeRow::Series { series: 0 },
                HomeRow::Shelf {
                    label: "Discover".to_string()
                },
                HomeRow::Media { item: 3 },
            ],
            "the number of every other media must not change"
        );
    }

    /// **The media of the line of the user went away, and the line below it
    /// takes that number of line.** The program must know it, therefore it can
    /// take the line away and say what happened. See T-160.
    #[test]
    fn the_media_of_the_line_of_the_user_can_go_away() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        // The line 2 holds the media 1: the line 0 names the shelf, and the
        // line 1 holds the media 0.
        assert_eq!(rows[2], HomeRow::Media { item: 1 });

        assert_eq!(
            the_media_of_the_line_that_went_away(&rows, Some(2), |item| item == 1),
            Some(1),
            "the media of the line of the user went away"
        );

        assert_eq!(
            the_media_of_the_line_that_went_away(&rows, Some(1), |item| item == 1),
            None,
            "a media that goes away below the line of the user changes no key"
        );
    }

    /// A line that holds no media, and a view that holds no line, give
    /// nothing. See T-160.
    #[test]
    fn a_line_of_no_media_never_says_that_a_media_went_away() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        assert!(matches!(rows[0], HomeRow::Shelf { .. }));

        assert_eq!(
            the_media_of_the_line_that_went_away(&rows, Some(0), |_| true),
            None,
            "the name of a shelf holds no media"
        );

        assert_eq!(
            the_media_of_the_line_that_went_away(&rows, None, |_| true),
            None,
            "no line of the user, therefore no media went away from under it"
        );

        assert_eq!(
            the_media_of_the_line_that_went_away(&rows, Some(rows.len()), |_| true),
            None,
            "a line outside the list holds no media"
        );
    }

    /// The text names the media, and it promises the two keys of the view
    /// only. See T-118, T-143, and T-160.
    #[test]
    fn the_text_names_the_media_that_went_away() {
        let text = the_text_of_the_media_that_went_away("A Book Of Many Hours");

        assert!(
            text.contains("A Book Of Many Hours"),
            "the user must read which media went away: {}",
            text
        );

        assert!(
            text.contains("the keys j and k"),
            "the text must say how the user selects a media again: {}",
            text
        );

        for key in ["l:", "M:", "press Enter"] {
            assert!(
                !text.contains(key),
                "the text must promise no other key: {}",
                text
            );
        }
    }

    /// A shelf that holds no media any more must give no name. This is the
    /// rule of `group_home`. See T-66.
    #[test]
    fn a_shelf_with_no_media_left_gives_no_name() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);
        let after = without_the_media_that_left(&rows, |item| item == 0 || item == 1);

        assert!(
            !after.contains(&HomeRow::Shelf {
                label: "Continue Listening".to_string()
            }),
            "the shelf holds no line, therefore it gives no name"
        );
        assert_eq!(
            after.first(),
            Some(&HomeRow::Shelf {
                label: "Recently Added".to_string()
            })
        );
        assert_eq!(
            after.iter().filter_map(HomeRow::item).collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    /// A shelf of series keeps its name, because a series is a line too.
    #[test]
    fn a_shelf_of_series_keeps_its_name() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);
        let after = without_the_media_that_left(&rows, |_| true);

        assert_eq!(
            after,
            vec![
                HomeRow::Shelf {
                    label: "Recent Series".to_string()
                },
                HomeRow::Series { series: 0 },
            ],
            "every media left, and the shelf of the series stays"
        );
    }

    /// Nothing left, therefore the lines do not change.
    #[test]
    fn no_media_that_left_keeps_every_line() {
        let rows = group_home(&the_shelves(), &[series("series-1", "A Series")]);

        assert_eq!(without_the_media_that_left(&rows, |_| false), rows);
    }

    /// A library of podcasts can give a shelf of Continue Listening, and it
    /// counts the entities that hold an episode.
    #[test]
    fn the_shelf_of_each_episode_of_a_library_of_podcasts() {
        let shelves: Vec<RootPod> = serde_json::from_value(serde_json::json!([
            { "id": "continue-listening", "label": "Continue Listening",
              "entities": [ { "id": "p1", "media": {},
                              "recentEpisode": { "id": "e1", "libraryItemId": "p1" } },
                            { "id": "p2", "media": {} } ] },
            { "id": "newest-episodes", "label": "Newest Episodes",
              "entities": [ { "id": "p3", "media": {},
                              "recentEpisode": { "id": "e2", "libraryItemId": "p3" } } ] }
        ]))
        .expect("the answer must read");

        assert_eq!(
            the_media_of_continue_listening_pod(&shelves),
            vec![true, false],
            "the entity with no episode gives no line and no value"
        );
    }

    /// A view of names only must not give a line, and it must not stop the
    /// program.
    #[test]
    fn a_view_of_names_only_gives_no_line() {
        let rows = vec![HomeRow::Shelf {
            label: "A Shelf".to_string(),
        }];

        assert_eq!(first_line(&rows), None);
        assert_eq!(last_line(&rows), None);
        assert_eq!(next_line(&rows, 0), None);
        assert_eq!(previous_line(&rows, 0), None);
    }

    /// **A shelf with no name keeps its media.** The label of a shelf is a
    /// name for the user and no address, therefore the line of that shelf
    /// takes the identity of the shelf. See T-190.
    #[test]
    fn the_name_of_a_shelf_comes_of_the_label_and_then_of_the_identity() {
        assert_eq!(
            the_name_of_the_shelf(Some("recently-added"), "Recently Added"),
            "Recently Added"
        );
        assert_eq!(
            the_name_of_the_shelf(Some("recently-added"), ""),
            "recently-added",
            "the identity is the name of a shelf that the server did not name"
        );
        assert_eq!(
            the_name_of_the_shelf(Some("recently-added"), "   "),
            "recently-added",
            "a name of no character is a name that the server did not give"
        );
        assert_eq!(
            the_name_of_the_shelf(None, ""),
            "A shelf with no name",
            "the program names a shelf that holds neither"
        );
        assert_eq!(
            the_name_of_the_shelf(Some("  "), ""),
            "A shelf with no name"
        );
    }

    /// The lines of a shelf with no name stay, and the line of that shelf
    /// holds its identity. See T-190.
    #[test]
    fn a_shelf_with_no_name_keeps_its_media() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "id": "continue-listening", "label": "Continue Listening",
              "entities": [ { "id": "a", "media": {} } ] },
            { "id": "recently-added",
              "entities": [ { "id": "b", "media": {} } ] }
        ]))
        .unwrap();

        let rows = group_home(&shelves, &[]);

        assert_eq!(
            rows,
            vec![
                HomeRow::Shelf {
                    label: "Continue Listening".to_string()
                },
                HomeRow::Media { item: 0 },
                HomeRow::Shelf {
                    label: "recently-added".to_string()
                },
                HomeRow::Media { item: 1 },
            ]
        );
    }

    /// The same for a library of podcasts. See T-190.
    #[test]
    fn a_shelf_of_podcasts_with_no_name_keeps_its_media() {
        let shelves: Vec<RootPod> = serde_json::from_value(serde_json::json!([
            { "id": "newest-episodes",
              "entities": [ { "id": "a", "media": {},
                              "recentEpisode": { "id": "e1", "libraryItemId": "a" } } ] }
        ]))
        .unwrap();

        assert_eq!(
            group_home_pod(&shelves),
            vec![
                HomeRow::Shelf {
                    label: "newest-episodes".to_string()
                },
                HomeRow::Media { item: 0 },
            ]
        );
    }
}
