//! The list of server addresses and the health of each address.
//!
//! One Audiobookshelf server can have more than one address. An example is a
//! fast local address and a slow public address. The pool always selects the
//! address that has the most importance and that answers.
//!
//! A low `priority` value gives more importance.

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

/// The health of one address.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Health {
    /// The address answered the last request.
    Up,
    /// The address did not answer. The probe task examines it again.
    Down,
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
    pub fn mark_down(&self, url: &str) {
        self.set_health(url, Health::Down);
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

    /// Forgets the requests of this address that stopped at their time limit.
    ///
    /// **The address answered**, therefore the requests before that answer say
    /// nothing about it now. See T-97.
    pub fn the_address_answered(&self, url: &str) {
        let Some(position) = self.endpoints.iter().position(|e| e.url == url) else {
            return;
        };

        if let Ok(mut timeouts) = self.timeouts.write() {
            if let Some(count) = timeouts.get_mut(position) {
                *count = 0;
            }
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
            .filter(|(_, state)| **state == Health::Down)
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
        pool.mark_down("http://lan");
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
        pool.mark_down("https://wan");
        assert_eq!(pool.next_after("http://lan").unwrap(), "https://backup");
    }

    #[test]
    fn the_active_endpoint_is_none_if_all_endpoints_are_down() {
        let pool = pool();
        pool.mark_down("http://lan");
        pool.mark_down("https://wan");
        pool.mark_down("https://backup");
        assert!(pool.active().is_none());
    }

    /// This test proves the behaviour that the user asked for. The
    /// application returns to the local address when that address works
    /// again.
    #[test]
    fn the_pool_returns_to_the_endpoint_with_more_importance() {
        let pool = pool();
        pool.mark_down("http://lan");
        assert_eq!(pool.active().unwrap(), "https://wan");

        pool.mark_up("http://lan");
        assert_eq!(pool.active().unwrap(), "http://lan");
    }

    #[test]
    fn the_pool_gives_the_down_endpoints_for_the_probe() {
        let pool = pool();
        pool.mark_down("http://lan");
        pool.mark_down("https://backup");

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
