//! Database location and schema migrations.
//!
//! The runner uses `PRAGMA user_version`. Each migration moves the schema
//! forward by one version. A migration must be safe to run on a database
//! that an older version of the program made.

use rusqlite::{Connection, Result};
use std::path::PathBuf;

/// The schema version that this build of the program expects.
pub const LATEST_VERSION: i64 = 9;

/// Gives the full path of the database file.
pub fn db_path() -> PathBuf {
    crate::paths::db_file()
}

/// Opens the database file and applies the migrations.
pub fn open_conn() -> Result<Connection> {
    let conn = Connection::open(db_path())?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Reads the current schema version.
pub fn schema_version(conn: &Connection) -> Result<i64> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
}

/// Applies each migration that the database does not have.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let mut version = schema_version(conn)?;

    // The database has the latest schema. No migration is necessary.
    if version >= LATEST_VERSION {
        return Ok(());
    }

    if version < 1 {
        migrate_to_v1(conn)?;
        version = 1;
        conn.execute_batch("PRAGMA user_version = 1")?;
    }

    if version < 2 {
        migrate_to_v2(conn)?;
        version = 2;
        conn.execute_batch("PRAGMA user_version = 2")?;
    }

    if version < 3 {
        migrate_to_v3(conn)?;
        version = 3;
        conn.execute_batch("PRAGMA user_version = 3")?;
    }

    if version < 4 {
        migrate_to_v4(conn)?;
        version = 4;
        conn.execute_batch("PRAGMA user_version = 4")?;
    }

    if version < 5 {
        migrate_to_v5(conn)?;
        conn.execute_batch("PRAGMA user_version = 5")?;
    }

    if version < 6 {
        migrate_to_v6(conn)?;
        version = 6;
        conn.execute_batch("PRAGMA user_version = 6")?;
    }

    if version < 7 {
        migrate_to_v7(conn)?;
        version = 7;
        conn.execute_batch("PRAGMA user_version = 7")?;
    }

    if version < 8 {
        migrate_to_v8(conn)?;
        version = 8;
        conn.execute_batch("PRAGMA user_version = 8")?;
    }

    if version < 9 {
        migrate_to_v9(conn)?;
        conn.execute_batch("PRAGMA user_version = 9")?;
    }

    Ok(())
}

