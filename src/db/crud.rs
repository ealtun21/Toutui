use crate::db::database_struct::ListeningSession;
use crate::db::database_struct::Others;
use crate::db::database_struct::User;
use log::{error, info, warn};
use rusqlite::{params, Connection, OptionalExtension, Result};

/// Says which program owns a row of `listening_session`. See T-140.
///
/// The identity of the process is enough, and it needs no dependency: two
/// programs of one machine never hold one number at one moment. A number that
/// the system gives again after a program stopped belongs to a row that stands
/// still already, and such a row goes to the program that asks.
pub fn the_owner_of_this_program() -> String {
    std::process::id().to_string()
}

/// Gives the moment of now, in seconds.
fn the_moment_of_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs() as i64)
        .unwrap_or(0)
}

/// Gives the moment before which a row belongs to a program that does not live
/// any more.
fn the_moment_of_a_program_that_died() -> i64 {
    the_moment_of_now().saturating_sub(THE_LIMIT_OF_THE_HEARTBEAT as i64)
}

/// The time that a row of `listening_session` may stand still before another
/// program of the same account takes it, in seconds. See T-140.
///
/// The loop of a playback writes the position of each second, therefore a
/// program that lives touches its row every second. **A row that stands still
/// belongs to a program that stopped without a correct exit**, and the rule of
/// T-4 says that the next program sends that position one time.
///
/// The limit is longer than one second, because the loop writes nothing while
/// the engine seeks to the place of the user (T-38).
pub const THE_LIMIT_OF_THE_HEARTBEAT: u64 = 30;

// Update is_show_key_bindings
pub fn update_is_show_key_bindings(value: &str, username: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE users SET is_show_key_bindings = ?1 WHERE username = ?2",
            params![value, username],
        )?;
    } else {
        crate::logic::message::say(err_message);
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

/// Reads the sequence and the filter of the library of an account. See T-24.
///
/// The three values are the name of the field, `1` for the other direction,
/// and the filter of the server. An account of an older database gives three
/// empty texts, and the program then asks the server as it did before.
pub fn get_library_sort(username: &str) -> (String, bool, String) {
    let nothing = (String::new(), false, String::new());

    let Ok(conn) = crate::db::migrate::open_conn() else {
        return nothing;
    };

    let Ok(mut stmt) = conn.prepare(
        "SELECT library_sort, library_desc, library_filter FROM users WHERE username = ?1",
    ) else {
        return nothing;
    };

    stmt.query_row(params![username], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)? == "1",
            row.get::<_, String>(2)?,
        ))
    })
    .unwrap_or(nothing)
}

/// Writes the sequence and the filter of the library of an account.
pub fn update_library_sort(username: &str, field: &str, desc: bool, filter: &str) -> Result<()> {
    let conn = match crate::db::migrate::open_conn() {
        Ok(conn) => conn,
        Err(error) => {
            error!("[update_library_sort] {}", error);
            return Ok(());
        }
    };

    conn.execute(
        "UPDATE users SET library_sort = ?1, library_desc = ?2, library_filter = ?3
         WHERE username = ?4",
        params![field, if desc { "1" } else { "0" }, filter, username],
    )?;

    Ok(())
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
        crate::logic::message::say(err_message);
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

/// Gives the listening session of one account, and nothing for the session of
/// another account. See T-138.
///
/// **A session belongs to one account of one server.** The program of a second
/// account read the row of the first one before this rule, and it then sent the
/// position of a media that its own server does not hold: the server refused it,
/// and the program removed the row. The place of the user went away.
///
/// A row that an older program wrote holds no account, therefore the account
/// that asks takes it: such a database holds the row of the one account that
/// program had.
pub fn get_listening_session(username: &str, server: &str) -> Result<Option<ListeningSession>> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        let mut stmt = conn.prepare(
            "SELECT id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter
             FROM listening_session
             WHERE ((username = ?1 AND server = ?2) OR (username = '' AND server = ''))
               AND (owner = ?3 OR heartbeat <= ?4)
             ORDER BY (owner = ?3) DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![
            username,
            server,
            the_owner_of_this_program(),
            the_moment_of_a_program_that_died()
        ])?;

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
        crate::logic::message::say(err_message);
        error!("[get_listening_session] {}", err_message);
    }

    Ok(None)
}

