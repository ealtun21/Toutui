use crate::api::libraries::get_library_perso_view::{Entity, Media, Metadata, Root};
use crate::utils::html_text::to_plain_text;

/// Gives every entity of the shelves that holds a media, with its media.
///
/// A shelf of the personalized view does not always hold a media: the shelf
/// `recent-series` holds a series, and the shelf `newest-authors` holds an
/// author. Such an entity has no `media`.
///
/// The Home view keeps one value in each of its lists for each media, and the
/// screen reads those lists by one number. Therefore every function of this
/// file must walk the same entities, in the same sequence. This function is
/// that sequence, and it is the only one. See T-24.
///
/// The functions of this file each read one entity and each push one value.
/// An earlier form of this file had a different rule in each function: one
/// pushed for every entity, and the others pushed for an entity with a media
/// and a metadata only. With the shelf `continue-listening` alone, every
/// entity held a media, and the lists agreed. A second shelf of a series would
/// have moved one list against the others, and the screen would have shown the
/// title of one book beside the author of a different book.
///
/// **An entity with no identity gives the program no road to a playback.**
/// `collect_ids_cnt_list` wrote the text "N/A" for such an entity, the key
/// `Enter` then sent `POST /api/items/N/A/play`, and the program said that the
/// server does not have an item that the server has (T-389, the same fault as
/// T-388). This function now drops such an entity, with a WARN that names the
/// title, before every collector of this file reads the sequence.
pub fn media_entities(shelves: &[Root]) -> impl Iterator<Item = (&Entity, &Media)> {
    shelves
        .iter()
        .flat_map(|shelf| shelf.entities.iter().flatten())
        .filter_map(|entity| entity.media.as_ref().map(|media| (entity, media)))
        .filter(|(entity, media)| {
            let has_an_identity = entity.id.as_deref().is_some_and(|id| !id.trim().is_empty());

            if !has_an_identity {
                let title = media
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.title.as_deref())
                    .unwrap_or("");
                log::warn!(
                    "The answer of the server holds the media \"{}\" with no identity. The program cannot play it, therefore the line goes away.",
                    title
                );
            }

            has_an_identity
        })
}

/// Reads one value of the metadata of a media, or gives `N/A`.
///
/// **A text of no letter is not a value**, and the server gives `""` for a book
/// that holds no tag of an author. See T-114.
fn from_metadata(media: &Media, read: impl Fn(&Metadata) -> Option<&String>) -> String {
    crate::utils::values_of_the_server::a_text_or_nothing(
        media.metadata.as_ref().and_then(read).map(String::as_str),
    )
}

/// collect titles
pub async fn collect_titles_cnt_list(continue_listening: &[Root]) -> Vec<String> {
    media_entities(continue_listening)
        .map(|(_, media)| from_metadata(media, |metadata| metadata.title.as_ref()))
        .collect()
}

/// collect author name
pub async fn collect_auth_names_cnt_list(continue_listening: &[Root]) -> Vec<String> {
    media_entities(continue_listening)
        .map(|(_, media)| from_metadata(media, |metadata| metadata.author_name.as_ref()))
        .collect()
}

/// collect published year
pub async fn collect_pub_year_cnt_list(continue_listening: &[Root]) -> Vec<String> {
    media_entities(continue_listening)
        .map(|(_, media)| from_metadata(media, |metadata| metadata.published_year.as_ref()))
        .collect()
}

/// collect duration
pub async fn collect_duration_cnt_list(continue_listening: &[Root]) -> Vec<f64> {
    media_entities(continue_listening)
        .map(|(_, media)| media.duration.unwrap_or(0.0))
        .collect()
}

/// collect description
pub async fn collect_desc_cnt_list(continue_listening: &[Root]) -> Vec<String> {
    media_entities(continue_listening)
        .map(|(_, media)| {
            // **A description of no letter is no description** (T-114), and the
            // panel of a description says why it holds no text (T-249). This
            // line held neither rule: a text of `""`, which is what the server
            // gives for a book of a scan with no tag, reached the panel of the
            // Home view as it stood, and that panel then held nothing at all;
            // and a description that the server does not have gave "N/A", which
            // stands on a line of its own with no label to name it.
            let text = media
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.description.as_ref())
                .map(|description| to_plain_text(description));

            crate::utils::values_of_the_server::a_description_or_nothing(text.as_deref())
        })
        .collect()
}

/// The facts of the panel 5 of the cover, one for each media of the shelves.
/// See T-326.
///
/// **The answer of the personalized view holds the same six facts as the
/// answer of the items of a library**, and the Home view read none of them:
/// `narratorName`, `seriesName`, and `genres` of the metadata, and
/// `numAudioFiles`, `size`, and `ebookFormat` of the media. The panel 5 of a
/// book of the Library view therefore said eight facts and a bar, and the
/// panel of that same book of the Home view said two lines.
///
/// The list walks [`media_entities`], which is the one sequence of the lists
/// of the Home view, therefore the number of a media of
/// [`collect_titles_cnt_list`] is the number of a media of this list.
pub async fn collect_the_facts_cnt_list(
    continue_listening: &[Root],
) -> Vec<crate::logic::the_facts_of_a_media::TheFactsOfAMedia> {
    media_entities(continue_listening)
        .map(|(_, media)| {
            let metadata = media.metadata.as_ref();

            crate::logic::the_facts_of_a_media::TheFactsOfAMedia {
                // **A text of no letter is not a value** (T-114), and a fact
                // of no value takes no line of the panel.
                series: metadata
                    .and_then(|metadata| metadata.series_name.clone())
                    .unwrap_or_default(),
                narrator: metadata
                    .and_then(|metadata| metadata.narrator_name.clone())
                    .unwrap_or_default(),
                genre: metadata
                    .and_then(|metadata| metadata.genres.as_ref())
                    .map(|genres| genres.join(", "))
                    .unwrap_or_default(),
                files: media.num_audio_files.unwrap_or_default(),
                // **The entity of a shelf holds no size of its own**: the
                // answer of the items gives `item.size` beside `media.size`,
                // and T-325 takes the first of the two because it holds the
                // ebook and the cover. A measurement of the sandbox of
                // 2026-08-16 gives the same number for the two of them for
                // each of the six media of the shelf Continue Listening,
                // therefore this list reads the one size that the entity has.
                size: media.size.unwrap_or_default(),
                the_ebook: media.ebook_format.clone().unwrap_or_default(),
            }
        })
        .collect()
}

