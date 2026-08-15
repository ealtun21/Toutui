//! The live messages of the server. See T-47.
//!
//! Audiobookshelf sends every change of every client over socket.io. A change
//! of a different client came to Toutui at the next `R` only.
//!
//! socket.io has two transports, and the client chooses. The transport
//! `websocket` needs a library, and the two crates of socket.io both bring
//! `native-tls`. The rule of T-20 refuses that. **The transport `polling` is
//! plain HTTP**, therefore `reqwest` does the whole work and the program needs
//! no new dependency.
//!
//! The flow, measured against an Audiobookshelf 2.36.0 on 2026-08-11:
//!
//! | Step | The request | The answer |
//! |---|---|---|
//! | 1 | `GET /socket.io/?EIO=4&transport=polling` | `0{"sid":"...","pingInterval":25000,...}` |
//! | 2 | `POST` the body `40` | `ok` |
//! | 3 | `GET` | `40{"sid":"..."}` |
//! | 4 | `POST` the body `42["auth","<the token>"]` | `ok` |
//! | 5 | `GET` | `42["user_online",{...}]` and `42["init",{...}]` |
//!
//! Three rules of that transport, and the measurement found all three:
//!
//! 1. **The server sends `2`, and the client must answer `3`.** The period is
//!    `pingInterval`. A client that does not answer gets `1` (close), and every
//!    later request of that identity gives `400`.
//! 2. **One `GET` at a time for one identity.** A `POST` beside the `GET` is
//!    correct.
//! 3. **One answer can hold more than one packet.** The separator is the byte
//!    `0x1e`.
//!
//! **The message `init` holds no `mediaProgress`.** A measurement of 2026-08-14
//! against the same server shows that the positions of the account come with the
//! `user_updated` of the step 5, and never with `init`. The two lists of the
//! screen take the place of the lists before them (T-184), therefore that
//! difference decides a rule: see `the_message_holds_the_positions`.
//!
//! **A message can hold a secret.** `user_updated` carries the account of the
//! user, and that object holds a new token. Therefore this module writes the
//! name of a message in the log, and never the body.
//!
//! Every function that reads a packet is pure. Therefore a test examines the
//! whole protocol with no server and no network.

use crate::api::client::endpoint::EndpointPool;
use log::{info, warn};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

/// The address of the handshake. The transport and the version of the protocol
/// stand in the address, therefore every request carries them.
pub const HANDSHAKE_PATH: &str = "/socket.io/?EIO=4&transport=polling";

/// The separator of two packets inside one answer.
pub const SEPARATOR: char = '\u{1e}';

/// The time to wait for one `GET` of the poll.
///
/// The server answers with a ping every `pingInterval`, therefore 25 seconds
/// with an Audiobookshelf 2.36.0. This value gives room for a slow answer, and
/// it stays short enough to find a connection that died.
const POLL_TIMEOUT: Duration = Duration::from_secs(60);

/// The time to wait for a connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The time to wait after a connection that ended, before a new one.
const WAIT_BEFORE_AGAIN: Duration = Duration::from_secs(10);

/// The longest time between two attempts. See T-61.
///
/// A server that gives no socket.io answers every attempt with a fault. The wait
/// therefore becomes longer after each fault, and it stops at this value: one
/// attempt of ten minutes costs the server almost nothing, and a user who mends
/// their server waits ten minutes at the most for the live messages.
const LONGEST_WAIT: Duration = Duration::from_secs(600);

/// The answer of the handshake.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handshake {
    /// The identity of the connection. Every later request carries it.
    pub sid: String,
    /// The period of the ping of the server, in milliseconds.
    #[serde(default)]
    pub ping_interval: u64,
    /// The time that the server waits for the pong, in milliseconds.
    #[serde(default)]
    pub ping_timeout: u64,
}

/// One packet of the protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    /// `0` and the answer of the handshake.
    Open(Handshake),
    /// `1`. The server closed the connection.
    Close,
    /// `2`. The client must answer with a pong.
    Ping,
    /// `3`. The answer of the client to a ping.
    Pong,
    /// `40`. The server accepted the connection of socket.io.
    Connected,
    /// `42` and a list of two values: the name and the body.
    Event {
        /// The name that the server gives the message.
        name: String,
        /// The body of the message.
        body: serde_json::Value,
    },
    /// A packet that this program does not use.
    Other(String),
}