/// Gives **every** session that this program may close, and not one of them.
/// See T-145.
///
/// `get_listening_session` gives one row, and the close of a session removed
/// every row that this program may take: the row of a program that died then
/// went away with no request, and the place of the user of that program went
/// away with it. **The program closes each of these rows, and it removes a row
/// after that row reached the server.**
///
/// The rows of a program that died come first, and the row of this program
/// comes last. Two rows of one media therefore leave the newest position on the
/// server.
pub fn get_the_sessions_to_close(username: &str, server: &str) -> Result<Vec<ListeningSession>> {
    let err_message = "Error connecting to the database.";
    let mut sessions = Vec::new();

    if let Ok(conn) = crate::db::migrate::open_conn() {
        let mut stmt = conn.prepare(
            "SELECT id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter
             FROM listening_session
             WHERE ((username = ?1 AND server = ?2) OR (username = '' AND server = ''))
               AND (owner = ?3 OR heartbeat <= ?4)
             ORDER BY (owner = ?3) ASC, heartbeat ASC",
        )?;

        let rows = stmt.query_map(
            params![
                username,
                server,
                the_owner_of_this_program(),
                the_moment_of_a_program_that_died()
            ],
            |row| {
                Ok(ListeningSession {
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
                })
            },
        )?;

        for session in rows {
            sessions.push(session?);
        }
    } else {
        crate::logic::message::say(err_message);
        error!("[get_the_sessions_to_close] {}", err_message);
    }

    Ok(sessions)
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
    username: &str,
    server: &str,
) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        // The row of this account goes away, and the row of another account
        // stays: that account sends its own position to its own server. A row
        // of an older program holds no account, and it belongs to the account
        // that writes here. See T-138.
        conn.execute(
            "DELETE FROM listening_session
             WHERE ((username = ?1 AND server = ?2) OR (username = '' AND server = ''))
               AND (owner = ?3 OR heartbeat <= ?4)",
            params![
                username,
                server,
                the_owner_of_this_program(),
                the_moment_of_a_program_that_died()
            ],
        )?;
        conn.execute(
            "INSERT INTO listening_session (id_session, id_item, current_time_playback, duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter, username, server, owner, heartbeat)
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![id_session, id_item, current_time, duration, id_pod, elapsed_time, title, author, is_playback, chapter, username, server, the_owner_of_this_program(), the_moment_of_now()],
        )?;
    } else {
        crate::logic::message::say(err_message);
        error!("[insert_listening_session] {}", err_message);
    }

    Ok(())
}

/// Removes the row of **one playback**, and no other row of the table. See
/// T-141.
///
/// The loop of a playback closes its session on the server itself, and it sends
/// the position: the place of the user is then safe, therefore the row must go
/// away. **A row that stays sends that position again at the next start or at
/// the key `Q`**, and it destroys a place that a different client wrote — that
/// is the fault of T-4, and a media that came to its end met it again.
///
/// The identity of the session is the key, because the loop knows its own
/// playback and nothing else: another program of the same account keeps its row.
pub fn delete_the_session_of_a_playback(id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "DELETE FROM listening_session WHERE id_session = ?1",
            params![id_session],
        )?;
    } else {
        crate::logic::message::say(err_message);
        error!("[delete_the_session_of_a_playback] {}", err_message);
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
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
        error!("[update_is_playback] {}", err_message);
    }

    Ok(())
}
/// Writes the position of one second of a playback.
///
/// **The second says that the program lives too.** The loop of the playback
/// calls this function every second, therefore the row of a program that stopped
/// without a correct exit stands still: another program of the same account then
/// takes that row, and it sends the position one time. See T-140 and T-4.
pub fn update_current_time(value: u32, id_session: &str) -> Result<()> {
    let err_message = "Error connecting to the database.";

    if let Ok(conn) = crate::db::migrate::open_conn() {
        conn.execute(
            "UPDATE listening_session SET current_time_playback = ?1, heartbeat = ?3 WHERE id_session = ?2",
            params![value, id_session, the_moment_of_now()],
        )?;
    } else {
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
        error!("[update_is_finished] {}", err_message);
    }

    Ok(())
}

