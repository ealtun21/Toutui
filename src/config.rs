use crate::api::client::endpoint::{Endpoint, EndpointPool};
use color_eyre::eyre::{Report, Result};
use config::{Config as ConfigLib, File};
use serde::Deserialize;
use std::path::Path;

/// The text of `config.example.toml`.
///
/// The program holds that text, therefore it can make the file itself. See
/// T-122.
pub const THE_EXAMPLE_OF_THE_CONFIGURATION: &str = include_str!("../config.example.toml");

#[derive(Debug, Deserialize, Default)]
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

/// The colors of the views.
///
/// **A key that the file does not hold takes the value of the program**, and it
/// keeps every other color of the file. A configuration file of an older
/// version holds no `player_background_color`, and such a file stopped the
/// program before T-122: the block `colors` is one value for `serde`, therefore
/// one key that was absent lost the whole block.
#[derive(Debug, Deserialize)]
#[serde(default)]
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

impl Default for Colors {
    /// The colors of `config.example.toml`.
    fn default() -> Self {
        Self {
            background_color: vec![40, 40, 40],
            log_background_color: vec![40, 40, 40],
            header_background_color: vec![60, 60, 60],
            line_header_color: vec![180, 180, 180],
            list_background_color: vec![50, 50, 50],
            list_background_color_alt_row: vec![60, 60, 60],
            list_selected_background_color: vec![80, 80, 80],
            list_selected_foreground_color: vec![180, 180, 180],
            search_bar_foreground_color: vec![180, 180, 180],
            login_foreground_color: vec![180, 180, 180],
            player_background_color: vec![80, 80, 80],
        }
    }
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

/// Makes `config.toml` when no file stands at that place. See T-122.
///
/// **A user who builds the program has no configuration file.** `install.sh`
/// copies `config.example.toml`, and `cargo install`, a package of a
/// distribution, and a move from the program of before this fork copy nothing:
/// the program then stopped with `configuration file … not found`, and it said
/// a line of its own source. The program holds the text of the example,
/// therefore it writes the file and it goes on.
///
/// The function gives `true` when a file stands at that place after it. A disk
/// that permits no write gives `false`, and `load_config_from` then uses the
/// values of the program.
pub fn make_the_configuration_if_it_is_absent(path: &Path) -> bool {
    if path.exists() {
        return true;
    }

    if let Some(directory) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(directory) {
            log::warn!(
                "[config] the directory {} does not open: {}",
                directory.display(),
                error
            );
            return false;
        }
    }

    match std::fs::write(path, THE_EXAMPLE_OF_THE_CONFIGURATION) {
        Ok(()) => {
            log::info!("[config] the program made {}.", path.display());
            true
        }
        Err(error) => {
            log::warn!(
                "[config] the program cannot make {}: {}. The values of the program stay.",
                path.display(),
                error
            );
            false
        }
    }
}

/// load config from `config.toml` file
pub fn load_config() -> Result<ConfigFile> {
    load_config_from(&crate::paths::config_file())
}

