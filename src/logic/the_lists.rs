//! The collections and the playlists, between the task and the screen.
//! See T-84.
//!
//! The program changes a list of the server now: the key `m` puts a media in a
//! list, and the key `X` takes one out. **The screen must follow that change**,
//! and the request runs in a task that holds no `App`. Therefore the task puts
//! the new lists here, and the render takes them at the next frame.
//!
//! This is the shape of `logic::the_downloads` and of `logic::authors`. The
//! difference: the answer goes into `App` itself (`app.lists`), because every
//! view of the lists reads that field already.

use crate::api::client::ApiClient;
use crate::api::utils::collect_lists::ListView;
use std::sync::{Mutex, OnceLock};

fn box_of_the_lists() -> &'static Mutex<Option<Vec<ListView>>> {
    static LISTS: OnceLock<Mutex<Option<Vec<ListView>>>> = OnceLock::new();
    LISTS.get_or_init(|| Mutex::new(None))
}

/// Writes the lists that the server gave. The task of the request calls this.
pub fn keep(lists: Vec<ListView>) {
    if let Ok(mut place) = box_of_the_lists().lock() {
        *place = Some(lists);
    }
}

/// Gives the lists one time, and it forgets them.
///
/// The render calls this at each frame. A frame that gets `None` draws the
/// lists that the program holds already.
pub fn take() -> Option<Vec<ListView>> {
    box_of_the_lists().lock().ok()?.take()
}

/// Asks the server for the collections and the playlists of one library.
///
/// **The caller must wait for the write of the list before it calls this.** A
/// question that goes with the write gives the lists of the moment before it,
/// and the screen then shows the state that came before the key of the user.
/// A measurement of 2026-08-11 showed that fault: the view held "[2 items]"
/// after the key `X` took one media out.
pub async fn ask(api: &ApiClient, library: &str) {
    let collections = crate::api::libraries::get_lists::get_all_collections(api, library).await;
    let playlists = crate::api::libraries::get_lists::get_all_playlists(api, library).await;

    match (collections, playlists) {
        (Ok(collections), Ok(playlists)) => {
            keep(crate::api::utils::collect_lists::collect_lists(
                &collections,
                &playlists,
            ));
        }
        (collections, playlists) => log::warn!(
            "[lists] the server did not give the lists again: {:?} {:?}",
            collections.err(),
            playlists.err()
        ),
    }
}

/// Forgets the answer. A test starts from a known condition.
pub fn forget() {
    if let Ok(mut place) = box_of_the_lists().lock() {
        *place = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::collect_lists::ListKind;

    fn one(name: &str) -> ListView {
        ListView {
            id: "a-list".to_string(),
            kind: ListKind::Playlist,
            name: name.to_string(),
            description: String::new(),
            entries: Vec::new(),
        }
    }

    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    /// The render takes the answer one time. A second frame must not write the
    /// same lists again.
    #[test]
    fn the_answer_comes_one_time() {
        let _guard = guard();
        forget();

        assert!(take().is_none());

        keep(vec![one("A Test Playlist")]);

        let lists = take().expect("the answer must come");
        assert_eq!(lists.len(), 1);

        assert!(take().is_none());
    }
}