/// Removes the row of an account, and it says nothing to the user.
///
/// Gives the number of rows that went away.
///
/// **The login screen says the sentence of a token that is not valid**, and
/// `delete_user` says a sentence of its own. Therefore the two conditions need
/// two functions, and the work of the database stands here one time. The rows of
/// the downloads, of the queue, and of the positions that wait hold the name of
/// the account, and no key of the database removes them with this row: a login
/// with the same name finds all of them again. See T-123.
pub fn remove_the_account(username: &str) -> Result<usize> {
    let conn = crate::db::migrate::open_conn()?;

    let rows_deleted = conn.execute("DELETE FROM users WHERE username = ?1", params![username])?;
    info!("[remove_the_account] {} row(s) went away.", rows_deleted);

    Ok(rows_deleted)
}

// Delete an user
pub fn delete_user(username: &str) -> Result<()> {
    // The words of a user, and not the words of the code. See T-118.
    let message = format!(
        "The program removed the account {}. Start the program again.",
        username
    );
    let err_message = "Error connecting to the database.";

    match remove_the_account(username) {
        Ok(rows_deleted) => {
            if rows_deleted > 0 {
                crate::logic::message::say(message.as_str());
                info!("[delete_user] User deleted.");
            }
        }
        Err(error) => {
            crate::logic::message::say(err_message);
            error!("[delete user] {}: {}", err_message, error);
        }
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
        crate::logic::message::say(err_message);
        error!("[update_is_loop_break] {}", err_message);
    }

    Ok(())
}

/// Tells if the loop of the playback before this one came to its end.
///
/// **`None` says that the account holds no row of the disk.** A second program
/// of one account logs out while this program runs, and the row of `users` then
/// goes away (T-155). The old form of this function gave the text "No db found"
/// for that condition, and its one caller waited for that text to become `1`:
/// **the program then waited for ever** (T-158).
pub fn get_is_loop_break(username: &str) -> Option<String> {
    let conn = crate::db::migrate::open_conn().ok()?;

    let mut stmt = conn
        .prepare("SELECT is_loop_break FROM users WHERE username = ?1")
        .ok()?;

    stmt.query_row(params![username], |row| row.get::<_, String>(0))
        .ok()
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
        crate::logic::message::say(err_message);
        error!("[update_has_played_before] {}", err_message);
    }

    Ok(())
}
/// Tells if the user played a media before. The application uses this value
/// to know that it can stop with no session.
///
/// **`None` says that the account holds no row of the disk**, as
/// `get_is_loop_break` does. See T-158.
pub fn get_has_played_before(username: &str) -> Option<String> {
    let conn = crate::db::migrate::open_conn().ok()?;

    let mut stmt = conn
        .prepare("SELECT has_played_before FROM users WHERE username = ?1")
        .ok()?;

    stmt.query_row(params![username], |row| row.get::<_, String>(0))
        .ok()
}
/// Writes the library of the account, and it gives the number of the rows that
/// changed.
///
/// **A name that no row holds changes no row, and the user must not read that
/// the program kept their choice.** A second program of one account logs out
/// while this program runs (T-155): the write then changed 0 rows, and the
/// program said "The library has been updated" all the same. See T-159.
pub fn update_id_selected_lib(id_selected_lib: &str, username: &str) -> Result<usize> {
    let message = "The library has been updated. Please refresh the app to apply the changes.";
    let err_message = "Error connecting to the database.";

    let Ok(conn) = crate::db::migrate::open_conn() else {
        crate::logic::message::say(err_message);
        error!("[update_id_selected_lib] {}", err_message);

        return Ok(0);
    };

    let rows = conn.execute(
        "UPDATE users SET id_selected_lib = ?1 WHERE username = ?2",
        params![id_selected_lib, username],
    )?;

    if rows > 0 {
        crate::logic::message::say(message);
        info!("[update_id_selected_lib] The library has been updated");
    } else {
        crate::logic::message::say(
            crate::logic::the_accounts::the_text_of_an_account_that_is_gone(username).as_str(),
        );
        log::warn!(
            "[update_id_selected_lib] the account {} stands in no row of the disk. \
             The library of that account did not change.",
            username
        );
    }

    Ok(rows)
}

