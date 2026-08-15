use crate::api::libraries::get_library_perso_view_pod::{Entity, Media, RecentEpisode, Root};
use crate::utils::convert_seconds::*;
use crate::utils::html_text::to_plain_text;

/// Gives every entity of the shelves that holds an episode.
///
/// A shelf of the personalized view of a library of podcasts does not always
/// hold an episode. A measurement against an Audiobookshelf 2.36.0 on
/// 2026-08-11 gives three shelves: `newest-episodes` and `listen-again` hold
/// an episode, and `recently-added` holds a podcast with no episode.
///
/// The Home view plays an episode, and it cannot play a podcast. Therefore the
/// view shows the entities that hold an episode, and this function gives them.
///
/// Every list of the Home view must hold one value for each line, and the
/// screen reads those lists by one number. Therefore every function of this
/// file walks these entities, in this sequence, and it pushes one value for
/// each of them. See T-24.
pub fn episode_entities(shelves: &[Root]) -> impl Iterator<Item = (&Entity, &RecentEpisode)> {
    shelves
        .iter()
        .flat_map(|shelf| shelf.entities.iter().flatten())
        .filter_map(|entity| {
            let episode = entity.recent_episode.as_ref()?;
            // The title of the podcast and its author come from the media of
            // the entity. An entity with no media would give no value for
            // those two lists, and every list after it would then stand
            // against the others.
            entity.media.as_ref()?;
            Some((entity, episode))
        })
}

/// Reads one text, or gives `N/A`.
///
/// **A text of no letter is not a value**, and the server gives `""` for a value
/// that a podcast does not hold. See T-114.
fn or_not_available(value: Option<&String>) -> String {
    crate::utils::values_of_the_server::a_text_or_nothing(value.map(String::as_str))
}

/// Gives the media of an entity that `episode_entities` gave.
fn media_of(entity: &Entity) -> &Media {
    entity
        .media
        .as_ref()
        .expect("episode_entities gives an entity that holds a media")
}

/// collect id pod for continue listening
pub async fn collect_ids_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(_, episode)| or_not_available(episode.library_item_id.as_ref()))
        .collect()
}

/// Collect subtitles from recent episodes
///
/// **This list holds the value of the server alone** (T-250): the panel of the
/// Home view of a library of podcasts says the description of the podcast when
/// the episode holds no subtitle, therefore the words of a subtitle that the
/// server does not have belong to `the_description_of_a_podcast` of the screen
/// and not to this box. See T-249 for the rule of a field that a fallback reads.
pub async fn collect_subtitles_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(_, episode)| {
            let text = episode.subtitle.as_deref().map(to_plain_text);

            crate::utils::values_of_the_server::a_text_or(text.as_deref(), "")
        })
        .collect()
}

/// Collect num episode
pub async fn collect_nums_ep_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(_, episode)| or_not_available(episode.episode.as_ref()))
        .collect()
}

/// collect season
pub async fn collect_seasons_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(_, episode)| or_not_available(episode.season.as_ref()))
        .collect()
}

/// Collect authors
pub async fn collect_authors_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(entity, _)| {
            or_not_available(
                media_of(entity)
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.author.as_ref()),
            )
        })
        .collect()
}

/// Collect description
///
/// The description of the podcast of the line. **This list holds the value of
/// the server alone** (T-250), and the panel of the screen gives the words of a
/// description that the server does not have.
pub async fn collect_descs_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(entity, _)| {
            let text = media_of(entity)
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.description.as_ref())
                .map(|description| to_plain_text(description));

            crate::utils::values_of_the_server::a_text_or(text.as_deref(), "")
        })
        .collect()
}

/// Collect podcast title
pub async fn collect_titles_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(entity, _)| {
            or_not_available(
                media_of(entity)
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.title.as_ref()),
            )
        })
        .collect()
}

pub async fn collect_durations_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    let durations: Vec<f64> = episode_entities(roots)
        .map(|(_, episode)| {
            episode
                .audio_file
                .as_ref()
                .and_then(|file| file.duration)
                .unwrap_or(0.0)
        })
        .collect();

    convert_seconds(durations)
}

/// Gives the length of each episode of the shelves, in seconds. See T-236.
///
/// `collect_durations_pod_cnt_list` gives the same lengths as a text, and a
/// text gives no number: the key `n` of this view therefore put an episode in
/// the queue with no length at all, and the line of that media of the view of
/// the queue said no time (T-234).
///
/// **A length of 0 is a length that the server did not give** (T-180): the
/// answer of an episode of no audio file holds no length, and a media of no
/// length keeps its line and it says no time.
///
/// The sequence is the sequence of `episode_entities`, therefore the number of
/// a line of the Home view reads this list.
pub async fn the_lengths_of_the_episodes_of_the_shelves(roots: &[Root]) -> Vec<Option<f64>> {
    episode_entities(roots)
        .map(|(_, episode)| {
            episode
                .audio_file
                .as_ref()
                .and_then(|file| file.duration)
                .filter(|length| *length > 0.0)
        })
        .collect()
}

