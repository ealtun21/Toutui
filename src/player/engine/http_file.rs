//! A reader that gets a file from the server with range requests.
//!
//! The type obeys `Read` and `Seek`. Therefore `rodio::Decoder` accepts it.
//!
//! `rodio` decodes on the callback thread of the sound card. That thread must
//! never wait for the network, because a wait makes a gap in the sound.
//! Therefore a thread fills a buffer in the memory, and the `Read` function
//! copies bytes from that buffer.

use crate::api::client::error::{classify_status, ApiError};
use log::{info, warn};
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// The number of bytes that the thread keeps in front of the cursor.
const BUFFER_TARGET: usize = 8 * 1024 * 1024;

/// The size of one read operation from the answer of the server.
const CHUNK: usize = 64 * 1024;

/// The time that a read operation waits for data before it looks again.
const READ_WAIT: Duration = Duration::from_millis(250);

/// The first delay after a failed request. The delay doubles after each
/// failure, to the value `MAX_BACKOFF`.
const FIRST_BACKOFF: Duration = Duration::from_millis(500);

/// The largest delay between two attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(10);

/// Reads the total size of the file from the header `Content-Range`.
///
/// The header has the form `bytes 0-1023/2797969`. The number after the
/// oblique is the total size. The header `Content-Length` gives the length of
/// the part only. Therefore the code must not use `Content-Length` here.
pub fn total_size_from_content_range(value: &str) -> Option<u64> {
    value.rsplit('/').next()?.trim().parse::<u64>().ok()
}

/// The data that the thread and the reader share.
struct Shared {
    /// The bytes that the thread read and the reader did not use.
    buffer: Mutex<VecDeque<u8>>,
    /// The thread adds data, and the reader takes data.
    signal: Condvar,
    /// The thread stops when this value is true.
    stop: AtomicBool,
    /// The thread read to the end of the file.
    finished: AtomicBool,
    /// The thread cannot get data now. The engine shows "Reconnecting".
    stalled: AtomicBool,
}

/// A file on the server that obeys `Read` and `Seek`.
pub struct HttpFile {
    shared: Arc<Shared>,
    /// The position of the next read operation.
    cursor: u64,
    /// The number of bytes of the whole file.
    size: u64,
    url: String,
    token: String,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl HttpFile {
    /// Opens a file on the server.
    ///
    /// The function sends one range request. It reads the total size from the
    /// header `Content-Range`. It then starts the thread that fills the
    /// buffer.
    pub fn open(
        base_url: &str,
        token: &str,
        item_id: &str,
        ino: &str,
    ) -> Result<HttpFile, ApiError> {
        let url = format!(
            "{}/api/items/{}/file/{}/download",
            base_url.trim_end_matches('/'),
            item_id,
            ino
        );

        let client = blocking_client()?;

        let response = client
            .get(&url)
            .bearer_auth(token)
            .header("Range", "bytes=0-0")
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        if let Some(error) = classify_status(response.status()) {
            return Err(error);
        }

        let size = response
            .headers()
            .get("content-range")
            .and_then(|value| value.to_str().ok())
            .and_then(total_size_from_content_range)
            .ok_or_else(|| {
                ApiError::Decode("The server gave no Content-Range header.".to_string())
            })?;

        let shared = Arc::new(Shared {
            buffer: Mutex::new(VecDeque::new()),
            signal: Condvar::new(),
            stop: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            stalled: AtomicBool::new(false),
        });

        let mut file = HttpFile {
            shared,
            cursor: 0,
            size,
            url,
            token: token.to_string(),
            handle: None,
        };

        file.start_thread(0);

        Ok(file)
    }

    /// Gives the number of bytes of the whole file.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Tells if the file has no bytes.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Tells if the reader waits for data from the server.
    pub fn is_stalled(&self) -> bool {
        self.shared.stalled.load(Ordering::Relaxed)
    }

    /// Stops the thread and empties the buffer.
    fn stop_thread(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.signal.notify_all();

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        if let Ok(mut buffer) = self.shared.buffer.lock() {
            buffer.clear();
        }
    }

    /// Starts a thread that reads the file from a position.
    fn start_thread(&mut self, from: u64) {
        let shared = Arc::clone(&self.shared);
        let url = self.url.clone();
        let token = self.token.clone();

        shared.stop.store(false, Ordering::SeqCst);
        shared.finished.store(false, Ordering::SeqCst);

        let handle = std::thread::Builder::new()
            .name("abstui-prefetch".to_string())
            .spawn(move || fill_buffer(shared, url, token, from));

        self.handle = match handle {
            Ok(handle) => Some(handle),
            Err(error) => {
                warn!("[HttpFile] the application cannot start the thread: {}", error);
                None
            }
        };
    }
}

impl Drop for HttpFile {
    fn drop(&mut self) {
        self.stop_thread();
    }
}

impl Read for HttpFile {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() || self.cursor >= self.size {
            return Ok(0);
        }

        let mut buffer = self
            .shared
            .buffer
            .lock()
            .map_err(|_| io::Error::other("The buffer lock is broken."))?;