/// Gives every account of the database, in the sequence of its row. See T-124.
///
/// The three values of a line are the name, the address of the server, and
/// "this account starts". The view of the accounts holds one line for each of
/// them, and the key `c` gives the mark to a different line.
pub fn select_every_usr() -> Result<Vec<(String, String, bool)>> {
    let conn = crate::db::migrate::open_conn()?;

    let mut stmt =
        conn.prepare("SELECT username, server_address, is_default_usr FROM users ORDER BY rowid")?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i32>(2)? != 0,
        ))
    })?;

    Ok(rows.flatten().collect())
}

/// Gives one account the start of the program, and it takes that mark from
/// every other account. See T-124.
///
/// `select_default_usr` reads `WHERE is_default_usr = 1 LIMIT 1`, therefore two
/// rows with that value let the **rowid** decide. The two writes stand in one
/// transaction: a program that stops between them would hold no account at all,
/// and the login screen would then come at the next start.
///
/// **An account that the database does not hold takes the mark from every other
/// account and it gives that mark to nobody.** The view of the accounts is the
/// list of one process, therefore a second program of the account can remove an
/// account that the line of the user still names (T-155). The function gives
/// the database back as it was then, and it gives `0` to its caller.
pub fn make_this_account_the_default(username: &str) -> Result<usize> {
    let mut conn = crate::db::migrate::open_conn()?;
    let work = conn.transaction()?;

    work.execute("UPDATE users SET is_default_usr = 0", [])?;
    let rows = work.execute(
        "UPDATE users SET is_default_usr = 1 WHERE username = ?1",
        params![username],
    )?;

    if rows == 0 {
        work.rollback()?;
        warn!(
            "[make_this_account_the_default] the database holds no account {}. \
             The account of the start stays.",
            username
        );
        return Ok(0);
    }

    work.commit()?;

    Ok(rows)
}

/// Gives the start of the program to the first account when no account holds
/// it. See T-155.
///
/// **The login screen comes when `select_default_usr` gives no row**, and the
/// database can hold an account with a valid token at that moment: a write of
/// the mark that named an account of no row left every row at `0` before
/// T-155. A user of such a database has no key that gives the mark back, in any
/// view and after every start — that is the shape of T-136, and the start is the
/// place of the answer.
///
/// Gives the name of the account that takes the start, and `None` when an
/// account holds the mark already or when the database holds no account.
pub fn an_account_takes_the_start_when_none_holds_it() -> Result<Option<String>> {
    let conn = crate::db::migrate::open_conn()?;

    let with_the_mark: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE is_default_usr = 1",
        [],
        |row| row.get(0),
    )?;

    if with_the_mark > 0 {
        return Ok(None);
    }

    let first: Option<String> = conn
        .query_row(
            "SELECT username FROM users ORDER BY rowid LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let Some(name) = first else {
        return Ok(None);
    };

    conn.execute(
        "UPDATE users SET is_default_usr = 1 WHERE username = ?1",
        params![name],
    )?;

    info!(
        "[an_account_takes_the_start] no account held the start. The account {} takes it.",
        name
    );

    Ok(Some(name))
}

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
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
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
        crate::logic::message::say(err_message);
        error!("[insert_download_file] {}", err_message);
    }

    Ok(())
}

