use crate::api::client::endpoint::{Endpoint, EndpointPool};
use crate::api::client::ApiClient;
use crate::api::libraries::get_all_libraries::*;
use crate::api::utils::collect_get_all_libraries::*;
use crate::config::{load_config, pool_for_address};
use crate::db::crud::*;
use crate::db::database_struct::User;
use crate::utils::encrypt_token::*;
use color_eyre::eyre::{Report, Result};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    user: UserInfo,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    token: String,
}

/// Gives the sentence of a login that the server refused. See T-92.
///
/// **The old code said "Login failed" for every status.** A user then read the
/// same four words for a wrong password, for a rate limit, and for a fault of
/// the server, and no line of the screen told them what to do. The status of
/// the answer holds that knowledge.
///
/// **The rate limit is the status that costs the most time.** Audiobookshelf
/// permits 40 requests of the login in 600 seconds, and it answers `429` after
/// that. A user who writes their password again and again reaches it, and
/// "Login failed" then sends them to look for a fault that does not exist.
///
/// The function is pure, therefore a test needs no server.
pub fn the_sentence_of_a_login_that_failed(status: u16) -> String {
    match status {
        401 | 403 => "The server refused the username or the password.".to_string(),
        404 => {
            "The server has no login at this address. Is it an Audiobookshelf server?".to_string()
        }
        429 => "The server took too many attempts of the login. Wait 10 minutes.".to_string(),
        code if code >= 500 => {
            format!("The server has a fault. It answered {}.", code)
        }
        code => format!("The server refused the login. It answered {}.", code),
    }
}

/// The sentence of a login of an account that reaches no library. See T-173.
///
/// **A server can answer the login and then give no library.** A new
/// Audiobookshelf server before its first library gives that answer, and an
/// account whose administrator gave it no library gives it too:
/// `GET /api/libraries` comes back with the status 200 and the body
/// `{"libraries": []}`.
///
/// The program takes the first library of that answer for the account of the
/// database, therefore an account of no library has no start. The login stops
/// here with this sentence, and the row of the account never comes.
pub const THE_SENTENCE_OF_A_LOGIN_WITH_NO_LIBRARY: &str =
    "The server gave no library for this account. Ask an administrator of the server for a library.";

/// The sentence of a login that wrote no row of the account. See T-199.
///
/// **A login that writes no row is a login that failed.** The old code wrote the
/// row with `let _ = db_insert_usr(&users)`, therefore a database that a second
/// program of this account held took the row away with no word at all: the log
/// said `Login successful`, the program came back to the login screen of a first
/// start, and the row of the message held no character. The user wrote the
/// address, the name, and the password again, and the login gave the login
/// screen back for ever.
///
/// The row of the message of the login holds one line, therefore the sentence
/// holds one line. See the trap 11 of the harness.
pub const THE_SENTENCE_OF_A_LOGIN_THAT_KEPT_NO_ACCOUNT: &str =
    "The program did not write the account in its database. Stop a second Toutui, and try the login again.";

