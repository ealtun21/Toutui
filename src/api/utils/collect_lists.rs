//! The display data of the collections and of the playlists.
//!
//! A collection and a playlist are two lists of media. The user interface
//! shows them together, thus this module gives one type for both. It has no
//! network code, therefore the tests need no server.

use crate::api::libraries::get_all_books::LibraryItem;
use crate::api::libraries::get_lists::{CollectionRoot, PlaylistRoot};
use crate::utils::html_text::to_plain_text;
use crate::utils::values_of_the_server::{a_text_or, a_text_or_nothing};

/// What the list is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    /// Every user of the server sees a collection. It holds books.
    Collection,
    /// A playlist belongs to one user. It holds books or episodes.
    Playlist,
}

impl ListKind {
    /// Gives the name of the kind for the screen.
    pub fn name(&self) -> &'static str {
        match self {
            ListKind::Collection => "Collection",
            ListKind::Playlist => "Playlist",
        }
    }
}

/// One medium of a collection or of a playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct ListEntry {
    /// The identity of the library item.
    pub id: String,
    /// The identity of the episode. A book has no episode.
    pub episode_id: Option<String>,
    pub title: String,
    pub author: String,
    pub duration: f64,
    pub description: String,
}

/// One collection or one playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct ListView {
    pub id: String,
    pub kind: ListKind,
    pub name: String,
    pub description: String,
    pub entries: Vec<ListEntry>,
}

impl ListView {
    /// Gives the line of this list in the list of the lists.
    pub fn line(&self) -> String {
        // `ui::keys::items` holds the rule of "1 item" for the whole program.
        // This function held a second copy of it. See T-85 and T-95.
        format!(
            "[{}] {} [{}]",
            self.kind.name(),
            self.name,
            crate::ui::keys::items(self.entries.len())
        )
    }
}

impl ListEntry {
    /// Gives the line of this medium in the list.
    pub fn line(&self) -> String {
        self.title.clone()
    }

    /// Tells if this medium is an episode of a podcast.
    pub fn is_episode(&self) -> bool {
        self.episode_id.is_some()
    }
}

/// Reads the title, the author, and the length of a library item.
fn from_item(item: &LibraryItem) -> ListEntry {
    let metadata = item
        .media
        .as_ref()
        .and_then(|media| media.metadata.as_ref());

    ListEntry {
        id: item.id.clone().unwrap_or_default(),
        episode_id: None,
        // **A text of no letter is not a value.** See T-114.
        title: a_text_or_nothing(metadata.and_then(|data| data.title.as_deref())),
        // A book gives `authorName`, and a podcast gives `author`.
        author: a_text_or_nothing(
            metadata.and_then(|data| data.author_name.as_deref().or(data.author.as_deref())),
        ),
        duration: item
            .media
            .as_ref()
            .and_then(|media| media.duration)
            .unwrap_or(0.0),
        description: a_description_or_nothing(
            metadata.and_then(|data| data.description.as_deref()),
        ),
    }
}

/// Gives the description of a media, or the words for a media that has none.
///
/// The text of the server can hold a web page, and a page that holds no text is
/// no description. See T-114.
fn a_description_or_nothing(text: Option<&str>) -> String {
    let plain = text.map(to_plain_text);

    a_text_or(plain.as_deref(), "No description available")
}

/// Makes the display data of the collections.
pub fn collect_collections(root: &CollectionRoot) -> Vec<ListView> {
    root.results
        .iter()
        .flatten()
        .map(|collection| ListView {
            id: collection.id.clone().unwrap_or_default(),
            kind: ListKind::Collection,
            name: a_text_or_nothing(collection.name.as_deref()),
            description: a_description_or_nothing(collection.description.as_deref()),
            entries: collection.books.iter().flatten().map(from_item).collect(),
        })
        .collect()
}