/// The position of one media, in the form that the mark of a line needs.
///
/// The two values have the exact form of
/// `collect_progress_percentage_book` and of `collect_is_finished_book`,
/// therefore the screen shows a live value and a value of a request in the
/// same way. See T-44.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    /// The part of the media that the user heard, in percent, as a text.
    pub percent: String,
    /// `Finished` or `Not finished`.
    pub finished: String,
}

/// One row of `mediaProgress` of the message `user_updated`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressRow {
    #[serde(default)]
    library_item_id: String,
    /// The part that the user heard, between 0 and 1.
    #[serde(default)]
    progress: f64,
    #[serde(default)]
    is_finished: bool,
    /// The identity of the episode, for a podcast. A book gives `null`.
    #[serde(default)]
    episode_id: Option<String>,
    /// The user asked the server to keep this media away from the shelf of
    /// Continue Listening. The key `N` of the program writes this value.
    #[serde(default)]
    hide_from_continue_listening: bool,
}

/// The messages that make every list of the screen old.
///
/// A change of the metadata of an item changes the title, the author, and the
/// cover of that item. Those values stand in the list of the library, in the
/// list of a series, and in the Home view. Therefore the program cannot correct
/// one line: the user must ask the server again with the key `R`.
///
/// `user_updated` is **not** in this list. The program itself sends the position
/// of the playback every ten seconds, and the server answers every such request
/// with `user_updated` to the client that sent it. That message would then keep
/// a notice on the screen for ever. The position of the message goes to the mark
/// of the line instead, and that needs no request. See `Progress`.
const MESSAGES_OF_THE_LIBRARY: [&str; 9] = [
    "item_updated",
    "items_updated",
    "item_added",
    "items_added",
    "item_removed",
    "items_removed",
    "library_updated",
    "library_added",
    "library_removed",
];

/// Tells if a message makes the lists of the screen old.
pub fn the_library_changed(name: &str) -> bool {
    MESSAGES_OF_THE_LIBRARY.contains(&name)
}

/// Gives the time to wait after a number of faults, one after the other.
///
/// The first fault waits `WAIT_BEFORE_AGAIN`, and each fault after it doubles
/// that time to `LONGEST_WAIT`. A connection that opened gives the count the
/// value 0, therefore a server that answers again gives the live messages back at
/// once.
///
/// **A server that gives no socket.io must not get a request every ten seconds
/// for ever.** A measurement on 2026-08-11 gave 6 requests in 65 seconds with the
/// old rule, and that is 8640 requests in one day. See T-61.
///
/// The function is pure, therefore a test needs no server and no clock.
pub fn wait_after_the_faults(faults: u32) -> Duration {
    if faults <= 1 {
        return WAIT_BEFORE_AGAIN;
    }

    // The value 2 gives two times the first wait, and the value n gives 2^(n-1)
    // times it. A large number of faults would make that product too large for
    // the type, therefore the count stops at 20.
    let steps = faults.saturating_sub(1).min(20);
    let seconds = WAIT_BEFORE_AGAIN.as_secs().saturating_mul(1 << steps);

    Duration::from_secs(seconds).min(LONGEST_WAIT)
}

/// Reads every packet of one answer.
///
/// One answer can hold more than one packet, and the separator is the byte
/// `0x1e`. A part that this function cannot read gives `Packet::Other`, because
/// a message of a newer server must not stop the connection.
pub fn packets_of_the_body(body: &str) -> Vec<Packet> {
    body.split(SEPARATOR)
        .filter(|part| !part.is_empty())
        .map(packet_of_the_text)
        .collect()
}

/// Reads one packet.
fn packet_of_the_text(text: &str) -> Packet {
    let mut letters = text.chars();

    match letters.next() {
        Some('0') => match serde_json::from_str::<Handshake>(letters.as_str()) {
            Ok(hand) => Packet::Open(hand),
            Err(_) => Packet::Other(text.to_string()),
        },
        Some('1') => Packet::Close,
        Some('2') => Packet::Ping,
        Some('3') => Packet::Pong,
        // The packets of socket.io start with `4`, and the second letter gives
        // the kind.
        Some('4') => match letters.next() {
            Some('0') => Packet::Connected,
            Some('2') => match event_of_the_text(letters.as_str()) {
                Some(packet) => packet,
                None => Packet::Other(text.to_string()),
            },
            _ => Packet::Other(text.to_string()),
        },
        _ => Packet::Other(text.to_string()),
    }
}

