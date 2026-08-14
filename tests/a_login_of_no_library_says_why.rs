//! A login of an account that reaches no library says why. See T-173.
//!
//! **A server that answers the login and that gives no library** is a condition
//! that no measurement of this repository reached. A new Audiobookshelf server
//! before its first library gives that answer, and an account whose
//! administrator gave it no library gives it too: `GET /api/libraries` comes
//! back with the status 200 and the body `{"libraries": []}`.
//!
//! A measurement of 2026-08-14 with the sandbox, tmux, and
//! `docs/harness/no_library.py`: the proxy forwarded `POST /login` to the
//! sandbox and it answered `GET /api/libraries` with the empty list. The
//! program took the token, it wrote `Token successfully encrypted` in its log,
//! and **the screen then held no character for ever**. The row of the account
//! holds the name and the id of the library of the start, and `library_names[0]`
//! of a list of nothing stopped the thread of the login. The hook of that panic
//! gives the terminal back on the standard output, and the screen of the login
//! holds the lock of it (T-133): the two threads waited for each other, and no
//! word came to the user, to the standard error, or to the log. T-174 gives the
//! terminal back on `/dev/tty` for that road.
//!
//! **This test needs no sandbox.** A host of a raw socket answers the login
//! with a token and the libraries with the empty list.

use toutui::api::server::auth_process::{auth_process, THE_SENTENCE_OF_A_LOGIN_WITH_NO_LIBRARY};

/// Starts a host that logs the user in and that holds no library.
async fn a_server_of_no_library() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                let mut request = Vec::new();
                let mut byte = [0u8; 1];
                while socket.read(&mut byte).await.unwrap_or(0) == 1 {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                let head_of_the_request = String::from_utf8_lossy(&request).to_string();

                // The login gives a token. The token of this test is not a
                // token of a server, and no request of this test uses it.
                let body = if head_of_the_request.starts_with("POST /login") {
                    r#"{"user":{"token":"a token of this test"}}"#.to_string()
                } else {
                    r#"{"libraries": []}"#.to_string()
                };

                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(head.as_bytes()).await;
                let _ = socket.write_all(body.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    });

    address
}

/// The login of an account that reaches no library gives one sentence, and it
/// writes no row of an account.
///
/// **The parts of this test stay in one function**: two test functions of one
/// binary fight for the boxes of the process (T-144 and T-157), and
/// `auth_process` reads the configuration file of the process.
#[tokio::test(flavor = "multi_thread")]
async fn a_login_of_an_account_of_no_library_says_why() {
    // The configuration and the database of this test belong to a directory of
    // its own. **A test never writes the account of the user.** The secret key
    // stands in the environment, therefore the login of this test reaches the
    // line of the library and not the line of the key.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::env::set_var("TOUTUI_SECRET_KEY", "the key of this test");

    let address = a_server_of_no_library().await;

    let answer = auth_process("a user", "a password", &address).await;

    // Without the correction this line never comes: the thread of the login
    // stops with `index out of bounds: the len is 0 but the index is 0`.
    let error = answer.expect_err("an account of no library has no start");

    assert_eq!(
        error.to_string(),
        THE_SENTENCE_OF_A_LOGIN_WITH_NO_LIBRARY,
        "the login must say that the server gave no library"
    );

    // The row of the message of the login holds one line. See the trap 11 of
    // the harness.
    assert!(
        THE_SENTENCE_OF_A_LOGIN_WITH_NO_LIBRARY.len() <= 150,
        "the sentence of a login of no library is too long"
    );

    // The sentence names the work of the user. A text that says a fault and no
    // work sends the user to look for a fault of their own (T-91).
    assert!(
        THE_SENTENCE_OF_A_LOGIN_WITH_NO_LIBRARY.contains("administrator"),
        "the sentence must name the work of the user"
    );
}
