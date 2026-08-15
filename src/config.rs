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

/// The fault of a configuration file that the program cannot read at all. See
/// T-265.
///
/// **A value of the file that the program cannot read is not a file that the
/// program cannot read.** T-258 to T-263 each take one value of the user away
/// and they keep every other value; a file whose shape the crate `config`
/// refuses gives no value at all, therefore the program stops. The words of
/// that crate name the line and the column of the fault, and they name no file
/// and no road back: the report of `main` said
/// `Error: TOML parse error at line 64, column 31` with
/// `Location: src/config.rs`, and a user must read no line of the source of
/// this program (T-172).
///
/// `the_words_of_a_program_that_stops` makes the sentence of the screen out of
/// this fault.
#[derive(Debug)]
pub struct TheConfigurationFileDidNotCome {
    /// The path of the file that the program cannot read.
    pub path: String,
    /// What the crate `config` said of that file. It names the line and the
    /// column, and no word of the program can give that.
    pub reason: String,
}

impl std::fmt::Display for TheConfigurationFileDidNotCome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The program cannot read the configuration file {}.\n{}",
            self.path, self.reason
        )
    }
}

impl std::error::Error for TheConfigurationFileDidNotCome {}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    pub colors: Colors,
    /// The servers that the configuration file gives.
    pub servers: Vec<ServerConfig>,
    /// The settings of the reader of the ebooks. See T-72.
    pub reader: ReaderConfig,
    /// The name of each value of the configuration file that the program does
    /// not use. See T-264.
    ///
    /// The read of the file takes a value of the user away for two reasons: the
    /// program cannot read it (T-258 and T-259), or a rule of the program
    /// refuses it (T-260 to T-263). Each of the two took a line of the log
    /// alone, and **the file belongs to the user**: the user wrote that value,
    /// and the program then used a different one with no word at all. This list
    /// holds the name of each such value, and
    /// `the_words_of_the_values_that_the_program_does_not_use` makes the
    /// sentence of the screen.
    #[serde(default, skip)]
    pub the_values_that_the_program_does_not_use: Vec<String>,
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

    // T-265. The words of the crate name the line and the column of the fault,
    // and they name no file and no road back. The fault of this program holds
    // both, and the caller that stands in front of the user says it.
    let config = ConfigLib::builder()
        .add_source(File::with_name(&config_path_str))
        .build()
        .map_err(|e| {
            Report::new(TheConfigurationFileDidNotCome {
                path: config_path_str.clone(),
                reason: e.to_string(),
            })
        })?;

    // A block `colors` that is absent, and a key of that block that is absent,
    // take the value of the program. See T-122. A colour that the program
    // cannot read takes the colour of the program alone, and the other colours
    // of the user stay. See T-258.
    // T-264. Every reader below takes a value of the user away in silence. The
    // name of each such value comes here, and the caller that holds a screen
    // says the sentence of it.
    let mut went_away = Vec::new();

    let colors = the_colours_of_the_file(&config, &mut went_away);
    // A configuration file that an older version made has no `servers`
    // block. An empty list is correct in that condition. A server of that list
    // that the program cannot read goes away alone, and every other server of
    // the user stays. See T-259.
    let servers = the_servers_of_the_file(&config, &mut went_away);
    // A configuration file of an older version has no block `reader`. Every
    // value of that block then takes the value of the program. See T-72. A
    // value of that block that the program cannot read takes the value of the
    // program alone. See T-259.
    let reader = the_reader_of_the_file(&config, &mut went_away);

    Ok(ConfigFile {
        colors,
        servers,
        reader,
        the_values_that_the_program_does_not_use: went_away,
    })
}

/// Gives the sentence of the screen for the values that the program does not
/// use, or nothing when the program uses every value of the file. See T-264.
///
/// **A value of the file that goes away took a line of the log alone.** The log
/// is the one word of a fault that no view of the user holds (T-177), and this
/// fault holds a view: the user wrote the file, the user can correct it, and the
/// screen stands in front of them at the start and at the key `R`. Therefore the
/// screen says the number, and the log keeps the name and the reason of each
/// value.
///
/// The sentence says "does not use" and not "cannot read", because the two
/// reasons of T-258 to T-263 stand together in this list: a value that the
/// program cannot read, and a value that a rule of the program refuses.
pub fn the_words_of_the_values_that_the_program_does_not_use(names: &[String]) -> Option<String> {
    match names.len() {
        // The program uses every value of the file. A message of no fault is a
        // message that hides the answer of a key for six seconds.
        0 => None,
        // A count of one takes no plural. The shape `1 value(s)` is no sentence
        // of a person.
        1 => Some(
            "The program does not use 1 value of the configuration file. \
             The log names it."
                .to_string(),
        ),
        many => Some(format!(
            "The program does not use {} values of the configuration file. \
             The log names each of them.",
            many
        )),
    }
}