/// Reads the list of two values of a message.
///
/// The server sends `["<the name>",<the body>]`. A message with no body gives
/// `Value::Null`, and that is not a fault.
fn event_of_the_text(text: &str) -> Option<Packet> {
    let list: Vec<serde_json::Value> = serde_json::from_str(text).ok()?;
    let name = list.first()?.as_str()?.to_string();
    let body = list.get(1).cloned().unwrap_or(serde_json::Value::Null);

    Some(Packet::Event { name, body })
}

/// Makes the body that sends the token to the server.
///
/// The token stands in the body, and never in the address. An address goes in
/// the log of the server of the user.
pub fn auth_message(token: &str) -> String {
    format!(
        "42[\"auth\",{}]",
        serde_json::Value::String(token.to_string())
    )
}

/// Gives the address of the poll of one identity.
///
/// The identity comes from the server, therefore this function examines it. An
/// identity of engine.io holds the letters of base64 of an address only. A
/// value with a different letter gives no address, and the connection then
/// starts again.
pub fn poll_path(sid: &str) -> Option<String> {
    if sid.is_empty() {
        return None;
    }

    let is_safe = sid
        .chars()
        .all(|letter| letter.is_ascii_alphanumeric() || letter == '-' || letter == '_');

    if !is_safe {
        return None;
    }

    Some(format!("{}&sid={}", HANDSHAKE_PATH, sid))
}

/// Tells if a message holds the list of the positions of the account.
///
/// **`progress_of_the_user` and `the_media_away_from_continue_listening` cannot
/// answer this question**: each of them gives a list of no row for a message that
/// holds no `mediaProgress`, and for a message that holds a `mediaProgress` of no
/// row. The two conditions need two different answers: the first one says nothing
/// of the positions, and the second one says that no media of the account holds a
/// position. See T-184.
///
/// The function is pure, therefore a test needs no server.
pub fn the_message_holds_the_positions(body: &serde_json::Value) -> bool {
    body.get("mediaProgress")
        .and_then(|value| value.as_array())
        .is_some()
}

/// Gives the position of every media of the message `user_updated`.
///
/// The message carries the whole account of the user. This function takes the
/// positions only, therefore the token of that message goes nowhere.
///
/// **A row of a podcast names an episode, and this function keeps it** (T-228).
/// The Home view of a library of podcasts holds one line for one episode, and
/// that line reads this list: a message that gave the rows of the books alone
/// left the mark of every episode at the value of the request of the start for
/// ever. The value is the key of `crate::logic::live::the_key_of_the_media`,
/// because the identity of the item names every episode of one podcast (T-223).
/// That is the value of `the_media_away_from_continue_listening` below.
pub fn progress_of_the_user(body: &serde_json::Value) -> Vec<(String, Progress)> {
    let Some(rows) = body.get("mediaProgress").and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    rows.iter()
        .filter_map(|row| serde_json::from_value::<ProgressRow>(row.clone()).ok())
        .filter(|row| !row.library_item_id.is_empty())
        .map(|row| {
            let progress = Progress {
                percent: format!("{}", (row.progress * 100.0).round() as i64),
                finished: if row.is_finished {
                    "Finished".to_string()
                } else {
                    "Not finished".to_string()
                },
            };

            let key = crate::logic::live::the_key_of_the_media(
                &row.library_item_id,
                row.episode_id.as_deref(),
            );

            (key, progress)
        })
        .collect()
}

/// Gives the media that must not stand on the shelf of Continue Listening.
///
/// The server keeps two values away from that shelf: a media that the user
/// finished, and a media that the user hid with `hideFromContinueListening`.
/// The shelf of a request holds neither of them, therefore a line of the screen
/// must go away when a message gives one of the two. See T-66.
///
/// The message carries the whole account, therefore this list is complete: a
/// media that is absent from it belongs on the shelf again.
///
/// **A row of a podcast names an episode, and this function keeps it** (T-226).
/// A line of the Home view of a library of podcasts is one episode, therefore a
/// row of an episode that the user finished or hid must take that line away.
/// The value is the key of `crate::logic::live::the_key_of_the_media`, because
/// the identity of the item names every episode of one podcast (T-223).
pub fn the_media_away_from_continue_listening(body: &serde_json::Value) -> Vec<String> {
    let Some(rows) = body.get("mediaProgress").and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    rows.iter()
        .filter_map(|row| serde_json::from_value::<ProgressRow>(row.clone()).ok())
        .filter(|row| !row.library_item_id.is_empty())
        .filter(|row| row.is_finished || row.hide_from_continue_listening)
        .map(|row| {
            crate::logic::live::the_key_of_the_media(
                &row.library_item_id,
                row.episode_id.as_deref(),
            )
        })
        .collect()
}

