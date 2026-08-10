use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use log::error;
use std::env;
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

/// Gives the secret key that encrypts the token.
pub fn secret_key() -> Result<String, String> {
    env::var("TOUTUI_SECRET_KEY").map_err(|_| {
        error!("{}", NO_KEY);
        NO_KEY.to_string()
    })
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

        assert_eq!(
            decrypt_token(from_version_4).unwrap(),
            "the-auth-token"
        );
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
