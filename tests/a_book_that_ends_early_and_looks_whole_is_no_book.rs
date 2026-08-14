//! A body of a book that ends early and that looks whole is no book. See T-196.
//!
//! **The measurement of 2026-08-14, with
//! `docs/harness/a_body_that_ends_early_and_looks_whole.py`.** The proxy gave
//! the head of `GET /api/items/:id/ebook` with **no** `Content-Length` and no
//! `Transfer-Encoding`, the first 20000 bytes of the EPUB of
//! `Alice in Wonderland` of 136761 bytes, and it then closed the connection. A
//! body with neither of those headers ends at the close of the connection (RFC
//! 9112, section 6.3), therefore `reqwest` read a **clean** end of that body:
//! no fault of the network, no fault of a status, and 116761 bytes fewer than
//! the file holds.
//!
//! The program wrote those 20000 bytes under the name of the whole book,
//! `downloads/toutuitest/8fda6e43-….epub`, and the screen said:
//!
//! ```text
//! This file is not an EPUB.
//! ```
//!
//! **That is the fault of the user of T-186 again, by the road that the
//! correction of T-186 does not hold.** The key `h` and the key `e` after it
//! made **no request of the server** (`grep -c` of the log of the proxy: the
//! same count before the key and after it): `get_the_ebook_of` finds the file
//! of the disk, and the book of the user is broken for every program of that
//! account until the key `X`.
//!
//! **The head of the answer named no length, therefore the client can count
//! nothing.** T-186 holds the other road: a head that names `Content-Length`
//! gives `reqwest` the fault of an incomplete message. The one truth of the
//! length of such a body stands in the answer of `GET /api/items/:id`, and
//! `metadata.size` of the file of the ebook is that number (T-179 gave the
//! same field to the download of the audio).
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

/// The identity of the file of the ebook, in the answer of the item.
const THE_INO: &str = "42";

/// A server of a raw socket that ends the body of the **first** answer of the
/// ebook early, with a head that names no length, and that then gives the whole
/// book.
///
/// **A server of a fault of this shape needs no network and no sandbox** (T-167
/// and T-169).
async fn the_server_that_ends_the_first_body_early(book: Vec<u8>) -> (String, Arc<AtomicUsize>) {
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

                    // **The head of the first answer names no length at all.**
                    // A body with no `Content-Length` and no
                    // `Transfer-Encoding` ends at the close of the connection,
                    // therefore the client reads a clean end of a body that
                    // holds a part of the file. See T-193.
                    let answer = if which == 0 {
                        "HTTP/1.1 200 OK\r\nContent-Type: application/epub+zip\r\n\
                         Connection: close\r\n\r\n"
                            .to_string()
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/epub+zip\r\n\
                             Content-Length: {}\r\nConnection: close\r\n\r\n",
                            book.len()
                        )
                    };

                    let _ = socket.write_all(answer.as_bytes()).await;

                    let body = if which == 0 {
                        &book[..THE_PART]
                    } else {
                        &book[..]
                    };

                    let _ = socket.write_all(body).await;
                    let _ = socket.flush().await;
                    return;
                }

                // The answer of the item names the size of the file of the
                // ebook, and that number is the one truth of the length of a
                // body that names none.
                let of_the_item = serde_json::json!({
                    "media": { "ebookFile": { "ino": THE_INO } },
                    "libraryFiles": [{
                        "ino": THE_INO,
                        "fileType": "ebook",
                        "metadata": { "filename": "a-book.epub", "size": book.len() }
                    }]
                });

                let body = serde_json::to_vec(&of_the_item).unwrap_or_default();

                let answer = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );

                let _ = socket.write_all(answer.as_bytes()).await;
                let _ = socket.write_all(&body).await;
                let _ = socket.flush().await;
            });
        }
    });

    (address, count)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_book_that_ends_early_and_looks_whole_is_no_book() {
    let book = the_book();
    let (address, requests) = the_server_that_ends_the_first_body_early(book.clone()).await;

    let directory = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_DATA_HOME", directory.path());
    std::env::set_var("XDG_CONFIG_HOME", directory.path());

    let pool = EndpointPool::new(vec![Endpoint::new(&address, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());

    let path = toutui::logic::reader::session::ebook_path_of("someone", "an-item", None);

    // 1. The body of the answer ends early, and it looks whole. The program
    //    counts the bytes against the size of the answer of the item, and it
    //    says the two numbers.
    let fault = toutui::logic::reader::session::get_the_ebook_of(&api, "someone", "an-item", None)
        .await
        .expect_err("a book that ends early is a fault");

    assert!(
        fault.contains(&THE_PART.to_string()) && fault.contains(&book.len().to_string()),
        "the sentence names the bytes that came and the bytes of the file: {}",
        fault
    );

    // 2. **The disk holds no book.** A part of a book with the name of a whole
    //    book is the fault of T-186 and of T-196: the reader of every program
    //    of this account after it opens that part, and it asks the server for
    //    nothing.
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
