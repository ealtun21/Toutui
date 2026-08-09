# Design: API client and multiple server endpoints

Date: 2026-08-09
Status: Approved
Sub-project: 1 of 5

## 1. Purpose

This design replaces the loose HTTP code in `src/api/` with one client type.
The client owns the HTTP connection pool, the server endpoints, and the token.
It selects an endpoint by priority. It changes to a different endpoint when the
first endpoint does not answer.

This is the first sub-project of the fork. The other sub-projects need this
work. The audio engine needs it to stream media. The offline mode needs it to
detect that the server is not available.

## 2. Scope

### 2.1 In scope

- A `ServerConfig` structure that reads endpoints from `config.toml`.
- An `EndpointPool` type that selects an endpoint and monitors its health.
- An `ApiClient` type that sends all HTTP requests.
- An `ApiError` type that gives a cause for each failure.
- A database migration runner.
- The first unit tests and integration tests in the repository.
- A continuous integration workflow.
- A change from OpenSSL to rustls.

### 2.2 Out of scope

These items belong to later sub-projects:

- The audio engine and the removal of VLC (sub-project 2).
- The removal of `unwrap()` calls in the user interface code (sub-project 3).
- The EPUB reader (sub-project 4).
- A full refactor of `src/db/crud.rs`.

## 3. Background

The code has these problems today:

1. Each API function calls `reqwest::Client::new()`. The client is not shared.
   Therefore the program does not use connection pooling.
2. No request has a timeout. If a server address is not available, the request
   waits for the operating system default. This time is about two minutes.
3. The server address is a `String`. The code sends this string through
   approximately 40 call sites.
4. The code discards all errors with `let _ = ...`. Therefore the program cannot
   tell the difference between an unavailable server, an expired token, and bad
   data.
5. `src/db/crud.rs` has 1181 lines. Almost all functions repeat the same 12 lines
   that find the database path and open a connection.
6. The repository has no tests.

## 4. Design

### 4.1 Configuration schema

`config.toml` gets a new repeatable block:

```toml
[[servers]]
name = "home"
endpoints = [
  { url = "http://192.168.1.10:13378", priority = 0 },
  { url = "https://abs.example.com",   priority = 1 },
]
```

A low `priority` value has more importance than a high value. The `name` field
is the identity of the server. The address is not the identity. Therefore the
user can change an address and keep the data.

### 4.2 EndpointPool

`EndpointPool` holds the endpoints in priority sequence. It also holds the index
of the active endpoint.

```rust
pub struct EndpointPool {
    endpoints: Vec<Endpoint>,   // sorted by priority, lowest value first
    active:    AtomicUsize,
}

struct Endpoint {
    url:      Url,
    priority: u8,
    health:   RwLock<Health>,
}

enum Health {
    Up,
    Down { since: Instant },
}
```

The pool obeys these rules:

- `active()` gives the endpoint with the most importance that has the state
  `Up`.
- A background task sends `GET /ping` every 60 seconds. It sends this request
  only to the endpoints that have more importance than the active endpoint. If
  an endpoint answers, the pool makes it active.
- If a request fails with a transport error, the pool sets that endpoint to
  `Down`. The client then sends the request again to the next endpoint.

### 4.3 Retry rule

The client sends a request again only if the request is idempotent.

| Method and path | Retry | Reason |
|---|---|---|
| All `GET` requests | Yes | A `GET` request does not change data. |
| `PATCH /api/me/progress/:id` | Yes | The request sets an absolute position. |
| `POST` session requests | No | A second request makes a second session. |

If a request is not idempotent, the client sets the endpoint to `Down` and gives
an error. The client does not send the request again.

### 4.4 ApiClient

```rust
pub struct ApiClient {
    http:  reqwest::Client,   // connect_timeout 3 s, timeout 15 s, pooled
    pool:  Arc<EndpointPool>,
    token: SecretString,      // decrypted one time at construction
}
```

One private function `request(method, path, body)` does this sequence:

1. It gets the active endpoint from the pool.
2. It sends the request.
3. It puts the result into an `ApiError` category.
4. If the rules in section 4.3 permit it, it sends the request again to the next
   endpoint.

Each typed method is a thin layer on this function. Therefore the failover logic
is in one place only.

Downloads use a different path. This path removes the total timeout and uses a
read timeout. A large M4B file must not stop after 15 seconds.

The `connect_timeout` value of 3 seconds is important. It makes the program
detect an unavailable server quickly.

### 4.5 Error taxonomy