/// Login
/// https://api.audiobookshelf.org/#server
///
/// The login function takes a username, password, url ans  makes a POST request and returns a token.
/// After, some data are fetched with this token and written in database
pub async fn auth_process(username: &str, password: &str, server_address: &str) -> Result<()> {
    let login_url = format!("{}/login", server_address);

    // This function runs before the `ApiClient` exists, because this function
    // gets the token. Therefore it makes its own client. The timeouts stop a
    // long wait if the address does not answer.
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(15))
        .build()?;

    // Struct for data request
    let login_data = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    // Send POST request
    let response = client
        .post(login_url)
        .header("Content-Type", "application/json")
        .json(&login_data)
        .send()
        .await?;

    // Checking the status of the response and fetch data
    if response.status().is_success() {
        let login_response: LoginResponse = response.json().await?;

        // Make a client for the first read. The configuration file can give
        // more than one address for this server. If the file has no entry for
        // the address, the pool holds the address that the user gave.
        let pool = match load_config() {
            Ok(config) => pool_for_address(&config.servers, server_address),
            Err(_) => EndpointPool::new(vec![Endpoint::new(server_address, 0)]),
        };

        let api = ApiClient::new(Arc::new(pool), login_response.user.token.clone())
            .map_err(Report::new)?;

        let all_libraries = get_all_libraries(&api).await.map_err(Report::new)?;
        let library_names = collect_library_names(&all_libraries).await;
        let _media_types = collect_media_types(&all_libraries).await;
        let library_ids = collect_library_ids(&all_libraries).await;

        // **An account of no library has no start.** The row of the account
        // holds the name and the id of the library of the start, and the old
        // code took the first name and the first id of a list that can hold
        // nothing: `library_names[0]` then stopped the thread of the login with
        // a panic. The screen of the login holds the lock of the standard
        // output, and the hook of that panic writes to it: the two threads
        // waited for each other, and the user read a screen of no character for
        // ever. See T-173, T-174, and T-133.
        if library_names.is_empty() || library_ids.is_empty() {
            error!("[auth_process] the account reaches no library of this server");

            return Err(Report::new(std::io::Error::other(
                THE_SENTENCE_OF_A_LOGIN_WITH_NO_LIBRARY.to_string(),
            )));
        }

        // Token encryption before insert it in the database
        //
        // **A login that keeps no token is a login that failed.** The old code
        // wrote the reason with `println!` and it kept an empty token: the row
        // of the account then held no token, and the next start showed the
        // login screen again with no word of the reason.
        //
        // `println!` also stopped the program. This function runs on the thread
        // of the login, and the screen of the login holds the lock of the
        // standard output while it waits for that thread: the two threads then
        // waited for each other for ever, and the user read a screen of no
        // character. **No line of this function writes to the terminal.** See
        // T-133.
        let token_encrypted = match encrypt_token(login_response.user.token.as_str()) {
            Ok(encrypted_token) => {
                info!("Token successfully encrypted");
                encrypted_token
            }
            Err(error) => {
                error!("[auth_process] the token has no cipher: {}", error);

                // The row of the message of the login holds one line. See the
                // trap 11 of the harness.
                return Err(Report::new(std::io::Error::other(
                    "The program has no secret key, therefore it keeps no token. See the log."
                        .to_string(),
                )));
            }
        };

        // Init for handle_l
        let is_loop_break = "0".to_string();
        // The user played no media yet. Therefore the application does not
        // wait for a loop of a playback before it.
        let has_played_before = "1".to_string();

        // Writting in database :

        // init a new user
        let users = vec![User {
            server_address: server_address.to_string(),
            username: username.to_string(),
            token: token_encrypted,
            is_default_usr: true,
            name_selected_lib: library_names[0].clone(), // by default we take the first library
            id_selected_lib: library_ids[0].clone(),
            is_loop_break,
            has_played_before,
            speed_rate: 1.0,
            is_show_key_bindings: "1".to_string(),
        }];

        // insert the new user in database
        //
        // **A login that writes no row is a login that failed.** The old line
        // was `let _ = db_insert_usr(&users)`. See T-199.
        if let Err(error) = db_insert_usr(&users) {
            error!(
                "[auth_process] the program did not write the row of the account: {}",
                error
            );

            return Err(Report::new(std::io::Error::other(
                THE_SENTENCE_OF_A_LOGIN_THAT_KEPT_NO_ACCOUNT.to_string(),
            )));
        }

        // **One account starts the program, and the database must hold that
        // rule.** A second login writes a second row, and two rows with
        // `is_default_usr = 1` let the rowid decide which account the program
        // takes: the user who added an account would then meet the account of
        // the login before it. The account of the newest login starts. See
        // T-124.
        // **This write holds the login too** (T-199). The old code wrote the
        // fault in the log alone, and it gave the login its success: two rows
        // with `is_default_usr = 1` then let the rowid decide, therefore the user
        // who logged in met the account of the login before theirs, with no word
        // at all.
        if let Err(error) = crate::db::crud::make_this_account_the_default(username) {
            error!(
                "[auth_process] the account of the login must start: {}",
                error
            );

            return Err(Report::new(std::io::Error::other(
                THE_SENTENCE_OF_A_LOGIN_THAT_KEPT_NO_ACCOUNT.to_string(),
            )));
        }

        Ok(())
    } else {
        // The status says why the server refused the login. See T-92.
        Err(Report::new(std::io::Error::other(
            the_sentence_of_a_login_that_failed(response.status().as_u16()),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each status of the login gives its own sentence, and the sentence says
    /// what the user must do. See T-92.
    #[test]
    fn the_login_says_why_the_server_refused_it() {
        assert_eq!(
            the_sentence_of_a_login_that_failed(401),
            "The server refused the username or the password."
        );
        assert_eq!(
            the_sentence_of_a_login_that_failed(403),
            "The server refused the username or the password."
        );

        // The rate limit of the login is 40 requests of 600 seconds.
        assert_eq!(
            the_sentence_of_a_login_that_failed(429),
            "The server took too many attempts of the login. Wait 10 minutes."
        );

        assert!(the_sentence_of_a_login_that_failed(404).contains("Audiobookshelf"));
        assert!(the_sentence_of_a_login_that_failed(500).contains("fault"));
        assert!(the_sentence_of_a_login_that_failed(503).contains("503"));
        assert!(the_sentence_of_a_login_that_failed(418).contains("418"));

        // The row of the message of the login holds one line. Every sentence
        // stays inside it. See the trap 11 of the harness.
        for status in [401, 403, 404, 429, 500, 418] {
            let text = the_sentence_of_a_login_that_failed(status);
            assert!(text.len() <= 150, "the sentence of {} is too long", status);
        }
    }
}
