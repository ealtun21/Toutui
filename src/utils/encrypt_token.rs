use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::env;
use log::error;

/// The message that tells the user how to make the secret key.
const NO_KEY: &str = "No secret key is present. Do this:\n\
    mkdir -p ~/.config/abstui\n\
    echo 'ABSTUI_SECRET_KEY=secret' >> ~/.config/abstui/.env";

/// Gives the secret key that encrypts the token.
///
/// The name `ABSTUI_SECRET_KEY` has the higher importance. The program also
/// accepts `TOUTUI_SECRET_KEY`, because a user who wrote `.env` by hand
/// before the fork has that name.
pub fn secret_key() -> Result<String, String> {
    env::var("ABSTUI_SECRET_KEY")
        .or_else(|_| env::var("TOUTUI_SECRET_KEY"))
        .map_err(|_| {
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