/// Version 9 gives the listening session the program that owns it. See T-140.
///
/// **One row stood for one account, and a user starts the program in two
/// terminals.** The two programs of that account then shared one row: the
/// playback of the second program closed the live session of the first one, the
/// key `Q` of the first program sent the position of the book of the second one,
/// and the key `Q` of the second program found no row at all. The place of that
/// user reached no server.
///
/// `owner` holds the program, and `heartbeat` holds the moment of the last
/// second of that playback. A program takes its own row, and a row that no
/// program touched for `THE_LIMIT_OF_THE_HEARTBEAT` seconds: **that is the row of
/// a program that stopped without a correct exit**, and the rule of T-4 keeps it.
///
/// **A row that an older program wrote holds no owner**, and its heartbeat is 0.
/// Such a row is therefore old, and the program that asks takes it: that is the
/// same answer that version 8 gives to a row with no account.
///
/// **The migration must be safe to run two times**, as the rule of the head of
/// this file says.
fn migrate_to_v9(conn: &Connection) -> Result<()> {
    if !has_table(conn, "listening_session")? {
        return Ok(());
    }

    if !has_column_in(conn, "listening_session", "owner")? {
        conn.execute(
            "ALTER TABLE listening_session ADD COLUMN owner TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    if !has_column_in(conn, "listening_session", "heartbeat")? {
        conn.execute(
            "ALTER TABLE listening_session ADD COLUMN heartbeat INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

/// Version 8 gives the listening session an account and a server. See T-138.
///
/// **The row of that table held no account at all**, and one row stands for the
/// whole program. A user with two accounts therefore lost the place of a media:
/// the program of the second account read the row of the first one, it sent that
/// position to **its own** server, the server answered "The server does not have
/// this item", and the program then removed the row. The place of the user went
/// away, and no line of the screen said it.
///
/// The rule of the queue of version 7 holds here now: the account and the server
/// keep the rows apart.
///
/// **A row that an older program wrote holds no account.** The two columns are
/// empty for such a row, and the program gives that row to the account that
/// asks: a database of an older version holds the row of the one account that
/// program had.
///
/// **The migration must be safe to run two times**, as the rule of the head of
/// this file says: a database that has the two columns already must not stop the
/// runner.
fn migrate_to_v8(conn: &Connection) -> Result<()> {
    if !has_table(conn, "listening_session")? {
        return Ok(());
    }

    for column in ["username", "server"] {
        if has_column_in(conn, "listening_session", column)? {
            continue;
        }

        conn.execute(
            &format!(
                "ALTER TABLE listening_session ADD COLUMN {} TEXT NOT NULL DEFAULT ''",
                column
            ),
            [],
        )?;
    }

    Ok(())
}

/// Version 7 adds the queue of the media. See T-56.
///
/// The queue lived in the memory of the process, therefore a user who stopped
/// the program lost it. The server holds no queue: Audiobookshelf keeps its own
/// queue inside the web page and it gives it to no client.
///
/// One row is one media that waits. `place` holds the sequence, and the account
/// and the server hold the queue apart: a user with an account on two servers
/// keeps one queue for each of them.
fn migrate_to_v7(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS queue (
            username   TEXT    NOT NULL,
            server     TEXT    NOT NULL DEFAULT '',
            place      INTEGER NOT NULL,
            id_item    TEXT    NOT NULL,
            id_pod     TEXT    NOT NULL DEFAULT '',
            title      TEXT    NOT NULL DEFAULT '',
            author     TEXT    NOT NULL DEFAULT '',
            duration   REAL,
            PRIMARY KEY (username, server, id_item, id_pod)
        );",
    )
}

/// Version 6 adds the sequence and the filter of the library. See T-24.
///
/// `GET /api/libraries/:id/items` takes `sort`, `desc`, and `filter`. The
/// choice of the user must stay after the program stops, therefore it belongs
/// to the account and not to the process.
///
/// An empty value means "the sequence of the server, and no filter". Every
/// account of an older database gets that value, thus the program behaves as
/// it did before this version.
fn migrate_to_v6(conn: &Connection) -> Result<()> {
    if !has_table(conn, "users")? {
        return Ok(());
    }

    for column in ["library_sort", "library_desc", "library_filter"] {
        if has_column_in(conn, "users", column)? {
            continue;
        }

        conn.execute(
            &format!(
                "ALTER TABLE users ADD COLUMN {} TEXT NOT NULL DEFAULT ''",
                column
            ),
            [],
        )?;
    }

    Ok(())
}

/// Version 5 adds the table of the progress that waits for the server.
///
/// The application plays a local copy when the server does not answer. The
/// position then goes in this table. The application sends each row when the
/// server answers again, and it removes the row. See T-25.
///
/// The column `server` holds the identity of the server. A user can have an
/// account on more than one server, and a position must go to the server that
/// holds the media. One server can have many addresses, thus the identity is
/// the name of the server, and not one address.
///
/// The column `id_pod` holds the identity of the episode. A book has an empty
/// value. The column `position_s` holds the position in seconds; the name
/// `current_time` is a keyword of SQLite, and a query then gives the time of
/// the day. The column `updated_at` holds the time of the local computer in
/// milliseconds. The application compares that value with `lastUpdate` of the
/// server, thus a newer position of a different client stays.
fn migrate_to_v5(conn: &Connection) -> Result<()> {
    // The table `downloads` holds one row for each download. The key of an
    // episode is the identity of the episode, and the server needs the
    // identity of the podcast also. Therefore the row holds both.
    // A database that has no `downloads` table needs no change. The version 1
    // makes that table for every new database.
    if has_table(conn, "downloads")? && !has_column_in(conn, "downloads", "item_id")? {
        conn.execute(
            "ALTER TABLE downloads ADD COLUMN item_id TEXT NOT NULL DEFAULT ''",
            [],
        )?;

        // A row of an older version is a book. A book has one identity.
        conn.execute(
            "UPDATE downloads SET item_id = id_item WHERE item_id = ''",
            [],
        )?;
    }

    // A user can have an account on more than one server. The row must
    // therefore name its server. A row of an older version has an empty value,
    // and the application accepts that row for every server.
    if has_table(conn, "downloads")? && !has_column_in(conn, "downloads", "server")? {
        conn.execute(
            "ALTER TABLE downloads ADD COLUMN server TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pending_progress (
            id_item      TEXT    NOT NULL,
            username     TEXT    NOT NULL,
            server       TEXT    NOT NULL DEFAULT '',
            id_pod       TEXT    NOT NULL DEFAULT '',
            position_s   REAL    NOT NULL,
            duration     REAL    NOT NULL,
            is_finished  INTEGER NOT NULL DEFAULT 0,
            updated_at   INTEGER NOT NULL,
            PRIMARY KEY (id_item, username, server, id_pod)
        );",
    )
}

/// Version 1 records the schema that the program had before the migration
/// runner. Each statement is safe on a database that has the tables.
fn migrate_to_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            server_address TEXT NOT NULL,
            token TEXT NOT NULL,
            is_default_usr INTEGER NOT NULL DEFAULT 0,
            name_selected_lib TEXT NOT NULL,
            id_selected_lib TEXT NOT NULL,
            is_loop_break TEXT NOT NULL,
            is_vlc_launched_first_time TEXT NOT NULL,
            speed_rate FLOAT NOT NULL,
            is_vlc_running TEXT NOT NULL,
            is_show_key_bindings TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS listening_session (
            id_session TEXT PRIMARY KEY,
            id_item TEXT NOT NULL,
            current_time_playback INTEGER NOT NULL,
            duration TEXT NOT NULL,
            is_finished INTEGER NOT NULL DEFAULT 0,
            id_pod TEXT NOT NULL,
            elapsed_time INTEGER NOT NULL,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            is_playback INTEGER NOT NULL DEFAULT 1,
            chapter TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS others (
            login_err TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS downloads (
            id_item TEXT NOT NULL,
            username TEXT NOT NULL,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            file_path TEXT NOT NULL,
            duration REAL NOT NULL DEFAULT 0,
            current_time_offline INTEGER NOT NULL DEFAULT 0,
            downloaded_at TEXT NOT NULL,
            PRIMARY KEY (id_item, username)
        );",
    )
}

/// Version 2 adds the name of the configured server to each user. The name
/// connects a user to a `[[servers]]` block in the configuration file.
fn migrate_to_v2(conn: &Connection) -> Result<()> {
    let has_column: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'server_name'",
        [],
        |row| row.get(0),
    )?;

    if has_column == 0 {
        conn.execute(
            "ALTER TABLE users ADD COLUMN server_name TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    Ok(())
}

/// Version 3 adds the table of the downloaded audio files. A book can have
/// more than one audio file. The table `downloads` holds one path only.
fn migrate_to_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS download_files (
            id_item   TEXT    NOT NULL,
            username  TEXT    NOT NULL,
            idx       INTEGER NOT NULL,
            ino       TEXT    NOT NULL,
            file_path TEXT    NOT NULL,
            size      INTEGER NOT NULL,
            duration  REAL    NOT NULL,
            PRIMARY KEY (id_item, username, idx)
        );",
    )
}

/// Version 4 removes VLC from the schema.
///
/// The application decodes the audio in the process now. The column
/// `is_vlc_running` has no use. The column `is_vlc_launched_first_time` keeps
/// its use, but its name mentions VLC. Therefore the migration changes the
/// name to `has_played_before`.
///
/// SQLite gives `DROP COLUMN` from version 3.35.0 and `RENAME COLUMN` from
/// version 3.25.0. The crate `rusqlite` has the feature `bundled`, thus the
/// version is newer than both. If a statement fails, the migration writes a
/// message and continues. A column that stays does no damage.
fn migrate_to_v4(conn: &Connection) -> Result<()> {
    if has_column(conn, "is_vlc_launched_first_time")? && !has_column(conn, "has_played_before")? {
        if let Err(error) = conn.execute(
            "ALTER TABLE users RENAME COLUMN is_vlc_launched_first_time TO has_played_before",
            [],
        ) {
            log::warn!("[migrate] the database keeps the old name: {}", error);
        }
    }

    // A database that has neither name needs the column.
    if !has_column(conn, "has_played_before")? {
        if let Err(error) = conn.execute(
            "ALTER TABLE users ADD COLUMN has_played_before TEXT NOT NULL DEFAULT '0'",
            [],
        ) {
            log::warn!("[migrate] the database has no has_played_before: {}", error);
        }
    }

    if has_column(conn, "is_vlc_running")? {
        if let Err(error) = conn.execute("ALTER TABLE users DROP COLUMN is_vlc_running", []) {
            log::warn!("[migrate] the database keeps is_vlc_running: {}", error);
        }
    }

    Ok(())
}

/// Tells if the table `users` has a column.
/// Tells if the database has the given table.
fn has_table(conn: &Connection, table: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [table],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

/// Tells if the given table has the given column.
fn has_column_in(conn: &Connection, table: &str, name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
        [table, name],
        |row| row.get(0),
    )?;

    Ok(count == 1)
}

fn has_column(conn: &Connection, name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = ?1",
        [name],
        |row| row.get(0),
    )?;

    Ok(count == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Counts the columns that have the given name in the given table.
    fn column_count(conn: &Connection, table: &str, column: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
            rusqlite::params![table, column],
            |row| row.get(0),
        )
        .unwrap()
    }

    /// Counts the tables that have the given name.
    fn table_count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            rusqlite::params![table],
            |row| row.get(0),
        )
        .unwrap()
    }

    /// A new database gets the latest schema version.
    #[test]
    fn migrations_set_the_latest_version() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
    }

    /// A new database gets the `server_name` column.
    #[test]
    fn migrations_add_the_server_name_column() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        assert_eq!(column_count(&conn, "users", "server_name"), 1);
    }

    /// A new database gets the `download_files` table.
    #[test]
    fn migrations_add_the_download_files_table() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        assert_eq!(table_count(&conn, "download_files"), 1);
    }

    /// A new database gets the `pending_progress` table.
    #[test]
    fn migrations_add_the_pending_progress_table() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        assert_eq!(table_count(&conn, "pending_progress"), 1);
    }

    /// The version 5 adds a column to the table `downloads`. A database that
    /// has no such table must not stop the runner.
    #[test]
    fn migration_v5_accepts_a_database_with_no_downloads_table() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 4").unwrap();

        run_migrations(&conn).unwrap();

        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
        assert_eq!(table_count(&conn, "pending_progress"), 1);
    }

    /// The columns of the sequence and of the filter come to a database of
    /// an older version, and every account keeps its other values.
    #[test]
    fn migration_v6_adds_the_sequence_and_the_filter() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO users (username, server_address, token, is_default_usr,
                 name_selected_lib, id_selected_lib, is_loop_break, speed_rate,
                 is_show_key_bindings, has_played_before)
             VALUES ('a', 'http://x', 't', 1, 'Books', 'lib-1', '0', 1.0, '1', '0')",
            [],
        )
        .unwrap();

        for column in ["library_sort", "library_desc", "library_filter"] {
            assert!(
                has_column_in(&conn, "users", column).unwrap(),
                "the column {} must exist",
                column
            );
        }

        // An empty value means "the sequence of the server, and no filter".
        let sort: String = conn
            .query_row(
                "SELECT library_sort FROM users WHERE username = 'a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(sort, "");
    }

    /// The session of a database of an older version keeps its position, and it
    /// gets the two columns of the account. See T-138.
    #[test]
    fn migration_v8_gives_the_session_an_account() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        for column in ["username", "server"] {
            assert!(
                has_column_in(&conn, "listening_session", column).unwrap(),
                "the column {} of the session must exist",
                column
            );
        }

        // A row of a program of an older version holds no account. The columns
        // are empty for such a row, therefore the account that reads it takes
        // it: a database of an older version holds the row of the one account
        // that program had.
        conn.execute(
            "INSERT INTO listening_session (id_session, id_item, current_time_playback,
                 duration, is_finished, id_pod, elapsed_time, title, author, is_playback, chapter)
             VALUES ('a-session', 'a-book', 810, '1800', 0, '', 0, 'A Book', 'An Author', 1, '')",
            [],
        )
        .unwrap();

        let (username, server): (String, String) = conn
            .query_row(
                "SELECT username, server FROM listening_session WHERE id_session = 'a-session'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(username, "");
        assert_eq!(server, "");
    }

    /// The runner must not add a column two times.
    #[test]
    fn migration_v6_runs_two_times_with_no_fault() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute_batch("PRAGMA user_version = 5").unwrap();
        run_migrations(&conn).unwrap();

        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
    }

    /// A row of an older version is a book. The migration gives it the
    /// identity of the item.
    #[test]
    fn migration_v5_gives_a_book_its_own_identity() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 4").unwrap();
        conn.execute(
            "CREATE TABLE downloads (
                id_item TEXT NOT NULL,
                username TEXT NOT NULL,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                file_path TEXT NOT NULL,
                duration REAL NOT NULL DEFAULT 0,
                current_time_offline INTEGER NOT NULL DEFAULT 0,
                downloaded_at TEXT NOT NULL,
                PRIMARY KEY (id_item, username)
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO downloads VALUES ('book-1', 'bob', 'A', 'B', '/a', 1.0, 0, 'now')",
            [],
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        let value: String = conn
            .query_row(
                "SELECT item_id FROM downloads WHERE id_item = 'book-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(value, "book-1");
    }

    /// The runner does not fail if it runs two times.
    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
    }

    /// An old database has the tables but no version. The runner must not
    /// fail, and it must add the new column.
    #[test]
    fn migrations_upgrade_an_old_database() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE users (
                username TEXT PRIMARY KEY,
                server_address TEXT NOT NULL,
                token TEXT NOT NULL,
                is_default_usr INTEGER NOT NULL DEFAULT 0,
                name_selected_lib TEXT NOT NULL,
                id_selected_lib TEXT NOT NULL,
                is_loop_break TEXT NOT NULL,
                is_vlc_launched_first_time TEXT NOT NULL,
                speed_rate FLOAT NOT NULL,
                is_vlc_running TEXT NOT NULL,
                is_show_key_bindings TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        assert_eq!(schema_version(&conn).unwrap(), LATEST_VERSION);
        assert_eq!(column_count(&conn, "users", "server_name"), 1);
    }
    /// Migration v4 removes VLC from the schema of a new database.
    #[test]
    fn migration_v4_removes_vlc_from_a_new_database() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        assert!(!has_column(&conn, "is_vlc_running").unwrap());
        assert!(!has_column(&conn, "is_vlc_launched_first_time").unwrap());
        assert!(has_column(&conn, "has_played_before").unwrap());
    }

    /// A database that an older version made has the two columns of VLC. The
    /// runner must change the name of one column and remove the other. It
    /// must also keep the value of the column that it renames.
    #[test]
    fn migration_v4_upgrades_a_database_that_has_the_vlc_columns() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 3").unwrap();
        conn.execute(
            "CREATE TABLE users (
                username TEXT PRIMARY KEY,
                server_address TEXT NOT NULL,
                token TEXT NOT NULL,
                is_vlc_running TEXT NOT NULL DEFAULT '0',
                is_vlc_launched_first_time TEXT NOT NULL DEFAULT '0'
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO users VALUES ('bob', 'http://a', 'tok', '1', '1')",
            [],
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        assert!(!has_column(&conn, "is_vlc_running").unwrap());
        assert!(!has_column(&conn, "is_vlc_launched_first_time").unwrap());
        assert!(has_column(&conn, "has_played_before").unwrap());

        // The value of the user must survive the change of the name.
        let value: String = conn
            .query_row(
                "SELECT has_played_before FROM users WHERE username = 'bob'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "1");
    }
}
