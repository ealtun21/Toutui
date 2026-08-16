use crate::api::libraries::get_all_books::Root;
use crate::utils::html_text::to_plain_text;
use crate::utils::values_of_the_server::a_text_or_nothing;

/// collect titles
pub async fn collect_titles_library(library: &Root) -> Vec<String> {
    let mut titles_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(metadata) = &media.metadata {
                    // **A text of no letter is not a value.** See T-114.
                    titles_library.push(a_text_or_nothing(metadata.title.as_deref()));
                }
            }
        }
    }

    titles_library
}

/// collect ID of library items
pub async fn collect_ids_library(library: &Root) -> Vec<String> {
    let mut ids_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(id) = &item.id {
                ids_library.push(id.clone());
            } else {
                ids_library.push("N/A".to_string());
            }
        }
    }

    ids_library
}

/// The facts of the panel 5 of the cover, one for each item. See T-325.
///
/// **The answer of the items holds these six facts already**, and the program
/// read none of them: `narratorName`, `seriesName`, and `genres` of the
/// metadata, and `numAudioFiles`, `size`, and `ebookFormat` of the media. The
/// panel 5 of the design says each of them on a line of its own.
///
/// The list walks the items in the sequence of the server, as every other list
/// of a row does, therefore the number of an item of `collect_titles_library`
/// is the number of an item of this list.
pub async fn collect_the_facts_library(
    library: &Root,
) -> Vec<crate::logic::the_facts_of_a_media::TheFactsOfAMedia> {
    let mut facts = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(metadata) = &media.metadata {
                    facts.push(crate::logic::the_facts_of_a_media::TheFactsOfAMedia {
                        // **A text of no letter is not a value** (T-114), and a
                        // fact of no value takes no line of the panel.
                        series: metadata.series_name.clone().unwrap_or_default(),
                        narrator: metadata.narrator_name.clone().unwrap_or_default(),
                        genre: metadata
                            .genres
                            .as_ref()
                            .map(|genres| genres.join(", "))
                            .unwrap_or_default(),
                        files: media.num_audio_files.unwrap_or_default(),
                        // **The size of the media is the size of the item**:
                        // `media.size` holds the audio alone, and `item.size`
                        // holds the ebook and the cover beside it.
                        size: item.size.or(media.size).unwrap_or_default(),
                        the_ebook: media.ebook_format.clone().unwrap_or_default(),
                    });
                }
            }
        }
    }

    facts
}

/// collect author name for book
pub async fn collect_auth_names_library(library: &Root) -> Vec<String> {
    let mut auth_names_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(metadata) = &media.metadata {
                    auth_names_library.push(a_text_or_nothing(metadata.author_name.as_deref()));
                }
            }
        }
    }

    auth_names_library
}

/// collect author name for podcast
pub async fn collect_auth_names_library_pod(library: &Root) -> Vec<String> {
    let mut auth_names_library_pod = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(metadata) = &media.metadata {
                    auth_names_library_pod.push(a_text_or_nothing(metadata.author.as_deref()));
                }
            }
        }
    }

    auth_names_library_pod
}
/// collect published year
pub async fn collect_published_year_library(library: &Root) -> Vec<String> {
    let mut published_year_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(metadata) = &media.metadata {
                    published_year_library
                        .push(a_text_or_nothing(metadata.published_year.as_deref()));
                }
            }
        }
    }

    published_year_library
}

/// collect description
pub async fn collect_desc_library(library: &Root) -> Vec<String> {
    let mut desc_library = Vec::new();

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(metadata) = &media.metadata {
                    // The description of the server can hold a web page. A page
                    // that holds no text is not a description. See T-114.
                    let text = metadata.description.as_deref().map(to_plain_text);

                    desc_library.push(
                        crate::utils::values_of_the_server::a_description_or_nothing(
                            text.as_deref(),
                        ),
                    );
                }
            }
        }
    }

    desc_library
}

/// collect duration
pub async fn collect_duration_library(library: &Root) -> Vec<f64> {
    let mut duration = vec![];

    if let Some(results) = &library.results {
        for item in results {
            if let Some(media) = &item.media {
                if let Some(dur) = &media.duration {
                    duration.push(*dur);
                } else {
                    duration.push(0.0);
                }
            }
        }
    }

    duration
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the server for a book that holds no tag at all.
    ///
    /// **The measurement of 2026-08-12**, of the library of 2056 items of the
    /// sandbox: `authorName`, `narratorName`, and `seriesName` come as `""`, and
    /// `publishedYear` comes as `null`. See T-114.
    fn a_book_with_no_tag() -> Root {
        serde_json::from_value(serde_json::json!({
            "results": [
                {
                    "id": "item-1",
                    "mediaType": "book",
                    "media": {
                        "metadata": {
                            "title": "Large Book 2056",
                            "authorName": "",
                            "narratorName": "",
                            "seriesName": "",
                            "publishedYear": null,
                            "description": ""
                        },
                        "duration": 1.0
                    }
                }
            ],
            "total": 1
        }))
        .expect("the answer of the server must read")
    }

    /// A text of no letter is not a value. See T-114.
    #[tokio::test]
    async fn an_empty_text_of_the_server_gives_the_words_of_a_value_that_is_absent() {
        let answer = a_book_with_no_tag();

        // The old code wrote `""` on the screen, and the line of the Library
        // view then said "Author:  - Year: N/A".
        assert_eq!(collect_auth_names_library(&answer).await, vec!["N/A"]);
        assert_eq!(collect_auth_names_library_pod(&answer).await, vec!["N/A"]);
        assert_eq!(collect_published_year_library(&answer).await, vec!["N/A"]);
        assert_eq!(
            collect_desc_library(&answer).await,
            vec!["No description available"]
        );

        // The title holds letters, therefore it stays as it stands.
        assert_eq!(
            collect_titles_library(&answer).await,
            vec!["Large Book 2056"]
        );
    }

    /// A description that holds a web page with no text is no description.
    #[tokio::test]
    async fn a_page_with_no_text_is_no_description() {
        let answer: Root = serde_json::from_value(serde_json::json!({
            "results": [ { "id": "item-1", "media": { "metadata": {
                "title": "A book", "description": "<p> </p>" } } } ]
        }))
        .expect("an answer");

        assert_eq!(
            collect_desc_library(&answer).await,
            vec!["No description available"]
        );
    }

    /// Every list of the screen holds one value for each item. A list that is
    /// short gives the author of a different book.
    #[tokio::test]
    async fn every_list_holds_one_value_for_each_item() {
        let answer = a_book_with_no_tag();

        assert_eq!(collect_ids_library(&answer).await.len(), 1);
        assert_eq!(collect_titles_library(&answer).await.len(), 1);
        assert_eq!(collect_auth_names_library(&answer).await.len(), 1);
        assert_eq!(collect_published_year_library(&answer).await.len(), 1);
        assert_eq!(collect_desc_library(&answer).await.len(), 1);
        assert_eq!(collect_duration_library(&answer).await.len(), 1);
    }
}
