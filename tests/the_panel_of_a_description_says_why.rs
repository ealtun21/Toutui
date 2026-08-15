//! The panel of a description says why it holds no text. See T-249.
//!
//! **"N/A" is a value of a field, and a description is a panel of its own.** The
//! words of `NOT_AVAILABLE` stand beside a label — the line of the Library view
//! says `Author: LibriVox - Year: N/A`, and that label tells the user which
//! value the server does not have. The panel of the description holds no label
//! at all, therefore `N/A` alone stands on a line of the screen and it says
//! nothing to the user (the rule of T-91, of T-114, and of T-174).
//!
//! **The measurement of 2026-08-15**, of `A Long Test Book` of the sandbox,
//! which holds no description at all. One book, and two views of one run of the
//! real program inside tmux:
//!
//! ```text
//! ────────Search result [1 item]────────        ────────Home [35 items]────────
//! ➤ 50% A Long Test Book                        ➤ 50% A Long Test Book
//! Author: Long Author - Year: N/A - …           Author: Long Author - Year: N/A - …
//! Progress: 50%, 15m left, Not finished         Progress: 50%, 15m left, Not finished
//! No description available                      N/A
//! ```
//!
//! The view of the episodes of a podcast of the same run said `N/A` too, for
//! every one of the 57 episodes of `Letters of Two Brides`: a measurement of
//! `GET /api/items/:id` of that podcast says that the server gives `""` for the
//! subtitle and for the description of each of them.
//!
//! **The Home view of a library of books held a second fault**: that one line
//! read the description of the server and it took no rule of T-114 at all,
//! therefore a description of `""` — which is what the server gives for a book
//! of a scan with no tag — reached the panel as it stood, and the panel then
//! held **nothing at all**.
//!
//! **The view of the books of a series held a third fault**, and that panel held
//! **no line at all**. The measurement of 2026-08-15, of `The Test Chronicles`
//! of the sandbox, after a `PATCH /api/items/:id/media` gave the second book the
//! description `<p> </p>` — a web page with no text, which the server keeps as
//! it stands:
//!
//! ```text
//! ─────────The Test Chronicles [3 items]─────────    ─────────The Test Chronicles [3 items]─────────
//! ➤ ✓   #1 - The Test Chronicles Volume 1              ✓   #1 - The Test Chronicles Volume 1
//!   41% #2 - The Test Chronicles Volume 2            ➤ 41% #2 - The Test Chronicles Volume 2
//!   ✓   #3 - The Test Chronicles Volume 3              ✓   #3 - The Test Chronicles Volume 3
//! Author: Series Author - Duration: 0m               Author: Series Author - Duration: 0m
//! Progress: 100%, 0m left, Finished                  Progress: 41%, 0m left, Not finished
//! No description available                           ⟨the panel holds no line⟩
//! ```
//!
//! The tests are pure, therefore they need no server and no screen.

use toutui::utils::values_of_the_server::{NOT_AVAILABLE, NO_DESCRIPTION};

/// The words of a description are not the words of a value of a field.
#[test]
fn the_words_of_a_description_are_not_the_words_of_a_field() {
    assert_ne!(NO_DESCRIPTION, NOT_AVAILABLE);

    // A panel with no label needs a sentence, and not two letters and a stroke.
    assert!(NO_DESCRIPTION.split_whitespace().count() >= 3);
    assert!(NO_DESCRIPTION.to_lowercase().contains("description"));
}

/// A description that the server does not have, and a description of no letter,
/// each give the words of a description that the program does not have.
#[test]
fn a_description_of_no_letter_gives_the_words_of_a_description_that_is_absent() {
    use toutui::utils::values_of_the_server::a_description_or_nothing;

    assert_eq!(a_description_or_nothing(None), NO_DESCRIPTION);
    assert_eq!(a_description_or_nothing(Some("")), NO_DESCRIPTION);
    assert_eq!(a_description_or_nothing(Some("   ")), NO_DESCRIPTION);
    assert_eq!(a_description_or_nothing(Some("\t\n")), NO_DESCRIPTION);

    // A description that holds a letter stays as it stands.
    assert_eq!(
        a_description_or_nothing(Some("Letters of Two Brides is a novel.")),
        "Letters of Two Brides is a novel."
    );
}

