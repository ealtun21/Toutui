//! The list of server addresses and the health of each address.
//!
//! One Audiobookshelf server can have more than one address. An example is a
//! fast local address and a slow public address. The pool always selects the
//! address that has the most importance and that answers.
//!
//! A low `priority` value gives more importance.

use log::info;
use std::sync::RwLock;

/// One address of a server.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// The base address. It has no slash at the end.
    pub url: String,
    /// A low value gives more importance.
    pub priority: u8,
}

impl Endpoint {
    /// Makes an endpoint. The function removes a slash at the end of the
    /// address, because the request path always starts with a slash.
    pub fn new(url: &str, priority: u8) -> Self {
        Endpoint {
            url: url.trim_end_matches('/').to_string(),
            priority,
        }
    }
}

/// Why an address has the state `Down`. See T-171.
///
/// **The two causes are not the same thing for the user.** An address that
/// takes no connection is a server that is away, and the media of the disk are
/// the road of that user (T-107). An address that answers `500` is a server
/// that stands and that works: every other request of that user comes back, and
/// the key `R` gives the lists of the server again. The header of the program
/// said "the server does not answer" for both of them, and the second sentence
/// is a reason that the program does not have (T-91).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyDown {
    /// No answer came: no machine took the connection, or the answer did not
    /// come in the permitted time.
    ItGaveNoAnswer,
    /// The address answered, and the answer holds a fault of the server.
    ItAnsweredWithAFault,
}

/// The health of one address.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Health {
    /// The address answered the last request.
    Up,
    /// The address did not give a good answer. The probe task examines it
    /// again.
    Down(WhyDown),
}

/// How many requests of one address must stop at their time limit, one after
/// the other, before that address takes the state `Down`. See T-97.
///
/// **A request that stops at its time limit is not evidence that the address is
/// down.** The server does slow work for some requests of a user:
/// `POST /api/podcasts/feed` makes it read a web site, and a scan of a library
/// reads every file. A measurement of 2026-08-11 gave that timeout for the
/// feed, and **every request after it said "No server address answered"** until
/// the probe task ran again, one minute later.
///
/// A connection that no machine takes is a different condition, and one of them
/// gives the state `Down` at once.
const TIMEOUTS_OF_ONE_ADDRESS: u8 = 2;

/// The addresses of one server, in priority sequence.
#[derive(Debug)]
pub struct EndpointPool {
    endpoints: Vec<Endpoint>,
    health: RwLock<Vec<Health>>,
    /// The requests of each address that stopped at their time limit, one
    /// after the other. An answer of the address puts it back to 0.
    timeouts: RwLock<Vec<u8>>,
}

impl EndpointPool {
    /// Makes a pool. The function sorts the endpoints by priority. The
    /// endpoint with the lowest value comes first. All endpoints start with
    /// the state `Up`.
    pub fn new(mut endpoints: Vec<Endpoint>) -> Self {
        endpoints.sort_by_key(|endpoint| endpoint.priority);
        let health = vec![Health::Up; endpoints.len()];

        let timeouts = vec![0; endpoints.len()];

        EndpointPool {
            endpoints,
            health: RwLock::new(health),
            timeouts: RwLock::new(timeouts),
        }
    }

    /// Gives the address that has the most importance and the state `Up`.
    ///
    /// Gives `None` if no address has the state `Up`. The caller then
    /// reports `ApiError::Unreachable`.
    pub fn active(&self) -> Option<String> {
        let health = self.health.read().ok()?;

        self.endpoints
            .iter()
            .zip(health.iter())
            .find(|(_, state)| **state == Health::Up)
            .map(|(endpoint, _)| endpoint.url.clone())
    }

    /// Gives the address for a request of the user.
    ///
    /// The function gives the address that has the most importance and the
    /// state `Up`. **If no address has that state, it gives the address that
    /// has the most importance**, and the request then tries that address.
    ///
    /// **A request must try an address before the program says that no address
    /// answered.** The state `Down` is the answer of an attempt that came
    /// before, and the probe task examines it every 60 seconds only: a
    /// measurement of 2026-08-12 marked the one address of the pool down for a
    /// connection that no machine took, and the program then said "No server
    /// address answered" for **31.6 seconds** while the server answered `curl`
    /// in 1.5 milliseconds. See T-128.
    ///
    /// The function gives `None` for a pool that holds no address.
    pub fn an_address(&self) -> Option<String> {
        self.active()
            .or_else(|| self.endpoints.first().map(|endpoint| endpoint.url.clone()))
    }

