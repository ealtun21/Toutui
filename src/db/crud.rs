use crate::db::database_struct::ListeningSession;
use crate::db::database_struct::Others;
use crate::db::database_struct::User;
use crate::utils::pop_up_message::*;
use log::{error, info};
use rusqlite::{params, Connection, Result};
use std::io::stdout;

// Update is_show_key_bindings
pub fn update_is_show_key_bindings(value: &str, username: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE users SET is_show_key_bindings = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_show_key_bindings] {}", err_message);
    }

    Ok(())
}

// get is_show_key_bindings
pub fn get_is_show_key_bindings(username: &str) -> String {
    let conn = match crate::db::migrate::open_conn() {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_show_key_bindings FROM users WHERE username = ?1")
    {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id.to_string(),
        Err(_) => String::from("No db found"),
    }
}

// Update speed_rate
pub fn update_speed_rate(username: &str, is_speed_rate_up: bool) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        if is_speed_rate_up {
            conn.execute(
                "UPDATE users SET speed_rate = speed_rate + 0.10 WHERE username = ?1",
                params![username],
            )?;
        } else {
            conn.execute(
                "UPDATE users SET speed_rate = speed_rate - 0.10 WHERE username = ?1",
                params![username],
            )?;
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_speed_rate] {}", err_message);
    }

    Ok(())
}

// get speed_rate
pub fn get_speed_rate(username: &str) -> String {
    let conn = match crate::db::migrate::open_conn() {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT speed_rate FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, f32>(0)) {
        Ok(id) => id.to_string(),
        Err(_) => String::from("No db found"),
    }
}

// get listening_session
pub fn get_listening_session() -> Result<Option<ListeningSession>> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        let mut stmt = conn.prepare(
            "SELECT id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter
             FROM listening_session
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![])?;

        if let Some(row) = rows.next()? {
            let session = ListeningSession {
                id_session: row.get(0)?,
                id_item: row.get(1)?,
                current_time: row.get(2)?,
                duration: row.get(3)?,
                is_finished: row.get(4)?,
                id_pod: row.get(5)?,
                elapsed_time: row.get(6)?,
                title: row.get(7)?,
                author: row.get(8)?,
                is_playback: row.get(9)?,
                chapter: row.get(10)?,
            };
            return Ok(Some(session));
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[get_listening_session] {}", err_message);
    }

    Ok(None)
}

// insert data into `listening_session` table
// The ApiClient refactor removes the token and the address parameters.
// See docs/superpowers/plans/2026-08-09-api-client-endpoints.md, task 10.
#[allow(clippy::too_many_arguments)]
pub fn insert_listening_session(
    id_session: String,
    id_item: String,
    current_time: u32,
    duration: String,
    id_pod: String,
    elapsed_time: u32,
    title: String,
    author: String,
    is_playback: bool,
    chapter: String,
) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute("DELETE FROM listening_session", params![])?;
        conn.execute(
            "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter) 
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id_session, id_item, current_time, duration, id_pod, elapsed_time, title, author, is_playback, chapter],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[insert_listening_session] {}", err_message);
    }

    Ok(())
}

/// Removes the listening session from the database.
///
/// The application calls this function after it closes a session and sends
/// the last position to the server. A row that stays makes the application
/// send that position again at the next start. Then a position that a
/// different client wrote is lost. See T-4.
pub fn delete_listening_session() -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute("DELETE FROM listening_session", params![])?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[delete_listening_session] {}", err_message);
    }

    Ok(())
}

// Update chapter (for `listening_session` table)
pub fn update_chapter(value: &str, id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE listening_session SET chapter = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_chapter] {}", err_message);
    }

    Ok(())
}
// Update is_playback (for `listening_session` table)
pub fn update_is_playback(value: &str, id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE listening_session SET is_playback = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_playback] {}", err_message);
    }

    Ok(())
}
// Update current_time (for `listening_session` table)
pub fn update_current_time(value: u32, id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE listening_session SET current_time_playback = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_current_time] {}", err_message);
    }

    Ok(())
}