/// collect ID of the library item
pub async fn collect_ids_cnt_list(continue_listening: &[Root]) -> Vec<String> {
    media_entities(continue_listening)
        .map(|(entity, _)| entity.id.clone().unwrap_or_else(|| "N/A".to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shelf of media, a shelf of series, and a shelf of media again. The
    /// shelf of the series holds no media, therefore it gives no value.
    fn the_shelves() -> Vec<Root> {
        serde_json::from_value(serde_json::json!([
            {
                "id": "continue-listening",
                "label": "Continue Listening",
                "entities": [
                    { "id": "book-1",
                      "media": { "duration": 60.0,
                                 "metadata": { "title": "The First Book",
                                               "authorName": "One Author",
                                               "publishedYear": "1999",
                                               "description": "<p>A text</p>" } } }
                ]
            },
            {
                "id": "recent-series",
                "label": "Recent Series",
                "entities": [
                    { "id": "series-1", "name": "A Series",
                      "books": [ { "id": "book-2" } ] }
                ]
            },
            {
                "id": "recently-added",
                "label": "Recently Added",
                "entities": [
                    { "id": "book-3", "media": { "metadata": { "title": "The Third Book" } } }
                ]
            }
        ]))
        .expect("the answer of the server must read")
    }

    #[tokio::test]
    async fn every_list_holds_one_value_for_each_media() {
        let shelves = the_shelves();

        let titles = collect_titles_cnt_list(&shelves).await;
        let authors = collect_auth_names_cnt_list(&shelves).await;
        let years = collect_pub_year_cnt_list(&shelves).await;
        let durations = collect_duration_cnt_list(&shelves).await;
        let descriptions = collect_desc_cnt_list(&shelves).await;
        let ids = collect_ids_cnt_list(&shelves).await;

        // The shelf of the series holds one entity, and that entity gives no
        // line. Therefore every list holds two values, and not three.
        for length in [
            titles.len(),
            authors.len(),
            years.len(),
            durations.len(),
            descriptions.len(),
            ids.len(),
        ] {
            assert_eq!(length, 2, "every list must hold one value for each media");
        }

        assert_eq!(titles, vec!["The First Book", "The Third Book"]);
        assert_eq!(ids, vec!["book-1", "book-3"]);
        assert_eq!(authors, vec!["One Author", "N/A"]);
        assert_eq!(years, vec!["1999", "N/A"]);
        assert_eq!(durations, vec![60.0, 0.0]);
        assert_eq!(descriptions[0], "A text");
        // **The panel of a description says why it holds no text** (T-249): a
        // book with no description said "N/A" here, and the same book of the
        // view of the search said "No description available".
        assert_eq!(
            descriptions[1],
            crate::utils::values_of_the_server::NO_DESCRIPTION
        );
    }

    /// A media with no identity, or with an identity of no character, gives
    /// no line: the program has no road to a playback of it. See T-389.
    #[tokio::test]
    async fn a_media_with_no_identity_takes_no_line() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "id": "continue-listening", "label": "Continue Listening",
              "entities": [
                  { "media": { "metadata": { "title": "No Id At All" } } },
                  { "id": "   ", "media": { "metadata": { "title": "An Id Of No Character" } } },
                  { "id": "book-9", "media": { "metadata": { "title": "A Real Book" } } }
              ] }
        ]))
        .expect("the answer must read");

        let titles = collect_titles_cnt_list(&shelves).await;
        let ids = collect_ids_cnt_list(&shelves).await;

        assert_eq!(titles, vec!["A Real Book"]);
        assert_eq!(ids, vec!["book-9"]);
    }

    #[tokio::test]
    async fn a_shelf_with_no_entity_gives_no_value() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "id": "discover", "label": "Discover" }
        ]))
        .expect("the answer must read");

        assert!(collect_titles_cnt_list(&shelves).await.is_empty());
        assert!(collect_ids_cnt_list(&shelves).await.is_empty());
    }

    #[tokio::test]
    async fn a_media_with_no_metadata_still_gives_one_line() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "id": "discover", "label": "Discover",
              "entities": [ { "id": "book-9", "media": {} } ] }
        ]))
        .expect("the answer must read");

        assert_eq!(collect_titles_cnt_list(&shelves).await, vec!["N/A"]);
        assert_eq!(collect_ids_cnt_list(&shelves).await, vec!["book-9"]);
        assert_eq!(collect_duration_cnt_list(&shelves).await, vec![0.0]);
    }
}
