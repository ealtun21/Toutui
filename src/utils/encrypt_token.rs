use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::env;
use log::error;

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

    /// A cipher that a different key wrote gives an error, and not a panic.
    #[test]
    fn a_cipher_of_a_different_key_gives_an_error() {
        with_key();

        assert!(decrypt_token("bm90LWEtY2lwaGVy").is_err());
    }
}
