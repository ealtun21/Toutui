use crate::api::client::endpoint::{Endpoint, EndpointPool};
use color_eyre::eyre::{Report, Result};
use config::{Config as ConfigLib, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub colors: Colors,
    /// The servers that the configuration file gives.
    pub servers: Vec<ServerConfig>,
    /// The settings of the reader of the ebooks. See T-72.
    pub reader: ReaderConfig,
}

/// The settings of the reader of the ebooks. See T-72.
#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ReaderConfig {
    /// The largest cache of the ebooks of the disk, in megabytes.
    ///
    /// The value 0, and a file with no such key, give the value of the program:
    /// `logic::reader::cache::LIMIT_OF_THE_CACHE`, of one gigabyte. **A cache of
    /// 0 bytes would remove every book of the disk**, therefore that value cannot
    /// mean itself.
    #[serde(default)]
    pub ebook_cache_mb: u64,
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
    let config_path = crate::paths::config_file();
    let config_path_str = config_path.to_str().unwrap().to_string();

    let config = ConfigLib::builder()
        .add_source(File::with_name(&config_path_str))
        .build()
        .map_err(|e| Report::new(e))?;

    let colors: Colors = config.get("colors").map_err(|e| Report::new(e))?;
    // A configuration file that an older version made has no `servers`
    // block. An empty list is correct in that condition.
    let servers: Vec<ServerConfig> = config.get("servers").unwrap_or_default();
    // A configuration file of an older version has no block `reader`. Every
    // value of that block then takes the value of the program. See T-72.
    let reader: ReaderConfig = config.get("reader").unwrap_or_default();

    Ok(ConfigFile {
        colors,
        servers,
        reader,
    })
}

/// The colour that a list with no value gives.
const DEFAULT_COMPONENT: u8 = 128;

/// Gives the three components of a colour of the configuration file.
///
/// A colour of the configuration file is a list of numbers. The user writes
/// that list, therefore the list does not always hold three values. The
/// program must not stop because of such a list.
///
/// The function repeats the last value for a component that the list does not
/// give. A list with no value gives a middle grey.
///
/// The old code read the three components with an index. A list that was too
/// short then stopped the program. `load_config` also gives an error for a
/// file that a person cannot parse, and the old code then read an empty list.
/// A measurement on 2026-08-10 stopped a thread with "index out of bounds: the
/// len is 0 but the index is 0".
pub fn rgb_parts(values: &[u8]) -> (u8, u8, u8) {
    let component = |index: usize| -> u8 {
        match values.get(index) {
            Some(value) => *value,
            None => *values.last().unwrap_or(&DEFAULT_COMPONENT),
        }
    };

    (component(0), component(1), component(2))
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

/// Gives the identity of the server that holds the stored address.
///
/// One server can have many addresses, and the pool changes between them. The
/// identity must therefore not be one address: a position of a local address
/// and a position of a public address belong to the same server.
///
/// The function gives the name of the configured server. A server that the
/// configuration file does not name gives its address. Two servers then never
/// have the same identity, and the application does not send the position of
/// one server to a different server. See T-25.
pub fn server_key(servers: &[ServerConfig], stored_address: &str) -> String {
    server_name_for_address(servers, stored_address)
        .unwrap_or_else(|| normalise(stored_address).to_string())
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
    /// A configuration file that an older version made still has a `[player]`
    /// block. That block controlled VLC. The application must read the file
    /// and must not fail. See T-14.
    #[test]
    fn an_old_player_block_does_not_stop_the_application() {
        let text = r#"
[player]
cvlc = "1"
cvlc_term = "0"
address = "localhost"
port = "1234"

[colors]
background_color = [40, 40, 40]
log_background_color = [40, 40, 40]
header_background_color = [60, 60, 60]
line_header_color = [80, 80, 80]
list_background_color = [40, 40, 40]
list_background_color_alt_row = [45, 45, 45]
list_selected_background_color = [70, 70, 70]
list_selected_foreground_color = [255, 255, 255]
search_bar_foreground_color = [255, 255, 255]
login_foreground_color = [255, 255, 255]
player_background_color = [80, 80, 80]

[[servers]]
name = "home"
endpoints = [ { url = "http://localhost:13378", priority = 0 } ]
"#;

        let parsed = ConfigLib::builder()
            .add_source(config::File::from_str(text, config::FileFormat::Toml))
            .build()
            .expect("the file must parse");

        // The application reads only the keys that it knows. The unknown
        // block does no damage.
        let colors: Colors = parsed.get("colors").expect("the colours must load");
        assert_eq!(colors.background_color, vec![40, 40, 40]);

        let servers: Vec<ServerConfig> = parsed.get("servers").unwrap_or_default();
        assert_eq!(servers.len(), 1);
    }

    /// Every address of one server gives the same identity. A position that
    /// the user made on the local address therefore goes to the same server
    /// through the public address.
    #[test]
    fn every_address_of_a_server_gives_the_same_identity() {
        let list = servers();

        let first = server_key(&list, "http://192.168.1.10:13378");
        let second = server_key(&list, "https://abs.example.com");

        assert_eq!(first, "home");
        assert_eq!(second, "home");
    }

    /// A server that the configuration file does not name gives its address.
    #[test]
    fn a_server_that_is_not_configured_gives_its_address() {
        assert_eq!(
            server_key(&servers(), "http://other:13378/"),
            "http://other:13378"
        );
        assert_eq!(server_key(&[], "http://only"), "http://only");
    }

    /// Two servers must never have the same identity.
    #[test]
    fn two_servers_have_two_identities() {
        let list = servers();

        assert_ne!(
            server_key(&list, "http://localhost:13378"),
            server_key(&list, "http://second-server:13378")
        );
    }

    #[test]
    fn a_complete_colour_gives_its_three_components() {
        assert_eq!(rgb_parts(&[40, 50, 60]), (40, 50, 60));
    }

    /// A list with more than three values gives the first three values.
    #[test]
    fn a_colour_that_is_too_long_gives_the_first_three_components() {
        assert_eq!(rgb_parts(&[40, 50, 60, 70]), (40, 50, 60));
    }

    /// This list stopped the program. `load_config` gives an error for a file
    /// that a person cannot parse, and the old code then read an empty list.
    #[test]
    fn a_colour_with_no_value_gives_a_grey() {
        assert_eq!(rgb_parts(&[]), (128, 128, 128));
    }

    /// The user writes the list. A list of one value or two values must not
    /// stop the program.
    #[test]
    fn a_colour_that_is_too_short_repeats_the_last_value() {
        assert_eq!(rgb_parts(&[40]), (40, 40, 40));
        assert_eq!(rgb_parts(&[40, 50]), (40, 50, 50));
    }
}