/// Follows the messages of the server, and it never gives up.
///
/// The task holds its own HTTP client. The client of the application changes
/// the address when one address does not answer, and an identity of socket.io
/// belongs to one address. Therefore this task asks the pool for an address one
/// time for each connection, and it keeps that address while the connection
/// lives.
pub fn spawn_the_live_task(pool: Arc<EndpointPool>, token: String) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let http = match reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(POLL_TIMEOUT)
            .build()
        {
            Ok(http) => http,
            Err(error) => {
                warn!("[live] The client did not start: {}", error);
                crate::logic::live::keep(crate::logic::live::State::Fault(
                    "the client did not start".to_string(),
                ));
                return;
            }
        };

        // The number of faults, one after the other. A connection that opened
        // gives it the value 0. See T-61.
        let mut faults: u32 = 0;

        loop {
            match pool.active() {
                Some(base) => {
                    crate::logic::live::keep(crate::logic::live::State::Waiting);

                    let attempt = one_connection(&http, &base, &token).await;

                    if attempt.opened {
                        faults = 0;
                    } else {
                        faults = faults.saturating_add(1);
                    }

                    if the_address_is_down(attempt.opened, attempt.fault.as_deref()) {
                        // **The program must not say "Connected" for a server
                        // that no address reaches.** This task tries again every
                        // few seconds, therefore it knows first. The probe task
                        // gives the address back. See T-107.
                        // The socket of this task did not open, therefore no
                        // answer came at all. See T-171.
                        pool.mark_down(
                            &base,
                            THE_PROGRAM_CANNOT_CONNECT,
                            crate::api::client::endpoint::WhyDown::ItGaveNoAnswer,
                        );
                    }

                    if let Some(text) = attempt.fault {
                        // The first fault of a server tells the user something.
                        // The tenth fault of the same server tells nothing, and
                        // it fills the log of a program that runs for days.
                        if faults <= 1 {
                            info!("[live] The connection ended: {}", text);
                        }

                        crate::logic::live::keep(crate::logic::live::State::Fault(text));
                    }
                }
                None => {
                    faults = faults.saturating_add(1);

                    crate::logic::live::keep(crate::logic::live::State::Fault(
                        "no address of the server answers".to_string(),
                    ));
                }
            }

            let wait = wait_after_the_faults(faults);

            if faults == 2 {
                info!(
                    "[live] The server gives no live message. The program tries \
                     again, and it waits longer after each fault, to {} seconds.",
                    LONGEST_WAIT.as_secs()
                );
            }

            tokio::time::sleep(wait).await;
        }
    })
}

/// The fault of a connection that no machine took.
///
/// **A connection that no machine takes is evidence that the address is down**,
/// and a request that stops at its time limit is not (T-97). Therefore the live
/// task compares with this text, and it marks the address down for this fault
/// only. See T-107.
pub const THE_PROGRAM_CANNOT_CONNECT: &str = "the program cannot connect to the server";

/// Tells if this attempt of the live connection is evidence that the address is
/// down.
///
/// A connection that opened says nothing: the server took the handshake, and a
/// later fault belongs to that one connection. A time limit says nothing
/// either: the server does slow work for some requests of a user (T-97). See
/// T-107.
pub fn the_address_is_down(opened: bool, fault: Option<&str>) -> bool {
    !opened && fault == Some(THE_PROGRAM_CANNOT_CONNECT)
}

/// What one attempt of a connection gave. See T-61.
pub struct Attempt {
    /// The connection opened: the handshake came, and the server took the token.
    /// The loop then waits the short time, and not the long one.
    pub opened: bool,
    /// The reason why the connection ended, if it ended with a fault.
    pub fault: Option<String>,
}

/// Makes one connection, and it reads the messages while that connection lives.
async fn one_connection(http: &reqwest::Client, base: &str, token: &str) -> Attempt {
    let mut opened = false;

    match one_connection_or_a_fault(http, base, token, &mut opened).await {
        Ok(()) => Attempt {
            opened,
            fault: None,
        },
        Err(text) => Attempt {
            opened,
            fault: Some(text),
        },
    }
}