        loop {
            if !buffer.is_empty() {
                let count = out.len().min(buffer.len());

                for (position, byte) in buffer.drain(..count).enumerate() {
                    out[position] = byte;
                }

                self.cursor += count as u64;
                self.shared.stalled.store(false, Ordering::Relaxed);
                self.shared.signal.notify_all();

                return Ok(count);
            }

            if self.shared.finished.load(Ordering::SeqCst) {
                return Ok(0);
            }

            // The buffer is empty. The thread did not get the data yet. The
            // engine reads this value and shows "Reconnecting".
            self.shared.stalled.store(true, Ordering::Relaxed);

            let (new_buffer, _) = self
                .shared
                .signal
                .wait_timeout(buffer, READ_WAIT)
                .map_err(|_| io::Error::other("The buffer lock is broken."))?;

            buffer = new_buffer;
        }
    }
}

impl Seek for HttpFile {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let target = match from {
            SeekFrom::Start(value) => value as i64,
            SeekFrom::End(value) => self.size as i64 + value,
            SeekFrom::Current(value) => self.cursor as i64 + value,
        };

        if target < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The position is before the start of the file.",
            ));
        }

        let target = (target as u64).min(self.size);

        if target == self.cursor {
            return Ok(target);
        }

        // The buffer holds the bytes that come after the cursor. A movement
        // forward inside the buffer only removes bytes. This is the usual
        // condition, and it sends no request.
        let inside = {
            let mut buffer = self
                .shared
                .buffer
                .lock()
                .map_err(|_| io::Error::other("The buffer lock is broken."))?;

            if target > self.cursor && target - self.cursor <= buffer.len() as u64 {
                let count = (target - self.cursor) as usize;
                buffer.drain(..count);
                true
            } else {
                false
            }
        };

        self.cursor = target;

        if !inside {
            info!("[HttpFile] the reader moves to the byte {}", target);
            self.stop_thread();
            self.start_thread(target);
        }

        Ok(target)
    }
}

/// Makes the HTTP client of the thread.
///
/// The thread is not an asynchronous task. Therefore it uses the blocking
/// client of `reqwest`.
fn blocking_client() -> Result<reqwest::blocking::Client, ApiError> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| ApiError::Unreachable)
}

/// Reads the file and fills the buffer.
///
/// The function sends the request again after a failure. The delay doubles
/// after each failure. The function stops when the reader asks it to stop.
fn fill_buffer(shared: Arc<Shared>, url: String, token: String, from: u64) {
    let mut position = from;
    let mut backoff = FIRST_BACKOFF;

    let client = match blocking_client() {
        Ok(client) => client,
        Err(_) => {
            shared.finished.store(true, Ordering::SeqCst);
            shared.signal.notify_all();
            return;
        }
    };

    while !shared.stop.load(Ordering::SeqCst) {
        let answer = client
            .get(&url)
            .bearer_auth(&token)
            .header("Range", format!("bytes={}-", position))
            .send();

        let mut response = match answer {
            Ok(response) if response.status().is_success() => {
                shared.stalled.store(false, Ordering::Relaxed);
                backoff = FIRST_BACKOFF;
                response
            }
            Ok(response) => {
                warn!("[HttpFile] the server gave the status {}", response.status());
                shared.stalled.store(true, Ordering::Relaxed);
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
            Err(error) => {
                warn!("[HttpFile] the request failed: {}", error);
                shared.stalled.store(true, Ordering::Relaxed);
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        let mut chunk = vec![0u8; CHUNK];

        loop {
            if shared.stop.load(Ordering::SeqCst) {
                return;
            }

            // Wait while the buffer is full. This keeps the memory bounded.
            {
                let mut buffer = match shared.buffer.lock() {
                    Ok(buffer) => buffer,
                    Err(_) => return,
                };

                while buffer.len() >= BUFFER_TARGET && !shared.stop.load(Ordering::SeqCst) {
                    let (new_buffer, _) = match shared.signal.wait_timeout(buffer, READ_WAIT) {
                        Ok(value) => value,
                        Err(_) => return,
                    };
                    buffer = new_buffer;
                }
            }

            match response.read(&mut chunk) {
                Ok(0) => {
                    shared.finished.store(true, Ordering::SeqCst);
                    shared.signal.notify_all();
                    return;
                }
                Ok(count) => {
                    let mut buffer = match shared.buffer.lock() {
                        Ok(buffer) => buffer,
                        Err(_) => return,
                    };
                    buffer.extend(&chunk[..count]);
                    position += count as u64;
                    shared.signal.notify_all();
                }
                Err(error) => {
                    // The connection stopped. The outer loop sends a new
                    // request from the current position.
                    warn!("[HttpFile] the connection stopped: {}", error);
                    shared.stalled.store(true, Ordering::Relaxed);
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::total_size_from_content_range;

    /// The header has the form `bytes 0-1023/2797969`. The number after the
    /// oblique is the total size. `Content-Length` gives the length of the
    /// part only, thus the code must not use it for the total size.
    #[test]
    fn the_content_range_header_gives_the_total_size() {
        assert_eq!(
            total_size_from_content_range("bytes 0-1023/2797969"),
            Some(2797969)
        );
        assert_eq!(
            total_size_from_content_range("bytes 100-1099/2797969"),
            Some(2797969)
        );
    }

    #[test]
    fn a_header_that_is_not_valid_gives_no_size() {
        assert_eq!(total_size_from_content_range("bytes 0-1023/*"), None);
        assert_eq!(total_size_from_content_range("nonsense"), None);
        assert_eq!(total_size_from_content_range(""), None);
    }
}
