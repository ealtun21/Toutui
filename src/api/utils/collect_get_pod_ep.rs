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
///
/// **This list holds the value of the server alone** (T-251, and the rule of
/// T-249 and T-250): the panel of the view of the episodes says the description
/// of the episode when that episode holds no subtitle, therefore the words of a
/// subtitle that the server does not have belong to
/// `crate::logic::the_panel_of_a_line::the_description_of_a_podcast` of the
/// screen and not to this box. A box that a fallback reads must hold no word of
/// the program, because those words are a text of a letter and the fallback then
/// stops at them.
pub async fn collect_subtitles_pod_ep(item: &Root) -> Vec<String> {
    let mut subtitles_pod_ep = Vec::new();

    if let Some(media) = &item.media {
        if let Some(episodes) = &media.episodes {
            for episode in episodes {
                let text = episode.subtitle.as_deref().map(to_plain_text);

                subtitles_pod_ep.push(crate::utils::values_of_the_server::a_text_or(
                    text.as_deref(),
                    "",
                ));
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

/// The description of each episode of a podcast, for the panel of the view of
/// the episodes. See T-251.
///
/// **The description of an episode is not the description of its podcast**, and
/// the program asked the server for neither of them at the panel: the render
/// read the subtitle of the episode alone, and this box held one value — the
/// description of the podcast — for a view of many lines.
///
/// The list holds one value for each episode now, as every other list of this
/// view does (the rule of T-24). The description of the episode comes first,
/// and the description of the podcast after it: the server gives the show notes
/// of an episode in `description`, and an episode of no show notes says what
/// the podcast is.
///
/// **The list holds the value of the server alone** (T-249 and T-250): the
/// screen gives the words of a description that the server does not have.
pub async fn collect_descs_pod_ep(item: &Root) -> Vec<String> {
    let Some(media) = &item.media else {
        return Vec::new();
    };

    let of_the_podcast = media
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_deref())
        .map(to_plain_text)
        .unwrap_or_default();

    let Some(episodes) = &media.episodes else {
        return Vec::new();
    };

    episodes
        .iter()
        .map(|episode| {
            let of_the_episode = episode.description.as_deref().map(to_plain_text);

            crate::utils::values_of_the_server::a_text_or(
                of_the_episode.as_deref(),
                &of_the_podcast,
            )
        })
        .collect()
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

/// The length of each episode of a podcast, as a text.
///
/// **The list holds one value for each episode** (T-288). This function pushed
/// a value for an episode of an audio file alone: an episode with no audio file
/// therefore took the length of the episode after it, and the last line of the
/// view held no value at all. The measurement of 2026-08-16, of a podcast of 11
/// episodes whose first episode lost its `audioFile`: the panel of the first
/// line said `Duration: 22m` for an episode of 5 minutes, and the panel of the
/// last line said `Error: Episode data unavailable or index out of bounds.`
///
/// **A length of 0 is a length that the server did not give** (T-180), and an
/// episode with no audio file holds no length. Each of them gives the words of
/// a value that the server did not give, beside the label `Duration:` (T-249).
pub async fn collect_durations_pod_ep(item: &Root) -> Vec<String> {
    the_lengths_of_the_episodes(item)
        .await
        .into_iter()
        .map(|length| match length {
            Some(length) => convert_seconds(vec![length]).remove(0),
            None => crate::utils::values_of_the_server::NOT_AVAILABLE.to_string(),
        })
        .collect()
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
/// **The list holds one value for each episode**, and the lists of this view
/// stand one against the other by the number of the line (T-24).
/// `collect_durations_pod_ep` reads this function now, and it therefore holds
/// that rule too: it pushed a value for an episode of an audio file alone, and
/// an episode with no audio file then took the length of the episode after it
/// (T-288).
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