/// The work of one connection. `opened` becomes true when the connection is
/// open, therefore the caller knows a fault of the handshake from a fault of a
/// connection that lived.
async fn one_connection_or_a_fault(
    http: &reqwest::Client,
    base: &str,
    token: &str,
    opened: &mut bool,
) -> Result<(), String> {
    let hand = handshake(http, base).await?;
    let path = poll_path(&hand.sid).ok_or("the server gave an identity that is not safe")?;
    let address = format!("{}{}", base, path);

    // The packet `40` opens the namespace of socket.io. The packet of the
    // token comes after it, and the server then sends `init`.
    send(http, &address, "40").await?;
    send(http, &address, &auth_message(token)).await?;

    info!(
        "[live] The connection is open. The ping comes every {} ms.",
        hand.ping_interval
    );

    *opened = true;
    crate::logic::live::keep(crate::logic::live::State::Ready);

    loop {
        let body = http
            .get(&address)
            .send()
            .await
            .map_err(|error| short_text(&error))?
            .text()
            .await
            .map_err(|error| short_text(&error))?;

        for packet in packets_of_the_body(&body) {
            match packet {
                // The server asks, and the client must answer. A client that
                // does not answer loses the connection.
                Packet::Ping => send(http, &address, "3").await?,
                Packet::Close => {
                    info!("[live] The server closed the connection.");
                    return Ok(());
                }
                Packet::Event { name, body } => take_the_message(&name, &body),
                Packet::Open(_) | Packet::Connected | Packet::Pong | Packet::Other(_) => {}
            }
        }
    }
}

/// Asks for the handshake, and it reads the identity of the connection.
async fn handshake(http: &reqwest::Client, base: &str) -> Result<Handshake, String> {
    let body = http
        .get(format!("{}{}", base, HANDSHAKE_PATH))
        .send()
        .await
        .map_err(|error| short_text(&error))?
        .text()
        .await
        .map_err(|error| short_text(&error))?;

    packets_of_the_body(&body)
        .into_iter()
        .find_map(|packet| match packet {
            Packet::Open(hand) => Some(hand),
            _ => None,
        })
        .ok_or_else(|| "the server gave no handshake".to_string())
}

/// Sends one packet to the server.
async fn send(http: &reqwest::Client, address: &str, body: &str) -> Result<(), String> {
    let answer = http
        .post(address)
        .header(reqwest::header::CONTENT_TYPE, "text/plain;charset=UTF-8")
        .body(body.to_string())
        .send()
        .await
        .map_err(|error| short_text(&error))?;

    if !answer.status().is_success() {
        return Err(format!("the server answered {}", answer.status().as_u16()));
    }

    Ok(())
}

/// Gives a short text of a fault of a request.
///
/// The text of `reqwest` holds the whole address. The address of the server of
/// the user must not go in the log, therefore this function gives the kind of
/// the fault only.
fn short_text(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "the server did not answer in time".to_string()
    } else if error.is_connect() {
        THE_PROGRAM_CANNOT_CONNECT.to_string()
    } else if let Some(status) = error.status() {
        format!("the server answered {}", status.as_u16())
    } else {
        "the request failed".to_string()
    }
}

