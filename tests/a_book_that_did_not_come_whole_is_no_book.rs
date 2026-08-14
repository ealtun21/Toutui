//! A book that did not come whole is no book of the disk. See T-186.
//!
//! **The measurement of 2026-08-14, with
//! `docs/harness/a_body_that_stops_in_the_middle.py`.** The proxy gave the head
//! of `GET /api/items/:id/ebook` and the first 60000 bytes of the EPUB of
//! `Alice in Wonderland` of 136761 bytes, and it then closed the connection.
//! The program said:
//!
//! ```text
//! The program did not get the book: The answer of the server is not valid:
//! error decoding response body
//! ```
//!
//! and a file of **60000 bytes** stood on the disk with the name of a whole
//! book: `downloads/toutuitest/8fda6e43-….epub`. The key `e` of the same media
//! after it made **no request of the server** (`grep -c` of the log of the
//! proxy: 3 before the key and 3 after it), and the program said:
//!
//! ```text
//! This file is not an EPUB.
//! ```
//!
//! **A program of the whole server said the same words.** The account took the
//! address of the sandbox again, a new program started, and the key `e` gave
//! that same sentence for a book that the server holds whole: the file of the
//! disk is the truth of the reader, and `get_the_ebook_of` asks the server for
//! nothing when that file exists. The one road out was the key `X` of the list,
//! and the sentence of the fault names no key at all. After that key the book
//! came whole (136761 bytes) and the reader opened it at its chapter 3.
//!
//! **`fetch.rs` of the download of the audio holds the rule already**: "the
//! function gives the file the name `.part` first … therefore a file without
//! `.part` is always complete" (T-179). The download of the ebook wrote the
//! name of the whole book from the first byte.
//!
//! **The parts of this test stay in one function.** It writes `XDG_DATA_HOME`
//! and `XDG_CONFIG_HOME`, and those are boxes of the process: two test
//! functions of one binary would fight for them (the shape of T-144 and of
//! T-157).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;

/// The bytes of the book of this test. The first bytes are the bytes of a file
/// of ZIP, therefore `the_file_is_a_pdf` gives `false` and the name of the file
/// stays the name of an EPUB.
fn the_book() -> Vec<u8> {
    let mut bytes = b"PK\x03\x04".to_vec();
    bytes.extend((0..4096u32).map(|number| (number % 251) as u8));
    bytes
}

/// The number of the bytes of the body that the first answer holds.
const THE_PART: usize = 100;

/// A server of a raw socket that stops the body of the **first** answer of the
/// ebook in the middle, and that then gives the whole book.
///
/// **A server of a fault of this shape needs no network and no sandbox** (T-167
/// and T-169): a port that no program holds refuses a connection at once, and
/// this test needs a server that answers with a head and that then goes away.
async fn the_server_that_stops_the_first_body(book: Vec<u8>) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let of_the_task = Arc::clone(&count);

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };

            let of_this = Arc::clone(&of_the_task);
            let book = book.clone();

            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut buffer = [0u8; 2048];

                loop {
                    match socket.read(&mut buffer).await {
                        Ok(0) | Err(_) => return,
                        Ok(read) => head.extend_from_slice(&buffer[..read]),
                    }

                    if head.windows(4).any(|four| four == b"\r\n\r\n") {
                        break;
                    }
                }

                let line = String::from_utf8_lossy(&head)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();

                if line.contains("/ebook") {
                    let which = of_this.fetch_add(1, Ordering::SeqCst);

                    let answer = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/epub+zip\r\n\
                         Content-Length: {}\r\nConnection: close\r\n\r\n",
                        book.len()
                    );

                    let _ = socket.write_all(answer.as_bytes()).await;

                    // **The head says the length of the whole book, and the
                    // body stops.** This is the fault of the network of the
                    // real world, and not a status of a fault.
                    let body = if which == 0 {
                        &book[..THE_PART]
                    } else {
                        &book[..]
                    };

                    let _ = socket.write_all(body).await;
                    let _ = socket.flush().await;
                    return;
                }

                // Every other path is the path of `why_the_book_did_not_come`.
                let body = b"{}";
                let answer = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(answer.as_bytes()).await;
                let _ = socket.write_all(body).await;
                let _ = socket.flush().await;
            });
        }
    });

    (address, count)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_book_that_did_not_come_whole_is_no_book() {
    let book = the_book();
    let (address, requests) = the_server_that_stops_the_first_body(book.clone()).await;

    let directory = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_DATA_HOME", directory.path());
    std::env::set_var("XDG_CONFIG_HOME", directory.path());

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    let path = toutui::logic::reader::session::ebook_path_of("someone", "an-item", None);

    // 1. The body of the answer stops in the middle. The program says so.
    let fault = toutui::logic::reader::session::get_the_ebook_of(&api, "someone", "an-item", None)
        .await
        .expect_err("a book that did not come whole is a fault");

    assert!(
        !fault.is_empty(),
        "the program says why the book did not come"
    );

    // 2. **The disk holds no book.** A part of a book with the name of a whole
    //    book is the fault of T-186: the reader of every program of this
    //    account after it opens that part, and it asks the server for nothing.
    assert!(
        !path.exists(),
        "a part of a book must not hold the name of a whole book: {} of {} byte(s)",
        path.display(),
        std::fs::metadata(&path).map(|of| of.len()).unwrap_or(0)
    );

    let of_the_pdf = toutui::logic::reader::session::pdf_path_of("someone", "an-item", None);

    assert!(
        !of_the_pdf.exists(),
        "the name of a PDF holds no part of a book either"
    );

    // 3. The key of the user asks the server again, and the book comes whole.
    let came = toutui::logic::reader::session::get_the_ebook_of(&api, "someone", "an-item", None)
        .await
        .expect("the book of the second request comes");

    assert_eq!(
        requests.load(Ordering::SeqCst),
        2,
        "the second open of the book asks the server again"
    );

    assert_eq!(
        std::fs::read(&came).expect("the file of the book"),
        book,
        "the file of the disk holds every byte of the book of the server"
    );
}
