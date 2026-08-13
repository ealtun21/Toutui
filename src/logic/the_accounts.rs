//! The accounts of the program, and the rules that no view holds. See T-124.
//!
//! **The program held one account before this work.** The view of the login
//! came only when the database held no account, the view of the accounts showed
//! one line, and no key chose the account that starts. The database held every
//! value of a second account already: a row of `users` with `is_default_usr`.
//!
//! The rules of this module are pure functions. The view reads them, and a test
//! holds each of them to the measurement that made it.

/// The name of the variable that asks the program for the login screen.
///
/// **A login that comes again needs a terminal that no view holds** (T-123).
/// The program therefore starts again in the place of its process, and a value
/// of this process does not cross `exec`: the variable carries the request to
/// the new program.
pub const THE_PROGRAM_ADDS_AN_ACCOUNT: &str = "TOUTUI_ADD_AN_ACCOUNT";

/// Tells if the program must show the login screen for a new account.
pub fn the_program_adds_an_account() -> bool {
    std::env::var(THE_PROGRAM_ADDS_AN_ACCOUNT)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

/// The mark of the account that the program starts with.
pub const THE_MARK_OF_THE_ACCOUNT_THAT_STARTS: &str = "▶ ";

/// The line of one account in the view of the accounts.
///
/// The line names the account and the address of its server: two accounts of
/// one name on two servers are two different accounts, and the name alone tells
/// them apart for no user. The account that starts holds the mark, and every
/// other line holds two spaces in its place, therefore the names stand in one
/// column.
pub fn the_line_of_an_account(name: &str, address: &str, starts: bool) -> String {
    let mark = if starts {
        THE_MARK_OF_THE_ACCOUNT_THAT_STARTS
    } else {
        "  "
    };

    if address.is_empty() {
        format!("{}{}", mark, name)
    } else {
        format!("{}{} — {}", mark, name, address)
    }
}

/// What the program must do after the user logged out of one account.
#[derive(Debug, PartialEq, Eq)]
pub enum AfterALogOut {
    /// The account that left did not start the program. The view takes its
    /// line away, and nothing else changes.
    TheViewOnly,
    /// The account that left started the program, and this account takes that
    /// work. Every list of the program comes from one account, therefore the
    /// program starts again.
    ThisAccountStarts(String),
    /// The account that left was the one account of the program. The login
    /// screen comes, and it asks for a server, a name, and a password.
    TheLoginScreen,
}

/// The rule of a log out. See T-124.
///
/// A log out of an account that does **not** start changes the view only. A log
/// out of the account that starts leaves the program with no account of a
/// start: the first account that stays takes that work, and with no account at
/// all the login screen comes.
pub fn the_account_after_a_log_out(
    accounts: &[(String, String, bool)],
    the_account_that_left: &str,
) -> AfterALogOut {
    let it_was_the_account_that_starts = accounts
        .iter()
        .any(|(name, _, starts)| name == the_account_that_left && *starts);

    if !it_was_the_account_that_starts {
        return AfterALogOut::TheViewOnly;
    }

    match accounts
        .iter()
        .find(|(name, _, _)| name != the_account_that_left)
    {
        Some((name, _, _)) => AfterALogOut::ThisAccountStarts(name.clone()),
        None => AfterALogOut::TheLoginScreen,
    }
}

/// What a key of the view of the accounts finds on the line of the user. See
/// T-155.
#[derive(Debug, PartialEq, Eq)]
pub enum TheAccountOfTheLine {
    /// The database holds that account. The key does its work.
    ItStays,
    /// A different program of this account removed it. The key changes no row,
    /// and it says the reason.
    ItIsGone,
}

/// Tells if the account of the line of the user stands in the database still.
///
/// **The list of the accounts is the list of one process**: `App::new` reads it
/// one time, and a second program of the account adds and removes a row while
/// that list stands. Every key of the view therefore reads the database before
/// it acts, and it acts on the **name** of its own line — that is the rule of
/// T-142 for the file of the settings, and the rule of T-147 for a line of the
/// queue.
pub fn the_account_of_the_line(
    accounts: &[(String, String, bool)],
    the_name_of_the_line: &str,
) -> TheAccountOfTheLine {
    if accounts
        .iter()
        .any(|(name, _, _)| name == the_name_of_the_line)
    {
        TheAccountOfTheLine::ItStays
    } else {
        TheAccountOfTheLine::ItIsGone
    }
}

/// The sentence of a key that found no account on its line. See T-155.
///
/// The sentence names the account, and it names no key: this program holds no
/// key that gives such an account back (T-118 and T-143).
pub fn the_text_of_an_account_that_is_gone(name: &str) -> String {
    format!(
        "A different program of this account removed the account \"{}\".",
        name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accounts() -> Vec<(String, String, bool)> {
        vec![
            (
                "toutuitest".to_string(),
                "127.0.0.1:13399".to_string(),
                true,
            ),
            (
                "secondtest".to_string(),
                "127.0.0.1:13400".to_string(),
                false,
            ),
        ]
    }

    /// The line names the account and the address, and the mark stands on the
    /// account that starts only.
    #[test]
    fn the_line_holds_the_name_the_address_and_the_mark() {
        assert_eq!(
            the_line_of_an_account("toutuitest", "127.0.0.1:13399", true),
            "▶ toutuitest — 127.0.0.1:13399"
        );
        assert_eq!(
            the_line_of_an_account("secondtest", "127.0.0.1:13400", false),
            "  secondtest — 127.0.0.1:13400"
        );
    }

    /// Every line starts at the same column: the mark holds two cells, and the
    /// line of an account that does not start holds two spaces.
    #[test]
    fn the_names_of_two_accounts_stand_in_one_column() {
        let with = the_line_of_an_account("a", "s", true);
        let without = the_line_of_an_account("a", "s", false);

        assert_eq!(
            with.chars().count(),
            without.chars().count(),
            "the two lines must hold the same number of cells: {:?} and {:?}",
            with,
            without
        );
    }

    /// An account of an older database holds no address. The line then names
    /// the account, and it holds no dash with nothing after it.
    #[test]
    fn a_line_with_no_address_holds_no_dash() {
        assert_eq!(
            the_line_of_an_account("toutuitest", "", false),
            "  toutuitest"
        );
    }

    /// A log out of an account that does not start changes no account of the
    /// start.
    #[test]
    fn a_log_out_of_a_second_account_changes_the_view_only() {
        assert_eq!(
            the_account_after_a_log_out(&accounts(), "secondtest"),
            AfterALogOut::TheViewOnly
        );
    }

    /// A log out of the account that starts gives the start to the account that
    /// stays.
    #[test]
    fn a_log_out_of_the_account_that_starts_gives_the_start_to_the_next_one() {
        assert_eq!(
            the_account_after_a_log_out(&accounts(), "toutuitest"),
            AfterALogOut::ThisAccountStarts("secondtest".to_string())
        );
    }

    /// The account of the line stands in the database, therefore the key does
    /// its work.
    #[test]
    fn an_account_of_the_database_stays() {
        assert_eq!(
            the_account_of_the_line(&accounts(), "secondtest"),
            TheAccountOfTheLine::ItStays
        );
    }

    /// A second program of the account removed the account of the line. The key
    /// must find it gone: it wrote the mark of the start on nobody before
    /// T-155, and the program then showed the login screen at every start.
    #[test]
    fn an_account_that_a_second_program_removed_is_gone() {
        let of_the_disk = vec![(
            "toutuitest".to_string(),
            "127.0.0.1:13399".to_string(),
            true,
        )];

        assert_eq!(
            the_account_of_the_line(&of_the_disk, "secondtest"),
            TheAccountOfTheLine::ItIsGone
        );
    }

    /// The sentence names the account, and it promises no key.
    #[test]
    fn the_sentence_of_an_account_that_is_gone_names_it() {
        let text = the_text_of_an_account_that_is_gone("secondtest");

        assert!(text.contains("secondtest"), "{}", text);
        assert!(
            !text.contains("Press"),
            "the sentence must promise no key: {}",
            text
        );
    }

    /// A log out of the one account of the program leaves no account, therefore
    /// the login screen comes.
    #[test]
    fn a_log_out_of_the_one_account_gives_the_login_screen() {
        let one = vec![(
            "toutuitest".to_string(),
            "127.0.0.1:13399".to_string(),
            true,
        )];

        assert_eq!(
            the_account_after_a_log_out(&one, "toutuitest"),
            AfterALogOut::TheLoginScreen
        );
    }
}
