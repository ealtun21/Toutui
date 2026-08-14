pub mod crud;
pub mod database_struct;
pub mod migrate;

/// The program did not read the accounts of its database. See T-199.
///
/// **A fault of the database is not a database with no account.** The old code
/// read the accounts with `if let Ok(...)`, therefore a database that a second
/// program held gave a list of no account: the program then drew the login
/// screen of a first start, and the row of the account stood on the disk all
/// the time.
///
/// The fault has a type of its own, because the words of the user must name the
/// database and not the server: `the_words_of_a_program_that_stops` reads this
/// type. See T-172 for the words of a fault of the server.
#[derive(Debug)]
pub struct TheAccountsDidNotCome(pub String);

impl std::fmt::Display for TheAccountsDidNotCome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The program did not read the accounts of its database: {}",
            self.0
        )
    }
}

impl std::error::Error for TheAccountsDidNotCome {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fault names the database, and it holds what the database said. See
    /// T-199.
    #[test]
    fn the_fault_of_the_accounts_names_the_database() {
        let fault = TheAccountsDidNotCome("database is locked".to_string());

        assert_eq!(
            fault.to_string(),
            "The program did not read the accounts of its database: database is locked"
        );
    }
}
