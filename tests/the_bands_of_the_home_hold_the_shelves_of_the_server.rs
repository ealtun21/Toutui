//! The bands of the Home view hold the shelves of the server. See T-335.
//!
//! The maintainer asked for a Home view of bands of covers on 2026-08-16
//! (T-331), and
//! `docs/superpowers/specs/2026-08-17-the-home-view-of-the-bands-of-covers-design.md`
//! holds the design of it. **The bands are a shape of the render, and the flat
//! list of the lines of `crate::logic::home_view` stays the data.** This file
//! measures the module of the bands against the answer of the sandbox.
//!
//! **The measurement of the sandbox of 2026-08-17**,
//! `GET /api/libraries/<id>/personalized`, and the real program of v0.8.167
//! inside tmux of 160 columns and 45 rows:
//!
//! | The library | The shelves of the server | The Home view of the program |
//! |---|---|---|
//! | `Books` | `continue-listening` (5), `recently-added` (10 of 22), `recent-series` (3), `discover` (7), `listen-again` (10), `newest-authors` (9) | `4 Home [35 items]`, five titles, and no title of the authors |
//! | `Podcasts` | `continue-listening` (4), `newest-episodes` (10 of 68), `recently-added` (2 podcasts), `listen-again` (3) | `4 Home [17 items]`, three titles, and no title of Recently Added |
//!
//! **A shelf that gives no line gives no band**, and the two libraries hold one
//! such shelf each: the shelf `newest-authors` of a library of books holds an
//! author and no media, and the shelf `recently-added` of a library of podcasts
//! holds a podcast with no `recentEpisode`. `group_home` and `group_home_pod`
//! drop the two of them already, therefore the bands of those lines hold five
//! bands and three bands.
//!
//! **The count of the title of a band says the media that the program holds**,
//! and never the field `total` of the shelf: the shelf `recently-added` of the
//! library `Books` says `total: 22` and it gives ten entities, and a band that
//! said `6 of 22` would promise 12 media that no key of the user can reach
//! (T-118).

use toutui::api::libraries::get_library_perso_view::Root;
use toutui::api::libraries::get_library_perso_view_pod::Root as RootPod;
use toutui::api::utils::collect_series::SeriesView;
use toutui::logic::home_view::{group_home, group_home_pod};
use toutui::logic::the_bands_of_the_home::{
    the_bands, the_cell_at_the_left, the_cell_at_the_right, the_count_of_a_band,
    the_first_cell_of_the_band, the_last_cell_of_the_band, the_place_of_the_line,
};

/// A series of the library, of `crate::api::utils::collect_series`.
fn series(id: &str) -> SeriesView {
    SeriesView {
        id: id.to_string(),
        name: format!("The series {id}"),
        description: String::new(),
        books: Vec::new(),
    }
}

/// The shape of the answer of the sandbox for the library `Books`, of
/// 2026-08-17. The numbers of the media are the numbers of the server.
fn the_shelves_of_the_library_of_books() -> Vec<Root> {
    let media_of = |how_many: usize, name: &str| {
        (0..how_many)
            .map(|one| serde_json::json!({ "id": format!("{name}-{one}"), "media": {} }))
            .collect::<Vec<_>>()
    };

    serde_json::from_value(serde_json::json!([
        { "id": "continue-listening", "label": "Continue Listening",
          "total": 5, "entities": media_of(5, "continue") },
        { "id": "recently-added", "label": "Recently Added",
          "total": 22, "entities": media_of(10, "recent") },
        { "id": "recent-series", "label": "Recent Series", "total": 3, "entities": [
            { "id": "series-1", "name": "A Series", "books": [] },
            { "id": "series-2", "name": "A Second Series", "books": [] },
            { "id": "series-3", "name": "A Third Series", "books": [] } ] },
        { "id": "discover", "label": "Discover",
          "total": 7, "entities": media_of(7, "discover") },
        { "id": "listen-again", "label": "Listen Again",
          "total": 10, "entities": media_of(10, "again") },
        { "id": "newest-authors", "label": "Newest Authors", "total": 9, "entities": [
            { "id": "author-1", "name": "An Author" } ] }
    ]))
    .expect("the answer of the server must read")
}

