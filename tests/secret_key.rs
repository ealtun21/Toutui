//! Tests of the name of the secret key.
//!
//! The program before the fork used `TOUTUI_SECRET_KEY`. This program uses
//! `TOUTUI_SECRET_KEY`, and it accepts the old name.
//!
//! The tests write variables of the environment, and variables are common to
//! the process. Therefore this file holds one test only.

use toutui::utils::encrypt_token::secret_key;

#[test]
fn the_program_accepts_the_two_names() {
    // No name is present.
    std::env::remove_var("TOUTUI_SECRET_KEY");
    std::env::remove_var("TOUTUI_SECRET_KEY");
    assert!(secret_key().is_err());

    // The old name only.
    std::env::set_var("TOUTUI_SECRET_KEY", "old");
    assert_eq!(secret_key().unwrap(), "old");

    // The two names. The new name has the higher importance.
    std::env::set_var("TOUTUI_SECRET_KEY", "new");
    assert_eq!(secret_key().unwrap(), "new");
}