/// Reads the configuration file of a path. `load_config` gives the path of the
/// user, and a test gives a path of its own.
pub fn load_config_from(path: &Path) -> Result<ConfigFile> {
    // A file that is absent comes into existence here. A disk that permits no
    // write gives the values of the program, and the program still starts.
    if !make_the_configuration_if_it_is_absent(path) {
        return Ok(ConfigFile::default());
    }

    let config_path_str = path.to_string_lossy().to_string();

    let config = ConfigLib::builder()
        .add_source(File::with_name(&config_path_str))
        .build()
        .map_err(|e| Report::new(e))?;

    // A block `colors` that is absent, and a key of that block that is absent,
    // take the value of the program. See T-122. A colour that the program
    // cannot read takes the colour of the program alone, and the other colours
    // of the user stay. See T-258.
    let colors = the_colours_of_the_file(&config);
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

/// Gives the text of `config.toml` with one value changed. See T-77.
///
/// **The file belongs to the user, and the user writes comments in it.**
/// `config.example.toml` is almost all comments, and a writer that makes the
/// file again from the values of the program would remove every one of them.
/// Therefore this function changes one line, and it keeps every other line as
/// it stands.
///
/// The three conditions:
///
/// - The block holds the key: the line takes the new value, and it keeps the
///   spaces at its start.
/// - The block exists and it holds no such key: the key comes after the last
///   line of the block that holds a value.
/// - The block does not exist: the block and the key come at the end of the
///   text. **A block that stands inside a comment is not a block**, therefore a
///   file with `# [reader]` gets a real block, and the comment stays.
///
/// The function is pure, therefore a test needs no file.
pub fn with_the_value(text: &str, block: &str, key: &str, value: &str) -> String {
    let head_of_the_block = format!("[{}]", block);
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();

    let mut inside = false;
    let mut head_of_the_lines: Option<usize> = None;
    let mut last_value_of_the_block: Option<usize> = None;
    let mut end_of_the_block: Option<usize> = None;

    for (number, line) in lines.iter().enumerate() {
        let clean = line.trim();

        if clean.starts_with('#') {
            continue;
        }

        if clean.starts_with('[') {
            if inside {
                end_of_the_block = Some(number);
                break;
            }

            inside = clean == head_of_the_block;

            if inside {
                head_of_the_lines = Some(number);
            }

            continue;
        }

        if !inside {
            continue;
        }

        if the_line_holds_the_key(clean, key) {
            let spaces = &line[..line.len() - line.trim_start().len()];
            lines[number] = format!("{}{} = {}", spaces, key, value);
            return join(&lines, text);
        }

        if !clean.is_empty() {
            last_value_of_the_block = Some(number);
        }
    }

    // The block stands in the text, and it holds no such key. The new line
    // comes after the last value of the block, or after the head of the block
    // when the block holds no value.
    if let Some(head) = head_of_the_lines {
        let after = last_value_of_the_block
            .map(|number| number + 1)
            .or(end_of_the_block)
            .unwrap_or(head + 1);

        lines.insert(after, format!("{} = {}", key, value));

        return join(&lines, text);
    }

    // The text holds no such block.
    let mut new = text.trim_end().to_string();

    if !new.is_empty() {
        new.push('\n');
        new.push('\n');
    }

    new.push_str(&format!("{}\n{} = {}\n", head_of_the_block, key, value));

    new
}

/// Says that a line of TOML gives a value to this key.
///
/// The name of a key can hold the name of a different key at its start
/// (`ebook_cache_mb` and `ebook_cache_mb_old`), therefore the sign `=` must come
/// after the name and after the spaces only.
fn the_line_holds_the_key(clean: &str, key: &str) -> bool {
    clean
        .strip_prefix(key)
        .is_some_and(|rest| rest.trim_start().starts_with('='))
}

/// Puts the lines together, and it keeps the end of the text as it was.
fn join(lines: &[String], text: &str) -> String {
    let mut new = lines.join("\n");

    if text.ends_with('\n') {
        new.push('\n');
    }

    new
}

/// Writes one value in `config.toml`, and it keeps every comment. See T-77.
///
/// The write goes to a file beside the file of the user, and a rename then puts
/// it in place: a program that stops in the middle of a write must not leave the
/// user with half a configuration file.
pub fn write_the_value(block: &str, key: &str, value: &str) -> Result<()> {
    let path = crate::paths::config_file();

    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let new = with_the_value(&text, block, key, value);

    let directory = path.parent().ok_or_else(|| {
        Report::msg("The program cannot name the directory of the configuration.")
    })?;

    std::fs::create_dir_all(directory)?;

    let beside = path.with_extension("toml.new");

    std::fs::write(&beside, new.as_bytes())?;
    std::fs::rename(&beside, &path)?;

    Ok(())
}

/// A colour of the configuration file holds three numbers.
const THE_NUMBERS_OF_A_COLOUR: usize = 3;

/// Reads one colour of the block `colors` of the configuration file.
///
/// **The program reads each colour of the file apart** (T-258). The block was
/// one value for `serde` before this item: `config.get::<Colors>("colors")`
/// gave an error for a block that held one number above 255, and
/// `unwrap_or_default` then took **every** colour of the user away. A
/// measurement of the real program v0.8.86, of a file that held
/// `background_color = [200, 0, 0]` and `list_selected_background_color =
/// [80, 80, 300]`, gave a screen of `48;2;40;40;40`: the red of the user went
/// away because a number of another colour stands above 255, and the log said
/// no word of it.
///
/// The three conditions:
///
/// - The key is absent: the colour of the program comes, and the log says
///   nothing. A file of an older version holds no `player_background_color`,
///   and that file is not a fault of the user. See T-122.
/// - The key holds a value that the program cannot read as a list of numbers
///   of 0 to 255: the colour of the program comes, and the log names that key.
/// - The key holds a list of no three numbers: **a colour of two numbers is a
///   colour that the program does not have**, therefore the colour of the
///   program comes and the log names that key. `rgb_parts` repeats the last
///   number of such a list, and the user then sees a colour that they did not
///   ask for.
fn the_colour_of_the_file(config: &ConfigLib, key: &str, of_the_program: Vec<u8>) -> Vec<u8> {
    match config.get::<Vec<u8>>(&format!("colors.{}", key)) {
        Ok(values) if values.len() == THE_NUMBERS_OF_A_COLOUR => values,
        Ok(values) => {
            log::warn!(
                "[config] the colour {} holds {} numbers and not three. \
                 The colour of the program stays.",
                key,
                values.len()
            );
            of_the_program
        }
        // The key is absent. That is not a fault of the user. See T-122.
        Err(config::ConfigError::NotFound(_)) => of_the_program,
        Err(error) => {
            log::warn!(
                "[config] the program cannot read the colour {}: {}. \
                 The colour of the program stays.",
                key,
                error
            );
            of_the_program
        }
    }
}

/// Reads the block `colors` of the configuration file, one colour at a time.
///
/// See `the_colour_of_the_file` for the rule of one colour. A block that is
/// absent gives every colour of the program, and it says nothing.
fn the_colours_of_the_file(config: &ConfigLib) -> Colors {
    let program = Colors::default();

    Colors {
        background_color: the_colour_of_the_file(
            config,
            "background_color",
            program.background_color,
        ),
        log_background_color: the_colour_of_the_file(
            config,
            "log_background_color",
            program.log_background_color,
        ),
        header_background_color: the_colour_of_the_file(
            config,
            "header_background_color",
            program.header_background_color,
        ),
        line_header_color: the_colour_of_the_file(
            config,
            "line_header_color",
            program.line_header_color,
        ),
        list_background_color: the_colour_of_the_file(
            config,
            "list_background_color",
            program.list_background_color,
        ),
        list_background_color_alt_row: the_colour_of_the_file(
            config,
            "list_background_color_alt_row",
            program.list_background_color_alt_row,
        ),
        list_selected_background_color: the_colour_of_the_file(
            config,
            "list_selected_background_color",
            program.list_selected_background_color,
        ),
        list_selected_foreground_color: the_colour_of_the_file(
            config,
            "list_selected_foreground_color",
            program.list_selected_foreground_color,
        ),
        search_bar_foreground_color: the_colour_of_the_file(
            config,
            "search_bar_foreground_color",
            program.search_bar_foreground_color,
        ),
        login_foreground_color: the_colour_of_the_file(
            config,
            "login_foreground_color",
            program.login_foreground_color,
        ),
        player_background_color: the_colour_of_the_file(
            config,
            "player_background_color",
            program.player_background_color,
        ),
    }
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

    /// The file of the user holds comments, and a write must keep every one of
    /// them. See T-77.
    #[test]
    fn a_write_of_a_new_block_keeps_every_line_of_the_file() {
        let text = include_str!("../config.example.toml");

        let new = with_the_value(text, "reader", "ebook_cache_mb", "2048");

        for line in text.lines() {
            assert!(
                new.lines().any(|of_the_new| of_the_new == line),
                "the line \"{}\" of the user went away",
                line
            );
        }

        assert!(new.contains("[reader]\nebook_cache_mb = 2048"));

        // The example file names the block inside a comment. That comment is
        // not a block, and it must stay as it stands.
        assert!(new.contains("# [reader]"));
    }

    /// The file that the write makes must still parse, and the value must come
    /// back.
    #[test]
    fn the_value_of_the_write_comes_back() {
        let text = include_str!("../config.example.toml");

        let new = with_the_value(text, "reader", "ebook_cache_mb", "2048");

        let parsed = ConfigLib::builder()
            .add_source(config::File::from_str(&new, config::FileFormat::Toml))
            .build()
            .expect("the file of the write must parse");

        let reader: ReaderConfig = parsed.get("reader").expect("the block must stand");
        assert_eq!(reader.ebook_cache_mb, 2048);

        // A second write changes the value, and it makes no second block.
        let again = with_the_value(&new, "reader", "ebook_cache_mb", "512");

        assert_eq!(
            again.matches("[reader]").count(),
            2,
            "one block, and one comment of the example file"
        );

        let parsed = ConfigLib::builder()
            .add_source(config::File::from_str(&again, config::FileFormat::Toml))
            .build()
            .expect("the file of the second write must parse");

        let reader: ReaderConfig = parsed.get("reader").expect("the block must stand");
        assert_eq!(reader.ebook_cache_mb, 512);
    }

    #[test]
    fn a_write_changes_the_line_of_the_key_and_it_keeps_the_spaces() {
        let text = "[reader]\n  ebook_cache_mb = 1\n";

        assert_eq!(
            with_the_value(text, "reader", "ebook_cache_mb", "2"),
            "[reader]\n  ebook_cache_mb = 2\n"
        );
    }

    /// The block stands between two other blocks, and it holds no such key.
    #[test]
    fn a_new_key_comes_inside_its_block() {
        let text = "[colors]\nbackground_color = [1, 2, 3]\n\n[reader]\n# a comment\nsomething = 1\n\n[[servers]]\nname = \"home\"\n";

        let new = with_the_value(text, "reader", "ebook_cache_mb", "64");

        assert_eq!(
            new,
            "[colors]\nbackground_color = [1, 2, 3]\n\n[reader]\n# a comment\nsomething = 1\nebook_cache_mb = 64\n\n[[servers]]\nname = \"home\"\n"
        );
    }

    /// A block with no value takes the new key after its head.
    #[test]
    fn a_block_with_no_value_takes_the_key_after_its_head() {
        assert_eq!(
            with_the_value("[reader]\n", "reader", "ebook_cache_mb", "64"),
            "[reader]\nebook_cache_mb = 64\n"
        );
    }

    /// A key that holds the name of the key at its start is a different key.
    #[test]
    fn a_key_of_a_name_that_is_longer_is_a_different_key() {
        let text = "[reader]\nebook_cache_mb_of_the_old = 1\n";

        let new = with_the_value(text, "reader", "ebook_cache_mb", "2");

        assert!(new.contains("ebook_cache_mb_of_the_old = 1"));
        assert!(new.contains("ebook_cache_mb = 2"));
    }

    /// A key of a different block must not change.
    #[test]
    fn the_write_stays_inside_its_block() {
        let text = "[colors]\nebook_cache_mb = 1\n\n[reader]\nebook_cache_mb = 2\n";

        let new = with_the_value(text, "reader", "ebook_cache_mb", "3");

        assert_eq!(
            new,
            "[colors]\nebook_cache_mb = 1\n\n[reader]\nebook_cache_mb = 3\n"
        );
    }

    /// A key inside a comment is not a key.
    #[test]
    fn a_key_of_a_comment_stays_a_comment() {
        let text = "[reader]\n# ebook_cache_mb = 1\n";

        let new = with_the_value(text, "reader", "ebook_cache_mb", "2");

        assert!(new.contains("# ebook_cache_mb = 1"));
        assert!(new.contains("\nebook_cache_mb = 2"));
    }

    /// A file with no line gives a file of one block.
    #[test]
    fn an_empty_file_gives_the_block_and_the_key() {
        assert_eq!(
            with_the_value("", "reader", "ebook_cache_mb", "64"),
            "[reader]\nebook_cache_mb = 64\n"
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

    /// **One colour that the program cannot read must not take the other
    /// colours of the user away** (T-258). The number 300 stands above 255,
    /// therefore the block was one error for `serde` and `unwrap_or_default`
    /// gave every colour of the program. A measurement of the real program
    /// v0.8.86 gave a screen of `48;2;40;40;40` for a file that asked for
    /// `48;2;200;0;0`.
    #[test]
    fn a_colour_that_the_program_cannot_read_keeps_the_other_colours() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[colors]\nbackground_color = [200, 0, 0]\n\
             list_selected_background_color = [80, 80, 300]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.colors.background_color,
            vec![200, 0, 0],
            "the colour of the user must stay"
        );
        assert_eq!(
            config.colors.list_selected_background_color,
            vec![80, 80, 80],
            "the colour that the program cannot read takes the colour of the program"
        );
    }

    /// **A colour that holds no three numbers is a colour that the program does
    /// not have** (T-258). `rgb_parts` repeats the last number of such a list,
    /// therefore a user who wrote `[50, 50]` saw a colour that they did not ask
    /// for. The colour of the program comes now, and the log names the key.
    #[test]
    fn a_colour_of_no_three_numbers_takes_the_colour_of_the_program() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[colors]\nlist_background_color = [50, 50]\n\
             header_background_color = [1, 2, 3, 4]\n\
             line_header_color = []\n\
             background_color = [7, 8, 9]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.colors.list_background_color, vec![50, 50, 50]);
        assert_eq!(config.colors.header_background_color, vec![60, 60, 60]);
        assert_eq!(config.colors.line_header_color, vec![180, 180, 180]);
        assert_eq!(
            config.colors.background_color,
            vec![7, 8, 9],
            "the colour of three numbers of the user must stay"
        );
    }

    /// A key that the file does not hold takes the colour of the program, and
    /// that is not a fault of the user. See T-122 and T-258.
    #[test]
    fn a_colour_that_the_file_does_not_hold_takes_the_colour_of_the_program() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(&path, "[colors]\nbackground_color = [1, 2, 3]\n")
            .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.colors.player_background_color, vec![80, 80, 80]);
    }

    /// **A user who builds the program has no configuration file**, and the
    /// program stopped with a line of its own source. It makes the file now,
    /// and the file is the example: every comment of it reaches the user. See
    /// T-122.
    #[test]
    fn a_configuration_that_is_absent_comes_into_existence() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("of_a_directory").join("config.toml");

        let config = load_config_from(&path).expect("the program must start");

        assert!(path.exists(), "the program must make the file");
        assert_eq!(
            std::fs::read_to_string(&path).expect("the file of the test"),
            THE_EXAMPLE_OF_THE_CONFIGURATION
        );
        assert_eq!(config.colors.background_color, vec![40, 40, 40]);
    }

    /// A second start must read the file of the user, and it must not write
    /// over it.
    #[test]
    fn a_configuration_that_exists_stays_as_it_stands() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(&path, "[colors]\nbackground_color = [1, 2, 3]\n")
            .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.colors.background_color, vec![1, 2, 3]);
        assert_eq!(
            std::fs::read_to_string(&path).expect("the file of the test"),
            "[colors]\nbackground_color = [1, 2, 3]\n"
        );
    }

    /// A file of an older version, and of the program before this fork, holds
    /// no `player_background_color`. Such a file stopped the program. Now the
    /// key that is absent takes the value of the program, and every color of
    /// the file stays. See T-122.
    #[test]
    fn a_key_of_a_colour_that_is_absent_keeps_every_other_colour() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[colors]\nbackground_color = [1, 2, 3]\nlogin_foreground_color = [4, 5, 6]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.colors.background_color, vec![1, 2, 3]);
        assert_eq!(config.colors.login_foreground_color, vec![4, 5, 6]);
        assert_eq!(config.colors.player_background_color, vec![80, 80, 80]);
    }

    /// A file with no block of colors at all must not stop the program.
    #[test]
    fn a_file_with_no_colour_at_all_starts_the_program() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(&path, "[reader]\nebook_cache_mb = 64\n").expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.reader.ebook_cache_mb, 64);
        assert_eq!(config.colors.background_color, vec![40, 40, 40]);
    }

    /// The example of the repository must hold every key of the program. A key
    /// that the example does not name would take the value of the program in
    /// silence, and the user would find no line to change.
    #[test]
    fn the_example_names_every_colour_of_the_program() {
        for key in [
            "background_color",
            "log_background_color",
            "header_background_color",
            "line_header_color",
            "list_background_color",
            "list_background_color_alt_row",
            "list_selected_background_color",
            "list_selected_foreground_color",
            "search_bar_foreground_color",
            "login_foreground_color",
            "player_background_color",
        ] {
            assert!(
                THE_EXAMPLE_OF_THE_CONFIGURATION
                    .lines()
                    .any(|line| line.trim_start().starts_with(key)),
                "the example does not hold {}",
                key
            );
        }
    }

    /// The values of the example and the values of the program must agree. A
    /// user who removes a line must see no change of the color.
    #[test]
    fn the_example_and_the_program_hold_the_same_colours() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");

        let of_the_example = load_config_from(&path).expect("the program must start");
        let of_the_program = Colors::default();

        assert_eq!(
            of_the_example.colors.background_color,
            of_the_program.background_color
        );
        assert_eq!(
            of_the_example.colors.player_background_color,
            of_the_program.player_background_color
        );
        assert_eq!(
            of_the_example.colors.list_selected_background_color,
            of_the_program.list_selected_background_color
        );
    }
}
