//! A reader of the stream of HLS of the server. See T-53.
//!
//! The type obeys `Read` and `Seek`, therefore `rodio::Decoder` accepts it. It
//! holds the same shape as `HttpFile`: a thread fills a buffer of the memory,
//! and `read` copies bytes of that buffer. The thread of the sound card must
//! never wait for the network.
//!
//! The difference with `HttpFile` is the source of the bytes. `HttpFile` reads
//! one file with requests of a range. This reader walks the parts of a playlist,
//! and it gives the audio of each part with no header of the container. See
//! `crate::player::engine::hls`.
//!
//! **This reader moves forward only.** A stream of HLS has no size in bytes,
//! and the bytes of a part come from ffmpeg of the server. A movement of the
//! playback therefore starts a new playback of the engine, and it does not move
//! inside this reader.

use crate::api::client::error::{classify_status, ApiError};
use crate::player::engine::hls::{self, Form, Segment};
use log::{info, warn};
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// The number of bytes that the thread keeps in front of the cursor.
const BUFFER_TARGET: usize = 4 * 1024 * 1024;

/// The time that a read operation waits for data before it looks again.
const READ_WAIT: Duration = Duration::from_millis(250);

/// The first delay after a failed request. The delay doubles after each
/// failure, to the value `MAX_BACKOFF`.
const FIRST_BACKOFF: Duration = Duration::from_millis(500);

/// The largest delay between two attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(10);

/// The number of attempts for one part of the stream.
///
/// ffmpeg of the server writes the parts while the client reads them.
/// Therefore a part that does not exist yet answers 404, and the reader tries
/// again.
const ATTEMPTS: usize = 20;

/// The time to wait for a connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// The time to wait for one part of the stream.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The data that the thread and the reader share.
struct Shared {
    buffer: Mutex<VecDeque<u8>>,
    signal: Condvar,
    stop: AtomicBool,
    finished: AtomicBool,
    stalled: AtomicBool,
}

/// The stream of one media of the server.
pub struct HlsFile {
    shared: Arc<Shared>,
    /// The number of bytes that the reader gave to the decoder.
    cursor: u64,
    /// The form of the audio inside the parts. The decoder needs the hint.
    form: Form,
    /// The place of the media at the first byte of this reader, in seconds.
    ///
    /// The reader starts at a part of the playlist, and that part starts inside
    /// the media. The engine adds this value to the position of the decoder.
    offset: f64,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl HlsFile {
    /// Opens the stream of a playlist, from the place `seconds` of the media.
    ///
    /// The function asks for the playlist, and it reads the first part. That
    /// first part names the form of the audio, therefore a form that no decoder
    /// of the program reads gives a fault **before** the playback starts. The
    /// caller then asks the server for a different form.
    pub fn open(
        base_url: &str,
        token: &str,
        playlist_path: &str,
        seconds: f64,
    ) -> Result<HlsFile, String> {
        let base = base_url.trim_end_matches('/').to_string();
        let client = client()?;

        let address = address_of(&base, playlist_path);
        let text = ask_for_the_text(&client, &address, token)?;
        let segments = hls::parse_playlist(&text);

        if segments.is_empty() {
            return Err("The playlist of the server holds no part.".to_string());
        }

        let (first, inside) = hls::place_in_the_playlist(&segments, seconds).unwrap_or((0, 0.0));

        // The first part gives the form of the audio. The bytes go in the buffer
        // at once, therefore the request happens one time only.
        let name = segments[first].name.clone();
        let bytes = ask_for_the_bytes(&client, &address_of_the_part(&address, &name), token)?;
        let stream = hls::audio_stream_of(&bytes)
            .ok_or_else(|| "The stream of the server holds no audio.".to_string())?;

        if !stream.form.a_decoder_of_the_program_reads_it() {
            return Err(format!(
                "The stream of the server holds the audio in the form {:?}, and \
                 no decoder of the program reads it.",
                stream.form
            ));
        }

        let audio = hls::audio_payload(&bytes, stream.pid);

        let shared = Arc::new(Shared {
            buffer: Mutex::new(VecDeque::from(audio)),
            signal: Condvar::new(),
            stop: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            stalled: AtomicBool::new(false),
        });

        info!(
            "[HlsFile] the stream holds {} part(s). The reader starts at the \
             part {} and at {:.1} seconds inside it. The audio is {:?}.",
            segments.len(),
            first,
            inside,
            stream.form
        );

        let mut file = HlsFile {
            shared,
            cursor: 0,
            form: stream.form,
            offset: hls::seconds_before(&segments, first),
            handle: None,
        };

        file.start_thread(&address, token, segments, first + 1, stream.pid);

        Ok(file)
    }

    /// Gives the form of the audio. The decoder takes it as a hint.
    pub fn form(&self) -> Form {
        self.form
    }

    /// Gives the place of the media at the first byte of the reader.
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// Tells if the reader waits for data of the server.
    pub fn is_stalled(&self) -> bool {
        self.shared.stalled.load(Ordering::Relaxed)
    }

    /// Starts the thread that reads the parts after the first one.
    fn start_thread(
        &mut self,
        address: &str,
        token: &str,
        segments: Vec<Segment>,
        from: usize,
        pid: u16,
    ) {
        let shared = Arc::clone(&self.shared);
        let address = address.to_string();
        let token = token.to_string();

        let handle = std::thread::Builder::new()
            .name("toutui-stream".to_string())
            .spawn(move || fill_buffer(shared, address, token, segments, from, pid));

        self.handle = match handle {
            Ok(handle) => Some(handle),
            Err(error) => {
                warn!("[HlsFile] the program cannot start the thread: {}", error);
                self.shared.finished.store(true, Ordering::SeqCst);
                None
            }
        };
    }
}

impl Drop for HlsFile {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.signal.notify_all();

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Read for HlsFile {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
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

            if self.shared.finished.load(Ordering::SeqCst)
                || self.shared.stop.load(Ordering::SeqCst)
            {
                return Ok(0);
            }

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

impl Seek for HlsFile {
    /// A stream moves forward only.
    ///
    /// The decoder asks for the place at the start, and `Current(0)` and
    /// `Start(<the place now>)` give that answer. A movement forward drops
    /// bytes of the buffer. Every other movement gives a fault, and the engine
    /// then starts a new playback at the new place. See T-53.
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let target = match from {
            SeekFrom::Current(0) => return Ok(self.cursor),
            SeekFrom::Start(value) => value,
            SeekFrom::Current(value) if value >= 0 => self.cursor + value as u64,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "The stream of the server moves forward only.",
                ))
            }
        };

