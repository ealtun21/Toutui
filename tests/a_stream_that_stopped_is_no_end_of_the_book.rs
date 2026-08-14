//! A body of the audio that stops in the middle is not the end of the book.
//! See T-193.
//!
//! `HttpFile` reads the file of a book with range requests, and a thread fills
//! a buffer. That thread reads the answer of the server byte by byte, and it
//! holds two roads: a read that fails is a connection that stopped, and the
//! thread asks the server again from the byte that it holds; a read of **zero**
//! bytes is the end of the file, and the thread stops.
//!
//! **A connection that closes with no fault gives a read of zero bytes too.** A
//! body with no `Content-Length` ends at the close of the connection (RFC 9112,
//! section 6.3): a proxy in front of Audiobookshelf that loses its own
//! connection to the server therefore gives the client a body of the status 206
//! that stops in the middle, and the client reads a clean end of the file.
//!
//! The measurement of this file: a server of a raw socket answers the first
//! range request with 100 bytes of a file of 1000, with no `Content-Length` and
//! with `Connection: close`. The reader must give the 1000 bytes of the book.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use toutui::player::engine::http_file::HttpFile;

/// The bytes of the file of the test. Each byte holds its own place, therefore
/// the test can name the byte where a read stopped.
fn content(size: usize) -> Vec<u8> {
    (0..size).map(|value| (value % 251) as u8).collect()
}

/// Reads the head of a request, and gives the first byte of its `Range` and
/// the last one.
///
/// **`HttpFile::open` and the thread of the buffer both ask for the byte 0**:
/// the request of `open` is `bytes=0-0`, and the request of the thread is
/// `bytes=0-`. The end of the range is therefore the one value that names the
/// two of them apart.
fn the_range_of(stream: &TcpStream) -> Option<(u64, Option<u64>)> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut range = None;

    loop {
        let mut line = String::new();

        if reader.read_line(&mut line).ok()? == 0 {
            return range;
        }

        let clean = line.trim_end();

        if clean.is_empty() {
            return range;
        }

        if let Some(value) = clean.to_ascii_lowercase().strip_prefix("range: bytes=") {
            let mut parts = value.split('-');
            let start = parts.next().and_then(|one| one.parse().ok());
            let end = parts.next().and_then(|one| one.parse().ok());

            if let Some(start) = start {
                range = Some((start, end));
            }
        }
    }
}

/// A server of a raw socket that stops the body of its first range request.
///
/// The answers, in the sequence of the requests of the reader:
///
/// 1. `Range: bytes=0-0` of `HttpFile::open`: one byte, with the header
///    `Content-Range` that gives the size 1000.
/// 2. The first request of the thread of the buffer: the status 206, **no**
///    `Content-Length`, `Connection: close`, 100 bytes, and the close.
/// 3. Every request after it: the bytes from the place of the request, with a
///    `Content-Length` of its own.
///
/// The count of the answers of the shape 2 comes back to the caller.
fn a_server_that_stops_the_body(body: Vec<u8>) -> (String, Arc<AtomicUsize>, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port of the loopback");
    let address = format!("http://{}", listener.local_addr().expect("the address"));
    let stopped = Arc::new(AtomicUsize::new(0));

    let inside = listener.try_clone().expect("a second handle of the port");
    let count = Arc::clone(&stopped);

    std::thread::spawn(move || {
        let total = body.len();

        for stream in inside.incoming() {
            let Ok(mut stream) = stream else { return };

            let Some((start, end)) = the_range_of(&stream) else {
                continue;
            };
            let start = (start as usize).min(total);

            // The request of `HttpFile::open`, which reads the size. It names
            // the last byte of its range, and no request of the thread does.
            if end.is_some() {
                let head = format!(
                    "HTTP/1.1 206 Partial Content\r\naccept-ranges: bytes\r\n\
                     content-range: bytes 0-0/{}\r\ncontent-length: 1\r\n\r\n",
                    total
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body[..1]);
                let _ = stream.flush();
                continue;
            }

            // The body that stops in the middle. It comes one time.
            if count.fetch_add(1, Ordering::SeqCst) == 0 {
                let head = format!(
                    "HTTP/1.1 206 Partial Content\r\naccept-ranges: bytes\r\n\
                     content-range: bytes {}-{}/{}\r\nconnection: close\r\n\r\n",
                    start,
                    total - 1,
                    total
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body[start..start + 100]);
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
                continue;
            }

            let rest = &body[start..];
            let head = format!(
                "HTTP/1.1 206 Partial Content\r\naccept-ranges: bytes\r\n\
                 content-range: bytes {}-{}/{}\r\ncontent-length: {}\r\n\r\n",
                start,
                total - 1,
                total,
                rest.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(rest);
            let _ = stream.flush();
        }
    });

    (address, stopped, listener)
}

/// **The parts of this test stay in one function.** The count of the answers
/// belongs to one server, and a second function of this module would take the
/// slot of the thread of that server. See the shape of T-144 and of T-157.
///
/// The fault, before the correction: the thread of the buffer read the close of
/// the connection as the end of the file, and `HttpFile` gave **100** bytes of
/// a book of 1000. The decoder then held a book that ends after its first
/// second, the playback went to its end with no word, and the queue took the
/// media after it.
#[test]
fn a_body_that_stops_in_the_middle_is_no_end_of_the_file() {
    let expected = content(1000);
    let (address, stopped, keep) = a_server_that_stops_the_body(expected.clone());

    // `reqwest::blocking` stops the program inside a task of tokio, therefore
    // the reader runs in a thread of its own. See the trap 25.
    let (send, take) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let got = (|| {
            let mut file = HttpFile::open(&address, "test-token", "item1", "ino1")
                .map_err(|error| error.to_string())?;
            let size = file.len();
            let mut got = Vec::new();
            file.read_to_end(&mut got).map_err(|e| e.to_string())?;
            Ok::<(u64, Vec<u8>), String>((size, got))
        })();

        let _ = send.send(got);
    });

    // A test must not call a function that may never come back. The reader
    // waits 500 milliseconds after the body that stopped, therefore ten
    // seconds is a long time for this work.
    let got = take
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the reader must come back")
        .expect("the reader must open the file");

    assert_eq!(got.0, 1000, "the header Content-Range gives the size");
    assert_eq!(
        got.1.len(),
        1000,
        "a body that stops in the middle is not the end of the book: the reader \
         must ask the server again from the byte that it holds"
    );
    assert_eq!(got.1, expected, "the bytes of the book must be correct");
    assert!(
        stopped.load(Ordering::SeqCst) >= 2,
        "the reader must send a second request after the body that stopped"
    );

    drop(keep);
}