/// Makes the display data of the playlists.
///
/// An entry that names an episode takes the title and the length from that
/// episode. The other entries take them from the library item.
pub fn collect_playlists(root: &PlaylistRoot) -> Vec<ListView> {
    root.results
        .iter()
        .flatten()
        .map(|playlist| ListView {
            id: playlist.id.clone().unwrap_or_default(),
            kind: ListKind::Playlist,
            name: a_text_or_nothing(playlist.name.as_deref()),
            description: a_description_or_nothing(playlist.description.as_deref()),
            entries: playlist
                .items
                .iter()
                .flatten()
                .map(|item| {
                    let mut entry = match &item.library_item {
                        Some(library_item) => from_item(library_item),
                        None => ListEntry {
                            id: String::new(),
                            episode_id: None,
                            title: "N/A".to_string(),
                            author: "N/A".to_string(),
                            duration: 0.0,
                            description: "No description available".to_string(),
                        },
                    };

                    if let Some(id) = item.library_item_id.clone() {
                        entry.id = id;
                    }

                    // The episode gives its own title and its own length. The
                    // podcast gives the author.
                    if let Some(episode) = &item.episode {
                        entry.episode_id = item.episode_id.clone();

                        if let Some(title) = episode["title"].as_str() {
                            entry.title = title.to_string();
                        }

                        if let Some(duration) = episode["audioFile"]["duration"].as_f64() {
                            entry.duration = duration;
                        } else if let Some(duration) = episode["duration"].as_f64() {
                            entry.duration = duration;
                        }

                        if let Some(description) = episode["description"].as_str() {
                            entry.description = to_plain_text(description);
                        }
                    }

                    entry
                })
                .collect(),
        })
        .collect()
}

