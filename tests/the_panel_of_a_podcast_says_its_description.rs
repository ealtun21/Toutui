//! **The panel of a podcast of the Home view says the description of that
//! podcast.** See T-250.
//!
//! The measurement of 2026-08-15, of the library `Podcasts` of the sandbox. The
//! server gives `""` for the subtitle of every one of the 57 episodes of
//! `Letters of Two Brides`, and it gives the description of the podcast beside
//! each of them. One frame of the real program inside tmux:
//!
//! ```text
//! ────────Home [18 items]────────
//! ➤ 3%  Letter 1
//! [Letters of Two Brides] - Author: LibriVox - Episode: 1 - Duration: 29m
//! Progress: 3%, 28m left, Not finished
//!
//! No description available
//! ```
//!
//! The program held those words in `descs_pod_cnt_list` already:
//! `collect_descs_pod_cnt_list` takes the description of the podcast of each
//! line, and `render_desc_home` of `src/ui/tui.rs` read
//! `subtitles_pod_cnt_list` alone. **A box that the render never reads is a road
//! of its own.**

use toutui::api::libraries::get_library_perso_view_pod::Root as ShelfRoot;
use toutui::api::utils::collect_personalized_view_pod::{
    collect_descs_pod_cnt_list, collect_subtitles_pod_cnt_list,
};
use toutui::logic::the_panel_of_a_line::the_description_of_a_podcast;
use toutui::utils::values_of_the_server::NO_DESCRIPTION;

/// The shelves of the sandbox, of three conditions of one panel.
///
/// **The parts of this test stay in one function.** See T-144 and T-157.
#[tokio::test]
async fn the_panel_of_a_podcast_says_the_description_of_that_podcast() {
    let shelves: Vec<ShelfRoot> = serde_json::from_value(serde_json::json!([
        {
            "id": "continue-listening",
            "entities": [
                // The condition of the sandbox: no subtitle, and a description.
                {
                    "id": "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806",
                    "mediaType": "podcast",
                    "media": { "metadata": {
                        "title": "Letters of Two Brides",
                        "description": "Letters of Two Brides is an epistolary novel."
                    } },
                    "recentEpisode": { "id": "episode-1",
                                       "libraryItemId": "9fa45bd1-66bc-4c17-ba49-a5a6a5ec8806",
                                       "title": "Letter 1", "subtitle": "" }
                },
                // A subtitle of the episode stays the text of the panel.
                {
                    "id": "31d0f7c9-0a2b-4a1e-9c8d-0e5b6a7c8d9e",
                    "mediaType": "podcast",
                    "media": { "metadata": {
                        "title": "Arthur Gordon Pym",
                        "description": "The Narrative of Arthur Gordon Pym of Nantucket."
                    } },
                    "recentEpisode": {
                        "id": "episode-2",
                        "libraryItemId": "31d0f7c9-0a2b-4a1e-9c8d-0e5b6a7c8d9e",
                        "title": "Chapter 01",
                        "subtitle": "The first chapter."
                    }
                },
                // The server holds neither of the two.
                {
                    "id": "5c4b3a29-1d0e-4f6a-8b7c-2e1d0c9b8a76",
                    "mediaType": "podcast",
                    "media": { "metadata": { "title": "A Podcast Of No Words" } },
                    "recentEpisode": { "id": "episode-3",
                                       "libraryItemId": "5c4b3a29-1d0e-4f6a-8b7c-2e1d0c9b8a76",
                                       "title": "The Only Episode" }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read");

    // **The two boxes hold the value of the server alone** (T-250 and T-249): a
    // box that a fallback reads must hold no word of the program, because those
    // words are a text of a letter and the fallback then stops at them.
    let subtitles = collect_subtitles_pod_cnt_list(&shelves).await;
    let descriptions = collect_descs_pod_cnt_list(&shelves).await;

    assert_eq!(subtitles, vec!["", "The first chapter.", ""]);
    assert_eq!(
        descriptions,
        vec![
            "Letters of Two Brides is an epistolary novel.",
            "The Narrative of Arthur Gordon Pym of Nantucket.",
            ""
        ]
    );

    // The panel of the screen, of each of the three lines.
    let the_panel: Vec<String> = subtitles
        .iter()
        .zip(descriptions.iter())
        .map(|(subtitle, description)| the_description_of_a_podcast(subtitle, description))
        .collect();

    assert_eq!(
        the_panel,
        vec![
            // The fault of the measurement: this line said the words of a panel
            // that holds no text, and the server gave the description.
            "Letters of Two Brides is an epistolary novel.".to_string(),
            // The subtitle of the episode is nearer to the line than the
            // description of the whole podcast, therefore it comes first.
            "The first chapter.".to_string(),
            // The rule of T-249 stands for the panel that holds neither.
            NO_DESCRIPTION.to_string(),
        ]
    );

    // A description of no character is no description (T-249), and a subtitle of
    // spaces alone is no subtitle.
    assert_eq!(
        the_description_of_a_podcast("   ", "A podcast."),
        "A podcast."
    );
    assert_eq!(the_description_of_a_podcast("   ", "  "), NO_DESCRIPTION);
}