/// collect ids ep
pub async fn collect_ids_ep_pod_cnt_list(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(_, episode)| or_not_available(episode.id.as_ref()))
        .collect()
}

/// collect titles pod for continue listening
pub async fn collect_titles_cnt_list_pod(roots: &[Root]) -> Vec<String> {
    episode_entities(roots)
        .map(|(_, episode)| or_not_available(episode.title.as_ref()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shelves of the library of podcasts of the sandbox, measured on
    /// 2026-08-11: a shelf of episodes, a shelf of podcasts with no episode,
    /// and a shelf of episodes again.
    fn the_shelves() -> Vec<Root> {
        serde_json::from_value(serde_json::json!([
            {
                "id": "newest-episodes",
                "label": "Newest Episodes",
                "type": "episode",
                "entities": [
                    { "id": "pod-1",
                      "media": { "metadata": { "title": "A Podcast", "author": "An Author",
                                               "description": "<p>A text</p>" } },
                      "recentEpisode": { "id": "ep-1", "libraryItemId": "pod-1",
                                         "title": "The First Episode", "season": "1",
                                         "episode": "1", "subtitle": "<i>A subtitle</i>",
                                         "audioFile": { "duration": 65.0 } } }
                ]
            },
            {
                "id": "recently-added",
                "label": "Recently Added",
                "type": "podcast",
                "entities": [
                    { "id": "pod-2", "media": { "metadata": { "title": "A Second Podcast" } } }
                ]
            },
            {
                "id": "listen-again",
                "label": "Listen Again",
                "type": "episode",
                "entities": [
                    { "id": "pod-1",
                      "media": { "metadata": { "title": "A Podcast" } },
                      "recentEpisode": { "id": "ep-2", "libraryItemId": "pod-1",
                                         "title": "The Second Episode" } }
                ]
            }
        ]))
        .expect("the answer of the server must read")
    }

    #[tokio::test]
    async fn every_list_holds_one_value_for_each_episode() {
        let shelves = the_shelves();

        let ids = collect_ids_pod_cnt_list(&shelves).await;
        let titles = collect_titles_cnt_list_pod(&shelves).await;
        let podcasts = collect_titles_pod_cnt_list(&shelves).await;
        let authors = collect_authors_pod_cnt_list(&shelves).await;
        let seasons = collect_seasons_pod_cnt_list(&shelves).await;
        let numbers = collect_nums_ep_pod_cnt_list(&shelves).await;
        let subtitles = collect_subtitles_pod_cnt_list(&shelves).await;
        let descriptions = collect_descs_pod_cnt_list(&shelves).await;
        let durations = collect_durations_pod_cnt_list(&shelves).await;
        let episodes = collect_ids_ep_pod_cnt_list(&shelves).await;

        // The shelf of the podcasts holds no episode, therefore it gives no
        // line. Every list holds two values, and not three.
        for length in [
            ids.len(),
            titles.len(),
            podcasts.len(),
            authors.len(),
            seasons.len(),
            numbers.len(),
            subtitles.len(),
            descriptions.len(),
            durations.len(),
            episodes.len(),
        ] {
            assert_eq!(length, 2, "every list must hold one value for each line");
        }

        assert_eq!(titles, vec!["The First Episode", "The Second Episode"]);
        assert_eq!(episodes, vec!["ep-1", "ep-2"]);
        assert_eq!(ids, vec!["pod-1", "pod-1"]);
        assert_eq!(authors, vec!["An Author", "N/A"]);
        assert_eq!(seasons, vec!["1", "N/A"]);
        // **The two boxes of the panel hold the value of the server alone**
        // (T-250): the subtitle of the episode falls back to the description of
        // the podcast, therefore the words of the program belong to the screen.
        // See T-249 for the words themselves.
        assert_eq!(subtitles, vec!["A subtitle", ""]);
        assert_eq!(descriptions, vec!["A text", ""]);
    }

    #[tokio::test]
    async fn an_entity_with_no_media_gives_no_line() {
        let shelves: Vec<Root> = serde_json::from_value(serde_json::json!([
            { "id": "newest-episodes", "label": "Newest Episodes",
              "entities": [ { "id": "pod-3",
                              "recentEpisode": { "id": "ep-3", "title": "An Episode" } } ] }
        ]))
        .expect("the answer must read");

        assert!(collect_titles_cnt_list_pod(&shelves).await.is_empty());
        assert!(collect_titles_pod_cnt_list(&shelves).await.is_empty());
    }
}