        if target == self.cursor {
            return Ok(self.cursor);
        }

        if target < self.cursor {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "The stream of the server moves forward only.",
            ));
        }

        let mut count = (target - self.cursor) as usize;

        while count > 0 {
            let mut buffer = self
                .shared
                .buffer
                .lock()
                .map_err(|_| io::Error::other("The buffer lock is broken."))?;

            if buffer.is_empty() {
                if self.shared.finished.load(Ordering::SeqCst) {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "The stream of the server has no more bytes.",
                    ));
                }

                let (new_buffer, _) = self
                    .shared
                    .signal
                    .wait_timeout(buffer, READ_WAIT)
                    .map_err(|_| io::Error::other("The buffer lock is broken."))?;
                buffer = new_buffer;
            }

            let taken = count.min(buffer.len());
            buffer.drain(..taken);
            drop(buffer);

            self.cursor += taken as u64;
            count -= taken;
        }

        Ok(self.cursor)
    }
}

/// Makes the address of the playlist.
fn address_of(base: &str, playlist_path: &str) -> String {
    if playlist_path.starts_with("http://") || playlist_path.starts_with("https://") {
        return playlist_path.to_string();
    }

    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        playlist_path.trim_start_matches('/')
    )
}

/// Makes the address of one part, from the address of the playlist.
///
/// The playlist names a part with no directory, therefore the part stands beside
/// the playlist.
fn address_of_the_part(playlist: &str, name: &str) -> String {
    match playlist.rfind('/') {
        Some(place) => format!("{}/{}", &playlist[..place], name),
        None => name.to_string(),
    }
}

/// Makes the HTTP client of the thread.
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| format!("The program cannot make the client: {}", error))
}

/// Asks for a text of the server.
fn ask_for_the_text(
    client: &reqwest::blocking::Client,
    address: &str,
    token: &str,
) -> Result<String, String> {
    let answer = client
        .get(address)
        .bearer_auth(token)
        .send()
        .map_err(|_| "The server did not answer.".to_string())?;

    if let Some(error) = classify_status(answer.status()) {
        return Err(text_of(&error));
    }

    answer
        .text()
        .map_err(|_| "The server gave no playlist.".to_string())
}