/// The panel of the Home view of a library of books.
///
/// **The parts of this test stay in one function**: two test functions of one
/// module fight for the slot of that module. See T-144 and T-157.
#[tokio::test]
async fn the_panel_of_the_home_view_says_why_it_holds_no_description() {
    use toutui::api::libraries::get_library_perso_view::Root;
    use toutui::api::utils::collect_personalized_view::collect_desc_cnt_list;

    // The shelf `continue-listening` of the sandbox on 2026-08-15: the server
    // gives `null` for the description of `A Long Test Book`.
    let no_description: Vec<Root> = serde_json::from_value(serde_json::json!([
        {
            "id": "continue-listening",
            "label": "Continue Listening",
            "entities": [
                {
                    "id": "item-1",
                    "mediaType": "book",
                    "media": {
                        "metadata": { "title": "A Long Test Book", "description": null },
                        "duration": 1800.0
                    }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read");

    assert_eq!(
        collect_desc_cnt_list(&no_description).await,
        vec![NO_DESCRIPTION],
        "the panel of the Home view said \"N/A\", and the same book of the view \
         of the search said \"No description available\""
    );

    // **A description of `""` is no description** (T-114). The server gives it
    // for a book of a scan that holds no tag, and this line of the Home view
    // wrote it on the screen as it stood: the panel then held nothing at all.
    let a_description_of_no_letter: Vec<Root> = serde_json::from_value(serde_json::json!([
        {
            "id": "recently-added",
            "entities": [
                {
                    "id": "item-2",
                    "media": {
                        "metadata": { "title": "Large Book 2056", "description": "" }
                    }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read");

    assert_eq!(
        collect_desc_cnt_list(&a_description_of_no_letter).await,
        vec![NO_DESCRIPTION],
        "a description of no letter reached the panel of the Home view as it stood"
    );

    // A page that holds no text is no description either.
    let a_page_with_no_text: Vec<Root> = serde_json::from_value(serde_json::json!([
        {
            "id": "discover",
            "entities": [
                {
                    "id": "item-3",
                    "media": { "metadata": { "title": "A book", "description": "<p> </p>" } }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read");

    assert_eq!(
        collect_desc_cnt_list(&a_page_with_no_text).await,
        vec![NO_DESCRIPTION]
    );

    // The description of the server stays as it stands.
    let a_description: Vec<Root> = serde_json::from_value(serde_json::json!([
        {
            "id": "discover",
            "entities": [
                {
                    "id": "item-4",
                    "media": { "metadata": { "title": "A book", "description": "A novel." } }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read");

    assert_eq!(
        collect_desc_cnt_list(&a_description).await,
        vec!["A novel."]
    );
}

/// The panel of the view of the episodes of a podcast, and the panel of the
/// Home view of a library of podcasts.
///
/// **The parts of this test stay in one function.** See T-144 and T-157.
#[tokio::test]
async fn the_panel_of_a_podcast_says_why_it_holds_no_description() {
    use toutui::api::libraries::get_library_perso_view_pod::Root as ShelfRoot;
    use toutui::api::library_items::get_pod_ep::Root as ItemRoot;
    use toutui::api::utils::collect_get_pod_ep::{collect_descs_pod_ep, collect_subtitles_pod_ep};
    use toutui::api::utils::collect_personalized_view_pod::collect_subtitles_pod_cnt_list;

    // `Letters of Two Brides` of the sandbox on 2026-08-15: every one of the 57
    // episodes holds `""` for the subtitle and for the description.
    let podcast: ItemRoot = serde_json::from_value(serde_json::json!({
        "id": "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806",
        "media": {
            "metadata": { "title": "Letters of Two Brides", "description": "" },
            "episodes": [
                { "id": "episode-1", "title": "Letter 1", "subtitle": "" },
                { "id": "episode-2", "title": "Letter 2", "subtitle": null },
                { "id": "episode-3", "title": "Letter 3", "subtitle": "The third letter." }
            ]
        }
    }))
    .expect("the answer of the server must read");

    // The view of the episodes said "N/A" for every episode of that podcast.
    assert_eq!(
        collect_subtitles_pod_ep(&podcast).await,
        vec![
            NO_DESCRIPTION.to_string(),
            NO_DESCRIPTION.to_string(),
            "The third letter.".to_string()
        ]
    );

    // The description of the podcast itself takes the same words.
    assert_eq!(collect_descs_pod_ep(&podcast).await, vec![NO_DESCRIPTION]);

    // The Home view of a library of podcasts holds the same panel.
    let shelves: Vec<ShelfRoot> = serde_json::from_value(serde_json::json!([
        {
            "id": "listen-again",
            "entities": [
                {
                    "id": "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806",
                    "mediaType": "podcast",
                    "media": { "metadata": { "title": "Letters of Two Brides" } },
                    "recentEpisode": { "id": "episode-1", "title": "Letter 1", "subtitle": "" }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read");

    assert_eq!(
        collect_subtitles_pod_cnt_list(&shelves).await,
        vec![NO_DESCRIPTION]
    );
}

/// The panel of the view of the books of a series, and the panel of the view of
/// the series itself.
///
/// **The parts of this test stay in one function.** See T-144 and T-157.
#[test]
fn the_panel_of_a_series_says_why_it_holds_no_description() {
    use toutui::api::libraries::get_all_series::SeriesRoot;
    use toutui::api::utils::collect_series::collect_series;

    // `The Test Chronicles` of the sandbox on 2026-08-15: the series holds a
    // description, the first book holds none, and the second book holds a web
    // page with no text.
    let root: SeriesRoot = serde_json::from_value(serde_json::json!({
        "results": [
            {
                "id": "series-1",
                "name": "The Test Chronicles",
                "description": "Three books of a test.",
                "books": [
                    { "id": "book-1", "media": { "metadata": {
                        "title": "The Test Chronicles Volume 1",
                        "seriesName": "The Test Chronicles #1" } } },
                    { "id": "book-2", "media": { "metadata": {
                        "title": "The Test Chronicles Volume 2",
                        "seriesName": "The Test Chronicles #2",
                        "description": "<p> </p>" } } }
                ]
            }
        ],
        "total": 1
    }))
    .expect("the answer of the server must read");

    let series = collect_series(&root);

    // The panel of the second book held no line at all.
    assert_eq!(
        series[0].books[1].description_for_the_screen(),
        NO_DESCRIPTION,
        "the panel of a book whose description is a page with no text held no line"
    );

    // The panel of the first book says the same words in the same view.
    assert_eq!(
        series[0].books[0].description_for_the_screen(),
        NO_DESCRIPTION
    );

    // **The panel of the series keeps its fallback** (T-43): the field of a book
    // holds the description of the server alone, therefore the words of the
    // program in it cannot hide the description of the book after it.
    let no_description_of_the_series: SeriesRoot = serde_json::from_value(serde_json::json!({
        "results": [
            {
                "id": "series-2",
                "name": "Second Series",
                "books": [
                    { "id": "book-3", "media": { "metadata": {
                        "title": "A book of no description",
                        "seriesName": "Second Series #1" } } },
                    { "id": "book-4", "media": { "metadata": {
                        "title": "A book of a description",
                        "seriesName": "Second Series #2",
                        "description": "The second book tells this." } } }
                ]
            }
        ],
        "total": 1
    }))
    .expect("the answer of the server must read");

    let second = collect_series(&no_description_of_the_series);

    assert_eq!(
        second[0].description_for_the_screen(),
        "The second book tells this."
    );

    // A series of no description, whose books hold no description either, says
    // why the panel holds no text.
    let no_description_at_all: SeriesRoot = serde_json::from_value(serde_json::json!({
        "results": [
            {
                "id": "series-3",
                "name": "A Third Series",
                "books": [
                    { "id": "book-5", "media": { "metadata": {
                        "title": "A book", "seriesName": "A Third Series #1" } } }
                ]
            }
        ],
        "total": 1
    }))
    .expect("the answer of the server must read");

    assert_eq!(
        collect_series(&no_description_at_all)[0].description_for_the_screen(),
        NO_DESCRIPTION
    );
}