/// Says the sentence of [`the_words_of_the_values_that_the_program_does_not_use`]
/// on the screen of the user. See T-264.
///
/// **The read of the configuration file has no screen of its own.** The program
/// reads that file at its start, at the key `R`, at the login, and at the moment
/// that a book comes into the cache of the ebooks. The first two of them stand
/// in front of the user, therefore they call this function; the other two say
/// nothing, because a message of a task that the user did not ask for belongs to
/// no view (T-164).
pub fn say_the_values_that_the_program_does_not_use(config: &ConfigFile) {
    if let Some(words) = the_words_of_the_values_that_the_program_does_not_use(
        &config.the_values_that_the_program_does_not_use,
    ) {
        crate::logic::message::say(&words);
    }
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
fn the_colour_of_the_file(
    config: &ConfigLib,
    key: &str,
    of_the_program: Vec<u8>,
    went_away: &mut Vec<String>,
) -> Vec<u8> {
    match config.get::<Vec<u8>>(&format!("colors.{}", key)) {
        Ok(values) if values.len() == THE_NUMBERS_OF_A_COLOUR => values,
        Ok(values) => {
            went_away.push(format!("colors.{}", key));
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
            went_away.push(format!("colors.{}", key));
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
fn the_colours_of_the_file(config: &ConfigLib, went_away: &mut Vec<String>) -> Colors {
    let program = Colors::default();

    Colors {
        background_color: the_colour_of_the_file(
            config,
            "background_color",
            program.background_color,
            went_away,
        ),
        log_background_color: the_colour_of_the_file(
            config,
            "log_background_color",
            program.log_background_color,
            went_away,
        ),
        header_background_color: the_colour_of_the_file(
            config,
            "header_background_color",
            program.header_background_color,
            went_away,
        ),
        line_header_color: the_colour_of_the_file(
            config,
            "line_header_color",
            program.line_header_color,
            went_away,
        ),
        list_background_color: the_colour_of_the_file(
            config,
            "list_background_color",
            program.list_background_color,
            went_away,
        ),
        list_background_color_alt_row: the_colour_of_the_file(
            config,
            "list_background_color_alt_row",
            program.list_background_color_alt_row,
            went_away,
        ),
        list_selected_background_color: the_colour_of_the_file(
            config,
            "list_selected_background_color",
            program.list_selected_background_color,
            went_away,
        ),
        list_selected_foreground_color: the_colour_of_the_file(
            config,
            "list_selected_foreground_color",
            program.list_selected_foreground_color,
            went_away,
        ),
        search_bar_foreground_color: the_colour_of_the_file(
            config,
            "search_bar_foreground_color",
            program.search_bar_foreground_color,
            went_away,
        ),
        login_foreground_color: the_colour_of_the_file(
            config,
            "login_foreground_color",
            program.login_foreground_color,
            went_away,
        ),
        player_background_color: the_colour_of_the_file(
            config,
            "player_background_color",
            program.player_background_color,
            went_away,
        ),
    }
}

/// Gives one value of the configuration file, and it names a value that the
/// program cannot read.
///
/// **A block of `serde` is one value, and one value is one fault** (T-258 and
/// T-259). This function reads one key of one block, therefore a value that the
/// program cannot read takes the value of the program alone.
///
/// A key that the file does not hold takes the value of the program in silence:
/// a file of an older version is not a fault of the user (T-122).
fn the_value_of_the_file<T>(
    config: &ConfigLib,
    key: &str,
    of_the_program: T,
    went_away: &mut Vec<String>,
) -> T
where
    T: for<'a> Deserialize<'a>,
{
    match config.get::<T>(key) {
        Ok(value) => value,
        // The key is absent. That is not a fault of the user. See T-122.
        Err(config::ConfigError::NotFound(_)) => of_the_program,
        Err(error) => {
            went_away.push(key.to_string());
            log::warn!(
                "[config] the program cannot read {}: {}. \
                 The value of the program stays.",
                key,
                error
            );
            of_the_program
        }
    }
}

/// Reads the block `reader` of the configuration file, one value at a time.
///
/// See T-259. `config.get("reader")` read the whole block as one value: a user
/// who wrote `ebook_cache_mb = -1` lost every value of the block, and the cache
/// of the ebooks then held one gigabyte for a limit of 512 megabytes that the
/// user asked for, with no word at all.
fn the_reader_of_the_file(config: &ConfigLib, went_away: &mut Vec<String>) -> ReaderConfig {
    let program = ReaderConfig::default();

    ReaderConfig {
        ebook_cache_mb: the_value_of_the_file(
            config,
            "reader.ebook_cache_mb",
            program.ebook_cache_mb,
            went_away,
        ),
    }
}

/// One server of the block `servers`, as the file gives it.
///
/// The addresses stay values that the program did not read yet, because it
/// reads each address of a server apart. See `the_servers_of_the_file`.
#[derive(Debug, Deserialize)]
struct TheRowOfAServer {
    name: String,
    endpoints: Vec<config::Value>,
}

/// Reads the block `servers` of the configuration file, one server at a time
/// and one address at a time.
///
/// See T-259. `config.get("servers")` read the whole list as one value:
/// **one address of one server that the program cannot read took every server
/// of the user away**. The name of a server is the identity of the place of the
/// user on the disk (`server_key`), therefore the queue and the downloads of
/// that user went away with the block, and the log said no word of it.
///
/// The rule of one server:
///
/// - A server with no name, or with no list of addresses, belongs to no pool:
///   the name is the identity, and the program cannot ask an address that it
///   cannot read. That server goes away, and the log names its place.
/// - An address that the program cannot read goes away, and every other address
///   of that server stays: a server has more than one address, and one of them
///   answers.
/// - A server that keeps no address belongs to no pool.
/// - A name that is an address belongs to no pool: the address of an account
///   that no server of the file names is the identity of the place of that
///   user, therefore a name of the prefix `http://` or `https://` can hold the
///   identity of a different server (T-262).
/// - An address that more than one server holds belongs to no server: an
///   address names one machine, and the name of the server that holds it is the
///   identity of the place of the user, therefore that address goes away from
///   each of those servers (T-263). See `the_addresses_of_one_server_alone`.
/// - A name that a server before it holds already belongs to no pool: two
///   servers of one name hold one identity, and the place of one server then
///   goes to a different server (T-261). The server of the first block keeps
///   the name, and every server after it that repeats that name goes away.
fn the_servers_of_the_file(config: &ConfigLib, went_away: &mut Vec<String>) -> Vec<ServerConfig> {
    let rows: Vec<config::Value> = match config.get("servers") {
        Ok(rows) => rows,
        // The block is absent. A file of an older version holds none. See
        // T-122.
        Err(config::ConfigError::NotFound(_)) => return Vec::new(),
        Err(error) => {
            went_away.push("servers".to_string());
            log::warn!(
                "[config] the program cannot read the block servers: {}. \
                 The program uses the address of the login screen.",
                error
            );
            return Vec::new();
        }
    };

    let mut servers = Vec::new();

    for (place, row) in rows.into_iter().enumerate() {
        let row: TheRowOfAServer = match row.try_deserialize() {
            Ok(row) => row,
            Err(error) => {
                went_away.push(format!("the server {} of the block servers", place + 1));
                log::warn!(
                    "[config] the program cannot read the server {} of the block servers: {}. \
                     That server goes away, and every other server stays.",
                    place + 1,
                    error
                );
                continue;
            }
        };

        // T-260. A name of no character is no name. `serde` reads `name = ""`
        // with no fault at all, and `server_key` then gives `""` for the
        // identity of the place of the user: the queue and the downloads of
        // that user go away, and the column `server` of those tables holds
        // `''` for a row of a server that no file names — therefore the place
        // of one server goes to a different server (T-25). A server of no name
        // goes away, and the address of the account then gives the identity.
        if row.name.trim().is_empty() {
            went_away.push(format!("the server {} of the block servers", place + 1));
            log::warn!(
                "[config] the server {} of the block servers has a name of no character. \
                 A name is the identity of the place of the user, therefore that server \
                 goes away and the program uses the address of the login screen.",
                place + 1
            );
            continue;
        }

        // T-262. A name that is an address holds the identity of a server that
        // the file does not name. `server_key` gives the name of the server for
        // an address that the file names, and it gives `normalise(the address)`
        // for every other address. `check_shape` of `src/api/server/address.rs`
        // gives an address of the prefix `http://` or `https://` alone,
        // therefore a name of one of those two prefixes can be the identity of
        // a different server: a measurement of the real program showed the
        // queue of the server of the port 13399 on the screen of the account of
        // the port 13500, whose server the file named `http://localhost:13399`.
        // Such a server goes away, and its address then gives its identity.
        let name = row.name.trim().to_ascii_lowercase();
        if name.starts_with("http://") || name.starts_with("https://") {
            went_away.push(format!("the server {} of the block servers", place + 1));
            log::warn!(
                "[config] the server {} of the block servers has the name {}, which is an \
                 address. The address of an account is the identity of the place of that user \
                 when no server of the file holds that address, therefore a name that is an \
                 address can give one identity to two servers: that server goes away and the \
                 program uses the address of the login screen.",
                place + 1,
                row.name
            );
            continue;
        }

        // T-261. Two servers of one name hold one identity. `server_key` gives
        // the name of the **first** server that holds the address, therefore
        // the account of the second server writes the column `server` of the
        // tables `queue` and `downloads` with the name of the first one: the
        // queue and the downloads of one server then go to a different server,
        // which the rule of T-25 does not permit. The server of the first block
        // keeps the name, and a server after it that repeats that name goes
        // away. The address of the account of that server then gives the
        // identity.
        if servers
            .iter()
            .any(|server: &ServerConfig| server.name == row.name)
        {
            went_away.push(format!("the server {} of the block servers", place + 1));
            log::warn!(
                "[config] the server {} of the block servers has the name {}, which a server \
                 before it has already. A name is the identity of the place of the user, and \
                 two servers must not hold one identity, therefore that server goes away and \
                 the program uses the address of the login screen.",
                place + 1,
                row.name
            );
            continue;
        }

        let mut endpoints = Vec::new();
        for address in row.endpoints {
            match address.try_deserialize::<EndpointConfig>() {
                Ok(endpoint) => endpoints.push(endpoint),
                Err(error) => {
                    went_away.push(format!("an address of the server {}", row.name));
                    log::warn!(
                        "[config] the program cannot read an address of the server {}: {}. \
                     That address goes away, and every other address of it stays.",
                        row.name,
                        error
                    );
                }
            }
        }

        if endpoints.is_empty() {
            went_away.push(format!("the server {} of the block servers", place + 1));
            log::warn!(
                "[config] the server {} has no address that the program can read. \
                 That server goes away.",
                row.name
            );
            continue;
        }

        servers.push(ServerConfig {
            name: row.name,
            endpoints,
        });
    }

    the_addresses_of_one_server_alone(servers, went_away)
}

/// Takes an address that more than one server of the block `servers` holds out
/// of every server that holds it.
///
/// See T-263. An address names one machine, and the name of the server that
/// holds that address is the identity of the place of the user on the disk
/// (`server_key`). Therefore an address that two servers hold gives **one**
/// identity to **two** servers: `server_name_for_address` and
/// `pool_for_address` each give the first server of the list that holds the
/// address, and the queue and the downloads of the second server then go to the
/// first one.
///
/// A measurement of the real program v0.8.91 showed the two faults of one file.
/// The file named the server `work` with the addresses `http://127.0.0.1:13500`
/// and `http://localhost:13399`, and the server `home` with the address
/// `http://localhost:13399`. The queue of the account of the port 13500 came to
/// the screen of the account of the port 13399, and the header of that second
/// account said `🔗 127.0.0.1:13500`: **the program of one server asked the
/// address of a different server.**
///
/// The program cannot know which of the two servers holds that machine, because
/// the file says both. Therefore that address belongs to **no** server: it goes
/// away from each of them, and the address of the login screen then gives the
/// identity of that place. A server that keeps no address belongs to no pool.
///
/// An address that **one** server holds two times is no address of two servers,
/// and it stays.
fn the_addresses_of_one_server_alone(
    mut servers: Vec<ServerConfig>,
    went_away: &mut Vec<String>,
) -> Vec<ServerConfig> {
    let mut the_addresses_of_two_servers: Vec<String> = Vec::new();

    for (place, server) in servers.iter().enumerate() {
        for endpoint in &server.endpoints {
            let address = normalise(&endpoint.url);

            let a_server_after_it_holds_it = servers.iter().enumerate().any(|(other, server)| {
                other != place
                    && server
                        .endpoints
                        .iter()
                        .any(|endpoint| normalise(&endpoint.url) == address)
            });

            if a_server_after_it_holds_it
                && !the_addresses_of_two_servers
                    .iter()
                    .any(|held| held == address)
            {
                the_addresses_of_two_servers.push(address.to_string());
            }
        }
    }

    for address in &the_addresses_of_two_servers {
        let names: Vec<String> = servers
            .iter()
            .filter(|server| {
                server
                    .endpoints
                    .iter()
                    .any(|endpoint| normalise(&endpoint.url) == address)
            })
            .map(|server| server.name.clone())
            .collect();

        went_away.push(format!("the address {} of the block servers", address));
        log::warn!(
            "[config] more than one server of the block servers has the address {}: {}. \
             The name of the server that holds an address is the identity of the place of \
             the user, therefore that address goes away from each of those servers and the \
             program uses the address of the login screen.",
            address,
            names.join(", ")
        );

        for server in servers.iter_mut() {
            server
                .endpoints
                .retain(|endpoint| normalise(&endpoint.url) != address);
        }
    }

    servers.retain(|server| {
        if server.endpoints.is_empty() {
            went_away.push(format!("the server {} of the block servers", server.name));
            log::warn!(
                "[config] the server {} keeps no address of its own. That server goes away.",
                server.name
            );
            return false;
        }

        true
    });

    servers
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

    /// **One address of one server that the program cannot read took every
    /// server of the user away** (T-259). The name of a server is the identity
    /// of the place of the user on the disk, therefore the queue and the
    /// downloads of that user went away with the block, and the log said no
    /// word of it. The server of the fault goes away alone now.
    #[test]
    fn a_server_that_the_program_cannot_read_keeps_the_other_servers() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"the sandbox\"\n\
             endpoints = [ { url = \"http://localhost:13399\", priority = 0 } ]\n\
             \n[[servers]]\nname = \"the server away from home\"\n\
             endpoints = [ { url = \"https://abs.example.com\", priority = 300 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.servers.len(),
            1,
            "the server of the user that the program can read must stay"
        );
        assert_eq!(config.servers[0].name, "the sandbox");
        assert_eq!(
            server_key(&config.servers, "http://localhost:13399"),
            "the sandbox",
            "the identity of the place of the user on the disk must stay"
        );
    }

    /// **A server has more than one address, and one of them answers** (T-259).
    /// An address that the program cannot read goes away, and every other
    /// address of that server stays.
    #[test]
    fn an_address_that_the_program_cannot_read_keeps_the_other_addresses() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\nendpoints = [\n\
             { url = \"http://192.168.1.10:13378\", priority = 0 },\n\
             { url = \"https://abs.example.com\", priority = 300 },\n]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.servers.len(), 1, "the server of the user must stay");
        assert_eq!(
            config.servers[0].endpoints.len(),
            1,
            "the address that the program can read must stay"
        );
        assert_eq!(
            config.servers[0].endpoints[0].url,
            "http://192.168.1.10:13378"
        );
    }

    /// **A name of no character is no name** (T-260). `serde` reads
    /// `name = ""` with no fault, therefore the server reached the pool and
    /// `server_key` gave `""` for the identity of the place of the user: the
    /// queue and the downloads of that user went away, and the log said no
    /// word at all. The server of no name goes away now, and the address of
    /// the account gives the identity again.
    #[test]
    fn a_server_of_no_name_keeps_the_place_of_the_user() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"\"\n\
             endpoints = [ { url = \"http://localhost:13399\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert!(
            config.servers.is_empty(),
            "a server of no name must belong to no pool"
        );
        assert_eq!(
            server_key(&config.servers, "http://localhost:13399"),
            "http://localhost:13399",
            "the address of the account must give the identity of the place of the user"
        );
    }

    /// **A name of spaces alone is a name of no character** (T-260): the user
    /// cannot tell it from an empty name in the file, and it gives the same
    /// identity of no word on the disk.
    #[test]
    fn a_server_of_a_name_of_spaces_keeps_the_place_of_the_user() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"   \"\n\
             endpoints = [ { url = \"http://localhost:13399\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert!(
            config.servers.is_empty(),
            "a server of a name of spaces must belong to no pool"
        );
    }

    /// **A server of no name goes away alone** (T-260). The rule of T-259
    /// holds: every other server of the user stays, and it keeps the identity
    /// of the place of that user.
    #[test]
    fn a_server_of_no_name_keeps_the_other_servers() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"\"\n\
             endpoints = [ { url = \"https://abs.example.com\", priority = 0 } ]\n\
             \n[[servers]]\nname = \"the sandbox\"\n\
             endpoints = [ { url = \"http://localhost:13399\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.servers.len(),
            1,
            "the server of the user that holds a name must stay"
        );
        assert_eq!(
            server_key(&config.servers, "http://localhost:13399"),
            "the sandbox",
            "the identity of the place of the user must stay"
        );
    }

    /// **Two servers of no name held one identity** (T-260, and the rule of
    /// T-25): `server_key` gave `""` for the address of each of them,
    /// therefore the place of one server went to a different server. Each of
    /// them gives its own address now.
    #[test]
    fn two_servers_of_no_name_hold_two_identities() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 } ]\n\
             \n[[servers]]\nname = \"\"\n\
             endpoints = [ { url = \"http://second:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        let first = server_key(&config.servers, "http://first:13378");
        let second = server_key(&config.servers, "http://second:13378");
        assert_ne!(
            first, second,
            "two servers must never hold the same identity of the place of the user"
        );
        assert_eq!(first, "http://first:13378");
        assert_eq!(second, "http://second:13378");
    }

    /// **Two servers of one name held one identity** (T-261, and the rule of
    /// T-25). `server_key` gives the name of the first server that holds the
    /// address, therefore the account of the second server wrote the queue and
    /// the downloads of the first one: a measurement of the real program showed
    /// the queue of the server of the port 13399 on the screen of the account
    /// of the port 13500. The second server goes away now, and its address
    /// gives its own identity.
    #[test]
    fn two_servers_of_one_name_hold_two_identities() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 } ]\n\
             \n[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://second:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        let first = server_key(&config.servers, "http://first:13378");
        let second = server_key(&config.servers, "http://second:13378");
        assert_ne!(
            first, second,
            "two servers must never hold the same identity of the place of the user"
        );
        assert_eq!(
            first, "home",
            "the server of the first block keeps the name"
        );
        assert_eq!(
            second, "http://second:13378",
            "the address of the account must give the identity of the second server"
        );
    }

    /// **A name that a server before it holds already belongs to no pool**
    /// (T-261): that server goes away whole, therefore no address of it stands
    /// in the pool of the first server. An address of one server must never
    /// answer for a different server, because the program sends the token of
    /// the account to it.
    #[test]
    fn a_server_of_a_name_that_stands_already_goes_away_whole() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 } ]\n\
             \n[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://second:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.servers.len(),
            1,
            "a server of a name that stands already must belong to no pool"
        );
        assert_eq!(
            pool_for_address(&config.servers, "http://first:13378").len(),
            1,
            "the pool of the first server must hold no address of the second one"
        );
        assert_eq!(
            pool_for_address(&config.servers, "http://second:13378").len(),
            1,
            "the pool of the second server must hold the address of the account alone"
        );
    }

    /// **A server that keeps its name goes away, and the server after it then
    /// keeps that name** (T-261). The name of a server that belongs to no pool
    /// is no identity of the disk, therefore it holds no name away from a
    /// server that the program reads.
    #[test]
    fn a_name_of_a_server_that_went_away_stays_free() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 300 } ]\n\
             \n[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://second:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.servers.len(),
            1,
            "the server that the program reads must stay"
        );
        assert_eq!(
            server_key(&config.servers, "http://second:13378"),
            "home",
            "the name of the server that went away must stay free"
        );
    }

    /// **Two names of two servers stay two identities** (T-261). This test
    /// passes on both builds, and it guards the road of the user who names each
    /// server of the file.
    #[test]
    fn two_servers_of_two_names_keep_their_identities() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 } ]\n\
             \n[[servers]]\nname = \"the server away from home\"\n\
             endpoints = [ { url = \"http://second:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.servers.len(), 2, "the two servers of the user stay");
        assert_eq!(server_key(&config.servers, "http://first:13378"), "home");
        assert_eq!(
            server_key(&config.servers, "http://second:13378"),
            "the server away from home"
        );
    }

    /// **A name that is an address held the identity of a different server**
    /// (T-262, and the rule of T-25). The address of an account that no server
    /// of the file names is the identity of the place of that user, therefore a
    /// server of the name `http://second:13378` took the queue and the
    /// downloads of the server at that address: a measurement of the real
    /// program showed the queue of the server of the port 13399 on the screen
    /// of the account of the port 13500. That server goes away now, and its
    /// own address gives its identity.
    #[test]
    fn a_name_that_is_an_address_holds_no_identity_of_a_different_server() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"http://second:13378\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        let first = server_key(&config.servers, "http://first:13378");
        let second = server_key(&config.servers, "http://second:13378");
        assert_ne!(
            first, second,
            "two servers must never hold the same identity of the place of the user"
        );
        assert_eq!(
            first, "http://first:13378",
            "the address of the account must give the identity of the server of that name"
        );
    }

    /// **A server of a name that is an address belongs to no pool** (T-262):
    /// that server goes away whole, therefore no address of it stands in the
    /// pool of the server of that address. The program sends the token of the
    /// account to an address of the pool, and an address of a different server
    /// must never get it (T-97 and T-128). The prefix `https://` is an address
    /// too, because `check_shape` gives one of the two prefixes alone.
    #[test]
    fn a_server_of_a_name_that_is_an_address_goes_away_whole() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"https://second:13378\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 },\n\
             { url = \"http://third:13378\", priority = 1 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.servers.len(),
            0,
            "a server of a name that is an address must belong to no pool"
        );
        assert_eq!(
            pool_for_address(&config.servers, "http://first:13378").len(),
            1,
            "the pool must hold the address of the account alone"
        );
    }

    /// **A name that holds an address inside it stays** (T-262). This test
    /// passes on both builds, and it guards the road of the user who names a
    /// server with words that hold an address: such a name starts with no
    /// prefix of an address, therefore it is the identity of no other server.
    #[test]
    fn a_name_that_holds_an_address_inside_it_stays() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"the server at http://second:13378\"\n\
             endpoints = [ { url = \"http://first:13378\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.servers.len(), 1, "the server of the user must stay");
        assert_eq!(
            server_key(&config.servers, "http://first:13378"),
            "the server at http://second:13378",
            "the identity of the place of the user must stay"
        );
    }

    /// **An address that two servers of the file hold holds the identity of a
    /// different server** (T-263). The measurement of the real program v0.8.91:
    /// the file named the server `work` with the addresses of the ports 13500
    /// and 13399, and the server `home` with the address of the port 13399. The
    /// queue of the account of the port 13500 came to the screen of the account
    /// of the port 13399, because `server_key` gave `work` for the two of them.
    ///
    /// The address of an account that no server of the file holds is the
    /// identity of the place of that user, therefore that address gives the two
    /// servers two identities again.
    #[test]
    fn an_address_of_two_servers_holds_no_identity_of_a_different_server() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"work\"\nendpoints = [\n\
             { url = \"http://127.0.0.1:13500\", priority = 0 },\n\
             { url = \"http://localhost:13399\", priority = 1 },\n]\n\n\
             [[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://localhost:13399\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            server_key(&config.servers, "http://localhost:13399"),
            "http://localhost:13399",
            "the address of two servers must hold the identity of no server of the file"
        );
        assert_eq!(
            server_key(&config.servers, "http://127.0.0.1:13500"),
            "work",
            "the address of one server must keep the name of that server"
        );
    }

    /// **An address of two servers goes away from each of them, and a server
    /// that keeps no address goes away** (T-263). The pool of the account of
    /// that address must hold that address alone: the measurement of the real
    /// program showed the header `🔗 127.0.0.1:13500` for the account of the
    /// port 13399, because the pool of it was the pool of a different server.
    ///
    /// The address of the server `home` holds a slash at its end, and the
    /// address of the server `work` holds none: two addresses of one machine
    /// are one address (`normalise`).
    #[test]
    fn an_address_of_two_servers_goes_away_from_each_of_them() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"work\"\nendpoints = [\n\
             { url = \"http://127.0.0.1:13500\", priority = 0 },\n\
             { url = \"http://localhost:13399\", priority = 1 },\n]\n\n\
             [[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://localhost:13399/\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.servers.len(),
            1,
            "the server that keeps no address of its own must go away"
        );
        assert_eq!(config.servers[0].name, "work");
        assert_eq!(
            config.servers[0].endpoints.len(),
            1,
            "the address of two servers must go away from the server that stays"
        );
        assert_eq!(
            pool_for_address(&config.servers, "http://localhost:13399").len(),
            1,
            "the pool of that account must hold the address of the login screen alone"
        );
    }

    /// **An address that one server holds two times is no address of two
    /// servers** (T-263). This test passes on both builds, and it guards the
    /// user who writes one address two times in one block: that server holds
    /// its identity still.
    #[test]
    fn an_address_that_one_server_holds_two_times_stays() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\nendpoints = [\n\
             { url = \"http://first:13378\", priority = 0 },\n\
             { url = \"http://first:13378\", priority = 1 },\n]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.servers.len(), 1, "the server of the user must stay");
        assert_eq!(
            server_key(&config.servers, "http://first:13378"),
            "home",
            "the identity of the place of the user must stay"
        );
    }

    /// **A value of the block `reader` that the program cannot read takes the
    /// value of the program** (T-259). A user who wrote `ebook_cache_mb = -1`
    /// got the limit of the program, of one gigabyte, for the 512 megabytes
    /// that they asked for.
    ///
    /// **This test passes on both builds of T-259**, and it says so: the block
    /// `reader` holds one value today, therefore the fault of the block and the
    /// fault of the value give the same number. The correction of that block
    /// gives the **word** alone, and the log of the real program is the
    /// evidence of it. A second key of the block would put this block under the
    /// gate of `a_server_that_the_program_cannot_read_keeps_the_other_servers`.
    #[test]
    fn a_value_of_the_reader_that_the_program_cannot_read_takes_the_value_of_the_program() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(&path, "[reader]\nebook_cache_mb = -1\n").expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.reader.ebook_cache_mb, 0,
            "the value of the program comes for a value that it cannot read"
        );
    }

    /// The guard of the rule of T-122: a block that the file does not hold is
    /// not a fault of the user, and it says nothing. This test passes on both
    /// builds of T-259.
    #[test]
    fn a_block_that_the_file_does_not_hold_takes_the_values_of_the_program() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(&path, "[colors]\nbackground_color = [1, 2, 3]\n")
            .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert!(config.servers.is_empty());
        assert_eq!(config.reader.ebook_cache_mb, 0);
    }

    /// **A server of the file that the program can read reaches the pool**
    /// (T-259). The correction must take no server away that stands.
    #[test]
    fn the_servers_of_the_file_that_the_program_reads_stay() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[reader]\nebook_cache_mb = 512\n\n\
             [[servers]]\nname = \"home\"\nendpoints = [\n\
             { url = \"http://192.168.1.10:13378\", priority = 0 },\n\
             { url = \"https://abs.example.com\", priority = 1 },\n]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(config.reader.ebook_cache_mb, 512);
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].endpoints.len(), 2);
        assert_eq!(
            pool_for_address(&config.servers, "https://abs.example.com").len(),
            2,
            "the pool of the user must hold the two addresses of that server"
        );
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
    /// T-264. **A value of the configuration file that the program does not use
    /// took a line of the log alone.** The file belongs to the user: they wrote
    /// that value, and the program then used a different one with no word at
    /// all. A measurement of the real program v0.8.92 inside tmux, with a file
    /// of a colour of two numbers, of a server with a name of no character, and
    /// of `ebook_cache_mb = "not a number"`, gave a screen of 45 rows with
    /// **no** word of the configuration: `grep -icE "config|colour|value|file"`
    /// of the whole screen gave 0.
    ///
    /// The two reasons of a value that goes away stand in one list: the program
    /// cannot read it (T-258 and T-259), and a rule of the program refuses it
    /// (T-260 to T-263).
    #[test]
    fn the_program_names_each_value_of_the_file_that_it_does_not_use() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[colors]\nbackground_color = [40, 40]\n\
             list_background_color = [50, 50, 50]\n\
             [reader]\nebook_cache_mb = \"not a number\"\n\
             [[servers]]\nname = \"\"\n\
             endpoints = [ { url = \"http://one.example.com\", priority = 0 } ]\n\
             [[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://two.example.com\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert_eq!(
            config.the_values_that_the_program_does_not_use,
            vec![
                "colors.background_color".to_string(),
                "the server 1 of the block servers".to_string(),
                "reader.ebook_cache_mb".to_string(),
            ],
            "the program must name the colour, the server, and the value of the reader"
        );

        // The values of the user that the program uses stay, and they name
        // nothing.
        assert_eq!(config.colors.list_background_color, vec![50, 50, 50]);
        assert_eq!(config.servers.len(), 1, "the server of a name stays");
    }

    /// A file whose every value the program uses names nothing. **A message of
    /// no fault hides the answer of a key for six seconds**, therefore the
    /// program must say nothing at all.
    #[test]
    fn a_file_that_the_program_reads_names_no_value_at_all() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[colors]\nbackground_color = [40, 40, 40]\n[reader]\nebook_cache_mb = 512\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert!(
            config.the_values_that_the_program_does_not_use.is_empty(),
            "a file of no fault must name no value"
        );
        assert_eq!(
            the_words_of_the_values_that_the_program_does_not_use(
                &config.the_values_that_the_program_does_not_use
            ),
            None,
            "a file of no fault must say nothing at all"
        );
    }

    /// The sentence of the screen. **A count of one takes no plural**: the shape
    /// `1 value(s)` is no sentence of a person.
    #[test]
    fn the_sentence_of_the_screen_counts_the_values() {
        assert_eq!(
            the_words_of_the_values_that_the_program_does_not_use(&[]),
            None
        );
        assert_eq!(
            the_words_of_the_values_that_the_program_does_not_use(&[
                "colors.background_color".to_string()
            ])
            .expect("one value gives a sentence"),
            "The program does not use 1 value of the configuration file. The log names it."
        );
        assert_eq!(
            the_words_of_the_values_that_the_program_does_not_use(&[
                "colors.background_color".to_string(),
                "reader.ebook_cache_mb".to_string(),
                "the server 1 of the block servers".to_string(),
            ])
            .expect("three values give a sentence"),
            "The program does not use 3 values of the configuration file. \
             The log names each of them."
        );
    }

    /// A server that a rule of the program refuses is a value that the program
    /// does not use (T-260 to T-263), and the user must read it too. The
    /// address `http://one.example.com` stands in the two servers, therefore it
    /// goes away from each of them (T-263), and the two servers then keep no
    /// address of their own.
    #[test]
    fn a_server_that_a_rule_of_the_program_refuses_names_itself() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(
            &path,
            "[[servers]]\nname = \"home\"\n\
             endpoints = [ { url = \"http://one.example.com\", priority = 0 } ]\n\
             [[servers]]\nname = \"work\"\n\
             endpoints = [ { url = \"http://one.example.com\", priority = 0 } ]\n",
        )
        .expect("the file of the test");

        let config = load_config_from(&path).expect("the program must start");

        assert!(config.servers.is_empty(), "the two servers go away");
        assert_eq!(
            config.the_values_that_the_program_does_not_use,
            vec![
                "the address http://one.example.com of the block servers".to_string(),
                "the server home of the block servers".to_string(),
                "the server work of the block servers".to_string(),
            ],
            "the address and the two servers that it took away must each name themselves"
        );
    }

    /// A file whose shape the crate `config` refuses gives no value of the user
    /// at all, therefore the fault of the program names that file and it holds
    /// what the crate said. See T-265.
    #[test]
    fn a_file_that_the_program_cannot_read_names_that_file() {
        let place = tempfile::tempdir().expect("a directory of a test");
        let path = place.path().join("config.toml");
        std::fs::write(&path, "[colors]\nbackground_color = [40, 40, 40\n")
            .expect("the file of the test");

        let report = load_config_from(&path).expect_err("a file of no shape stops the read");

        let fault = report
            .chain()
            .find_map(|cause| cause.downcast_ref::<TheConfigurationFileDidNotCome>())
            .expect("the fault of the configuration file must stand in the report");

        assert_eq!(
            fault.path,
            path.to_string_lossy(),
            "the fault must name the file of the user"
        );
        assert!(
            fault.reason.contains("unclosed array"),
            "the fault must hold what the crate said: {}",
            fault.reason
        );

        let words = fault.to_string();
        assert!(
            words.contains("cannot read the configuration file"),
            "{}",
            words
        );
        assert!(!words.contains("Location"), "{}", words);
    }
}
