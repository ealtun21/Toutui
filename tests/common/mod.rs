//! What every test of the sandbox needs. See T-96.
//!
//! **The login of the server permits 40 requests of 600 seconds.** Sixteen
//! files of `tests/` held the same `token()`, and each of them asked the server
//! for a token of its own: one run of every test of the sandbox therefore made
//! about twenty logins. Two runs inside ten minutes reached the limit, the
//! server answered `429`, and the test said "the answer must hold a token".
//! **That happened three times in the session of 2026-08-11**, and each time it
//! cost a restart of the container and a run of the tests again.
//!
//! This module keeps the token in a file of `CARGO_TARGET_TMPDIR`. A run that
//! finds a token there **examines it with one request that is not a login**
//! (`GET /api/libraries`), and it asks for a new token only when that request
//! fails. One run of every test of the sandbox therefore makes **one** login,
//! and a second run makes none.

use std::path::PathBuf;

pub const SERVER: &str = "http://127.0.0.1:13399";
pub const USER: &str = "toutuitest";
pub const PASSWORD: &str = "toutuitest";

/// The file that holds the token between two runs.
///
/// `CARGO_TARGET_TMPDIR` belongs to the tests of this package, and
/// `cargo clean` removes it with everything else.
fn the_file_of_the_token() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("the-token-of-the-sandbox")
}

/// Says if the server takes this token.
///
/// **This request is not a login**, therefore it does not reach the rate limit
/// of the login.
async fn the_server_takes_the_token(token: &str) -> bool {
    let answer = reqwest::Client::new()
        .get(format!("{}/api/libraries", SERVER))
        .bearer_auth(token)
        .send()
        .await;

    matches!(answer, Ok(answer) if answer.status().is_success())
}

/// Asks the server for a new token.
async fn a_new_token() -> String {
    let answer: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/login", SERVER))
        .json(&serde_json::json!({ "username": USER, "password": PASSWORD }))
        .send()
        .await
        .expect("the sandbox server must answer")
        .json()
        .await
        .expect("the answer of the login must hold JSON");

    answer["user"]["token"]
        .as_str()
        .expect(
            "the answer must hold a token. A run of the tests that comes after \
             many logins meets the rate limit of 40 requests of 600 seconds: \
             read `podman logs abs-test`, and give the container a restart.",
        )
        .to_string()
}

/// Gives a token of the sandbox.
///
/// The token of the run before this one comes back when the server still takes
/// it. See T-96.
#[allow(dead_code)]
pub async fn token() -> String {
    let file = the_file_of_the_token();

    if let Ok(text) = std::fs::read_to_string(&file) {
        let old = text.trim().to_string();

        if !old.is_empty() && the_server_takes_the_token(&old).await {
            return old;
        }
    }

    let fresh = a_new_token().await;

    // A file beside it and a rename: two tests that write together then never
    // give a half of a token to a third one.
    let beside = file.with_extension(format!("{}", std::process::id()));

    if std::fs::write(&beside, &fresh).is_ok() {
        let _ = std::fs::rename(&beside, &file);
    }

    fresh
}
