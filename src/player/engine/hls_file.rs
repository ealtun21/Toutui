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

/// The number of attempts for one part of the stream, for the thread that fills
/// the buffer.
///
/// ffmpeg of the server writes the parts while the client reads them. Therefore a
/// part that does not exist yet answers 404, and the reader tries again.
const ATTEMPTS: usize = 20;

/// The number of attempts for the **first** part, when the reader opens.
///
/// `HlsFile::open` runs on the thread of the engine, and that thread reads the
/// commands of the user. **A command of the user stops this wait**, therefore the
/// number can be large: `a_command_waits` of the engine gives the answer at each
/// attempt. See T-68.
///
/// A movement of the playback asks for a part before the part where the transcode
/// of the server began. The server answers 404 and it starts the transcode again
/// at the new place: its log says "Segment #N Request is before starting segment
/// number #M - Reset Transcode". A measurement of 2026-08-11 gave the first part
/// of such a start in 0.5 seconds for a file of MP3.
///
/// **A copy that fails costs ten seconds more.** ffmpeg of the server writes no
/// part when the codec of the file does not fit a transport stream, and the
/// server needs ten seconds to see that its own transcode died: its log says
/// "Transcode never closed...". It then starts ffmpeg again with `-c:a aac`.
/// Fourteen attempts with `LONGEST_WAIT_OF_THE_OPEN` give about 25 seconds, and
/// that holds the ten seconds of the server and the first part of the second try.
/// See T-68.
const ATTEMPTS_OF_THE_OPEN: usize = 14;

/// The largest delay between two attempts of the open.
const LONGEST_WAIT_OF_THE_OPEN: Duration = Duration::from_secs(2);

/// The number of attempts of a part before the reader asks for the playlist.
///
/// A part that ffmpeg did not write yet answers 404, and that answer is the
/// common one. Therefore the reader does not ask for the playlist at the first
/// attempts. See T-68.
const ATTEMPTS_BEFORE_THE_PLAYLIST: usize = 2;

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

/// A stream of the server that did not reach its last part. See T-194.
#[derive(Debug, Clone, PartialEq)]
pub struct TheStreamStopped {
    /// The place of the media that the reader reached, in seconds.
    ///
    /// **The playlist is the truth of the length of a stream**: it names every
    /// part of the media, and the time of each of them. A reader that stops at
    /// the part N therefore holds the media to the sum of the times of the
    /// parts before N, and no second more.
    pub seconds: f64,
    /// What the user reads.
    pub why: String,
}

/// What the reader of a stream tells the engine. See T-194.
///
/// **The end of a reader is not the end of the media.** The thread of the
/// buffer meets a part that does not come, and it then has no more bytes: the
/// decoder reads that as the end of the book, the engine writes the whole place
/// of the media, and the program tells the server that the user finished the
/// book. The reader writes here what it really reached, and the engine reads
/// it.
#[derive(Debug, Default)]
pub struct StreamReport {
    stopped: Mutex<Option<TheStreamStopped>>,
}

impl StreamReport {
    /// Gives the report of a stream that did not reach its last part.
    pub fn the_stream_stopped(&self) -> Option<TheStreamStopped> {
        self.stopped.lock().ok().and_then(|value| value.clone())
    }

    /// Says that the stream stopped at a place of the media.
    fn say(&self, value: TheStreamStopped) {
        if let Ok(mut place) = self.stopped.lock() {
            *place = Some(value);
        }
    }
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
    /// What the thread of the buffer tells the engine. See T-194.
    report: Arc<StreamReport>,
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
        // **A playlist that stops in the middle is not a short playlist.** The
        // parts that the body does not hold belong to no playback, and the
        // program then tells the server that the user listened to the whole
        // book. See T-193.
        if !hls::the_playlist_is_whole(&text) {
            warn!("[HlsFile] the body of the playlist stopped in the middle.");
            return Err(hls::the_sentence_of_a_playlist_that_stopped());
        }

        let segments = hls::parse_playlist(&text);

