pub mod crud;
pub mod database_struct;
pub mod migrate;
pub mod the_wait_of_the_disk;

/// Does the work of the disk on a thread of the pool of the blocking work, and
/// it waits for the answer. See T-204.
///
/// **A call of the database that stands on a thread of the runtime stops the
/// screen of the program.** rusqlite waits for a lock of the file, and it holds
/// the thread that calls it: a second program of the account that writes the
/// database (T-140) therefore gives that thread the busy timeout of five
/// seconds. A measurement of 2026-08-14 with `docs/harness/hold_the_lock.py`
/// and `strace`: the three writes of one second of the loop of the playback
/// took a thread of the runtime for 15 seconds, that thread is the driver of
/// the runtime, and the loop of the screen waited on it — **the row of the
/// player, the timer for sleep, and every key of the user stopped for those 15
/// seconds**, and the playback went on.
///
/// The work of this function stands on a thread of the pool of the blocking
/// work of tokio, therefore no driver of the runtime waits for the disk. The
/// caller waits for the answer, and the sequence of two calls of one loop
/// stays: a later position must never reach the disk before an earlier one.
pub async fn the_work_of_the_disk<T, W>(work: W) -> Option<T>
where
    T: Send + 'static,
    W: FnOnce() -> T + Send + 'static,
{
    match tokio::task::spawn_blocking(work).await {
        Ok(answer) => Some(answer),
        Err(why) => {
            log::error!(
                "[the work of the disk] the thread of the disk did not come back: {}",
                why
            );

            None
        }
    }
}

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