/// Removes the rows of the files of a download that the book no longer holds.
///
/// **A book of the server can lose a file** (T-187). `insert_download_file`
/// writes over the row of a file of the same number, and it leaves the row of
/// every number after it: the offline playback then holds a file that the book
/// does not hold, and it plays that file. This function takes those rows away.
///
/// The list holds the numbers of the files of the plan of the download. A list
/// with no number removes every row of that download.
pub fn keep_the_files_of_the_download(
    id_item: &str,
    username: &str,
    the_numbers: &[u32],
) -> Result<()> {
    let err_message = "Error connecting to the database.";

    let Ok(conn) = crate::db::migrate::open_conn() else {
        crate::logic::message::say(err_message);
        error!("[keep_the_files_of_the_download] {}", err_message);
        return Ok(());
    };

    // The numbers come from the plan of the download, and each of them is a
    // number of the program. Therefore the statement holds them, and no value
    // of the server reaches this text.
    let of_the_book: Vec<String> = if the_numbers.is_empty() {
        // `NOT IN ()` is no statement of SQL. No row holds the number -1,
        // therefore this list removes every row of the download.
        vec!["-1".to_string()]
    } else {
        the_numbers.iter().map(|one| one.to_string()).collect()
    };

    conn.execute(
        &format!(
            "DELETE FROM download_files WHERE id_item = ?1 AND username = ?2 AND idx NOT IN ({})",
            of_the_book.join(",")
        ),
        params![id_item, username],
    )?;

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
        crate::logic::message::say(err_message);
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

/// Tells if a program of this account plays this media from the disk now. See
/// T-156.
///
/// **An offline playback opens no session on the server** (T-152), therefore the
/// table `listening_session` holds no row of it and a second program of the
/// account can see nothing of that work. The loop of that playback keeps the
/// place of the user in `pending_progress` **at each second** since T-152, and
/// that moment is the heartbeat: a media whose place moved inside
/// `THE_LIMIT_OF_THE_HEARTBEAT` seconds belongs to a playback that runs. It is
/// the rule of T-140, of T-148, and of T-153, and it needs no new column and no
/// call of the system.
///
/// `updated_at` holds milliseconds.
pub fn a_program_keeps_the_place_of_this_media(
    username: &str,
    id_item: &str,
    id_pod: &str,
) -> bool {
    let Ok(conn) = crate::db::migrate::open_conn() else {
        return false;
    };

    let newest: Option<i64> = conn
        .query_row(
            "SELECT MAX(updated_at) FROM pending_progress \
             WHERE username = ?1 AND id_item = ?2 AND id_pod = ?3",
            params![username, id_item, id_pod],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let Some(newest) = newest else {
        return false;
    };

    let limit = the_moment_of_now().saturating_sub(THE_LIMIT_OF_THE_HEARTBEAT as i64) * 1000;

    newest >= limit
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

/// Writes the whole queue of one account of one server. See T-56.
///
/// The queue is short: a user puts some media in it. Therefore the function
/// writes every row again, and it needs no rule for a row that changed.
pub fn save_the_queue(username: &str, server: &str, rows: &[QueueRow]) -> Result<()> {
    let Ok(mut conn) = crate::db::migrate::open_conn() else {
        error!("[save_the_queue] the program cannot open the database.");
        return Ok(());
    };

    let work = conn.transaction()?;

    work.execute(
        "DELETE FROM queue WHERE username = ?1 AND server = ?2",
        params![username, server],
    )?;

    for (place, row) in rows.iter().enumerate() {
        work.execute(
            "INSERT OR REPLACE INTO queue
             (username, server, place, id_item, id_pod, title, author, duration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                username,
                server,
                place as i64,
                row.id_item,
                row.id_pod,
                row.title,
                row.author,
                row.duration,
            ],
        )?;
    }

    work.commit()?;

    Ok(())
}

/// Reads the queue of one account of one server, in the sequence of the queue.
pub fn read_the_queue(username: &str, server: &str) -> Vec<QueueRow> {
    let Ok(conn) = crate::db::migrate::open_conn() else {
        error!("[read_the_queue] the program cannot open the database.");
        return Vec::new();
    };

    let mut statement = match conn.prepare(
        "SELECT id_item, id_pod, title, author, duration FROM queue
         WHERE username = ?1 AND server = ?2 ORDER BY place",
    ) {
        Ok(statement) => statement,
        Err(error) => {
            error!("[read_the_queue] {}", error);
            return Vec::new();
        }
    };

    let rows = statement.query_map(params![username, server], |row| {
        Ok(QueueRow {
            id_item: row.get(0)?,
            id_pod: row.get(1)?,
            title: row.get(2)?,
            author: row.get(3)?,
            duration: row.get(4)?,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(|row| row.ok()).collect(),
        Err(error) => {
            error!("[read_the_queue] {}", error);
            Vec::new()
        }
    }
}

/// One row of the table of the queue. See T-56.
#[derive(Debug, Clone, PartialEq)]
pub struct QueueRow {
    pub id_item: String,
    /// The identity of the episode. A book gives an empty text.
    pub id_pod: String,
    pub title: String,
    pub author: String,
    /// The length of the media, in seconds. A view that holds no length gives
    /// nothing here.
    pub duration: Option<f64>,
}