        if segments.is_empty() {
            return Err("The playlist of the server holds no part.".to_string());
        }

        let (first, inside) = hls::place_in_the_playlist(&segments, seconds).unwrap_or((0, 0.0));

        // The first part gives the form of the audio. The bytes go in the buffer
        // at once, therefore the request happens one time only.
        let name = segments[first].name.clone();

        let bytes = ask_for_the_bytes_with_a_limit(
            &client,
            &address_of_the_part(&address, &name),
            token,
            Patience {
                attempts: ATTEMPTS_OF_THE_OPEN,
                longest: LONGEST_WAIT_OF_THE_OPEN,
                a_command_stops_it: true,
                playlist: &address,
            },
        )?;
        let stream = hls::audio_stream_of(&bytes)
            .ok_or_else(|| "The stream of the server holds no audio.".to_string())?;

        if !stream.form.a_decoder_of_the_program_reads_it() {
            // ffmpeg of the server copies the codec of the file when that codec
            // fits a transport stream, and it gives a fault when it does not.
            //
            // **A measurement of 2026-08-11 with a real file of xHE-AAC gave no
            // LATM at all.** T-53 expected that form here. ffmpeg stops at the
            // header of the transport stream instead: "Could not write header
            // (incorrect codec parameters ?)", and it writes no part. The server
            // then starts ffmpeg again with `-c:a aac`. Therefore this arm holds
            // a form that no measurement met yet, and it stays for a server that
            // gives one. See T-68.
            return Err(format!(
                "The stream of the server holds the audio in the form {:?}, and \
                 no decoder of the program reads it. The file of this media \
                 needs a form that the server can put in a transport stream.",
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
            report: Arc::new(StreamReport::default()),
            handle: None,
        };

        file.start_thread(&address, token, segments, first + 1, stream.pid);

        Ok(file)
    }

    /// Gives the form of the audio. The decoder takes it as a hint.
    pub fn form(&self) -> Form {
        self.form
    }

    /// Gives the box where the thread of the buffer says what it reached.
    ///
    /// The engine keeps this box, and it reads that box when the decoder has no
    /// more bytes: a stream that stopped before its last part is not the end of
    /// the media. See T-194.
    pub fn report(&self) -> Arc<StreamReport> {
        Arc::clone(&self.report)
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
        let report = Arc::clone(&self.report);
        let address = address.to_string();
        let token = token.to_string();
        let place_of_the_start = self.offset;

        let handle = std::thread::Builder::new()
            .name("toutui-stream".to_string())
            .spawn(move || fill_buffer(shared, report, address, token, segments, from, pid));

        self.handle = match handle {
            Ok(handle) => Some(handle),
            Err(error) => {
                warn!("[HlsFile] the program cannot start the thread: {}", error);

                // The reader holds the first part alone, therefore the media
                // stops at the end of that part and it is not finished. See
                // T-194.
                self.report.say(TheStreamStopped {
                    seconds: place_of_the_start,
                    why: hls::the_sentence_of_a_stream_that_stopped(),
                });
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

/// Gives the name of the file of an address.
fn name_of(address: &str) -> &str {
    match address.rfind('/') {
        Some(place) => &address[place + 1..],
        None => address,
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

/// How long a reader of the parts waits, and what stops that wait.
struct Patience<'a> {
    /// The number of attempts for one part.
    attempts: usize,
    /// The largest delay between two attempts.
    longest: Duration,
    /// A command of the user stops the wait. The open gives `true`, because it
    /// runs on the thread of the engine. See T-68.
    a_command_stops_it: bool,
    /// The address of the playlist. A reader that holds it can see a stream that
    /// the server ended. See T-68.
    playlist: &'a str,
}

/// Asks for the bytes of one part of the stream, for the thread of the buffer.
fn ask_for_the_bytes(
    client: &reqwest::blocking::Client,
    address: &str,
    token: &str,
    playlist: &str,
) -> Result<Vec<u8>, String> {
    ask_for_the_bytes_with_a_limit(
        client,
        address,
        token,
        Patience {
            attempts: ATTEMPTS,
            longest: MAX_BACKOFF,
            a_command_stops_it: false,
            playlist,
        },
    )
}

/// Gives the sentence for a stream that the server ended.
///
/// **A stream of the server can go away while the client reads it.** A
/// measurement of 2026-08-11 with a file of xHE-AAC: the decoder of ffmpeg gave
/// samples of NaN to its encoder, ffmpeg stopped with the code 234, and the
/// server then wrote "Closing Stream" and "Deleted session data". Every part of
/// that stream answers 404 after that moment, and the answer never changes.
///
/// A reader that does not see this waits for every attempt, and the user waits
/// with it. Therefore the reader asks for the playlist, and a playlist that
/// answers 404 says that the stream is gone.
///
/// The function is pure, therefore a test needs no server. See T-68.
pub fn the_sentence_of_a_stream_that_ended() -> String {
    "The server ended the stream of this media. Its ffmpeg cannot read the form \
     of this audio."
        .to_string()
}

/// Tells if the stream of the server is gone.
///
/// A playlist that answers 404 says that the server removed the session of the
/// stream. Any other answer, and a fault of the network, give `false`: the
/// reader must not stop for a server that is slow. See T-68.
fn the_stream_is_gone(client: &reqwest::blocking::Client, playlist: &str, token: &str) -> bool {
    match client.get(playlist).bearer_auth(token).send() {
        Ok(answer) => classify_status(answer.status()) == Some(ApiError::NotFound),
        Err(_) => false,
    }
}

/// Gives the sentence for a part that the server never made.
///
/// **A 404 of a part is not a media that the server does not have.** ffmpeg of
/// the server writes a part when it made that part, therefore a part that answers
/// 404 to every attempt says that the server made no part at all. The old
/// sentence came from `classify_status`, and it said "The server does not have
/// this item": the user then reads that their book is absent, and the book is
/// there.
///
/// A measurement of 2026-08-11 with a file of xHE-AAC: ffmpeg cannot read that
/// form, therefore it wrote no part and the program said the wrong sentence.
///
/// The function is pure, therefore a test needs no server. See T-68.
pub fn the_sentence_of_no_part(attempts: usize) -> String {
    format!(
        "The server made no part of the stream after {} attempts. Its ffmpeg \
         cannot read the form of this audio.",
        attempts
    )
}

/// Asks for the bytes of one part, with a number of attempts and a largest delay.
///
/// The thread that fills the buffer waits long, because a wait there costs the
/// user nothing. The open waits long too, and **a command of the user stops that
/// wait**: `stops_for_a_command` gives `true` for the open. See T-63 and T-68.
fn ask_for_the_bytes_with_a_limit(
    client: &reqwest::blocking::Client,
    address: &str,
    token: &str,
    patience: Patience,
) -> Result<Vec<u8>, String> {
    let mut backoff = FIRST_BACKOFF;
    let mut last = String::from("The server did not answer.");
    let mut every_answer_was_404 = true;

    for attempt in 0..patience.attempts {
        // The open runs on the thread of the engine. A key of the user must not
        // wait for a server that makes no part. See T-68.
        if patience.a_command_stops_it && crate::player::engine::a_command_waits() {
            info!("[HlsFile] a command of the user stops the wait of the open.");
            return Err("The user asked for something else.".to_string());
        }

        let answer = match client.get(address).bearer_auth(token).send() {
            Ok(answer) => answer,
            Err(_) => {
                last = "The server did not answer.".to_string();
                every_answer_was_404 = false;
                wait(&mut backoff, attempt, patience.longest);
                continue;
            }
        };

        // ffmpeg of the server writes the parts while the client reads them.
        // A part that does not exist yet answers 404.
        if let Some(error) = classify_status(answer.status()) {
            every_answer_was_404 = every_answer_was_404 && error == ApiError::NotFound;
            last = text_of(&error);

            // **A stream that the server ended answers 404 for ever.** One
            // request of the playlist gives that answer, therefore the reader
            // stops at once and the user reads the true cause. The reader asks
            // after the second attempt, because a part that is not ready yet is
            // the common answer and it needs no second request. See T-68.
            if error == ApiError::NotFound
                && attempt >= ATTEMPTS_BEFORE_THE_PLAYLIST
                && the_stream_is_gone(client, patience.playlist, token)
            {
                warn!("[HlsFile] the server ended the stream of this media.");
                return Err(the_sentence_of_a_stream_that_ended());
            }

            wait(&mut backoff, attempt, patience.longest);
            continue;
        }

        // **A body that did not come is not a part that did not come, and both
        // need the same road.** The old code gave the answer of `bytes()` back
        // at once: a body that stopped in the middle therefore took **no**
        // second attempt, while every other fault of a part takes twenty of
        // them. A measurement of 2026-08-14 with
        // `docs/harness/a_body_that_stops_in_the_middle.py`: one request of
        // `output-7.ts`, and the book of ten minutes stopped after 42 seconds
        // of it. See T-194.
        every_answer_was_404 = false;

        match answer.bytes() {
            Ok(bytes) if hls::the_part_is_whole(&bytes) => return Ok(bytes.to_vec()),
            Ok(bytes) => {
                // A body with no `Content-Length` ends at the close of the
                // connection, therefore this body holds no fault of its own:
                // the packets of 188 bytes are the truth of its length. See
                // T-194.
                warn!(
                    "[HlsFile] the part of the stream holds {} bytes, and that \
                     is no whole number of packets. The reader asks again.",
                    bytes.len()
                );
                last = hls::the_sentence_of_a_part_that_stopped(name_of(address));
            }
            Err(error) => {
                warn!("[HlsFile] the body of a part did not come: {}", error);
                last = "The part of the stream did not come.".to_string();
            }
        }

        wait(&mut backoff, attempt, patience.longest);
    }

    if every_answer_was_404 {
        return Err(the_sentence_of_no_part(patience.attempts));
    }

    Err(last)
}

/// Waits, and it makes the delay longer.
fn wait(backoff: &mut Duration, attempt: usize, longest: Duration) {
    std::thread::sleep(*backoff);
    *backoff = (*backoff * 2).min(longest);

    if attempt == 0 {
        info!("[HlsFile] the part of the stream is not ready. The reader waits.");
    }
}

/// Gives a short text of a fault of a request.
fn text_of(error: &ApiError) -> String {
    format!("The stream of the server gave a fault: {}", error)
}

/// Reads the parts of the playlist, and it fills the buffer.
/// **A part that does not come is not the end of the media.** The thread says in
/// `report` which place of the media it reached, therefore the engine never
/// holds the end of this reader for the end of the book. See T-194.
fn fill_buffer(
    shared: Arc<Shared>,
    report: Arc<StreamReport>,
    address: String,
    token: String,
    segments: Vec<Segment>,
    from: usize,
    pid: u16,
) {
    let the_stream_stopped = |index: usize| TheStreamStopped {
        seconds: hls::seconds_before(&segments, index),
        why: hls::the_sentence_of_a_stream_that_stopped(),
    };

    let client = match client() {
        Ok(client) => client,
        Err(error) => {
            warn!("[HlsFile] {}", error);
            report.say(the_stream_stopped(from));
            shared.finished.store(true, Ordering::SeqCst);
            shared.signal.notify_all();
            return;
        }
    };

    for (index, part) in segments.iter().enumerate().skip(from) {
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

        let bytes = match ask_for_the_bytes(
            &client,
            &address_of_the_part(&address, &part.name),
            &token,
            &address,
        ) {
            Ok(bytes) => bytes,
            Err(error) => {
                warn!(
                    "[HlsFile] the part {} did not come: {}. The stream stops at \
                     {:.0} seconds of the media, and that is not its end.",
                    part.name,
                    error,
                    hls::seconds_before(&segments, index)
                );
                report.say(the_stream_stopped(index));
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

    /// A 404 of a part is not a media that the server does not have. The old
    /// sentence came from `classify_status`, and the user read "The server does
    /// not have this item" for a book that stands in their library. See T-68.
    #[test]
    fn the_sentence_of_a_part_that_the_server_never_made() {
        let sentence = the_sentence_of_no_part(14);

        assert!(
            sentence.contains("made no part"),
            "the sentence must name the part: {}",
            sentence
        );
        assert!(
            sentence.contains("14"),
            "the sentence must hold the number of the attempts: {}",
            sentence
        );
        assert!(
            !sentence.contains("does not have this item"),
            "the sentence must not say that the item is absent: {}",
            sentence
        );
        assert!(
            sentence.contains("form of this audio"),
            "the sentence must name the true cause: {}",
            sentence
        );
    }

    /// The commands of the user that make a long open pointless. `SetVolume`
    /// and `SetSpeed` must not stop an open, because the sleep timer sends them
    /// while a playback runs. See T-68.
    #[test]
    fn a_key_of_the_user_stops_the_wait_of_the_open() {
        use crate::player::engine::PlayerCommand;

        assert!(PlayerCommand::Stop.the_user_does_not_want_the_open());
        assert!(PlayerCommand::Pause.the_user_does_not_want_the_open());
        assert!(PlayerCommand::SeekTo(10.0).the_user_does_not_want_the_open());
        assert!(PlayerCommand::SeekBy(-10.0).the_user_does_not_want_the_open());

        assert!(!PlayerCommand::SetVolume(0.5).the_user_does_not_want_the_open());
        assert!(!PlayerCommand::SetSpeed(1.5).the_user_does_not_want_the_open());
        assert!(!PlayerCommand::Resume.the_user_does_not_want_the_open());
    }

    /// A stream that the server ended must stop the reader at once. The
    /// sentence must not say that the item is absent. See T-68.
    #[test]
    fn the_sentence_of_a_stream_that_the_server_ended() {
        let sentence = the_sentence_of_a_stream_that_ended();

        assert!(sentence.contains("ended the stream"), "{}", sentence);
        assert!(sentence.contains("form of this audio"), "{}", sentence);
        assert!(
            !sentence.contains("does not have this item"),
            "{}",
            sentence
        );
    }

    /// The reader asks for the playlist after the first attempts only. A part
    /// that ffmpeg did not write yet is the common answer, and it must cost one
    /// request. See T-68.
    #[test]
    fn the_reader_does_not_ask_for_the_playlist_at_the_first_attempt() {
        // The open must reach the examination of the playlist, and it must not
        // make that request at the first attempt of a part.
        let attempts_that_ask: Vec<usize> = (0..ATTEMPTS_OF_THE_OPEN)
            .filter(|attempt| *attempt >= ATTEMPTS_BEFORE_THE_PLAYLIST)
            .collect();

        assert_eq!(attempts_that_ask.first(), Some(&2));
        assert_eq!(attempts_that_ask.len(), ATTEMPTS_OF_THE_OPEN - 2);
    }

    /// The open must wait longer than the server needs to see that its own
    /// transcode died. A measurement of 2026-08-11 gave ten seconds for that,
    /// and the server then starts ffmpeg again. See T-68.
    #[test]
    fn the_open_waits_longer_than_the_server_needs_for_its_second_try() {
        // The first delay, then the double of it, and then the longest wait for
        // every attempt after those two.
        let mut total = Duration::from_secs(0);
        let mut backoff = FIRST_BACKOFF;

        for _ in 0..ATTEMPTS_OF_THE_OPEN {
            total += backoff;
            backoff = (backoff * 2).min(LONGEST_WAIT_OF_THE_OPEN);
        }

        assert!(
            total >= Duration::from_secs(20),
            "the open waits {:?}, and the server needs ten seconds to see a \
             transcode that died, and more for the first part of the second try",
            total
        );
    }

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
