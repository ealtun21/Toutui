use reqwest::Client;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::{Result, Report};
use crate::api::client::endpoint::{Endpoint, EndpointPool};
use crate::api::client::ApiClient;
use crate::config::{load_config, pool_for_address};
use crate::db::crud::*;
use std::sync::Arc;
use std::time::Duration;
use crate::db::database_struct::User;
use crate::api::libraries::get_all_libraries::*;
use crate::api::utils::collect_get_all_libraries::*;
use crate::utils::encrypt_token::*;
use log::info;


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
        let is_vlc_running = "0".to_string();
        let is_vlc_launched_first_time = "1".to_string();


        // Writting in database : 

        // init a new user
        let users = vec![
            User {
                server_address: server_address.to_string(),
                username: username.to_string(),
                token: token_encrypted,
                is_default_usr: true,
                name_selected_lib: library_names[0].clone(), // by default we take the first library
                id_selected_lib: library_ids[0].clone(),
                is_loop_break,
                is_vlc_launched_first_time,
                speed_rate: 1.0,
                is_vlc_running,
                is_show_key_bindings: "1".to_string()
            }
        ];

        // insert the new user in database
        let _ = db_insert_usr(&users);

        Ok(()) 
    } else {
        Err(Report::new(std::io::Error::other("Login failed"))) 
    }
}