    /// Gives the next address that has the state `Up` after the given
    /// address.
    ///
    /// The client uses this function for the second attempt.
    pub fn next_after(&self, url: &str) -> Option<String> {
        let health = self.health.read().ok()?;
        let position = self.endpoints.iter().position(|e| e.url == url)?;

        self.endpoints
            .iter()
            .zip(health.iter())
            .skip(position + 1)
            .find(|(_, state)| **state == Health::Up)
            .map(|(endpoint, _)| endpoint.url.clone())
    }

    /// Records that an address does not answer.
    ///
    /// `reason` says what the program measured. **The log must hold the moment
    /// that the program stopped to use an address**: the measurement of T-128
    /// read the fault of the live task and it made a guess, because no line of
    /// the log named the address. See T-128.
    pub fn mark_down(&self, url: &str, reason: &str, why: WhyDown) {
        if let Some(Health::Down(before)) = self.health_of(url) {
            // **The cause can change while the address stays down.** A server
            // that answers `500` and that then goes away must not keep the
            // words of a server that answers: the header says what the program
            // measured last. The log holds the first fault alone, because a
            // program that runs for days must not fill it. See T-171.
            if before != why {
                self.set_health(url, Health::Down(why));
            }

            return;
        }

        info!(
            "[api] The program does not use the address {} now: {}. It examines \
             that address every {} seconds, and a request of the user tries it.",
            url,
            reason,
            super::probe::PROBE_INTERVAL.as_secs()
        );

        self.set_health(url, Health::Down(why));
    }

    /// Tells that no address has the state `Up`, and that every address that
    /// went down answered with a fault of the server. See T-171.
    ///
    /// **A server that answers `500` for one path is not a server that is
    /// away.** A measurement of 2026-08-14 with `docs/harness/one_path_fails.py`
    /// gave the status 500 to `GET /api/libraries/:id/authors` alone: the header
    /// of the program then said `⚠ toutuitest: the server does not answer` and
    /// `🔗 127.0.0.1:13500 does not answer` for 10.5 seconds, while `curl` got
    /// an answer of that same address in 1.4 milliseconds. The header offers the
    /// media of the disk to that user (T-107), and the key `R` gives the lists
    /// of the server. See T-91 and T-170.
    ///
    /// A pool with no address gives `false`.
    pub fn every_address_answers_with_a_fault(&self) -> bool {
        let Ok(health) = self.health.read() else {
            return false;
        };

        !health.is_empty()
            && health
                .iter()
                .all(|state| *state == Health::Down(WhyDown::ItAnsweredWithAFault))
    }

    /// Records one request of this address that stopped at its time limit.
    ///
    /// The function gives `true` when the address must take the state `Down`:
    /// **one such request is not evidence that an address is down**, and
    /// `TIMEOUTS_OF_ONE_ADDRESS` of them, one after the other, are. See T-97.
    pub fn a_request_stopped_at_its_time_limit(&self, url: &str) -> bool {
        let Some(position) = self.endpoints.iter().position(|e| e.url == url) else {
            return true;
        };

        let Ok(mut timeouts) = self.timeouts.write() else {
            return true;
        };

        let Some(count) = timeouts.get_mut(position) else {
            return true;
        };

        *count = count.saturating_add(1);

        *count >= TIMEOUTS_OF_ONE_ADDRESS
    }

    /// Forgets the requests of this address that stopped at their time limit,
    /// and it gives the address the state `Up`.
    ///
    /// **The address answered**, therefore the requests before that answer say
    /// nothing about it now (T-97), and the state of that address is not `Down`
    /// (T-128).
    pub fn the_address_answered(&self, url: &str) {
        let Some(position) = self.endpoints.iter().position(|e| e.url == url) else {
            return;
        };

        if let Ok(mut timeouts) = self.timeouts.write() {
            if let Some(count) = timeouts.get_mut(position) {
                *count = 0;
            }
        }

        if matches!(self.health_of(url), Some(Health::Down(_))) {
            info!(
                "[api] The address {} answers again, therefore the program uses it.",
                url
            );
            self.set_health(url, Health::Up);
        }
    }

    /// Records that an address answers again.
    pub fn mark_up(&self, url: &str) {
        self.set_health(url, Health::Up);
    }