/// The shape of the answer of the sandbox for the library `Podcasts`, of
/// 2026-08-17. The shelf `recently-added` of a library of podcasts holds the
/// podcast itself, and it holds no `recentEpisode`.
fn the_shelves_of_the_library_of_podcasts() -> Vec<RootPod> {
    let episodes_of = |how_many: usize, name: &str| {
        (0..how_many)
            .map(|one| {
                serde_json::json!({
                    "id": format!("{name}-{one}"), "media": {},
                    "recentEpisode": { "id": format!("{name}-episode-{one}") } })
            })
            .collect::<Vec<_>>()
    };

    serde_json::from_value(serde_json::json!([
        { "id": "continue-listening", "label": "Continue Listening",
          "total": 4, "entities": episodes_of(4, "continue") },
        { "id": "newest-episodes", "label": "Newest Episodes",
          "total": 68, "entities": episodes_of(10, "newest") },
        { "id": "recently-added", "label": "Recently Added", "total": 2, "entities": [
            { "id": "a-podcast", "media": {} },
            { "id": "a-second-podcast", "media": {} } ] },
        { "id": "listen-again", "label": "Listen Again",
          "total": 3, "entities": episodes_of(3, "again") }
    ]))
    .expect("the answer of the server must read")
}

#[test]
fn the_library_of_books_of_the_sandbox_gives_five_bands() {
    let series = vec![series("series-1"), series("series-2"), series("series-3")];
    let rows = group_home(&the_shelves_of_the_library_of_books(), &series);
    let bands = the_bands(&rows);

    let names: Vec<&str> = bands.iter().map(|band| band.the_title.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Continue Listening",
            "Recently Added",
            "Recent Series",
            "Discover",
            "Listen Again"
        ],
        "the shelf of the authors holds no media, therefore it gives no band"
    );

    let how_many: Vec<usize> = bands.iter().map(|band| band.the_cells.len()).collect();
    assert_eq!(how_many, vec![5, 10, 3, 7, 10]);

    // The 35 media of the screen of the program stand in the cells, and no cell
    // holds the line of a title.
    let cells: Vec<usize> = bands
        .iter()
        .flat_map(|band| band.the_cells.iter().copied())
        .collect();
    assert_eq!(cells.len(), 35);
    for line in &cells {
        assert!(
            rows[*line].is_a_line_of_the_user(),
            "the line {line} of a cell must hold a media and not a title"
        );
    }
}

#[test]
fn the_library_of_podcasts_of_the_sandbox_gives_three_bands() {
    let rows = group_home_pod(&the_shelves_of_the_library_of_podcasts());
    let bands = the_bands(&rows);

    let names: Vec<&str> = bands.iter().map(|band| band.the_title.as_str()).collect();
    assert_eq!(
        names,
        vec!["Continue Listening", "Newest Episodes", "Listen Again"],
        "the shelf of the podcasts holds no episode, therefore it gives no band"
    );

    let how_many: Vec<usize> = bands.iter().map(|band| band.the_cells.len()).collect();
    assert_eq!(how_many, vec![4, 10, 3]);
}

#[test]
fn the_count_of_a_title_says_the_media_of_the_program_and_not_the_total() {
    let series = vec![series("series-1"), series("series-2"), series("series-3")];
    let rows = group_home(&the_shelves_of_the_library_of_books(), &series);
    let bands = the_bands(&rows);

    // The shelf `recently-added` says `total: 22`, and it gives ten media.
    let recently_added = &bands[1];
    assert_eq!(
        the_count_of_a_band(6, recently_added.the_cells.len()),
        "6 of 10"
    );
    assert_eq!(
        the_count_of_a_band(10, recently_added.the_cells.len()),
        "10 of 10"
    );
}

#[test]
fn the_moves_of_the_bands_of_the_sandbox_hold_their_two_ends() {
    let series = vec![series("series-1"), series("series-2"), series("series-3")];
    let rows = group_home(&the_shelves_of_the_library_of_books(), &series);
    let bands = the_bands(&rows);

    // The first cell of the first band: the line 1 of the flat list, under the
    // title of Continue Listening at the line 0.
    let first = bands[0].the_cells[0];
    assert_eq!(first, 1);
    assert_eq!(the_place_of_the_line(&bands, first), Some((0, 0)));

    // The key `h` of the first cell of a band stays on that cell.
    assert_eq!(the_cell_at_the_left(&bands, first), Some(first));

    // The key `G` gives the last cell of that band, and the key `l` of it stays.
    let last = the_last_cell_of_the_band(&bands, first).expect("the band holds a cell");
    assert_eq!(last, bands[0].the_cells[4]);
    assert_eq!(the_cell_at_the_right(&bands, last), Some(last));
    assert_eq!(the_first_cell_of_the_band(&bands, last), Some(first));

    // The line of a title stands in no band, and every move of it gives the
    // first media of the view.
    assert_eq!(the_place_of_the_line(&bands, 0), None);
    assert_eq!(the_cell_at_the_right(&bands, 0), Some(first));
}
