//! Database location and schema migrations.
//!
//! The runner uses `PRAGMA user_version`. Each migration moves the schema
//! forward by one version. A migration must be safe to run on a database
//! that an older version of the program made.

use rusqlite::{Connection, Result};
use std::env;
use std::path::PathBuf;

/// The schema version that this build of the program expects.
pub const LATEST_VERSION: i64 = 4;

/// Gives the full path of the database file.
///
/// The path follows `XDG_CONFIG_HOME` if that variable is set. If it is not
/// set, the path is `~/.config` on Linux and `~/Library/Preferences` on
/// macOS.
pub fn db_path() -> PathBuf {
    let config_home = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::home_dir().expect("Unable to find the user's home directory");
            if cfg!(target_os = "macos") {
                path.push("Library/Preferences");
            } else {
                path.push(".config");
            }
            path
        });

    config_home.join("toutui/db.sqlite3")
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
        conn.execute_batch("PRAGMA user_version = 4")?;
    }

    Ok(())
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