    /// Gives the addresses that have the state `Down`. The probe task
    /// examines these addresses.
    pub fn down_urls(&self) -> Vec<String> {
        let health = match self.health.read() {
            Ok(health) => health,
            Err(_) => return Vec::new(),
        };

        self.endpoints
            .iter()
            .zip(health.iter())
            .filter(|(_, state)| matches!(state, Health::Down(_)))
            .map(|(endpoint, _)| endpoint.url.clone())
            .collect()
    }

    /// Gives the number of addresses.
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Tells if the pool has no address.
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    /// Gives the health of one address, and `None` for an address that the pool
    /// does not hold.
    fn health_of(&self, url: &str) -> Option<Health> {
        let position = self.endpoints.iter().position(|e| e.url == url)?;
        let health = self.health.read().ok()?;

        health.get(position).copied()
    }

    /// Writes a new health state for one address.
    fn set_health(&self, url: &str, state: Health) {
        if let Some(position) = self.endpoints.iter().position(|e| e.url == url) {
            if let Ok(mut health) = self.health.write() {
                health[position] = state;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> EndpointPool {
        EndpointPool::new(vec![
            Endpoint::new("http://lan", 0),
            Endpoint::new("https://wan", 1),
            Endpoint::new("https://backup", 2),
        ])
    }

    /// **A server that answers with a fault is not a server that is away.**
    /// The header of the program read `active()` alone, therefore it said "the
    /// server does not answer" for an address that answered in 1.4
    /// milliseconds. See T-171.
    #[test]
    fn the_pool_knows_that_every_address_answers_with_a_fault() {
        let pool = pool();

        // Every address is well.
        assert!(!pool.every_address_answers_with_a_fault());

        pool.mark_down("http://lan", "a fault", WhyDown::ItAnsweredWithAFault);

        // One address of three answers with a fault, and two of them are well:
        // the header says "Connected", because `active` holds an address.
        assert!(!pool.every_address_answers_with_a_fault());
        assert!(pool.active().is_some());

        pool.mark_down("https://wan", "a fault", WhyDown::ItAnsweredWithAFault);
        pool.mark_down("https://backup", "a fault", WhyDown::ItAnsweredWithAFault);

        assert!(pool.active().is_none());
        assert!(pool.every_address_answers_with_a_fault());

        // **One address that gives no answer at all takes the rule away.** The
        // program cannot say that the server answers, because one address of it
        // does not.
        let mixed = EndpointPool::new(vec![
            Endpoint::new("http://lan", 0),
            Endpoint::new("https://wan", 1),
            Endpoint::new("https://backup", 2),
        ]);
        mixed.mark_down("http://lan", "a fault", WhyDown::ItAnsweredWithAFault);
        mixed.mark_down("https://wan", "no answer", WhyDown::ItGaveNoAnswer);
        mixed.mark_down("https://backup", "a fault", WhyDown::ItAnsweredWithAFault);

        assert!(mixed.active().is_none());
        assert!(!mixed.every_address_answers_with_a_fault());

        // A pool with no address says nothing of a fault.
        assert!(!EndpointPool::new(Vec::new()).every_address_answers_with_a_fault());

        // **The cause changes while the address stays down.** The server
        // answered with a fault, and it then went away: the header must say the
        // words of a server that is away, and not the words of T-171.
        pool.mark_down("http://lan", "no answer", WhyDown::ItGaveNoAnswer);
        assert!(!pool.every_address_answers_with_a_fault());

        // The address answers again, therefore the words of the fault go away.
        pool.the_address_answered("http://lan");
        assert!(!pool.every_address_answers_with_a_fault());
    }

    /// **One request that stops at its time limit does not take an address
    /// away.** The server does slow work for some requests of a user: the
    /// measurement of 2026-08-11 gave a timeout of `POST /api/podcasts/feed`,
    /// and every request after it said "No server address answered". See T-97.
    #[test]
    fn one_request_that_stops_at_its_time_limit_keeps_the_address() {
        let pool = pool();

        // The first one says nothing about the address.
        assert!(!pool.a_request_stopped_at_its_time_limit("http://lan"));

        // The second one, with no answer between them, says it.
        assert!(pool.a_request_stopped_at_its_time_limit("http://lan"));

        // An answer of the address forgets the two of them.
        pool.the_address_answered("http://lan");
        assert!(!pool.a_request_stopped_at_its_time_limit("http://lan"));

        // Each address holds its own count.
        assert!(!pool.a_request_stopped_at_its_time_limit("https://wan"));
        assert!(pool.a_request_stopped_at_its_time_limit("https://wan"));

        // An address that the pool does not hold gives the state `Down` at
        // once: the program knows nothing of it.
        assert!(pool.a_request_stopped_at_its_time_limit("http://nothing"));
    }

    /// **A request must try an address before the program says that no address
    /// answered.** `active` gives nothing when every address holds the state
    /// `Down`, and `an_address` then gives the address of the most importance.
    /// See T-128.
    #[test]
    fn an_address_gives_an_address_that_holds_the_state_down() {
        let pool = pool();
        assert_eq!(pool.an_address().unwrap(), "http://lan");

        pool.mark_down(
            "http://lan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        assert_eq!(pool.an_address().unwrap(), "https://wan");

        pool.mark_down(
            "https://wan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        pool.mark_down(
            "https://backup",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );

        assert!(pool.active().is_none());
        assert_eq!(pool.an_address().unwrap(), "http://lan");
    }

    /// A pool with no address gives nothing, and the caller then reports
    /// `ApiError::Unreachable`. See T-128.
    #[test]
    fn a_pool_with_no_address_gives_no_address_to_a_request() {
        let pool = EndpointPool::new(Vec::new());

        assert!(pool.an_address().is_none());
    }

    /// **An address that answered holds the state `Up`.** The request of the
    /// user is the newest measurement of that address, and the probe task waits
    /// 60 seconds. See T-128.
    #[test]
    fn an_address_that_answered_holds_the_state_up() {
        let pool = pool();
        pool.mark_down(
            "http://lan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        assert_eq!(pool.active().unwrap(), "https://wan");

        pool.the_address_answered("http://lan");

        assert_eq!(pool.active().unwrap(), "http://lan");
        assert!(pool.down_urls().is_empty());
    }

    #[test]
    fn the_pool_sorts_by_priority() {
        let pool = EndpointPool::new(vec![
            Endpoint::new("https://wan", 5),
            Endpoint::new("http://lan", 0),
        ]);
        assert_eq!(pool.active().unwrap(), "http://lan");
    }

    #[test]
    fn the_active_endpoint_has_the_most_importance() {
        assert_eq!(pool().active().unwrap(), "http://lan");
    }

    #[test]
    fn a_down_endpoint_is_not_active() {
        let pool = pool();
        pool.mark_down(
            "http://lan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        assert_eq!(pool.active().unwrap(), "https://wan");
    }

    #[test]
    fn the_pool_gives_the_next_endpoint_after_a_failure() {
        let pool = pool();
        assert_eq!(pool.next_after("http://lan").unwrap(), "https://wan");
        assert_eq!(pool.next_after("https://wan").unwrap(), "https://backup");
    }

    #[test]
    fn the_last_endpoint_has_no_next_endpoint() {
        let pool = pool();
        assert!(pool.next_after("https://backup").is_none());
    }

    #[test]
    fn next_after_does_not_give_a_down_endpoint() {
        let pool = pool();
        pool.mark_down(
            "https://wan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        assert_eq!(pool.next_after("http://lan").unwrap(), "https://backup");
    }

    #[test]
    fn the_active_endpoint_is_none_if_all_endpoints_are_down() {
        let pool = pool();
        pool.mark_down(
            "http://lan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        pool.mark_down(
            "https://wan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        pool.mark_down(
            "https://backup",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        assert!(pool.active().is_none());
    }

    /// This test proves the behaviour that the user asked for. The
    /// application returns to the local address when that address works
    /// again.
    #[test]
    fn the_pool_returns_to_the_endpoint_with_more_importance() {
        let pool = pool();
        pool.mark_down(
            "http://lan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        assert_eq!(pool.active().unwrap(), "https://wan");

        pool.mark_up("http://lan");
        assert_eq!(pool.active().unwrap(), "http://lan");
    }

    #[test]
    fn the_pool_gives_the_down_endpoints_for_the_probe() {
        let pool = pool();
        pool.mark_down(
            "http://lan",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );
        pool.mark_down(
            "https://backup",
            "the measurement of the test",
            WhyDown::ItGaveNoAnswer,
        );

        let mut down = pool.down_urls();
        down.sort();
        assert_eq!(down, vec!["http://lan", "https://backup"]);
    }

    #[test]
    fn a_pool_with_one_endpoint_works() {
        let pool = EndpointPool::new(vec![Endpoint::new("http://only", 0)]);
        assert_eq!(pool.active().unwrap(), "http://only");
        assert!(pool.next_after("http://only").is_none());
    }
}
