use crate::api::client::endpoint::{Endpoint, EndpointPool};
use config::{Config as ConfigLib, File};
use serde::Deserialize;
use color_eyre::eyre::{Result, Report};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub colors: Colors,
    pub player: Player,
    /// The servers that the configuration file gives.
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Colors {
    pub background_color: Vec<u8>,
    pub log_background_color: Vec<u8>,
    pub header_background_color: Vec<u8>,
    pub line_header_color: Vec<u8>,
    pub list_background_color: Vec<u8>,
    pub list_background_color_alt_row: Vec<u8>,
    pub list_selected_background_color: Vec<u8>,
    pub list_selected_foreground_color: Vec<u8>,
    pub search_bar_foreground_color: Vec<u8>,
    pub login_foreground_color: Vec<u8>,
    pub player_background_color: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub cvlc: String,
    pub cvlc_term: String,
    pub address: String,
    pub port: String,
}

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

/// load config from `config.toml` file
pub fn load_config() -> Result<ConfigFile> {
    let config_home_path = env::var("XDG_CONFIG_HOME")
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

    let config_path = config_home_path.join("toutui/config.toml");
    let config_path_str = config_path.to_str().unwrap().to_string();

    let config = ConfigLib::builder()
        .add_source(File::with_name(&config_path_str))
        .build()
        .map_err(|e| Report::new(e))?;

    let colors: Colors = config.get("colors")
        .map_err(|e| Report::new(e))?;
    let player: Player = config.get("player")
        .map_err(|e| Report::new(e))?;

    // A configuration file that an older version made has no `servers`
    // block. An empty list is correct in that condition.
    let servers: Vec<ServerConfig> = config.get("servers").unwrap_or_default();

    Ok(ConfigFile { colors, player, servers })
}

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