// Update elapsed_time (for `listening_session` table)
pub fn update_elapsed_time(value: u32, id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE listening_session SET elapsed_time = elapsed_time + ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_elapsed_time] {}", err_message);
    }

    Ok(())
}

// Update is_finished (for `listening_session` table)
pub fn update_is_finished(value: &str, id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE listening_session SET is_finished = ?1 WHERE id_session = ?2",
            params![value, id_session],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_finished] {}", err_message);
    }

    Ok(())
}

// Delete an user
pub fn delete_user(username: &str) -> Result<()> {
    let message = format!(
        "User '{}' deleted. Please restart the app to apply the changes.",
        username
    );
    let err_message = "Error connecting to the database.";
    if let Ok(conn) = crate::db::migrate::open_conn() {
        let rows_deleted =
            conn.execute("DELETE FROM users WHERE username = ?1", params![username])?;

        if rows_deleted > 0 {
            let mut stdout = stdout();
            let _ = pop_message(&mut stdout, 3, message.as_str());
            info!("[delete_user] User deleted.");
        } else {
            //println!("No user found with this username '{}'.", username);
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[delete user] {}", err_message);
    }

    Ok(())
}

// Update is_loop_break
pub fn update_is_loop_break(value: &str, username: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE users SET is_loop_break = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_is_loop_break] {}", err_message);
    }

    Ok(())
}

// get is_loop_break
pub fn get_is_loop_break(username: &str) -> String {
    let conn = match crate::db::migrate::open_conn() {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT is_loop_break FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id,
        Err(_) => String::from("No db found"),
    }
}

// Update is_vlv_launched_first_time
pub fn update_has_played_before(value: &str, username: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE users SET has_played_before = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_has_played_before] {}", err_message);
    }

    Ok(())
}
/// Tells if the user played a media before. The application uses this value
/// to know that it can stop with no session.
pub fn get_has_played_before(username: &str) -> String {
    let conn = match crate::db::migrate::open_conn() {
        Ok(c) => c,
        Err(_) => return String::from("Error: unable open database"),
    };

    let mut stmt = match conn.prepare("SELECT has_played_before FROM users WHERE username = ?1") {
        Ok(s) => s,
        Err(_) => return String::from("Error to prepare reqwest"),
    };

    match stmt.query_row(params![username], |row| row.get::<_, String>(0)) {
        Ok(id) => id,
        Err(_) => String::from("No db found"),
    }
}
// Update id_selected_lib
pub fn update_id_selected_lib(id_selected_lib: &str, username: &str) -> Result<()> {
    let message = "The library has been updated. Please refresh the app to apply the changes.";
    let err_message = "Error connecting to the database.";
    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE users SET id_selected_lib = ?1 WHERE username = ?2",
            params![id_selected_lib, username],
        )?;
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, message);
        info!("[update_id_selected_lib] The library has been updated");
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_id_selected_lib] {}", err_message);
    }

    Ok(())
}

// update default user
//pub fn update_default_user(conn: &Connection, username: &str) -> Result<()> {
//    // Mark all user as 0 by default
//    conn.execute(
//        "UPDATE users SET is_default_usr = 0",
//        [],
//    )?;
//
//    // Put the desired user as default
//    conn.execute(
//        "UPDATE users SET is_default_usr = 1 WHERE username = ?1",
//        params![username],
//    )?;
//
//    Ok(())
//}

// Insert user in database
pub fn db_insert_usr(users: &Vec<User>) -> Result<()> {
    let conn = crate::db::migrate::open_conn()?;
    for user in users {
        conn.execute(
            "INSERT OR REPLACE INTO users (username, server_address, token, is_default_usr, name_selected_lib, id_selected_lib, is_loop_break, has_played_before, speed_rate, is_show_key_bindings) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
            user.username,
            user.server_address,
            user.token,
            if user.is_default_usr { 1 } else { 0 },
            user.name_selected_lib,
            user.id_selected_lib,
            user.is_loop_break,
            user.has_played_before,
            user.speed_rate,
            user.is_show_key_bindings,
            ],
        )?;
    }
    Ok(())
}