```rust
pub enum ApiError {
    Unreachable,               // no endpoint answered
    Timeout,
    Unauthorized,              // 401: the token is not valid
    Forbidden,                 // 403: the account has no permission
    NotFound,
    Server(StatusCode),
    Decode(serde_json::Error),
}
```

`ApiError::Unreachable` is the signal for offline mode. Sub-project 2 uses this
signal. `ApiError::Forbidden` lets the download function tell the user that the
account does not have the "download" permission.

### 4.6 Database migration runner

The code uses `CREATE TABLE IF NOT EXISTS` today. This statement cannot change a
table that exists. Therefore this design adds a migration runner in
`src/db/migrate.rs`. The runner uses `PRAGMA user_version`.

- Migration v1 records the schema that exists now.
- Migration v2 adds the column `users.server_name`.

### 4.7 Cleanup of crud.rs

The migration runner needs one function that gives the database path. This
design extracts `db_path()` and a shared connection helper. Then it removes the
repeated blocks from the functions in `crud.rs`.

This design does not make other changes to `crud.rs`.

## 5. Data flow

```
config.toml ──▶ ServerConfig ──▶ EndpointPool ◀── background probe task
                                      │
                                      ▼
users table (token) ─────────────▶ ApiClient ──▶ typed API methods ──▶ UI code
                                      │
                                      ▼
                                  ApiError ──▶ offline mode (sub-project 2)
```

## 6. Backwards compatibility

An installation that exists has a value in `users.server_address` and no
`[[servers]]` block. The program does this at start:

1. If the stored address is in the endpoint list of a configured server, the
   program connects the user to that server.
2. If it is not, the program makes a pool that has one endpoint. This endpoint
   is the stored address.

Therefore a user that does not change the configuration file sees no difference.

## 7. Test plan

### 7.1 Unit tests

These tests do not use the network:

- The pool selects the endpoint with the most importance.
- The pool changes to the next endpoint after a transport failure.
- The background probe makes an endpoint with more importance active again.
- The configuration parser reads the `[[servers]]` block.
- The configuration parser makes a pool with one endpoint from an old
  installation.
- The error classifier puts each HTTP status into the correct category.

### 7.2 Integration tests

These tests use `wiremock`:

- The client changes to the second endpoint when the first endpoint refuses the
  connection.
- The client does not send a `POST` request again.
- The client stops a request after the connect timeout.

### 7.3 Continuous integration

A new workflow `ci.yml` runs these commands:

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo test`
- The ASD-STE100 linter on the Markdown files.

## 8. File layout

| Path | Contents |
|---|---|
| `src/api/client/mod.rs` | `ApiClient` |
| `src/api/client/endpoint.rs` | `EndpointPool`, `Endpoint`, `Health` |
| `src/api/client/error.rs` | `ApiError` |
| `src/db/migrate.rs` | Migration runner |
| `src/config.rs` | Add `ServerConfig` |
| `.github/workflows/ci.yml` | Continuous integration |

## 9. Dependency policy

The project must not need a program that the user installs separately. The
current code needs VLC. Sub-project 2 removes this need.

This sub-project makes two changes:

- It changes OpenSSL to rustls. Then the build does not need a C toolchain.
- It keeps all other dependencies as Rust crates.

The intention is a static binary. One exception can stay: the operating system
audio interface. Sub-project 2 examines this exception.

## 10. Known limitations

- The `users` table uses `username` as the primary key. Therefore two accounts
  with the same name on different servers are in conflict. This defect exists
  today. This design does not correct it.
- The ASD-STE100 linter examines Markdown files only. Rust documentation
  comments stay a convention. A person examines them in review.

## 11. Decisions

| Decision | Selection | Reason |
|---|---|---|
| Endpoint storage | Configuration file | The user can edit it. It does not need a change to the login screen. |
| Failover behaviour | Prefer the endpoint with more importance, then probe | The program returns to the local address automatically. |
| Refactor shape | One client with typed methods | The failover logic stays in one place. |
| Parallel requests | Not used | Parallel requests can send a `POST` request two times. |

## 12. Next sub-projects

The linter, the test harness, and the continuous integration workflow are part
of this sub-project. Section 7.3 gives the details. The fork identity in the
README is a separate small task.

| Number | Sub-project |
|---|---|
| 2 | Audio engine in the process. Remove VLC. |
| 3 | Robustness: remove `unwrap()` calls, add tests |
| 4 | EPUB reader in the terminal interface |
