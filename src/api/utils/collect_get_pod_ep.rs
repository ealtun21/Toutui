use crate::api::library_items::get_pod_ep::Root;
use crate::utils::convert_seconds::*;
use crate::utils::html_text::to_plain_text;
use crate::utils::values_of_the_server::a_text_or_nothing;

/// collect title podact episode
pub async fn collect_titles_pod_ep(item: &Root) -> Vec<String> {
    let mut titles_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                titles_pod_ep.push(a_text_or_nothing(episode.title.as_deref()));
            }
        }
    }

    titles_pod_ep
}

/// collect ID of podcast episode
pub async fn collect_ids_pod_ep(item: &Root) -> Vec<String> {
    let mut ids_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                if let Some(id) = &episode.id {
                    ids_pod_ep.push(id.clone());
                } else {
                    ids_pod_ep.push("N/A".to_string());
                }
            }
        }
    }

    ids_pod_ep
}

/// collect subtiles
pub async fn collect_subtitles_pod_ep(item: &Root) -> Vec<String> {
    let mut subtitles_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                let text = episode.subtitle.as_deref().map(to_plain_text);

                subtitles_pod_ep.push(a_text_or_nothing(text.as_deref()));
            }
        }
    }

    subtitles_pod_ep
}

/// collect seasons
pub async fn collect_seasons_pod_ep(item: &Root) -> Vec<String> {
    let mut seasons_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                seasons_pod_ep.push(a_text_or_nothing(episode.season.as_deref()));
            }
        }
    }

    seasons_pod_ep
}

/// collect episodes
pub async fn collect_episodes_pod_ep(item: &Root) -> Vec<String> {
    let mut episodes_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                episodes_pod_ep.push(a_text_or_nothing(episode.episode.as_deref()));
            }
        }
    }

    episodes_pod_ep
}

/// collect authors
pub async fn collect_authors_pod_ep(item: &Root) -> Vec<String> {
    let mut authors_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(metadata) = &media.metadata {
            authors_pod_ep.push(a_text_or_nothing(metadata.author.as_deref()));
        }
    }

    authors_pod_ep
}

/// collect desc
pub async fn collect_descs_pod_ep(item: &Root) -> Vec<String> {
    let mut descs_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(metadata) = &media.metadata {
            let text = metadata.description.as_deref().map(to_plain_text);

            descs_pod_ep.push(a_text_or_nothing(text.as_deref()));
        }
    }

    descs_pod_ep
}

/// collect title of podcast (no of podcast episode)
pub async fn collect_titles_pod(item: &Root) -> Vec<String> {
    let mut titles_pod = Vec::new();

    if let Some(media) = &item.media {
        if let Some(metadata) = &media.metadata {
            titles_pod.push(a_text_or_nothing(metadata.title.as_deref()));
        }
    }

    titles_pod
}

// collect duration
pub async fn collect_durations_pod_ep(item: &Root) -> Vec<String> {
    let mut durations = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                if let Some(audio_file) = &episode.audio_file {
                    if let Some(duration) = audio_file.duration {
                        durations.push(duration);
                    } else {
                        durations.push(0.0);
                    }
                }
            }
        }
    }

    convert_seconds(durations)
}

/// Gives the length of each episode of a podcast, in seconds. See T-236.
///
/// `collect_durations_pod_ep` gives the same lengths as a text, and a text
/// gives no number: the key `n` of the view of the episodes therefore put an
/// episode in the queue with no length at all, and the line of that media of
/// the view of the queue said no time (T-234).
///
/// **A length of 0 is a length that the server did not give** (T-180), and an
/// episode of no audio file holds no length. Each of them gives `None`, and the
/// media of that line says no time.
///
/// **The list holds one value for each episode**: `collect_durations_pod_ep`
/// pushes a value for an episode of an audio file alone, therefore an episode
/// with no audio file takes the length of the episode after it. The lists of
/// this view stand one against the other by the number of the line (T-24).
pub async fn the_lengths_of_the_episodes(item: &Root) -> Vec<Option<f64>> {
    let Some(episodes) = item
        .media
        .as_ref()
        .and_then(|media| media.episodes.as_ref())
    else {
        return Vec::new();
    };

    episodes
        .iter()
        .map(|episode| {
            episode
                .audio_file
                .as_ref()
                .and_then(|file| file.duration)
                .filter(|length| *length > 0.0)
        })
        .collect()
}