// get others
pub fn get_others() -> Result<Option<Others>> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        let mut stmt = conn.prepare(
            "SELECT login_err
             FROM others
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![])?;

        if let Some(row) = rows.next()? {
            let others = Others {
                login_err: row.get(0)?,
            };
            return Ok(Some(others));
        }
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[get_others] {}", err_message);
    }

    Ok(None)
}
// Update login_err (for `others` table)
pub fn update_login_err(value: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "INSERT INTO others (login_err) SELECT '' WHERE NOT EXISTS (SELECT 1 FROM others LIMIT 1)",
            [],
        )?;
        conn.execute(
            "UPDATE others SET login_err = ?1 WHERE rowid = 1",
            params![value],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_login_err] {}", err_message);
    }

    Ok(())
}

// Select default user
pub fn select_default_usr() -> Result<Vec<String>> {
    let conn = crate::db::migrate::open_conn()?;

    let mut stmt = conn.prepare(
        "SELECT username, server_address, token, is_default_usr, name_selected_lib, id_selected_lib, is_loop_break, has_played_before, speed_rate, is_show_key_bindings
         FROM users WHERE is_default_usr = 1 LIMIT 1"
    )?;

    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            username: row.get(0)?,
            server_address: row.get(1)?,
            token: row.get(2)?,
            is_default_usr: row.get::<_, i32>(3)? != 0, // convert 0/1 in bool
            name_selected_lib: row.get(4)?,
            id_selected_lib: row.get(5)?,
            is_loop_break: row.get(6)?,
            has_played_before: row.get(7)?,
            speed_rate: row.get(8)?,
            is_show_key_bindings: row.get(9)?,
        })
    })?;

    let mut result = Vec::new();

    for user in user_iter {
        match user {
            Ok(user) => {
                result.push(user.username);
                result.push(user.server_address);
                result.push(user.token);
                result.push(user.is_default_usr.to_string());
                result.push(user.name_selected_lib);
                result.push(user.id_selected_lib);
                result.push(user.is_loop_break);
                result.push(user.has_played_before);
                result.push(user.speed_rate.to_string());
                result.push(user.is_show_key_bindings);
            }
            Err(e) => {
                println!("Error occurred: {}", e);
                //return Err(rusqlite::Error::FromSqlConversionFailure(0, "Failed to map user".to_string()));
            }
        }
    }

    if result.is_empty() {
        //println!("No default user found.");
    }

    Ok(result)
}

/// Opens the database and applies the migrations.
pub fn init_db() -> Result<()> {
    let conn = Connection::open(crate::db::migrate::db_path())?;
    crate::db::migrate::run_migrations(&conn)?;
    Ok(())
}

// Insert (or replace) a downloaded item (for `downloads` table)
/// The parameter `id_item` is the key of the download: the item of a book, or
/// the episode of a podcast. The parameter `item_id` is always the identity of
/// the library item, because the server needs that value for the progress.
#[allow(clippy::too_many_arguments)]
pub fn insert_download(
    id_item: &str,
    username: &str,
    title: &str,
    author: &str,
    file_path: &str,
    duration: f64,
    item_id: &str,
    server: &str,
) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "INSERT OR REPLACE INTO downloads (id_item, username, title, author, file_path, duration, current_time_offline, downloaded_at, item_id, server)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, datetime('now'), ?7, ?8)",
            params![id_item, username, title, author, file_path, duration, item_id, server],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[insert_download] {}", err_message);
    }

    Ok(())
}

// Get a downloaded item: (file_path, current_time_offline, duration, title, author) (for `downloads` table)
pub fn get_download(id_item: &str, username: &str) -> Option<(String, u32, f64, String, String)> {
    let conn = crate::db::migrate::open_conn().ok()?;

    let mut stmt = conn.prepare(
        "SELECT file_path, current_time_offline, duration, title, author FROM downloads WHERE id_item = ?1 AND username = ?2"
    ).ok()?;

    stmt.query_row(params![id_item, username], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, u32>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })
    .ok()
}

