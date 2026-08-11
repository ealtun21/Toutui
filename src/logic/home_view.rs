//! The lines of the Home view. See T-24.
//!
//! `GET /api/libraries/:id/personalized` gives six shelves for a library of
//! books, and the program kept one of them. This module makes the lines of
//! every shelf: a line for the name of the shelf, and a line for each media of
//! that shelf.
//!
//! The functions here are pure, therefore a test needs no server and no
//! screen.

use crate::api::libraries::get_library_perso_view::Root;
use crate::api::libraries::get_library_perso_view_pod::Root as RootPod;
use crate::api::utils::collect_series::SeriesView;

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
            if entity.media.is_some() {
                of_this_shelf.push(HomeRow::Media { item });
                item += 1;
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
            label: shelf.label.clone(),
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
            if entity.recent_episode.is_some() && entity.media.is_some() {
                of_this_shelf.push(HomeRow::Media { item });
                item += 1;
            }
        }

        if of_this_shelf.is_empty() {
            continue;
        }

        rows.push(HomeRow::Shelf {
            label: shelf.label.clone(),
        });
        rows.append(&mut of_this_shelf);
    }

    rows
}

/// Tells, for each media of the lists of the Home view, if that media stands on
/// the shelf of Continue Listening.
///
/// The number of the value is the number of `HomeRow::Media`, therefore this
/// function counts the entities in the same way as `group_home`.
pub fn the_media_of_continue_listening(shelves: &[Root]) -> Vec<bool> {
    let mut of_the_shelf: Vec<bool> = Vec::new();

    for shelf in shelves {
        let is_the_shelf = the_shelf_of_continue_listening(shelf.id.as_deref(), &shelf.label);

        for entity in shelf.entities.iter().flatten() {
            if entity.media.is_some() {
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
            if entity.recent_episode.is_some() && entity.media.is_some() {
                of_the_shelf.push(is_the_shelf);
            }
        }
    }

    of_the_shelf
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
              "entities": [ { "id": "p1", "media": {}, "recentEpisode": { "id": "e1" } } ] },
            { "id": "recently-added", "label": "Recently Added",
              "entities": [ { "id": "p2", "media": {} } ] },
            { "id": "listen-again", "label": "Listen Again",
              "entities": [ { "id": "p1", "media": {}, "recentEpisode": { "id": "e2" } } ] }
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
              "entities": [ { "id": "p1", "media": {}, "recentEpisode": { "id": "e1" } },
                            { "id": "p2", "media": {} } ] },
            { "id": "newest-episodes", "label": "Newest Episodes",
              "entities": [ { "id": "p3", "media": {}, "recentEpisode": { "id": "e2" } } ] }
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
}
