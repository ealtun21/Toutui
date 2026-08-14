use crate::db::crud::*;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

pub struct Database {
    pub users: Vec<User>,
    pub default_usr: Vec<String>,
    pub listening_session: ListeningSession,
    pub others: Others,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub server_address: String,
    pub username: String,
    pub token: String,
    pub is_default_usr: bool,
    pub name_selected_lib: String,
    pub id_selected_lib: String,
    pub is_loop_break: String,
    /// Tells if the user played a media before.
    pub has_played_before: String,
    pub speed_rate: f32,
    pub is_show_key_bindings: String,
}

#[derive(Serialize, Deserialize, Debug)]
// currently use for close listening session when app is quit
// but in future could be used to sync offline items
pub struct ListeningSession {
    pub id_session: String,
    pub id_item: String,
    pub current_time: u32,
    pub duration: String,
    pub is_finished: bool,
    pub id_pod: String,
    pub elapsed_time: u32,
    pub title: String,
    pub author: String,
    pub is_playback: bool,
    pub chapter: String,
}

pub struct Others {
    pub login_err: String,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // open db and create table if there is none
        let _ = init_db();

        // init empty Vec<User> for future add of users
        let users: Vec<User> = vec![];

        // retrieve default user
        //
        // **A read of the accounts that failed is not a database with no
        // account.** The old line was `if let Ok(result) = ...`, therefore a
        // database that a second program of this account held gave a list of no
        // account: `src/main.rs` drew the login screen of a first start, the
        // user wrote the address, the name, and the password again, and the row
        // of their account stood on the disk all the time. **A table with no row
        // and a database that says nothing are two conditions**, and this
        // function keeps the two apart now: an empty table gives an empty list,
        // and a fault of the database stops the program with words of its own.
        // See T-199.
        let default_usr = select_default_usr()
            .map_err(|error| crate::db::TheAccountsDidNotCome(error.to_string()))?;

        // init listening_session
        let listening_session = ListeningSession {
            id_session: String::new(),
            id_item: String::new(),
            current_time: 0,
            duration: String::new(),
            is_finished: false,
            id_pod: String::new(),
            elapsed_time: 0,
            title: String::new(),
            author: String::new(),
            is_playback: false,
            chapter: String::new(),
        };

        let others = Others {
            login_err: String::new(),
        };

        Ok(Self {
            users,
            default_usr,
            listening_session,
            others,
        })
    }
}