// Update current_time_offline (for `downloads` table)
pub fn update_download_current_time(id_item: &str, username: &str, value: u32) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE downloads SET current_time_offline = ?1 WHERE id_item = ?2 AND username = ?3",
            params![value, id_item, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[update_download_current_time] {}", err_message);
    }

    Ok(())
}

// Insert (or replace) one audio file of a downloaded item (for `download_files` table)
pub fn insert_download_file(
    id_item: &str,
    username: &str,
    idx: u32,
    ino: &str,
    file_path: &str,
    size: u64,
    duration: f64,
) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "INSERT OR REPLACE INTO download_files (id_item, username, idx, ino, file_path, size, duration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id_item, username, idx, ino, file_path, size as i64, duration],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[insert_download_file] {}", err_message);
    }

    Ok(())
}

// Get the audio files of a downloaded item: (idx, file_path, duration) (for `download_files` table)
// The offline player reads this list. No caller exists yet.
#[allow(dead_code)]
pub fn get_download_files(id_item: &str, username: &str) -> Vec<(u32, String, f64)> {
    let Ok(conn) = crate::db::migrate::open_conn() else {
        return Vec::new();
    };

    let Ok(mut stmt) = conn.prepare(
        "SELECT idx, file_path, duration FROM download_files WHERE id_item = ?1 AND username = ?2 ORDER BY idx"
    ) else {
        return Vec::new();
    };

    let rows = stmt.query_map(params![id_item, username], |row| {
        Ok((
            row.get::<_, u32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
        ))
    });

    match rows {
        Ok(rows) => rows.filter_map(|row| row.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

// Delete a downloaded item (for `downloads` and `download_files` tables)
pub fn delete_download(id_item: &str, username: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "DELETE FROM downloads WHERE id_item = ?1 AND username = ?2",
            params![id_item, username],
        )?;

        conn.execute(
            "DELETE FROM download_files WHERE id_item = ?1 AND username = ?2",
            params![id_item, username],
        )?;
    } else {
        let mut stdout = stdout();
        let _ = pop_message(&mut stdout, 3, err_message);
        error!("[delete_download] {}", err_message);
    }

    Ok(())
}

/// One media that the user downloaded.
///
/// The offline mode makes its lists from these rows, because the server gives
/// no answer. See T-25.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadRow {
    /// The identity of the download. It is the item for a book, and the
    /// episode for one episode of a podcast.
    pub key: String,
    /// The identity of the library item. It is the podcast for an episode.
    pub item_id: String,
    pub title: String,
    pub author: String,
    pub file_path: String,
    pub duration: f64,
    /// The position of the local playback, in seconds.
    pub current_time: u32,
}