/// Puts the collections and the playlists in one list.
///
/// The collections come first, because every user of the server sees them.
pub fn collect_lists(collections: &CollectionRoot, playlists: &PlaylistRoot) -> Vec<ListView> {
    let mut all = collect_collections(collections);
    all.extend(collect_playlists(playlists));
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collections() -> CollectionRoot {
        serde_json::from_str(
            r#"{"results": [
                 {"id": "c1", "name": "A Collection", "description": "<b>Two</b> books",
                  "books": [
                    {"id": "b1", "media": {"duration": 10.0, "metadata":
                       {"title": "First", "authorName": "An Author"}}},
                    {"id": "b2", "media": {"duration": 20.0, "metadata":
                       {"title": "Second", "authorName": "An Author"}}}
                  ]}
               ], "total": 1}"#,
        )
        .unwrap()
    }

    fn playlists() -> PlaylistRoot {
        serde_json::from_str(
            r#"{"results": [
                 {"id": "p1", "name": "A Playlist", "description": null,
                  "items": [
                    {"libraryItemId": "b1", "libraryItem": {"id": "b1", "media":
                       {"duration": 10.0, "metadata": {"title": "First", "authorName": "An Author"}}}},
                    {"libraryItemId": "pod1", "episodeId": "ep1",
                     "episode": {"id": "ep1", "title": "An Episode",
                                 "audioFile": {"duration": 55.0}},
                     "libraryItem": {"id": "pod1", "media":
                       {"metadata": {"title": "A Podcast", "authorName": "A Voice"}}}}
                  ]}
               ], "total": 1}"#,
        )
        .unwrap()
    }

    #[test]
    fn a_collection_gives_its_books() {
        let lists = collect_collections(&collections());

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].kind, ListKind::Collection);
        assert_eq!(lists[0].entries.len(), 2);
        assert_eq!(lists[0].entries[0].title, "First");
        assert_eq!(lists[0].entries[0].id, "b1");
        assert_eq!(lists[0].entries[1].duration, 20.0);
        assert!(!lists[0].entries[0].is_episode());
    }

    #[test]
    fn the_description_of_a_collection_holds_no_html() {
        assert_eq!(
            collect_collections(&collections())[0].description,
            "Two books"
        );
    }

    /// A playlist can hold a book and an episode together.
    #[test]
    fn a_playlist_gives_its_books_and_its_episodes() {
        let lists = collect_playlists(&playlists());

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].kind, ListKind::Playlist);
        assert_eq!(lists[0].entries.len(), 2);

        let book = &lists[0].entries[0];
        assert!(!book.is_episode());
        assert_eq!(book.id, "b1");

        let episode = &lists[0].entries[1];
        assert!(episode.is_episode());
        assert_eq!(episode.id, "pod1");
        assert_eq!(episode.episode_id.as_deref(), Some("ep1"));
    }

    /// The title of an entry that names an episode is the title of the
    /// episode, and not the title of the podcast.
    #[test]
    fn an_episode_gives_its_own_title_and_length() {
        let lists = collect_playlists(&playlists());
        let episode = &lists[0].entries[1];

        assert_eq!(episode.title, "An Episode");
        assert_eq!(episode.duration, 55.0);
        assert_eq!(episode.author, "A Voice");
    }

    #[test]
    fn a_list_with_no_description_gives_a_message() {
        assert_eq!(
            collect_playlists(&playlists())[0].description,
            "No description available"
        );
    }

    #[test]
    fn the_line_names_the_kind_and_the_number_of_items() {
        assert_eq!(
            collect_collections(&collections())[0].line(),
            "[Collection] A Collection [2 items]"
        );
        assert_eq!(
            collect_playlists(&playlists())[0].line(),
            "[Playlist] A Playlist [2 items]"
        );
    }

    #[test]
    fn a_list_of_one_item_uses_the_singular() {
        let mut list = collect_collections(&collections()).remove(0);
        list.entries.truncate(1);

        assert_eq!(list.line(), "[Collection] A Collection [1 item]");
    }

    /// The collections come before the playlists.
    #[test]
    fn the_collections_come_first() {
        let all = collect_lists(&collections(), &playlists());

        assert_eq!(all.len(), 2);
        assert_eq!(all[0].kind, ListKind::Collection);
        assert_eq!(all[1].kind, ListKind::Playlist);
    }

    #[test]
    fn an_empty_answer_gives_an_empty_list() {
        let empty = collect_lists(&CollectionRoot::default(), &PlaylistRoot::default());
        assert!(empty.is_empty());
    }

    /// A list with no medium must not stop the application.
    #[test]
    fn a_list_with_no_item_gives_no_entry() {
        let root: CollectionRoot =
            serde_json::from_str(r#"{"results": [{"id": "c", "name": "Empty"}], "total": 1}"#)
                .unwrap();
        let lists = collect_collections(&root);

        assert_eq!(lists[0].entries.len(), 0);
        assert_eq!(lists[0].line(), "[Collection] Empty [0 items]");
    }

    /// These answers come from a real Audiobookshelf 2.36.0 server.
    #[test]
    fn the_module_reads_a_real_collection() {
        let raw = include_str!("../../../tests/fixtures/library_collections.json");
        let root: CollectionRoot = serde_json::from_str(raw).unwrap();
        let lists = collect_collections(&root);

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].name, "A Test Collection");
        assert_eq!(lists[0].entries.len(), 3);
        assert_eq!(lists[0].entries[0].title, "The Test Chronicles Volume 1");
    }

    #[test]
    fn the_module_reads_a_real_playlist_of_episodes() {
        let raw = include_str!("../../../tests/fixtures/library_playlists_podcast.json");
        let root: PlaylistRoot = serde_json::from_str(raw).unwrap();
        let lists = collect_playlists(&root);

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].entries.len(), 2);
        assert!(lists[0].entries[0].is_episode());
        assert_eq!(lists[0].entries[0].title, "Letter 1");
        assert!(lists[0].entries[0].duration > 1700.0);

        // A podcast gives the author in the field `author`, and a book gives
        // it in the field `authorName`.
        assert_eq!(lists[0].entries[0].author, "LibriVox");
    }
}
