# API Client and Multiple Server Endpoints Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the ad-hoc HTTP code in `src/api/` with one `ApiClient` that owns a pooled HTTP client, a priority-ordered endpoint list with automatic failover, and a typed error taxonomy.

**Architecture:** A pure `EndpointPool` holds endpoints sorted by priority and tracks health per endpoint. `ApiClient` wraps a single `reqwest::Client` and asks the pool for the active endpoint on each request. One private `send()` function performs selection, classification, and a single retry on the next endpoint — but only for idempotent requests. A background task probes down endpoints so the app returns to the local address automatically.

**Tech Stack:** Rust 2021, tokio, reqwest 0.11 with rustls, rusqlite, serde, wiremock (dev), GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-09-api-client-endpoints-design.md`

## Global Constraints

- Write all documentation, doc comments, and user-facing strings in ASD-STE100 Simplified Technical English. Use short sentences. Use the active voice. Use the present tense. One sentence gives one instruction.
- Do not add a dependency that needs a program the user installs separately.
- Do not add a dependency that needs a C toolchain. This is why OpenSSL goes away.
- Never send a `POST` request a second time. A second `POST` request makes a duplicate session on the server.
- `PATCH /api/me/progress/:id` sets an absolute position. Therefore it is safe to send again.
- Keep `reqwest` at version `0.11`. A major upgrade is out of scope.
- Every new public item gets a doc comment.
- Do not change the behaviour of the VLC player code. Sub-project 2 removes it.
- Run `cargo clippy -- -D warnings` before every commit.

---

## File Structure

| Path | Responsibility | Task |
|---|---|---|
| `Cargo.toml` | rustls instead of OpenSSL, add `wiremock` dev dependency | 1 |
| `.github/workflows/ci.yml` | Build, clippy, test, lint | 1, 11 |
| `src/db/migrate.rs` | Database path, shared connection, `PRAGMA user_version` runner | 2 |
| `src/api/client/error.rs` | `ApiError` and classification | 3 |
| `src/api/client/endpoint.rs` | `Endpoint`, `Health`, `EndpointPool` | 4 |
| `src/config.rs` | `ServerConfig`, `EndpointConfig`, pool construction | 5 |
| `src/api/client/mod.rs` | `ApiClient`, `send()`, typed methods | 6, 7, 8, 9 |
| `src/app.rs`, `src/logic/**` | Use `ApiClient` instead of `server_address` strings | 10 |
| `README.md`, `CONTRIBUTING.md` | Fork identity, ASD-STE100 rule | 11 |

---

### Task 1: Build tooling — rustls, test harness, continuous integration

Removes the C toolchain requirement and gives the repository its first test run. Every later task needs `cargo test` to work.

**Files:**
- Modify: `Cargo.toml`
- Create: `.github/workflows/ci.yml`
- Create: `tests/smoke.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: a working `cargo test` command, and `reqwest` built with rustls.

- [ ] **Step 1: Write the failing test**

Create `tests/smoke.rs`:

```rust
//! Smoke test. It proves that the test harness runs.

#[test]
fn the_test_harness_runs() {
    assert_eq!(2 + 2, 4);
}
```

- [ ] **Step 2: Run the test to see the harness work**

Run: `cargo test --test smoke`
Expected: PASS, `1 passed`. If the harness is broken, this fails first.

- [ ] **Step 3: Change OpenSSL to rustls in `Cargo.toml`**

Delete this line:

```toml
openssl = { version = "0.10.71", features = ["vendored"] }
```

Replace the `reqwest` line with:

```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls", "stream"] }
```

Add a `[dev-dependencies]` section at the end of the file:

```toml
[dev-dependencies]
wiremock = "0.6"
```

`#[tokio::test]` needs no extra crate. The `tokio` dependency has the `full`
feature, and that feature contains the test macro.

`default-features = false` is required. The default feature set pulls in
`native-tls`, and `native-tls` needs OpenSSL.

- [ ] **Step 4: Prove that OpenSSL is gone**

Run: `cargo tree -i openssl-sys`
Expected: the command fails, or prints nothing. If it prints a dependency
tree, a crate still needs OpenSSL. Find the crate in the output and give it
`default-features = false` too.

- [ ] **Step 5: Build and test**

Run: `cargo build && cargo test && cargo clippy -- -D warnings`
Expected: the build completes, `1 passed`, no clippy warnings.

If clippy reports warnings in code this task does not touch, do not correct
them now. Add `-A clippy::all` temporarily is **not** permitted. Instead,
record the warnings in the commit message and correct them in Task 10.

- [ ] **Step 6: Create the continuous integration workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main, offline-mode ]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install the Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache the Cargo directories
        uses: Swatinem/rust-cache@v2

      - name: Install the audio system headers
        run: sudo apt-get update && sudo apt-get install -y libasound2-dev

      - name: Build
        run: cargo build --verbose

      - name: Examine the code with clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Run the tests
        run: cargo test --verbose
```

The `libasound2-dev` step is not needed today. Sub-project 2 adds the audio
engine and needs it. The step is here so that the workflow does not change
later.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock tests/smoke.rs .github/workflows/ci.yml
git commit -m "build: change OpenSSL to rustls and add the CI workflow

The build no longer needs a C toolchain. The repository now has a test
harness and a continuous integration workflow."
```

---

### Task 2: Database migration runner

`init_db()` uses `CREATE TABLE IF NOT EXISTS`. That statement cannot change a table that exists. Task 5 needs a new column. This task adds the runner. It also extracts the database path, which almost every function in `crud.rs` repeats.

**Files:**
- Create: `src/db/migrate.rs`
- Modify: `src/db/mod.rs`
- Modify: `src/db/crud.rs` (replace `init_db`, extract `db_path`)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub fn db_path() -> std::path::PathBuf`
  - `pub fn open_conn() -> rusqlite::Result<rusqlite::Connection>`
  - `pub fn run_migrations(conn: &rusqlite::Connection) -> rusqlite::Result<()>`
  - The column `users.server_name TEXT NOT NULL DEFAULT ''`.

- [ ] **Step 1: Write the failing tests**

Create `src/db/migrate.rs` with only the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

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
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'server_name'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
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

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'server_name'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run: `cargo test --lib db::migrate`
Expected: FAIL, `cannot find function run_migrations in this scope`.

- [ ] **Step 3: Write the implementation**

Put this above the test module in `src/db/migrate.rs`:

```rust
//! Database location and schema migrations.
//!
//! The runner uses `PRAGMA user_version`. Each migration moves the schema
//! forward by one version. A migration must be safe to run on a database
//! that an older version of the program made.

use rusqlite::{Connection, Result};
use std::env;
use std::path::PathBuf;

/// The schema version that this build of the program expects.
pub const LATEST_VERSION: i64 = 2;

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

    if version < 1 {
        migrate_to_v1(conn)?;
        version = 1;
        conn.execute_batch("PRAGMA user_version = 1")?;
    }

    if version < 2 {
        migrate_to_v2(conn)?;
        conn.execute_batch("PRAGMA user_version = 2")?;
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
```

- [ ] **Step 4: Register the module**

In `src/db/mod.rs`, add:

```rust
pub mod migrate;
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run: `cargo test --lib db::migrate`
Expected: PASS, `4 passed`.

- [ ] **Step 6: Make `init_db` call the runner**

In `src/db/crud.rs`, replace the whole body of `init_db()` with:

```rust
/// Opens the database and applies the migrations.
pub fn init_db() -> Result<()> {
    let conn = Connection::open(crate::db::migrate::db_path())?;
    crate::db::migrate::run_migrations(&conn)?;
    Ok(())
}
```

Delete the four `CREATE TABLE IF NOT EXISTS` blocks from the old body. The
migration runner has them now.

- [ ] **Step 7: Remove the repeated path code from `crud.rs`**

Every other function in `crud.rs` starts with the same 12 lines that build
`config_home_path` and then `db_path`. In each function, replace those lines
and the `Connection::open(db_path)` call with one call:

```rust
let conn = crate::db::migrate::open_conn()?;
```

Some functions use `if let Ok(conn) = Connection::open(db_path)`. In those
functions use:

```rust
if let Ok(conn) = crate::db::migrate::open_conn() {
```

Do this for every function in the file. Do not change any SQL statement. Do
not change any function signature.

- [ ] **Step 8: Verify the build and the tests**

Run: `cargo build && cargo test && cargo clippy -- -D warnings`
Expected: the build completes. All tests pass. No warnings.

Also confirm the file got shorter:
Run: `wc -l src/db/crud.rs`
Expected: near 700 lines, against 1181 before.

- [ ] **Step 9: Commit**

```bash
git add src/db/migrate.rs src/db/mod.rs src/db/crud.rs
git commit -m "feat(db): add a schema migration runner

The runner uses PRAGMA user_version. Version 2 adds the column
users.server_name. The database path and the connection are now in one
place, which removes the repeated blocks in crud.rs."
```

---

### Task 3: The `ApiError` type

Today the code discards every failure with `let _ = ...`. Therefore the
program cannot tell an unavailable server from an expired token. This type
gives each failure a cause.

**Files:**
- Create: `src/api/client/error.rs`
- Create: `src/api/client/mod.rs` (module declarations only)
- Modify: `src/api/mod.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub enum ApiError { Unreachable, Timeout, Unauthorized, Forbidden, NotFound, Server(u16), Decode(String) }`
  - `pub fn classify_status(status: reqwest::StatusCode) -> Option<ApiError>`
  - `pub fn classify_transport(error: &reqwest::Error) -> ApiError`
  - `pub fn ApiError::is_endpoint_fault(&self) -> bool`

- [ ] **Step 1: Write the failing tests**

Create `src/api/client/error.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn a_success_status_is_not_an_error() {
        assert!(classify_status(StatusCode::OK).is_none());
        assert!(classify_status(StatusCode::NO_CONTENT).is_none());
    }

    #[test]
    fn client_statuses_get_their_own_category() {
        assert!(matches!(
            classify_status(StatusCode::UNAUTHORIZED),
            Some(ApiError::Unauthorized)
        ));
        assert!(matches!(
            classify_status(StatusCode::FORBIDDEN),
            Some(ApiError::Forbidden)
        ));
        assert!(matches!(
            classify_status(StatusCode::NOT_FOUND),
            Some(ApiError::NotFound)
        ));
    }

    #[test]
    fn a_server_status_keeps_its_code() {
        assert!(matches!(
            classify_status(StatusCode::BAD_GATEWAY),
            Some(ApiError::Server(502))
        ));
    }

    /// Only these categories permit a change to a different endpoint. A 404
    /// comes from the server, and a different endpoint gives the same
    /// answer.
    #[test]
    fn only_endpoint_faults_permit_a_change_of_endpoint() {
        assert!(ApiError::Unreachable.is_endpoint_fault());
        assert!(ApiError::Timeout.is_endpoint_fault());
        assert!(ApiError::Server(503).is_endpoint_fault());

        assert!(!ApiError::NotFound.is_endpoint_fault());
        assert!(!ApiError::Unauthorized.is_endpoint_fault());
        assert!(!ApiError::Forbidden.is_endpoint_fault());
        assert!(!ApiError::Decode("bad".to_string()).is_endpoint_fault());
    }
}
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run: `cargo test --lib api::client::error`
Expected: FAIL, `cannot find type ApiError in this scope`.

- [ ] **Step 3: Write the implementation**

Put this above the test module:

```rust
//! The failure categories of an API request.
//!
//! The category tells the caller what to do. `Unreachable` starts the
//! offline mode. `Unauthorized` asks the user to log in again. `Forbidden`
//! tells the user that the account has no permission.

use std::fmt;

/// The cause of a failed API request.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    /// No endpoint answered.
    Unreachable,
    /// The endpoint did not answer in the permitted time.
    Timeout,
    /// The server refused the token. The token is not valid.
    Unauthorized,
    /// The account does not have the necessary permission.
    Forbidden,
    /// The server does not have the item.
    NotFound,
    /// The server reported an internal fault. The value is the HTTP status.
    Server(u16),
    /// The answer of the server does not agree with the expected format.
    Decode(String),
}

impl ApiError {
    /// Tells if a different endpoint can give a better answer.
    ///
    /// A fault of the endpoint permits a second attempt. A fault of the
    /// request does not, because each endpoint gives the same answer.
    pub fn is_endpoint_fault(&self) -> bool {
        matches!(self, ApiError::Unreachable | ApiError::Timeout | ApiError::Server(_))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unreachable => write!(f, "No server address answered."),
            ApiError::Timeout => write!(f, "The server did not answer in time."),
            ApiError::Unauthorized => write!(f, "The token is not valid. Log in again."),
            ApiError::Forbidden => write!(f, "Your account does not have this permission."),
            ApiError::NotFound => write!(f, "The server does not have this item."),
            ApiError::Server(code) => write!(f, "The server reported a fault. Status {}.", code),
            ApiError::Decode(detail) => write!(f, "The answer of the server is not valid: {}", detail),
        }
    }
}

impl std::error::Error for ApiError {}

/// Puts an HTTP status into a category.
///
/// Gives `None` if the status shows success.
pub fn classify_status(status: reqwest::StatusCode) -> Option<ApiError> {
    if status.is_success() {
        return None;
    }

    match status.as_u16() {
        401 => Some(ApiError::Unauthorized),
        403 => Some(ApiError::Forbidden),
        404 => Some(ApiError::NotFound),
        code => Some(ApiError::Server(code)),
    }
}

/// Puts a transport fault into a category.
pub fn classify_transport(error: &reqwest::Error) -> ApiError {
    if error.is_timeout() {
        ApiError::Timeout
    } else if error.is_decode() {
        ApiError::Decode(error.to_string())
    } else {
        ApiError::Unreachable
    }
}
```

- [ ] **Step 4: Create the module files**

Create `src/api/client/mod.rs`. `endpoint` does not exist yet. Task 4 adds
it.

```rust
//! The HTTP client of the application.

pub mod error;
```

In `src/api/mod.rs`, add this line with the other module declarations:

```rust
pub mod client;
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run: `cargo test --lib api::client::error`
Expected: PASS, `4 passed`.

- [ ] **Step 6: Commit**

```bash
git add src/api/client/mod.rs src/api/client/error.rs src/api/mod.rs
git commit -m "feat(api): add the ApiError category type

Each failure now has a cause. ApiError::Unreachable is the signal for the
offline mode. Only a fault of the endpoint permits a second attempt on a
different address."
```

---

### Task 4: The `EndpointPool`

The pool holds the endpoints in priority sequence and records the health of
each one. The logic is pure. Therefore the tests need no network.

**Files:**
- Create: `src/api/client/endpoint.rs`
- Modify: `src/api/client/mod.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct Endpoint { pub url: String, pub priority: u8 }` with `Endpoint::new(url: &str, priority: u8) -> Endpoint`
  - `pub struct EndpointPool` with:
    - `EndpointPool::new(endpoints: Vec<Endpoint>) -> EndpointPool`
    - `active(&self) -> Option<String>`
    - `next_after(&self, url: &str) -> Option<String>`
    - `mark_down(&self, url: &str)`
    - `mark_up(&self, url: &str)`
    - `down_urls(&self) -> Vec<String>`
    - `len(&self) -> usize`
    - `is_empty(&self) -> bool`

- [ ] **Step 1: Write the failing tests**

Create `src/api/client/endpoint.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> EndpointPool {
        EndpointPool::new(vec![
            Endpoint::new("http://lan", 0),
            Endpoint::new("https://wan", 1),
            Endpoint::new("https://backup", 2),
        ])
    }

    #[test]
    fn the_pool_sorts_by_priority() {
        let pool = EndpointPool::new(vec![
            Endpoint::new("https://wan", 5),
            Endpoint::new("http://lan", 0),
        ]);
        assert_eq!(pool.active().unwrap(), "http://lan");
    }

    #[test]
    fn the_active_endpoint_has_the_most_importance() {
        assert_eq!(pool().active().unwrap(), "http://lan");
    }

    #[test]
    fn a_down_endpoint_is_not_active() {
        let pool = pool();
        pool.mark_down("http://lan");
        assert_eq!(pool.active().unwrap(), "https://wan");
    }

    #[test]
    fn the_pool_gives_the_next_endpoint_after_a_failure() {
        let pool = pool();
        assert_eq!(pool.next_after("http://lan").unwrap(), "https://wan");
        assert_eq!(pool.next_after("https://wan").unwrap(), "https://backup");
    }

    #[test]
    fn the_last_endpoint_has_no_next_endpoint() {
        let pool = pool();
        assert!(pool.next_after("https://backup").is_none());
    }

    #[test]
    fn next_after_does_not_give_a_down_endpoint() {
        let pool = pool();
        pool.mark_down("https://wan");
        assert_eq!(pool.next_after("http://lan").unwrap(), "https://backup");
    }

    #[test]
    fn the_active_endpoint_is_none_if_all_endpoints_are_down() {
        let pool = pool();
        pool.mark_down("http://lan");
        pool.mark_down("https://wan");
        pool.mark_down("https://backup");
        assert!(pool.active().is_none());
    }

    /// This test proves the behaviour that the user asked for. The
    /// application returns to the local address when that address works
    /// again.
    #[test]
    fn the_pool_returns_to_the_endpoint_with_more_importance() {
        let pool = pool();
        pool.mark_down("http://lan");
        assert_eq!(pool.active().unwrap(), "https://wan");

        pool.mark_up("http://lan");
        assert_eq!(pool.active().unwrap(), "http://lan");
    }

    #[test]
    fn the_pool_gives_the_down_endpoints_for_the_probe() {
        let pool = pool();
        pool.mark_down("http://lan");
        pool.mark_down("https://backup");

        let mut down = pool.down_urls();
        down.sort();
        assert_eq!(down, vec!["http://lan", "https://backup"]);
    }

    #[test]
    fn a_pool_with_one_endpoint_works() {
        let pool = EndpointPool::new(vec![Endpoint::new("http://only", 0)]);
        assert_eq!(pool.active().unwrap(), "http://only");
        assert!(pool.next_after("http://only").is_none());
    }
}
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run: `cargo test --lib api::client::endpoint`
Expected: FAIL, `cannot find type EndpointPool in this scope`.

- [ ] **Step 3: Write the implementation**

Put this above the test module:

```rust
//! The list of server addresses and the health of each address.
//!
//! One Audiobookshelf server can have more than one address. An example is a
//! fast local address and a slow public address. The pool always selects the
//! address that has the most importance and that answers.
//!
//! A low `priority` value gives more importance.

use std::sync::RwLock;

/// One address of a server.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// The base address. It has no slash at the end.
    pub url: String,
    /// A low value gives more importance.
    pub priority: u8,
}

impl Endpoint {
    /// Makes an endpoint. The function removes a slash at the end of the
    /// address, because the request path always starts with a slash.
    pub fn new(url: &str, priority: u8) -> Self {
        Endpoint {
            url: url.trim_end_matches('/').to_string(),
            priority,
        }
    }
}

/// The health of one address.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Health {
    /// The address answered the last request.
    Up,
    /// The address did not answer. The probe task examines it again.
    Down,
}

/// The addresses of one server, in priority sequence.
#[derive(Debug)]
pub struct EndpointPool {
    endpoints: Vec<Endpoint>,
    health: RwLock<Vec<Health>>,
}

impl EndpointPool {
    /// Makes a pool. The function sorts the endpoints by priority. The
    /// endpoint with the lowest value comes first. All endpoints start with
    /// the state `Up`.
    pub fn new(mut endpoints: Vec<Endpoint>) -> Self {
        endpoints.sort_by_key(|endpoint| endpoint.priority);
        let health = vec![Health::Up; endpoints.len()];

        EndpointPool {
            endpoints,
            health: RwLock::new(health),
        }
    }

    /// Gives the address that has the most importance and the state `Up`.
    ///
    /// Gives `None` if no address has the state `Up`. The caller then
    /// reports `ApiError::Unreachable`.
    pub fn active(&self) -> Option<String> {
        let health = self.health.read().ok()?;

        self.endpoints
            .iter()
            .zip(health.iter())
            .find(|(_, state)| **state == Health::Up)
            .map(|(endpoint, _)| endpoint.url.clone())
    }

    /// Gives the next address that has the state `Up` after the given
    /// address.
    ///
    /// The client uses this function for the second attempt.
    pub fn next_after(&self, url: &str) -> Option<String> {
        let health = self.health.read().ok()?;
        let position = self.endpoints.iter().position(|e| e.url == url)?;

        self.endpoints
            .iter()
            .zip(health.iter())
            .skip(position + 1)
            .find(|(_, state)| **state == Health::Up)
            .map(|(endpoint, _)| endpoint.url.clone())
    }

    /// Records that an address does not answer.
    pub fn mark_down(&self, url: &str) {
        self.set_health(url, Health::Down);
    }

    /// Records that an address answers again.
    pub fn mark_up(&self, url: &str) {
        self.set_health(url, Health::Up);
    }

    /// Gives the addresses that have the state `Down`. The probe task
    /// examines these addresses.
    pub fn down_urls(&self) -> Vec<String> {
        let health = match self.health.read() {
            Ok(health) => health,
            Err(_) => return Vec::new(),
        };

        self.endpoints
            .iter()
            .zip(health.iter())
            .filter(|(_, state)| **state == Health::Down)
            .map(|(endpoint, _)| endpoint.url.clone())
            .collect()
    }

    /// Gives the number of addresses.
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Tells if the pool has no address.
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    fn set_health(&self, url: &str, state: Health) {
        if let Some(position) = self.endpoints.iter().position(|e| e.url == url) {
            if let Ok(mut health) = self.health.write() {
                health[position] = state;
            }
        }
    }
}
```

- [ ] **Step 4: Register the module**

In `src/api/client/mod.rs`, add:

```rust
pub mod endpoint;
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run: `cargo test --lib api::client::endpoint`
Expected: PASS, `10 passed`.

- [ ] **Step 6: Commit**

```bash
git add src/api/client/endpoint.rs src/api/client/mod.rs
git commit -m "feat(api): add the EndpointPool with priority selection

The pool always gives the address that has the most importance and that
answers. It returns to a better address when that address works again."
```

---

### Task 5: Server configuration and the old single address

The configuration file gets the `[[servers]]` block. An installation that
exists has no such block. That installation must continue to work.

**Files:**
- Modify: `src/config.rs`
- Modify: `config.example.toml`

**Interfaces:**
- Consumes: `Endpoint`, `EndpointPool` from Task 4.
- Produces:
  - `pub struct EndpointConfig { pub url: String, pub priority: u8 }`
  - `pub struct ServerConfig { pub name: String, pub endpoints: Vec<EndpointConfig> }`
  - `ConfigFile.servers: Vec<ServerConfig>`
  - `pub fn pool_for_address(servers: &[ServerConfig], stored_address: &str) -> EndpointPool`
  - `pub fn server_name_for_address(servers: &[ServerConfig], stored_address: &str) -> Option<String>`

- [ ] **Step 1: Write the failing tests**

Add this test module at the end of `src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn servers() -> Vec<ServerConfig> {
        vec![ServerConfig {
            name: "home".to_string(),
            endpoints: vec![
                EndpointConfig {
                    url: "http://192.168.1.10:13378".to_string(),
                    priority: 0,
                },
                EndpointConfig {
                    url: "https://abs.example.com".to_string(),
                    priority: 1,
                },
            ],
        }]
    }

    /// The user logged in with the public address. The pool must still
    /// contain both addresses, and the local address must come first.
    #[test]
    fn a_known_address_gives_the_full_pool() {
        let pool = pool_for_address(&servers(), "https://abs.example.com");
        assert_eq!(pool.len(), 2);
        assert_eq!(pool.active().unwrap(), "http://192.168.1.10:13378");
    }

    #[test]
    fn a_known_address_gives_the_name_of_the_server() {
        let name = server_name_for_address(&servers(), "http://192.168.1.10:13378");
        assert_eq!(name.unwrap(), "home");
    }

    /// This is the behaviour for an installation that exists. The
    /// configuration file has no `[[servers]]` block.
    #[test]
    fn an_unknown_address_gives_a_pool_with_one_endpoint() {
        let pool = pool_for_address(&[], "https://other.example.com");
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.active().unwrap(), "https://other.example.com");
    }

    #[test]
    fn an_unknown_address_gives_no_server_name() {
        assert!(server_name_for_address(&servers(), "https://other.example.com").is_none());
    }

    /// A slash at the end must not stop the comparison.
    #[test]
    fn a_slash_at_the_end_does_not_change_the_result() {
        let pool = pool_for_address(&servers(), "https://abs.example.com/");
        assert_eq!(pool.len(), 2);
    }
}
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run: `cargo test --lib config`
Expected: FAIL, `cannot find type ServerConfig in this scope`.

- [ ] **Step 3: Write the implementation**

In `src/config.rs`, add these imports at the top:

```rust
use crate::api::client::endpoint::{Endpoint, EndpointPool};
```

Add these structures after the `Player` structure:

```rust
/// One address of a server, from the configuration file.
#[derive(Debug, Deserialize, Clone)]
pub struct EndpointConfig {
    /// The base address of the server.
    pub url: String,
    /// A low value gives more importance. The default value is 0.
    #[serde(default)]
    pub priority: u8,
}

/// One Audiobookshelf server that has one address or more.
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// The identity of the server. The address is not the identity.
    pub name: String,
    /// The addresses of this server.
    pub endpoints: Vec<EndpointConfig>,
}
```

Add this field to `ConfigFile`:

```rust
pub servers: Vec<ServerConfig>,
```

In `load_config()`, add this line before the `Ok(ConfigFile { ... })` line:

```rust
    // A configuration file that an older version made has no `servers`
    // block. An empty list is correct in that condition.
    let servers: Vec<ServerConfig> = config.get("servers").unwrap_or_default();
```

Change the return value to:

```rust
    Ok(ConfigFile { colors, player, servers })
```

Add these two functions at the end of the file, above the test module:

```rust
/// Removes a slash at the end of an address, for a comparison.
fn normalise(url: &str) -> &str {
    url.trim_end_matches('/')
}

/// Makes the endpoint pool for a user.
///
/// The function looks for the stored address in the configured servers. If it
/// finds the address, the pool gets all addresses of that server. If it does
/// not find the address, the pool gets the stored address only. Therefore an
/// installation that has no `[[servers]]` block continues to work.
pub fn pool_for_address(servers: &[ServerConfig], stored_address: &str) -> EndpointPool {
    let target = normalise(stored_address);

    for server in servers {
        let is_match = server
            .endpoints
            .iter()
            .any(|endpoint| normalise(&endpoint.url) == target);

        if is_match {
            let endpoints = server
                .endpoints
                .iter()
                .map(|endpoint| Endpoint::new(&endpoint.url, endpoint.priority))
                .collect();

            return EndpointPool::new(endpoints);
        }
    }

    EndpointPool::new(vec![Endpoint::new(stored_address, 0)])
}

/// Gives the name of the server that has the stored address.
///
/// Gives `None` if no configured server has the address.
pub fn server_name_for_address(servers: &[ServerConfig], stored_address: &str) -> Option<String> {
    let target = normalise(stored_address);

    servers
        .iter()
        .find(|server| {
            server
                .endpoints
                .iter()
                .any(|endpoint| normalise(&endpoint.url) == target)
        })
        .map(|server| server.name.clone())
}
```

- [ ] **Step 4: Run the tests to verify that they pass**

Run: `cargo test --lib config`
Expected: PASS, `5 passed`.

- [ ] **Step 5: Add the example to the configuration file**

Add this block at the top of `config.example.toml`:

```toml
#### SERVERS ####
# One Audiobookshelf server can have more than one address. An example is a
# fast local address at home and a public address away from home.
#
# The application always uses the address that has the most importance and
# that answers. A low `priority` value gives more importance.
#
# The application examines an address that does not answer every 60 seconds.
# Therefore it returns to the local address automatically.
#
# This block is optional. If you do not add it, the application uses the
# address that you gave at the login screen.
#
# [[servers]]
# name = "home"
# endpoints = [
#   { url = "http://192.168.1.10:13378", priority = 0 },
#   { url = "https://abs.example.com",   priority = 1 },
# ]

```

- [ ] **Step 6: Verify the build**

Run: `cargo build && cargo test && cargo clippy -- -D warnings`
Expected: the build completes. All tests pass. No warnings.

- [ ] **Step 7: Commit**

```bash
git add src/config.rs config.example.toml
git commit -m "feat(config): read more than one address for a server

The [[servers]] block gives an address list with a priority. An
installation that has no such block keeps the address from the login
screen."
```

---

### Task 6: The `ApiClient` request core

This is the centre of the design. One private function does the selection,
the classification, and the second attempt.

**Files:**
- Modify: `src/api/client/mod.rs`
- Create: `tests/api_client.rs`

**Interfaces:**
- Consumes: `EndpointPool` (Task 4), `ApiError`, `classify_status`, `classify_transport` (Task 3).
- Produces:
  - `pub enum Idempotent { Yes, No }`
  - `pub struct ApiClient`
  - `ApiClient::new(pool: Arc<EndpointPool>, token: String) -> Result<ApiClient, ApiError>`
  - `ApiClient::pool(&self) -> Arc<EndpointPool>`
  - `ApiClient::send(&self, method: reqwest::Method, path: &str, body: Option<serde_json::Value>, idempotent: Idempotent) -> Result<reqwest::Response, ApiError>`
  - `ApiClient::get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError>`
  - `ApiClient::patch_json<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError>`
  - `ApiClient::post_json<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T, ApiError>`

- [ ] **Step 1: Write the failing tests**

Create `tests/api_client.rs`:

```rust
//! Tests of the request core. The tests use a mock server, because the
//! behaviour depends on real HTTP answers.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::error::ApiError;
use toutui::api::client::ApiClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(urls: Vec<&str>) -> ApiClient {
    let endpoints = urls
        .iter()
        .enumerate()
        .map(|(index, url)| Endpoint::new(url, index as u8))
        .collect();

    ApiClient::new(Arc::new(EndpointPool::new(endpoints)), "test-token".to_string()).unwrap()
}

#[tokio::test]
async fn the_client_reads_json_from_the_first_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "value": 42
        })))
        .mount(&server)
        .await;

    let client = client(vec![&server.uri()]);
    let body: serde_json::Value = client.get_json("/api/libraries").await.unwrap();

    assert_eq!(body["value"], 42);
}

/// The first address refuses the connection. The client must use the second
/// address and give the answer.
#[tokio::test]
async fn the_client_changes_to_the_second_endpoint() {
    let good = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .mount(&good)
        .await;

    // Port 1 is not open. Therefore the connection fails immediately.
    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    let body: serde_json::Value = client.get_json("/api/libraries").await.unwrap();

    assert_eq!(body["ok"], true);
}

/// After a failure, the pool must record that the address does not answer.
#[tokio::test]
async fn a_failure_marks_the_endpoint_down() {
    let good = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .mount(&good)
        .await;

    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    let _: serde_json::Value = client.get_json("/api/libraries").await.unwrap();

    assert_eq!(client.pool().down_urls(), vec!["http://127.0.0.1:1"]);
}

/// This is the most important test of the task. A second POST request makes
/// a duplicate listening session on the server.
#[tokio::test]
async fn the_client_does_not_send_a_post_request_a_second_time() {
    let good = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/items/abc/play"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": "s1" })))
        .expect(0)
        .mount(&good)
        .await;

    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    let result: Result<serde_json::Value, ApiError> = client
        .post_json("/api/items/abc/play", &serde_json::json!({}))
        .await;

    assert!(matches!(result, Err(ApiError::Unreachable)));
    // The mock has `expect(0)`. The check happens when the server stops.
    drop(good);
}

#[tokio::test]
async fn all_endpoints_down_gives_unreachable() {
    let client = client(vec!["http://127.0.0.1:1", "http://127.0.0.1:2"]);
    let result: Result<serde_json::Value, ApiError> = client.get_json("/api/libraries").await;

    assert!(matches!(result, Err(ApiError::Unreachable)));
}

#[tokio::test]
async fn a_status_401_gives_unauthorized_and_does_not_change_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(vec![&server.uri(), "http://127.0.0.1:1"]);
    let result: Result<serde_json::Value, ApiError> = client.get_json("/api/libraries").await;

    assert!(matches!(result, Err(ApiError::Unauthorized)));
}

#[tokio::test]
async fn a_status_403_gives_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let client = client(vec![&server.uri()]);
    let result: Result<serde_json::Value, ApiError> = client.get_json("/api/items/abc/download").await;

    assert!(matches!(result, Err(ApiError::Forbidden)));
}

#[tokio::test]
async fn the_client_sends_the_token() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/libraries"))
        .and(wiremock::matchers::header(
            "authorization",
            "Bearer test-token",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(vec![&server.uri()]);
    let body: serde_json::Value = client.get_json("/api/libraries").await.unwrap();
    assert_eq!(body["ok"], true);
}
```

- [ ] **Step 2: Make the crate available to the integration test**

An integration test in `tests/` needs a library target. `src/main.rs` is a
binary only.

`app` and `login_app` must go into the library too. `src/ui/tui.rs`,
`src/ui/login_tui.rs`, `src/logic/search/search_active.rs`, and
`src/logic/auth/auth_input.rs` all use `crate::app::AppView` or
`crate::login_app::AppLogin`. If `app` stays in the binary only, the library
does not compile.

Create `src/lib.rs`:

```rust
//! The library target of Toutui. It exists so that the integration tests in
//! `tests/` can use the modules of the application.

pub mod api;
pub mod app;
pub mod config;
pub mod db;
pub mod login_app;
pub mod logic;
pub mod player;
pub mod ui;
pub mod utils;
```

In `src/main.rs`, replace all nine module declarations at the top:

```rust
mod login_app;
mod app;
mod config;
mod api;
mod ui;
mod player;
mod logic;
mod db;
mod utils;
```

with these two lines:

```rust
use toutui::{api, app, config, db, login_app, logic, player, ui, utils};
```

Then delete the `use toutui::...` names that `main.rs` does not use, because
clippy reports an unused import as a warning.

Every `crate::` path inside `src/app.rs`, `src/ui/`, `src/logic/`,
`src/player/`, `src/db/`, and `src/utils/` stays correct. Those files are now
in the library, and `crate` means the library there. Only `src/main.rs`
changes.

- [ ] **Step 3: Run the tests to verify that they fail**

Run: `cargo test --test api_client`
Expected: FAIL, `cannot find struct ApiClient`.

- [ ] **Step 4: Write the implementation**

Replace `src/api/client/mod.rs` with:

```rust
//! The HTTP client of the application.
//!
//! The client owns one `reqwest::Client`. Therefore the program uses
//! connection pooling. The client asks the endpoint pool for an address
//! before each request. If the address does not answer, and if the request
//! is idempotent, the client sends the request one time to the next address.

pub mod endpoint;
pub mod error;

use endpoint::EndpointPool;
use error::{classify_status, classify_transport, ApiError};
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// The time to wait for a connection. A short time makes the program detect
/// an address that does not answer quickly.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// The time to wait for a full answer. Downloads do not use this value.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Tells if the client can send a request a second time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Idempotent {
    /// The request does not change data, or it sets an absolute value.
    Yes,
    /// A second request makes a duplicate. An example is a session request.
    No,
}

/// The HTTP client of the application.
pub struct ApiClient {
    http: reqwest::Client,
    pool: Arc<EndpointPool>,
    token: String,
}

// The token is a secret. Therefore the debug output does not show it.
impl fmt::Debug for ApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiClient")
            .field("pool", &self.pool)
            .field("token", &"<hidden>")
            .finish()
    }
}

impl ApiClient {
    /// Makes a client.
    ///
    /// The token must be the decrypted token. Give the token one time. The
    /// client does not decrypt a token for each request.
    pub fn new(pool: Arc<EndpointPool>, token: String) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| classify_transport(&error))?;

        Ok(ApiClient { http, pool, token })
    }

    /// Gives the endpoint pool. The probe task and the tests use it.
    pub fn pool(&self) -> Arc<EndpointPool> {
        Arc::clone(&self.pool)
    }

    /// Sends a request. This function has all the failover logic.
    ///
    /// The `path` value must start with a slash.
    pub async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
        idempotent: Idempotent,
    ) -> Result<Response, ApiError> {
        let first = self.pool.active().ok_or(ApiError::Unreachable)?;

        let first_error = match self.attempt(&first, method.clone(), path, body.clone()).await {
            Ok(response) => return Ok(response),
            Err(error) => error,
        };

        // A fault of the request gives the same answer on each address.
        if !first_error.is_endpoint_fault() {
            return Err(first_error);
        }

        self.pool.mark_down(&first);

        if idempotent == Idempotent::No {
            return Err(first_error);
        }

        let second = match self.pool.next_after(&first) {
            Some(url) => url,
            None => return Err(ApiError::Unreachable),
        };

        match self.attempt(&second, method, path, body).await {
            Ok(response) => Ok(response),
            Err(error) => {
                if error.is_endpoint_fault() {
                    self.pool.mark_down(&second);
                }
                Err(error)
            }
        }
    }

    /// Sends one request to one address.
    async fn attempt(
        &self,
        base_url: &str,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response, ApiError> {
        let mut request = self
            .http
            .request(method, format!("{}{}", base_url, path))
            .bearer_auth(&self.token);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|e| classify_transport(&e))?;

        match classify_status(response.status()) {
            Some(error) => Err(error),
            None => Ok(response),
        }
    }

    /// Reads JSON with a `GET` request. A `GET` request is idempotent.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = self.send(Method::GET, path, None, Idempotent::Yes).await?;

        response
            .json::<T>()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))
    }

    /// Sends a `PATCH` request. The Audiobookshelf progress endpoint sets an
    /// absolute position. Therefore this request is idempotent.
    pub async fn patch_json<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        self.send(Method::PATCH, path, Some(value), Idempotent::Yes)
            .await?;

        Ok(())
    }

    /// Sends a `POST` request and reads the answer.
    ///
    /// The client never sends this request a second time. A second request
    /// makes a duplicate listening session on the server.
    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        let response = self
            .send(Method::POST, path, Some(value), Idempotent::No)
            .await?;

        response
            .json::<T>()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))
    }

    /// Sends a `POST` request that has no answer body.
    pub async fn post_no_content<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError> {
        let value =
            serde_json::to_value(body).map_err(|error| ApiError::Decode(error.to_string()))?;

        self.send(Method::POST, path, Some(value), Idempotent::No)
            .await?;

        Ok(())
    }
}
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run: `cargo test --test api_client`
Expected: PASS, `8 passed`.

If `the_client_does_not_send_a_post_request_a_second_time` fails, the
`Idempotent::No` branch is wrong. That branch must give the error before it
calls `next_after`.

- [ ] **Step 6: Run all the tests**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: all tests pass. No warnings.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs src/main.rs src/api/client/mod.rs tests/api_client.rs
git commit -m "feat(api): add the ApiClient request core

One function does the address selection, the classification of the answer,
and one second attempt. A POST request never goes a second time, because a
duplicate request makes a duplicate session on the server.

The client now has a connect timeout of 3 seconds. Before this change an
address that did not answer stopped the program for about two minutes."
```

---

### Task 7: The health probe task

The pool marks an address `Down` after a failure. Without a probe, that
address stays `Down` for ever. This task adds the background probe.

**Files:**
- Create: `src/api/client/probe.rs`
- Modify: `src/api/client/mod.rs`
- Modify: `tests/api_client.rs`

**Interfaces:**
- Consumes: `ApiClient`, `EndpointPool`.
- Produces:
  - `pub const PROBE_INTERVAL: Duration`
  - `pub async fn probe_once(http: &reqwest::Client, pool: &EndpointPool)`
  - `pub fn spawn_probe_task(client: Arc<ApiClient>) -> tokio::task::JoinHandle<()>`
  - `ApiClient::http(&self) -> &reqwest::Client`

- [ ] **Step 1: Write the failing test**

Add to `tests/api_client.rs`:

```rust
/// The probe must make an address active again after the address answers.
#[tokio::test]
async fn the_probe_makes_a_down_endpoint_active_again() {
    use toutui::api::client::probe::probe_once;

    let primary = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&primary)
        .await;

    let client = client(vec![&primary.uri(), "http://127.0.0.1:1"]);

    client.pool().mark_down(&primary.uri());
    assert_eq!(client.pool().active().unwrap(), "http://127.0.0.1:1");

    probe_once(client.http(), &client.pool()).await;

    assert_eq!(
        client.pool().active().unwrap(),
        primary.uri().trim_end_matches('/')
    );
}

/// The probe must not make an address active if the address does not answer.
#[tokio::test]
async fn the_probe_keeps_a_dead_endpoint_down() {
    use toutui::api::client::probe::probe_once;

    let good = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&good)
        .await;

    let client = client(vec!["http://127.0.0.1:1", &good.uri()]);
    client.pool().mark_down("http://127.0.0.1:1");

    probe_once(client.http(), &client.pool()).await;

    assert_eq!(client.pool().down_urls(), vec!["http://127.0.0.1:1"]);
}
```

- [ ] **Step 2: Run the tests to verify that they fail**

Run: `cargo test --test api_client the_probe`
Expected: FAIL, `could not find probe in client`.

- [ ] **Step 3: Write the implementation**

Create `src/api/client/probe.rs`:

```rust
//! The background task that examines the addresses that do not answer.
//!
//! Without this task, an address stays in the state `Down` for ever. With
//! this task, the application returns to the address that has the most
//! importance automatically. An example is the local address when the user
//! comes home.

use super::endpoint::EndpointPool;
use super::ApiClient;
use log::info;
use std::sync::Arc;
use std::time::Duration;

/// The time between two probes.
pub const PROBE_INTERVAL: Duration = Duration::from_secs(60);

/// Examines each address that has the state `Down` one time.
///
/// The function sends `GET /ping` to each such address. Audiobookshelf
/// answers this path without a token. If an address answers, the function
/// gives it the state `Up`.
pub async fn probe_once(http: &reqwest::Client, pool: &EndpointPool) {
    for url in pool.down_urls() {
        let is_up = http
            .get(format!("{}/ping", url))
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false);

        if is_up {
            info!("[probe] The address {} answers again.", url);
            pool.mark_up(&url);
        }
    }
}

/// Starts the probe task. The task runs until the program stops.
pub fn spawn_probe_task(client: Arc<ApiClient>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let pool = client.pool();

        loop {
            tokio::time::sleep(PROBE_INTERVAL).await;
            probe_once(client.http(), &pool).await;
        }
    })
}
```

- [ ] **Step 4: Give access to the HTTP client**

In `src/api/client/mod.rs`, add `pub mod probe;` with the other module
declarations. Add this method to `impl ApiClient`:

```rust
    /// Gives the HTTP client. The probe task uses it.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
```

- [ ] **Step 5: Run the tests to verify that they pass**

Run: `cargo test --test api_client`
Expected: PASS, `10 passed`.

- [ ] **Step 6: Commit**

```bash
git add src/api/client/probe.rs src/api/client/mod.rs tests/api_client.rs
git commit -m "feat(api): add the endpoint health probe

The task examines each address that does not answer every 60 seconds. The
application returns to the address that has the most importance
automatically."
```

---

### Task 8: Typed methods for the read requests and the progress requests

These are the `GET` and `PATCH` calls. All of them are idempotent.

**Files:**
- Modify: `src/api/libraries/get_all_libraries.rs`
- Modify: `src/api/libraries/get_all_books.rs`
- Modify: `src/api/libraries/get_library_perso_view.rs`
- Modify: `src/api/libraries/get_library_perso_view_pod.rs`
- Modify: `src/api/me/get_media_progress.rs`
- Modify: `src/api/me/update_media_progress.rs`
- Modify: `src/api/library_items/get_pod_ep.rs`

**Interfaces:**
- Consumes: `ApiClient::get_json`, `ApiClient::patch_json`.
- Produces (each keeps its `Root` type and its name, and loses the
  `server_address` and `token` parameters):
  - `get_all_libraries(client: &ApiClient) -> Result<Root, ApiError>`
  - `get_all_books(client: &ApiClient, id_selected_lib: &str) -> Result<Root, ApiError>`
  - `get_continue_listening(client: &ApiClient, id_selected_lib: &str) -> Result<Vec<Root>, ApiError>`
  - `get_continue_listening_pod(client: &ApiClient, id_selected_lib: &str) -> Result<Vec<Root>, ApiError>`
  - `get_book_progress(client: &ApiClient, book_id: &str) -> Result<Root, ApiError>`
  - `get_pod_ep(client: &ApiClient, id: &str) -> Result<Root, ApiError>`
  - `update_media_progress_book(client: &ApiClient, id_library_item: &str, current_time: Option<u32>, duration: &str) -> Result<(), ApiError>`
  - `update_media_progress2_book(client: &ApiClient, id_library_item: &str, current_time: Option<u32>, duration: &str, is_finished: bool) -> Result<(), ApiError>`
  - `update_media_progress_pod(client: &ApiClient, id_library_item: &str, current_time: Option<u32>, duration: &str, ep_id: &str) -> Result<(), ApiError>`
  - `update_media_progress2_pod(client: &ApiClient, id_library_item: &str, current_time: Option<u32>, duration: &str, is_finished: bool, ep_id: &str) -> Result<(), ApiError>`

- [ ] **Step 1: Convert one function and see the pattern**

In `src/api/libraries/get_all_libraries.rs`, replace the function body. Keep
every `struct` in the file without a change.

Before:

```rust
pub async fn get_all_libraries(token: &str, server_address: String) -> Result<Root> {
```

After:

```rust
use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;

/// Gets all libraries of the server.
///
/// See <https://api.audiobookshelf.org/#get-all-libraries>.
pub async fn get_all_libraries(client: &ApiClient) -> Result<Root, ApiError> {
    client.get_json("/api/libraries").await
}
```

Delete the `reqwest::Client`, the `AUTHORIZATION` header, and the manual
`response.json()` call. `get_json` does all of that.

- [ ] **Step 2: Verify that the file compiles**

Run: `cargo build 2>&1 | head -40`
Expected: errors in the call sites only, not in `get_all_libraries.rs`. Task
10 corrects the call sites. Errors in other files are correct at this point.

- [ ] **Step 3: Convert the other read functions**

Use the same pattern. The paths are:

| Function | Path |
|---|---|
| `get_all_books` | `/api/libraries/{id_selected_lib}/items` |
| `get_continue_listening` | `/api/libraries/{id_selected_lib}/personalized` |
| `get_continue_listening_pod` | `/api/libraries/{id_selected_lib}/personalized` |
| `get_book_progress` | `/api/me/progress/{book_id}` |
| `get_pod_ep` | `/api/items/{id}` |

Read the current URL in each file before you change it. Use the exact path
and the exact query string that the file has now. Do not invent a path.

`get_continue_listening` and `get_continue_listening_pod` give
`Result<Vec<Root>>`. Keep that return type. The body becomes:

```rust
pub async fn get_continue_listening(
    client: &ApiClient,
    id_selected_lib: &str,
) -> Result<Vec<Root>, ApiError> {
    client
        .get_json(&format!("/api/libraries/{}/personalized", id_selected_lib))
        .await
}
```

- [ ] **Step 4: Convert the progress functions**

In `src/api/me/update_media_progress.rs`, all four functions follow one
shape. Read the current body of each function to get the exact JSON keys.
The shape is:

```rust
use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::json;

/// Sends the listening position of a book to the server.
///
/// The server sets an absolute position. Therefore the client can send this
/// request a second time without a risk.
pub async fn update_media_progress_book(
    client: &ApiClient,
    id_library_item: &str,
    current_time: Option<u32>,
    duration: &str,
) -> Result<(), ApiError> {
    let current_time = current_time.unwrap_or(0);
    let duration_value: f64 = duration.parse().unwrap_or(0.0);
    let progress = if duration_value > 0.0 {
        current_time as f64 / duration_value
    } else {
        0.0
    };

    let body = json!({
        "duration": duration_value,
        "currentTime": current_time,
        "progress": progress,
    });

    client
        .patch_json(&format!("/api/me/progress/{}", id_library_item), &body)
        .await
}
```

Compare the JSON keys with the current file. If the current file sends a
different set of keys, use the keys of the current file. Do not change the
data that the program sends to the server.

The `2` variants add `"isFinished": is_finished`. The `pod` variants use the
path `/api/me/progress/{id_library_item}/{ep_id}`.

- [ ] **Step 5: Verify that the API files compile**

Run: `cargo build 2>&1 | grep -c "^error"`
Expected: a number greater than zero, and every error names a file in
`src/logic/`, `src/app.rs`, or `src/main.rs`. No error names a file in
`src/api/`.

Run this to confirm:
Run: `cargo build 2>&1 | grep "^error" -A2 | grep "src/api/" | wc -l`
Expected: `0`

- [ ] **Step 6: Commit**

```bash
git add src/api/libraries src/api/me src/api/library_items/get_pod_ep.rs
git commit -m "refactor(api): move the read requests to the ApiClient

Each function is now a thin layer on the client. The failover logic is in
one place. The call sites do not compile yet. Task 10 corrects them."
```

---

### Task 9: Typed methods for the session requests and the download

These requests are not idempotent, or they need a different timeout.

**Files:**
- Modify: `src/api/library_items/play_lib_item_or_pod.rs`
- Modify: `src/api/sessions/sync_open_session.rs`
- Modify: `src/api/sessions/close_open_session.rs`
- Modify: `src/api/library_items/download_item.rs`
- Modify: `src/api/client/mod.rs` (add the download method)

**Interfaces:**
- Consumes: `ApiClient::post_json`, `ApiClient::post_no_content`.
- Produces:
  - `post_start_playback_session_book(client: &ApiClient, id_library_item: &str) -> Result<Vec<String>, ApiError>`
  - `post_start_playback_session_pod(client: &ApiClient, id_library_item: &str, pod_ep_id: &str) -> Result<Vec<String>, ApiError>`
  - `sync_session(client: &ApiClient, session_id: &str, current_time: Option<u32>, time_listened: u32) -> Result<(), ApiError>`
  - `close_session_without_send_prg_data(client: &ApiClient, session_id: &str) -> Result<(), ApiError>`
  - `ApiClient::download_to_file(&self, path: &str, dest_dir: &Path, fallback_filename: &str) -> Result<PathBuf, ApiError>`
  - `download_library_item_file(client: &ApiClient, id_library_item: &str, dest_dir: &Path, fallback_filename: &str) -> Result<PathBuf, ApiError>`

- [ ] **Step 1: Write the failing test for the download timeout**

Add to `tests/api_client.rs`:

```rust
/// A large audiobook takes more than the normal request timeout. The
/// download must not stop after 15 seconds.
#[tokio::test]
async fn the_download_ignores_the_normal_request_timeout() {
    use std::time::Duration;

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"audio-data".to_vec())
                // This delay is longer than a short test timeout, and it
                // proves that the download path uses its own timeout.
                .set_delay(Duration::from_millis(300)),
        )
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let client = client(vec![&server.uri()]);

    let file = client
        .download_to_file("/api/items/abc/download", dir.path(), "abc.m4b")
        .await
        .unwrap();

    assert_eq!(std::fs::read(&file).unwrap(), b"audio-data");
}

/// A user without the download permission must get a clear category.
#[tokio::test]
async fn a_download_without_permission_gives_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/items/abc/download"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let client = client(vec![&server.uri()]);

    let result = client
        .download_to_file("/api/items/abc/download", dir.path(), "abc.m4b")
        .await;

    assert!(matches!(result, Err(ApiError::Forbidden)));
}
```

Add `tempfile = "3"` to `[dev-dependencies]` in `Cargo.toml`.

- [ ] **Step 2: Run the tests to verify that they fail**

Run: `cargo test --test api_client the_download`
Expected: FAIL, `no method named download_to_file`.

- [ ] **Step 3: Add the download method**

In `src/api/client/mod.rs`, add these imports:

```rust
use reqwest::header::CONTENT_DISPOSITION;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
```

Add this method to `impl ApiClient`:

```rust
    /// Downloads a file to a directory.
    ///
    /// This method does not use the normal request timeout. An audiobook can
    /// be some gigabytes, and the download can take many minutes. The
    /// connect timeout is still active. Therefore an address that does not
    /// answer still fails quickly.
    ///
    /// The method takes the file name from the `Content-Disposition` header.
    /// If the header has no name, the method uses `fallback_filename`.
    pub async fn download_to_file(
        &self,
        path: &str,
        dest_dir: &Path,
        fallback_filename: &str,
    ) -> Result<PathBuf, ApiError> {
        let base_url = self.pool.active().ok_or(ApiError::Unreachable)?;

        let response = self
            .http
            .get(format!("{}{}", base_url, path))
            .bearer_auth(&self.token)
            .timeout(Duration::from_secs(60 * 60 * 6))
            .send()
            .await
            .map_err(|error| {
                let error = classify_transport(&error);
                if error.is_endpoint_fault() {
                    self.pool.mark_down(&base_url);
                }
                error
            })?;

        if let Some(error) = classify_status(response.status()) {
            return Err(error);
        }

        let filename = response
            .headers()
            .get(CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split("filename=").nth(1))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| fallback_filename.to_string());

        tokio::fs::create_dir_all(dest_dir)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        let dest_path = dest_dir.join(&filename);

        let bytes = response
            .bytes()
            .await
            .map_err(|error| classify_transport(&error))?;

        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        file.write_all(&bytes)
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        Ok(dest_path)
    }
```

The download is not idempotent in cost. Therefore it does not change to a
different address. It marks the address `Down` and gives the error. The user
starts the download again.

- [ ] **Step 4: Run the tests to verify that they pass**

Run: `cargo test --test api_client the_download`
Expected: PASS, `2 passed`.

- [ ] **Step 5: Convert `download_item.rs`**

Replace the function with:

```rust
use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use std::path::{Path, PathBuf};

/// Downloads a library item for offline listening.
///
/// The account must have the "download" permission. Without the permission
/// the function gives `ApiError::Forbidden`.
///
/// See <https://api.audiobookshelf.org/#download-a-library-item>.
pub async fn download_library_item_file(
    client: &ApiClient,
    id_library_item: &str,
    dest_dir: &Path,
    fallback_filename: &str,
) -> Result<PathBuf, ApiError> {
    client
        .download_to_file(
            &format!("/api/items/{}/download", id_library_item),
            dest_dir,
            fallback_filename,
        )
        .await
}
```

- [ ] **Step 6: Convert the session functions**

In `src/api/library_items/play_lib_item_or_pod.rs`, keep the `Vec<String>`
return value and the fields that the current code reads from the answer.
Read the current body first. The new shape is:

```rust
use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde_json::json;

/// Starts a listening session for a book.
///
/// The client never sends this request a second time. A second request makes
/// a duplicate session on the server.
///
/// See <https://api.audiobookshelf.org/#play-a-library-item-or-podcast-episode>.
pub async fn post_start_playback_session_book(
    client: &ApiClient,
    id_library_item: &str,
) -> Result<Vec<String>, ApiError> {
    let body = json!({
        "deviceInfo": { "clientName": "Toutui" },
        "forceDirectPlay": true,
        "mediaPlayer": "vlc",
    });

    let answer: serde_json::Value = client
        .post_json(&format!("/api/items/{}/play", id_library_item), &body)
        .await?;

    // Keep the same sequence of values that the caller expects today.
    Ok(collect_session_fields(&answer))
}
```

Copy the exact body JSON and the exact field extraction from the current
file into `collect_session_fields`. Do not change the values that go to the
server, and do not change the sequence of the returned `Vec<String>`. The
callers read that vector by index.

Do the same for `post_start_playback_session_pod`, with the path
`/api/items/{id_library_item}/play/{pod_ep_id}`.

For `sync_open_session.rs` and `close_open_session.rs`, use
`post_no_content`. The paths are `/api/session/{session_id}/sync` and
`/api/session/{session_id}/close`.

- [ ] **Step 7: Verify that no file in `src/api/` has an error**

Run: `cargo build 2>&1 | grep "src/api/" | wc -l`
Expected: `0`

- [ ] **Step 8: Commit**

```bash
git add src/api Cargo.toml Cargo.lock tests/api_client.rs
git commit -m "refactor(api): move the session and download requests to the client

A POST session request never goes a second time. The download uses its own
long timeout, because an audiobook can be some gigabytes. A user without
the download permission now gets a clear message."
```

---

### Task 10: Connect the application to the client

The API layer compiles. The application does not. This task builds one
`ApiClient` at start and gives it to the code that needs it.

**Files:**
- Modify: `src/main.rs`
- Modify: `src/app.rs`
- Modify: `src/logic/handle_input/*.rs`
- Modify: `src/logic/sync_session/sync_session_from_database.rs`
- Modify: `src/logic/download.rs`
- Modify: `src/api/server/auth_process.rs`
- Modify: `src/db/crud.rs` (write `server_name`)

**Interfaces:**
- Consumes: everything from Tasks 3 to 9.
- Produces: `Arc<ApiClient>` in `App`, and no `server_address: String`
  parameter in the application code.

- [ ] **Step 1: Build the client at start**

`_database.default_usr` is a flat `Vec<String>`. `select_default_usr` in
`src/db/crud.rs` fills it in this sequence: index 0 is the user name, index 1
is the server address, index 2 is the encrypted token.

In `src/main.rs`, inside the `if _database_ready {` block, replace these
lines:

```rust
        // init current username
        let mut username: String = String::new();
        if let Some(var_username) = _database.default_usr.get(0) {
            username = var_username.clone();
        }
```

with:

```rust
        // The default user gives the name, the address, and the token.
        let username = _database.default_usr.first().cloned().unwrap_or_default();
        let server_address = _database
            .default_usr
            .get(1)
            .cloned()
            .unwrap_or_default();
        let encrypted_token = _database
            .default_usr
            .get(2)
            .cloned()
            .unwrap_or_default();

        // Build one HTTP client for the whole program. The client owns the
        // addresses of the server and the token. The program decrypts the
        // token one time here, not one time for each request.
        let config_file = config::load_config()?;
        let token = utils::encrypt_token::decrypt_token(&encrypted_token).unwrap_or_default();

        let pool = std::sync::Arc::new(config::pool_for_address(
            &config_file.servers,
            &server_address,
        ));

        let api_client = std::sync::Arc::new(
            api::client::ApiClient::new(pool, token).expect("Unable to build the HTTP client"),
        );

        // The probe task returns the program to the address that has the
        // most importance.
        let _probe = api::client::probe::spawn_probe_task(std::sync::Arc::clone(&api_client));
```

`username` is no longer `mut`. The rest of the block uses
`username.as_str()` and does not change it.

`App::new()` needs the client. Change the two calls in this file from
`App::new().await?` to `App::new(std::sync::Arc::clone(&api_client)).await?`.
There are two calls. One is before the loop. One is in the `KeyCode::Char('R')`
branch.

- [ ] **Step 2: Give the client to `App`**

In `src/app.rs`, add a field:

```rust
    /// The HTTP client of the server. It has all the addresses and the token.
    pub api: std::sync::Arc<crate::api::client::ApiClient>,
```

Change the signature of the constructor:

```rust
    pub async fn new(api: std::sync::Arc<crate::api::client::ApiClient>) -> Result<Self> {
```

Set the field from the parameter. Then delete the fields and the local
variables that hold `server_address` and the decrypted token, because the
client has them now.

- [ ] **Step 3: Correct each call site**

Run: `cargo build 2>&1 | grep "^error" -A3 | head -60`

Correct the errors one file at a time. The change is always the same shape:

```rust
// Before
let _ = update_media_progress_book(id, token.as_ref(), Some(time), &duration, server_address.clone()).await;

// After
if let Err(error) = update_media_progress_book(&app.api, id, Some(time), &duration).await {
    log::warn!("[progress] The server did not accept the position: {}", error);
}
```

Do not write `let _ = ...` for a new call. Record the error in the log. This
is the defect that section 3 of the specification describes.

For a function that does not have `app` in scope, add a parameter:

```rust
api: std::sync::Arc<crate::api::client::ApiClient>,
```

Then delete the `server_address: String` parameter and the
`token: Option<String>` parameter of that function.

- [ ] **Step 4: Correct `auth_process`**

`auth_process` runs before a client exists, because it gets the token. Keep
its own `reqwest::Client`, but add the timeouts:

```rust
    let client = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(3))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
```

Then, after the login succeeds, write the name of the server. Add this
before the `User` value:

```rust
    let config_file = crate::config::load_config().ok();
    let server_name = config_file
        .as_ref()
        .and_then(|file| {
            crate::config::server_name_for_address(&file.servers, server_address)
        })
        .unwrap_or_default();
```

Add `server_name` to the `User` structure in
`src/db/database_struct.rs`, and add the column to the `INSERT` statement in
`db_insert_usr` in `src/db/crud.rs`. The column exists after Task 2.

**Warning about the sequence of the columns.** `select_default_usr` gives a
flat `Vec<String>`. The callers read that vector by index. Step 1 of this
task reads index 0, index 1, and index 2. If you put `server_name` in the
middle of the `SELECT` list, every index after it changes, and the program
reads the wrong value.

Therefore put `server_name` at the end of the `SELECT` list, and push it last
into the result vector. Then no index changes.

- [ ] **Step 5: Verify the build**

Run: `cargo build && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: the build completes. All tests pass. No warnings.

- [ ] **Step 6: Test the program by hand**

Run: `cargo run`

Do these checks:

1. Log in with a valid address. The library appears.
2. Play a book. The player starts.
3. Stop the program. Start it again. The position is correct.
4. Add a `[[servers]]` block that has a wrong address with `priority = 0` and
   your real address with `priority = 1`. Start the program. The library
   must appear in about 3 seconds, not in 2 minutes.
5. Look in the log file. It must contain a line from the probe task, or a
   line that reports the address that does not answer.

Correct any fault before you continue.

- [ ] **Step 7: Commit**

```bash
git add src
git commit -m "refactor: give the ApiClient to the application

The program builds one client at start. The server address string is no
longer a parameter of the application functions. Each failed request now
writes a line in the log with the cause."
```

---

### Task 11: The ASD-STE100 linter and the fork identity

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Create: `docs/style.md`

**Interfaces:**
- Consumes: the workflow from Task 1.
- Produces: a lint step, and the writing rule for contributors.

- [ ] **Step 1: Examine the linter on your computer first**

Do not add a command to the workflow before it works on your computer.

Run:

```bash
go install github.com/stuffbucket/vale@latest
```

Then run the tool against one file:

```bash
"$(go env GOPATH)/bin/vale" README.md
```

Write down the exact command, the exact binary name, and the exact
arguments. If the module path is wrong, look at
<https://github.com/stuffbucket/vale> and use the path in that README.

If the tool does not work in 15 minutes, stop. Use this alternative: add the
lint step with `continue-on-error: true` and open an issue. Do not stop the
whole task.

- [ ] **Step 2: Add the lint step to the workflow**

Add this job to `.github/workflows/ci.yml`. Use the exact command from Step
1 in the `run` line:

```yaml
  documentation-style:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Go
        uses: actions/setup-go@v5
        with:
          go-version: stable

      - name: Install the Simplified Technical English linter
        run: go install github.com/stuffbucket/vale@latest

      - name: Examine the documentation
        run: |
          "$(go env GOPATH)/bin/vale" \
            README.md \
            CONTRIBUTING.md \
            docs/style.md \
            docs/superpowers/specs/*.md
```

- [ ] **Step 3: Write the style document**

Create `docs/style.md`:

```markdown
# The writing style of this project

This project writes all documentation, all doc comments, and all messages to
the user in ASD-STE100 Simplified Technical English.

## The rules

1. Write short sentences. A sentence that gives an instruction has 20 words
   or less. A sentence that describes something has 25 words or less.
2. Use the active voice. Write "The client sends the request". Do not write
   "The request is sent".
3. Use the present tense.
4. One sentence gives one instruction.
5. Use one word for one meaning. Do not use two words for the same thing.
6. Write the article. Write "the client", not "client".
7. Do not use a noun group of more than three words.
8. Start a paragraph with the topic of the paragraph.

## Why

The people who read this project do not all have English as a first
language. Simple text is faster to read and gives fewer mistakes. The style
also makes a code review easier, because two people cannot read one sentence
in two ways.

## What the tools examine

The continuous integration workflow examines the Markdown files. A person
examines the Rust doc comments in the code review. Use the same rules in
both.
```

- [ ] **Step 4: Add the rule to the contribution guide**

Add this section to `CONTRIBUTING.md`:

```markdown
## The writing style

Write all documentation, all doc comments, and all messages to the user in
ASD-STE100 Simplified Technical English. See [docs/style.md](docs/style.md).
The continuous integration workflow examines the Markdown files.
```

- [ ] **Step 5: Correct the README**

Replace the first line of `README.md`:

```markdown
## ⚠️ I'm not able to properly maintain this project anymore. That's why I archived this repo. ...
```

with:

```markdown
> **This is a maintained fork.** The original project at
> [AlbanDAVID/Toutui](https://github.com/AlbanDAVID/Toutui) is archived. This
> fork continues the work. Thank you to the original author.
```

Add this section after the feature list:

```markdown
## More than one server address

One Audiobookshelf server can have more than one address. An example is a
fast address on your local network and a public address for other locations.

Add a `[[servers]]` block to your configuration file:

    [[servers]]
    name = "home"
    endpoints = [
      { url = "http://192.168.1.10:13378", priority = 0 },
      { url = "https://abs.example.com",   priority = 1 },
    ]

A low `priority` value gives more importance. The application always uses the
address that has the most importance and that answers. If an address does not
answer, the application changes to the next address. The application examines
the addresses that do not answer every 60 seconds. Therefore it returns to
your local address automatically when you come home.

This block is optional. Without it, the application uses the address that you
gave at the login screen.
```

- [ ] **Step 6: Run the linter on your computer**

Run the exact command from Step 1 against each file that Step 2 names.
Correct each report. A report of a word that the tool does not know, and that
is correct for this project (an example is "Audiobookshelf"), goes in the
vocabulary file of the tool.

- [ ] **Step 7: Commit**

```bash
git add .github/workflows/ci.yml README.md CONTRIBUTING.md docs/style.md
git commit -m "docs: add the ASD-STE100 style rule and the fork identity

The workflow examines the Markdown files with a Simplified Technical
English linter. The README describes the fork and the new [[servers]]
block."
```

---

## Verification of the whole plan

After Task 11, run these commands. Each one must succeed.

```bash
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo tree -i openssl-sys    # must find nothing
```

Then do this test by hand:

1. Put a wrong address at `priority = 0` and your real address at
   `priority = 1`.
2. Start the program. The library must appear in about 3 seconds.
3. Look in the log file. It must show that the program changed to the second
   address.
4. Correct the first address. Wait 60 seconds. The log must show that the
   probe found the address again.

---

## Self-review notes

**Spec coverage:**

| Specification section | Task |
|---|---|
| 4.1 Configuration schema | 5 |
| 4.2 EndpointPool | 4 |
| 4.3 Retry rule | 6, 9 |
| 4.4 ApiClient | 6 |
| 4.5 Error taxonomy | 3 |
| 4.6 Migration runner | 2 |
| 4.7 Cleanup of crud.rs | 2 |
| 6 Backwards compatibility | 5 |
| 7.1 Unit tests | 2, 3, 4, 5 |
| 7.2 Integration tests | 6, 7, 9 |
| 7.3 Continuous integration | 1, 11 |
| 9 Dependency policy (rustls) | 1 |

The health probe of section 4.2 gets its own task, Task 7, because it needs
the client from Task 6.