/// Gives every download of one user on one server, with the newest first.
///
/// A row of an older version has no server, and it belongs to the server that
/// the user has now.
pub fn get_all_downloads(username: &str, server: &str) -> Vec<DownloadRow> {
    let Ok(conn) = crate::db::migrate::open_conn() else {
        return Vec::new();
    };

    let Ok(mut stmt) = conn.prepare(
        "SELECT id_item, title, author, file_path, duration, current_time_offline, item_id
         FROM downloads
         WHERE username = ?1 AND (server = ?2 OR server = '')
         ORDER BY downloaded_at DESC, title",
    ) else {
        return Vec::new();
    };

    let rows = stmt.query_map(params![username, server], |row| {
        Ok(DownloadRow {
            key: row.get::<_, String>(0)?,
            title: row.get::<_, String>(1)?,
            author: row.get::<_, String>(2)?,
            file_path: row.get::<_, String>(3)?,
            duration: row.get::<_, f64>(4)?,
            current_time: row.get::<_, u32>(5)?,
            item_id: row.get::<_, String>(6)?,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(|row| row.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Gives one download by its key.
pub fn get_download_row(key: &str, username: &str) -> Option<DownloadRow> {
    let conn = crate::db::migrate::open_conn().ok()?;

    let mut stmt = conn
        .prepare(
            "SELECT id_item, title, author, file_path, duration, current_time_offline, item_id
             FROM downloads WHERE id_item = ?1 AND username = ?2",
        )
        .ok()?;

    stmt.query_row(params![key, username], |row| {
        Ok(DownloadRow {
            key: row.get::<_, String>(0)?,
            title: row.get::<_, String>(1)?,
            author: row.get::<_, String>(2)?,
            file_path: row.get::<_, String>(3)?,
            duration: row.get::<_, f64>(4)?,
            current_time: row.get::<_, u32>(5)?,
            item_id: row.get::<_, String>(6)?,
        })
    })
    .ok()
}

/// One position that waits for the server.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProgress {
    /// The identity of the library item.
    pub id_item: String,
    /// The identity of the episode. A book has an empty value.
    pub id_pod: String,
    pub current_time: f64,
    pub duration: f64,
    pub is_finished: bool,
    /// The time of the local computer in milliseconds.
    pub updated_at: i64,
}

/// The statement that writes a position that waits for the server.
///
/// The name of a column must not be a keyword of SQLite. `current_time` is a
/// keyword: a query then gives the time of the day, and not the value of the
/// row. The test `a_pending_position_comes_back_as_a_number` guards this.
const INSERT_PENDING: &str = "INSERT OR REPLACE INTO pending_progress
     (id_item, username, server, id_pod, position_s, duration, is_finished, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

/// The statement that reads the positions that wait for the server.
/// The application sends the rows of one server only. A row of an older
/// version has an empty value, and it belongs to the server that the user has
/// now.
const SELECT_PENDING: &str = "SELECT id_item, id_pod, position_s, duration, is_finished, updated_at
     FROM pending_progress
     WHERE username = ?1 AND (server = ?2 OR server = '')
     ORDER BY updated_at";

/// Writes a position that waits for the server.
///
/// A newer position of the same media replaces the older position. The
/// application sends the last position only.
pub fn insert_pending_progress(
    username: &str,
    server: &str,
    progress: &PendingProgress,
) -> Result<()> {
    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            INSERT_PENDING,
            params![
                progress.id_item,
                username,
                server,
                progress.id_pod,
                progress.current_time,
                progress.duration,
                if progress.is_finished { 1 } else { 0 },
                progress.updated_at,
            ],
        )?;
    } else {
        error!("[insert_pending_progress] Error connecting to the database.");
    }

    Ok(())
}

/// Gives every position that waits for the given server, with the oldest
/// first.
pub fn get_pending_progress(username: &str, server: &str) -> Vec<PendingProgress> {
    let Ok(conn) = crate::db::migrate::open_conn() else {
        return Vec::new();
    };

    let Ok(mut stmt) = conn.prepare(SELECT_PENDING) else {
        return Vec::new();
    };

    let rows = stmt.query_map(params![username, server], |row| {
        Ok(PendingProgress {
            id_item: row.get::<_, String>(0)?,
            id_pod: row.get::<_, String>(1)?,
            current_time: row.get::<_, f64>(2)?,
            duration: row.get::<_, f64>(3)?,
            is_finished: row.get::<_, i64>(4)? != 0,
            updated_at: row.get::<_, i64>(5)?,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(|row| row.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Removes a position that the server has now.
pub fn delete_pending_progress(username: &str, id_item: &str, id_pod: &str) -> Result<()> {
    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "DELETE FROM pending_progress WHERE username = ?1 AND id_item = ?2 AND id_pod = ?3",
            params![username, id_item, id_pod],
        )?;
    } else {
        error!("[delete_pending_progress] Error connecting to the database.");
    }

    Ok(())
}

/// Gives the number of positions that wait for the given server.
pub fn count_pending_progress(username: &str, server: &str) -> usize {
    let Ok(conn) = crate::db::migrate::open_conn() else {
        return 0;
    };

    conn.query_row(
        "SELECT COUNT(*) FROM pending_progress
         WHERE username = ?1 AND (server = ?2 OR server = '')",
        params![username, server],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// The two statements of the table `pending_progress` must give a number
    /// back.
    ///
    /// A column with the name `current_time` gave the time of the day,
    /// because that name is a keyword of SQLite. The row then did not agree
    /// with the type, and the application sent no position at all.
    #[test]
    fn a_pending_position_comes_back_as_a_number() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run_migrations(&conn).unwrap();

        conn.execute(
            INSERT_PENDING,
            params![
                "item-1",
                "bob",
                "home",
                "",
                61.5_f64,
                120.0_f64,
                1_i64,
                1_700_000_000_000_i64
            ],
        )
        .unwrap();

        let mut stmt = conn.prepare(SELECT_PENDING).unwrap();

        let rows: Vec<(String, String, f64, f64, i64, i64)> = stmt
            .query_map(params!["bob", "home"], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .unwrap()
            .map(|row| row.unwrap())
            .collect();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "item-1");
        assert_eq!(rows[0].2, 61.5);
        assert_eq!(rows[0].3, 120.0);
        assert_eq!(rows[0].4, 1);
        assert_eq!(rows[0].5, 1_700_000_000_000);
    }

    /// A newer position of the same media replaces the older position.
    #[test]
    fn a_newer_position_replaces_the_older_position() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run_migrations(&conn).unwrap();

        for position in [10.0_f64, 40.0_f64] {
            conn.execute(
                INSERT_PENDING,
                params!["item-1", "bob", "home", "", position, 120.0_f64, 0_i64, 1_i64],
            )
            .unwrap();
        }

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM pending_progress", [], |row| {
                row.get(0)
            })
            .unwrap();
        let value: f64 = conn
            .query_row("SELECT position_s FROM pending_progress", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(value, 40.0);
    }

    /// The same media on two servers gives two rows, and the application reads
    /// the rows of one server only. A position must never go to a different
    /// server.
    #[test]
    fn two_servers_keep_two_separate_positions() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run_migrations(&conn).unwrap();

        for (server, position) in [("home", 10.0_f64), ("work", 20.0_f64)] {
            conn.execute(
                INSERT_PENDING,
                params!["item-1", "bob", server, "", position, 120.0_f64, 0_i64, 1_i64],
            )
            .unwrap();
        }

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM pending_progress", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 2);

        let mut stmt = conn.prepare(SELECT_PENDING).unwrap();
        let positions: Vec<f64> = stmt
            .query_map(params!["bob", "work"], |row| row.get::<_, f64>(2))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();

        assert_eq!(positions, vec![20.0]);
    }

    /// A row of an older version has no server. The application must still
    /// send it.
    #[test]
    fn a_row_with_no_server_belongs_to_the_server_of_now() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run_migrations(&conn).unwrap();

        conn.execute(
            INSERT_PENDING,
            params!["item-1", "bob", "", "", 30.0_f64, 120.0_f64, 0_i64, 1_i64],
        )
        .unwrap();

        let mut stmt = conn.prepare(SELECT_PENDING).unwrap();
        let count = stmt
            .query_map(params!["bob", "any-server"], |row| row.get::<_, f64>(2))
            .unwrap()
            .count();

        assert_eq!(count, 1);
    }

    /// A book and an episode of the same podcast are two separate rows.
    #[test]
    fn an_episode_and_a_book_are_two_rows() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run_migrations(&conn).unwrap();

        conn.execute(
            INSERT_PENDING,
            params!["pod-1", "bob", "home", "ep-1", 10.0_f64, 120.0_f64, 0_i64, 1_i64],
        )
        .unwrap();
        conn.execute(
            INSERT_PENDING,
            params!["pod-1", "bob", "home", "ep-2", 20.0_f64, 120.0_f64, 0_i64, 2_i64],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM pending_progress", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 2);
    }
}
