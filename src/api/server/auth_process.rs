use crate::api::client::endpoint::{Endpoint, EndpointPool};
use crate::api::client::ApiClient;
use crate::api::libraries::get_all_libraries::*;
use crate::api::utils::collect_get_all_libraries::*;
use crate::config::{load_config, pool_for_address};
use crate::db::crud::*;
use crate::db::database_struct::User;
use crate::utils::encrypt_token::*;
use color_eyre::eyre::{Report, Result};
use log::info;
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

        // Token encryption before insert it in the database
        let _token_to_encrypt = login_response.user.token.as_str();
        let mut token_encrypted = "".to_string();
        match encrypt_token(_token_to_encrypt) {
            Ok(encrypted_token) => {
                token_encrypted = encrypted_token;
                info!("Token successfully encrypted")
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

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
        let _ = db_insert_usr(&users);

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