/// Puts one message in the box that the screen reads.
///
/// This function writes the name of the message in the log, and never the body.
/// `user_updated` carries a new token of the user.
fn take_the_message(name: &str, body: &serde_json::Value) {
    if name == "user_updated" || name == "init" {
        // **A message that holds no list of the positions says nothing of the
        // positions.** The two lists below take the place of the lists that came
        // before them, therefore a message of another shape must not empty them.
        //
        // **The message `init` is that message.** A measurement of 2026-08-14
        // against an Audiobookshelf 2.36.0 shows that `init` carries no
        // `mediaProgress` at all: the positions come with the `user_updated` that
        // follows it. A connection that starts again after a fault would
        // therefore take every position of the screen away with no such rule.
        // See T-184.
        if !the_message_holds_the_positions(body) {
            info!(
                "[live] {}: the message holds no list of the positions. The lists of \
                 the screen stay.",
                name
            );
            return;
        }

        // **The message holds the whole account, therefore each of these two
        // lists takes the place of the list that came before it** (T-66 and
        // T-184). A list of no row is an answer too: an account whose media hold
        // no position holds no line of the shelf of Continue Listening either.
        let rows = progress_of_the_user(body);
        info!("[live] {}: the position of {} media.", name, rows.len());
        crate::logic::live::note_the_progress(rows);

        let away = the_media_away_from_continue_listening(body);
        info!(
            "[live] {} media of the account stay away from Continue Listening.",
            away.len()
        );
        crate::logic::live::note_the_media_away_from_continue_listening(away);

        return;
    }

    if the_library_changed(name) {
        info!("[live] {}: the lists of the screen are old.", name);
        crate::logic::live::note_that_the_lists_are_old();
    }

    // The queue of the episodes that the server downloads moved. The view of
    // that queue asks the server again, and it needs no key of the user. See
    // T-81.
    if crate::logic::the_downloads::the_queue_of_the_downloads_changed(name) {
        info!("[live] {}: the queue of the downloads changed.", name);
        crate::logic::the_downloads::note_that_the_queue_changed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact answer of the handshake of an Audiobookshelf 2.36.0, of the
    /// measurement of 2026-08-11.
    const HANDSHAKE_OF_THE_SERVER: &str = "0{\"sid\":\"HtEDXf_uhkTPO7cnAAAA\",\"upgrades\":[\"websocket\"],\"pingInterval\":25000,\"pingTimeout\":20000,\"maxPayload\":1000000}";

    #[test]
    fn the_handshake_gives_the_identity_and_the_period_of_the_ping() {
        match packets_of_the_body(HANDSHAKE_OF_THE_SERVER).as_slice() {
            [Packet::Open(hand)] => {
                assert_eq!(hand.sid, "HtEDXf_uhkTPO7cnAAAA");
                assert_eq!(hand.ping_interval, 25000);
                assert_eq!(hand.ping_timeout, 20000);
            }
            other => panic!("the answer must give one open packet: {:?}", other),
        }
    }

    /// One answer holds more than one packet, and the separator is the byte
    /// `0x1e`. A reader that takes the whole body as one packet loses every
    /// message after the first. See T-47.
    #[test]
    fn one_answer_holds_more_than_one_message() {
        let body = format!(
            "42[\"user_online\",{{\"id\":\"a\"}}]{}42[\"init\",{{\"userId\":\"a\"}}]",
            SEPARATOR
        );

        let packets = packets_of_the_body(&body);
        assert_eq!(packets.len(), 2);

        match &packets[0] {
            Packet::Event { name, .. } => assert_eq!(name, "user_online"),
            other => panic!("the first packet must be a message: {:?}", other),
        }

        match &packets[1] {
            Packet::Event { name, body } => {
                assert_eq!(name, "init");
                assert_eq!(body.get("userId").and_then(|v| v.as_str()), Some("a"));
            }
            other => panic!("the second packet must be a message: {:?}", other),
        }
    }

    /// **The live task tells the pool that no machine takes the connection.**
    ///
    /// The pool learned nothing of a server that went away: no request of the
    /// program failed while the user pressed no key, therefore `active()` still
    /// gave an address and the header said "Connected". This task tries again
    /// every few seconds, therefore it knows first. See T-107.
    #[test]
    fn a_connection_that_no_machine_takes_marks_the_address_down() {
        assert!(the_address_is_down(false, Some(THE_PROGRAM_CANNOT_CONNECT)));

        // A connection that opened says nothing of the address: the fault
        // belongs to that one connection.
        assert!(!the_address_is_down(true, Some(THE_PROGRAM_CANNOT_CONNECT)));

        // A time limit is not evidence that an address is down. See T-97.
        assert!(!the_address_is_down(
            false,
            Some("the server did not answer in time")
        ));
        assert!(!the_address_is_down(false, Some("the server answered 500")));
        assert!(!the_address_is_down(false, None));
    }

    #[test]
    fn the_program_knows_the_ping_the_pong_and_the_close() {
        assert_eq!(packets_of_the_body("2"), vec![Packet::Ping]);
        assert_eq!(packets_of_the_body("3"), vec![Packet::Pong]);
        assert_eq!(packets_of_the_body("1"), vec![Packet::Close]);
        assert_eq!(
            packets_of_the_body("40{\"sid\":\"40j8IkJgf7aoZxi4AAAC\"}"),
            vec![Packet::Connected]
        );
    }

    /// An empty answer gives no packet. The body of the answer of a `POST` is
    /// `ok`, and that value is not a packet of the protocol.
    #[test]
    fn a_body_that_the_program_does_not_know_stops_nothing() {
        assert!(packets_of_the_body("").is_empty());
        assert_eq!(
            packets_of_the_body("ok"),
            vec![Packet::Other("ok".to_string())]
        );
        assert_eq!(
            packets_of_the_body("44{\"message\":\"no\"}"),
            vec![Packet::Other("44{\"message\":\"no\"}".to_string())]
        );
        // A message with no body must not stop the connection.
        assert_eq!(
            packets_of_the_body("42[\"pong\"]"),
            vec![Packet::Event {
                name: "pong".to_string(),
                body: serde_json::Value::Null
            }]
        );
    }

    #[test]
    fn the_token_stands_in_the_body_of_the_message() {
        assert_eq!(auth_message("abc.def"), "42[\"auth\",\"abc.def\"]");
        // A token is a JWT, and a JWT holds no quotation mark. A value that
        // holds one must still give a body that a parser can read.
        let message = auth_message("a\"b");
        let packets = packets_of_the_body(&message);

        match packets.as_slice() {
            [Packet::Event { name, body }] => {
                assert_eq!(name, "auth");
                assert_eq!(body.as_str(), Some("a\"b"));
            }
            other => panic!("the message must hold the token: {:?}", other),
        }
    }

    #[test]
    fn the_address_of_the_poll_refuses_an_identity_that_is_not_safe() {
        assert_eq!(
            poll_path("HtEDXf_uhkTPO7cnAAAA"),
            Some("/socket.io/?EIO=4&transport=polling&sid=HtEDXf_uhkTPO7cnAAAA".to_string())
        );
        assert_eq!(poll_path(""), None);
        assert_eq!(poll_path("a&transport=websocket"), None);
        assert_eq!(poll_path("a b"), None);
        assert_eq!(poll_path("../../api/me"), None);
    }

    /// The exact shape of one row of `mediaProgress`, of the measurement of
    /// 2026-08-11.
    #[test]
    fn the_position_of_a_media_comes_from_the_message_of_the_user() {
        let body = serde_json::json!({
            "id": "5484c9aa",
            "username": "toutuitest",
            "token": "a token that the log must never hold",
            "mediaProgress": [
                {
                    "libraryItemId": "9a671047",
                    "episodeId": null,
                    "duration": 1800.0,
                    "progress": 0.4315,
                    "currentTime": 776.7,
                    "isFinished": false
                },
                {
                    "libraryItemId": "8fda6e43",
                    "episodeId": null,
                    "progress": 1.0,
                    "isFinished": true
                },
                {
                    "libraryItemId": "a podcast",
                    "episodeId": "an episode",
                    "progress": 0.5,
                    "isFinished": false
                }
            ]
        });

        let rows = progress_of_the_user(&body);

        // **The row of an episode of a podcast comes too** (T-228): a line of
        // the Home view of a library of podcasts is one episode, and that line
        // reads this list. Its key names the episode after the item, because the
        // identity of the item names every episode of one podcast (T-223).
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[2].0, "a podcast/an episode");
        assert_eq!(rows[2].1.percent, "50");

        assert_eq!(rows[0].0, "9a671047");
        // 0.4315 gives 43 percent, and the form is the form of
        // `collect_progress_percentage_book`.
        assert_eq!(rows[0].1.percent, "43");
        assert_eq!(rows[0].1.finished, "Not finished");

        assert_eq!(rows[1].0, "8fda6e43");
        assert_eq!(rows[1].1.percent, "100");
        assert_eq!(rows[1].1.finished, "Finished");
    }

    /// The exact shape of `mediaProgress` of an Audiobookshelf 2.36.0, of the
    /// measurement of 2026-08-11. The two values that take a media away from
    /// the shelf of Continue Listening are `isFinished` and
    /// `hideFromContinueListening`. See T-66.
    #[test]
    fn the_finished_media_and_the_hidden_media_leave_continue_listening() {
        let body = serde_json::json!({
            "mediaProgress": [
                { "libraryItemId": "a book that plays", "episodeId": null,
                  "progress": 0.4315, "isFinished": false,
                  "hideFromContinueListening": false },
                { "libraryItemId": "a book that ended", "episodeId": null,
                  "progress": 1.0, "isFinished": true,
                  "hideFromContinueListening": false },
                { "libraryItemId": "a book that the user hid", "episodeId": null,
                  "progress": 0.2, "isFinished": false,
                  "hideFromContinueListening": true },
                { "libraryItemId": "a podcast", "episodeId": "an episode",
                  "progress": 1.0, "isFinished": true }
            ]
        });

        // **The row of an episode comes with the key of that episode** (T-226).
        // A line of the Home view of a library of podcasts is one episode, and
        // the identity of the item names every episode of one podcast (T-223).
        assert_eq!(
            the_media_away_from_continue_listening(&body),
            vec![
                "a book that ended".to_string(),
                "a book that the user hid".to_string(),
                "a podcast/an episode".to_string()
            ],
            "the book that plays stays, and the episode names itself"
        );
    }

    /// A server that gives no such field must give no media away. An old
    /// server, and a message of a different shape, keep every line.
    #[test]
    fn a_message_with_no_such_field_takes_no_media_away() {
        assert!(the_media_away_from_continue_listening(&serde_json::Value::Null).is_empty());
        assert!(the_media_away_from_continue_listening(
            &serde_json::json!({"mediaProgress": [{"libraryItemId": "a book"}]})
        )
        .is_empty());
        assert!(the_media_away_from_continue_listening(
            &serde_json::json!({"mediaProgress": "not a list"})
        )
        .is_empty());
    }

    #[test]
    fn a_message_with_no_position_gives_no_row() {
        assert!(progress_of_the_user(&serde_json::Value::Null).is_empty());
        assert!(progress_of_the_user(&serde_json::json!({"mediaProgress": []})).is_empty());
        assert!(
            progress_of_the_user(&serde_json::json!({"mediaProgress": "not a list"})).is_empty()
        );
    }

    /// **A message that holds no list of the positions is not a message of an
    /// account whose media hold no position.** The two lists of the screen take
    /// the place of the lists before them, therefore the first condition must
    /// keep those lists and the second one must empty them. See T-184.
    #[test]
    fn a_message_that_holds_no_list_of_the_positions_is_not_a_list_of_no_row() {
        assert!(!the_message_holds_the_positions(&serde_json::Value::Null));
        assert!(!the_message_holds_the_positions(&serde_json::json!({
            "id": "u1", "username": "toutuitest"
        })));
        assert!(!the_message_holds_the_positions(&serde_json::json!({
            "mediaProgress": "not a list"
        })));

        assert!(the_message_holds_the_positions(&serde_json::json!({
            "mediaProgress": []
        })));
    }

    /// A server that gives no socket.io must not get a request every ten seconds
    /// for ever. See T-61.
    #[test]
    fn the_wait_becomes_longer_after_each_fault() {
        // No fault, and the first fault: the short wait. A connection that ends
        // in the way of the protocol comes back at once.
        assert_eq!(wait_after_the_faults(0), WAIT_BEFORE_AGAIN);
        assert_eq!(wait_after_the_faults(1), WAIT_BEFORE_AGAIN);

        // Each fault after the first one doubles the wait.
        assert_eq!(wait_after_the_faults(2), WAIT_BEFORE_AGAIN * 2);
        assert_eq!(wait_after_the_faults(3), WAIT_BEFORE_AGAIN * 4);
        assert_eq!(wait_after_the_faults(4), WAIT_BEFORE_AGAIN * 8);

        // The wait stops at the longest value, and it stays there.
        assert_eq!(wait_after_the_faults(20), LONGEST_WAIT);
        assert_eq!(wait_after_the_faults(1000), LONGEST_WAIT);
        assert_eq!(wait_after_the_faults(u32::MAX), LONGEST_WAIT);

        // The number of requests of one hour of a server that answers nothing.
        let mut spent = Duration::from_secs(0);
        let mut requests = 0;
        let mut faults = 0;

        while spent < Duration::from_secs(3600) {
            faults += 1;
            requests += 1;
            spent += wait_after_the_faults(faults);
        }

        // The old rule gave 360 requests in one hour. This rule gives fewer than
        // twenty.
        assert!(requests < 20, "one hour gives {} requests", requests);
    }

    /// The program itself makes `user_updated` every ten seconds while a media
    /// plays. That message must not put a notice on the screen. See T-47.
    #[test]
    fn the_message_of_the_user_does_not_make_the_lists_old() {
        assert!(!the_library_changed("user_updated"));
        assert!(!the_library_changed("init"));
        assert!(!the_library_changed("user_online"));
        assert!(!the_library_changed("stream_progress"));

        assert!(the_library_changed("item_updated"));
        assert!(the_library_changed("items_added"));
        assert!(the_library_changed("library_updated"));
    }
}