/// Asks for the bytes of one part of the stream.
fn ask_for_the_bytes(
    client: &reqwest::blocking::Client,
    address: &str,
    token: &str,
) -> Result<Vec<u8>, String> {
    let mut backoff = FIRST_BACKOFF;
    let mut last = String::from("The server did not answer.");

    for attempt in 0..ATTEMPTS {
        let answer = match client.get(address).bearer_auth(token).send() {
            Ok(answer) => answer,
            Err(_) => {
                last = "The server did not answer.".to_string();
                wait(&mut backoff, attempt);
                continue;
            }
        };

        // ffmpeg of the server writes the parts while the client reads them.
        // A part that does not exist yet answers 404.
        if let Some(error) = classify_status(answer.status()) {
            last = text_of(&error);
            wait(&mut backoff, attempt);
            continue;
        }

        return answer
            .bytes()
            .map(|bytes| bytes.to_vec())
            .map_err(|_| "The part of the stream did not come.".to_string());
    }

    Err(last)
}

/// Waits, and it makes the delay longer.
fn wait(backoff: &mut Duration, attempt: usize) {
    std::thread::sleep(*backoff);
    *backoff = (*backoff * 2).min(MAX_BACKOFF);

    if attempt == 0 {
        info!("[HlsFile] the part of the stream is not ready. The reader waits.");
    }
}

/// Gives a short text of a fault of a request.
fn text_of(error: &ApiError) -> String {
    format!("The stream of the server gave a fault: {}", error)
}

/// Reads the parts of the playlist, and it fills the buffer.
fn fill_buffer(
    shared: Arc<Shared>,
    address: String,
    token: String,
    segments: Vec<Segment>,
    from: usize,
    pid: u16,
) {
    let client = match client() {
        Ok(client) => client,
        Err(error) => {
            warn!("[HlsFile] {}", error);
            shared.finished.store(true, Ordering::SeqCst);
            shared.signal.notify_all();
            return;
        }
    };

    for part in segments.iter().skip(from) {
        // The buffer must not hold the whole book. The thread waits while the
        // reader is far behind.
        loop {
            if shared.stop.load(Ordering::SeqCst) {
                return;
            }

            let buffer = match shared.buffer.lock() {
                Ok(buffer) => buffer,
                Err(_) => return,
            };

            if buffer.len() < BUFFER_TARGET {
                break;
            }

            let _ = shared.signal.wait_timeout(buffer, READ_WAIT);
        }

        let bytes =
            match ask_for_the_bytes(&client, &address_of_the_part(&address, &part.name), &token) {
                Ok(bytes) => bytes,
                Err(error) => {
                    warn!(
                        "[HlsFile] the part {} did not come: {}. The playback stops \
                     there.",
                        part.name, error
                    );
                    break;
                }
            };

        let audio = hls::audio_payload(&bytes, pid);

        if audio.is_empty() {
            warn!("[HlsFile] the part {} holds no audio.", part.name);
            continue;
        }

        if let Ok(mut buffer) = shared.buffer.lock() {
            buffer.extend(audio);
        }

        shared.signal.notify_all();
    }

    shared.finished.store(true, Ordering::SeqCst);
    shared.signal.notify_all();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_of_the_playlist_takes_the_base_of_the_server() {
        assert_eq!(
            address_of("http://server:13378", "/hls/abc/output.m3u8"),
            "http://server:13378/hls/abc/output.m3u8"
        );
        // A base with an oblique at the end gives one oblique only.
        assert_eq!(
            address_of("http://server:13378/", "hls/abc/output.m3u8"),
            "http://server:13378/hls/abc/output.m3u8"
        );
        // The server can give a whole address.
        assert_eq!(
            address_of("http://server:13378", "https://other/hls/a.m3u8"),
            "https://other/hls/a.m3u8"
        );
    }

    /// The playlist names a part with no directory. Therefore the part stands
    /// beside the playlist, and the address of the playlist gives the
    /// directory.
    #[test]
    fn the_address_of_a_part_stands_beside_the_playlist() {
        assert_eq!(
            address_of_the_part("http://server/hls/abc/output.m3u8", "output-12.ts"),
            "http://server/hls/abc/output-12.ts"
        );
    }
}
