use log::error;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::env;
use std::io::{Read, Write};
use std::path::Path;

/// Reads the file `.env` and puts every value of it in the environment.
///
/// A file that is absent is not a fault, because the user can give the key in
/// the environment itself. Therefore the function gives `false` and writes
/// nothing.
///
/// The fork moved from `dotenv` to `dotenvy`, because `dotenv` has no
/// maintainer since 2019. This function holds that call in one place, and a
/// test confirms that a key in the file arrives in the environment. A silent
/// fault here makes every token unreadable, and no user could enter.
pub fn load_env_file(path: &Path) -> bool {
    dotenvy::from_filename(path).is_ok()
}

/// The message that tells the user how to make the secret key.
const NO_KEY: &str = "No secret key is present. Do this:\n\
    mkdir -p ~/.config/toutui\n\
    echo 'TOUTUI_SECRET_KEY=secret' >> ~/.config/toutui/.env";

/// The name of the variable of the environment that holds the secret key.
pub const THE_KEY: &str = "TOUTUI_SECRET_KEY";

/// Gives the secret key that encrypts the token.
pub fn secret_key() -> Result<String, String> {
    match env::var(THE_KEY) {
        Ok(key) if !key.is_empty() => Ok(key),
        _ => {
            error!("{}", NO_KEY);
            Err(NO_KEY.to_string())
        }
    }
}

/// Makes the secret key when the machine has none, and writes it in `.env`.
/// See T-133.
///
/// **A program that has no key keeps no token.** `install.sh` makes the key,
/// and a user who builds the program with `cargo`, with `nix`, or with a
/// package of their system gets no such file. The login of that user then
/// asked the server, took the token, and wrote nothing: the next start showed
/// the login screen again, and no message said why.
///
/// The function gives `true` when it made a key, and `false` when the machine
/// held one already. It never makes a second key: a new key makes every token
/// of the database unreadable, and every account would have to log in again.
///
/// The key is 32 bytes of the machine, in the form of the hexadecimal, and the
/// file belongs to the user alone.
pub fn the_program_makes_a_key_if_it_has_none(config_dir: &Path) -> Result<bool, String> {
    if matches!(env::var(THE_KEY), Ok(key) if !key.is_empty()) {
        return Ok(false);
    }

    let path = config_dir.join(".env");

    // **The file of a start before this one holds the key.** The caller reads
    // `.env` already, and this function reads it again: a caller that forgets
    // that line must not make a second key.
    load_env_file(&path);

    if matches!(env::var(THE_KEY), Ok(key) if !key.is_empty()) {
        return Ok(false);
    }

    let key = a_new_key()?;

    std::fs::create_dir_all(config_dir).map_err(|error| {
        format!(
            "The program did not make {}: {}",
            config_dir.display(),
            error
        )
    })?;

    let mut options = std::fs::OpenOptions::new();
    options.create(true).append(true);

    // Nobody else reads the key. The mode belongs to the moment of the making,
    // therefore no other program reads the file between the two lines.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(&path)
        .map_err(|error| format!("The program did not open {}: {}", path.display(), error))?;

    // A file of another line keeps that line, and the key stands on a line of
    // its own.
    let the_line = if the_file_ends_with_a_new_line(&path) {
        format!("{}={}\n", THE_KEY, key)
    } else {
        format!("\n{}={}\n", THE_KEY, key)
    };

    file.write_all(the_line.as_bytes())
        .map_err(|error| format!("The program did not write {}: {}", path.display(), error))?;

    env::set_var(THE_KEY, &key);

    Ok(true)
}

/// Says whether the file ends with a new line. A file that does not exist and a
/// file of no byte need no new line before the key.
fn the_file_ends_with_a_new_line(path: &Path) -> bool {
    match std::fs::read(path) {
        Ok(bytes) => bytes.is_empty() || bytes.ends_with(b"\n"),
        Err(_) => true,
    }
}

/// Gives 32 bytes of the machine, in the form of the hexadecimal.
///
/// The bytes come from `/dev/urandom`. Linux and macOS both hold that file, and
/// it needs no crate: a key of a generator of the program itself would be a key
/// that another program can make again.
fn a_new_key() -> Result<String, String> {
    let mut bytes = [0u8; 32];

    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(|error| format!("The program did not read /dev/urandom: {}", error))?;

    Ok(bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>())
}

pub fn encrypt_token(token_to_encrypt: &str) -> Result<String, String> {
    let key = secret_key()?;
    let mc = new_magic_crypt!(key, 256);
    Ok(mc.encrypt_str_to_base64(token_to_encrypt))
}

pub fn decrypt_token(encrypted_token: &str) -> Result<String, String> {
    let key = secret_key()?;
    let mc = new_magic_crypt!(key, 256);
    mc.decrypt_base64_to_string(encrypted_token).map_err(|_| {
        error!("Failed to decrypt the token.");
        "Failed to decrypt the token.".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::{decrypt_token, encrypt_token};

    /// The key of the test. Only the tests of this module read this variable,
    /// therefore one value for the whole process is correct here.
    fn with_key() {
        std::env::set_var("TOUTUI_SECRET_KEY", "a-test-secret-key");
    }

    /// The program reads what it wrote.
    #[test]
    fn a_token_comes_back_from_its_own_cipher() {
        with_key();

        let cipher = encrypt_token("the-auth-token").unwrap();

        assert_eq!(decrypt_token(&cipher).unwrap(), "the-auth-token");
    }

    /// The program reads a token that magic-crypt 4 wrote.
    ///
    /// The fork moved from magic-crypt 4.0.1 to 5.0.1. A change of the form of
    /// the cipher would make every token that a user has unreadable, and every
    /// user would have to give their password again. magic-crypt 4.0.1 wrote
    /// this text for the token "the-auth-token" with the key of the test.
    #[test]
    fn a_token_of_magic_crypt_4_stays_readable() {
        with_key();

        let from_version_4 = "6rP+/cWyTHmupXVAA1oH9Q==";

        assert_eq!(decrypt_token(from_version_4).unwrap(), "the-auth-token");
    }

    /// A key in the file `.env` arrives in the environment.
    ///
    /// The test uses its own name for the variable, so that it does not change
    /// the key that the other tests of this module read.
    #[test]
    fn a_key_in_the_file_arrives_in_the_environment() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".env");
        std::fs::write(&file, "TOUTUI_TEST_ENV_KEY=from-the-file\n").unwrap();

        assert!(super::load_env_file(&file));
        assert_eq!(
            std::env::var("TOUTUI_TEST_ENV_KEY").unwrap(),
            "from-the-file"
        );
    }

    /// A file that is absent gives `false`, and it is not a fault.
    #[test]
    fn a_file_that_is_absent_gives_false() {
        let dir = tempfile::tempdir().unwrap();

        assert!(!super::load_env_file(&dir.path().join("no-such-file")));
    }

    /// A cipher that a different key wrote gives an error, and not a panic.
    #[test]
    fn a_cipher_of_a_different_key_gives_an_error() {
        with_key();

        assert!(decrypt_token("bm90LWEtY2lwaGVy").is_err());
    }
}
