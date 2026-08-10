//! The background task that examines the addresses that do not answer.
//!
//! Without this task, an address stays in the state `Down` for ever. With
//! this task, the application returns to the address that has the most
//! importance automatically. An example is the local address when the user
//! comes home.

use super::endpoint::EndpointPool;
use super::ApiClient;
use log::info;
use std::sync::Arc;
use std::time::Duration;

/// The time between two probes.
pub const PROBE_INTERVAL: Duration = Duration::from_secs(60);

/// Examines each address that has the state `Down` one time.
///
/// The function sends `GET /ping` to each such address. Audiobookshelf
/// answers this path without a token. If an address answers, the function
/// gives it the state `Up`.
pub async fn probe_once(http: &reqwest::Client, pool: &EndpointPool) {
    for url in pool.down_urls() {
        let is_up = http
            .get(format!("{}/ping", url))
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false);

        if is_up {
            info!("[probe] The address {} answers again.", url);
            pool.mark_up(&url);
        }
    }
}

/// Starts the probe task. The task runs until the program stops.
pub fn spawn_probe_task(client: Arc<ApiClient>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let pool = client.pool();

        loop {
            tokio::time::sleep(PROBE_INTERVAL).await;
            probe_once(client.http(), &pool).await;
        }
    })
}
