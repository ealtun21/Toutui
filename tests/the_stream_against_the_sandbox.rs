//! The stream of the server plays a file that no decoder of the program reads.
//! See T-53.
//!
//! The program reads the file of the server itself, and it plays 17 forms. A
//! file of a form that it does not read stopped a part of a book before this
//! work. Audiobookshelf gives every media as a stream of HLS as well, and
//! ffmpeg of the server makes that stream. Therefore **every codec that ffmpeg
//! reads becomes a codec that Toutui plays.**
//!
//! Continuous integration does not run this test, because it needs a server.
//! Start the sandbox of `docs/TEST-SERVER.md`, and then run:
//!
//! ```text
//! ALSA_CONFIG_PATH=/dev/null cargo test --test the_stream_against_the_sandbox \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **The test needs a book of two files, and one file of a form that the
//! program does not read.** `docs/TEST-SERVER.md` says how to make it with
//! ffmpeg. The test says what is absent, and it does not fail for a sandbox
//! that has no such book.

use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::library_items::play_lib_item_or_pod::post_a_stream_session;
use toutui::player::engine::hls;
use toutui::player::engine::hls_file::HlsFile;

const SERVER: &str = "http://127.0.0.1:13399";

/// The title of the book of the measurement. It holds `01 - Part 1.mp3` and
/// `02 - Part 2.wma`, and the program plays no WMA file. See T-18.
const TITLE: &str = "One File With No Decoder";

mod common;
use common::token;

/// The whole way: the session of the stream, the playlist, and the audio of one
/// part.
///
/// The parts of this test stay in one function, because the session of the
/// server holds the state of ffmpeg.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the sandbox server of docs/TEST-SERVER.md on port 13399, and a book of two files"]
async fn the_stream_of_the_server_gives_audio_that_the_program_reads() {
    let token = token().await;
    let pool = Arc::new(EndpointPool::new(vec![Endpoint::new(SERVER, 0)]));
    let api = Arc::new(ApiClient::new(Arc::clone(&pool), token.clone()).unwrap());

    let libraries: serde_json::Value = api
        .get_json("/api/libraries")
        .await
        .expect("the server must give the libraries");

    let library = libraries["libraries"]
        .as_array()
        .expect("the answer must hold a list")
        .iter()
        .find(|library| library["mediaType"] == "book")
        .expect("the sandbox must hold a library of books")["id"]
        .as_str()
        .expect("a library must hold an identity")
        .to_string();

    let items: serde_json::Value = api
        .get_json(&format!("/api/libraries/{}/items?limit=100", library))
        .await
        .expect("the server must give the items");

    let item = items["results"]
        .as_array()
        .expect("the answer must hold a list")
        .iter()
        .find(|item| item["media"]["metadata"]["title"].as_str() == Some(TITLE));

    let Some(item) = item else {
        println!(
            "the sandbox holds no book \"{}\". Read docs/TEST-SERVER.md and make \
             it. The test stops here, and it does not fail.",
            TITLE
        );
        return;
    };

    let item_id = item["id"].as_str().expect("an item holds an identity");
    println!("the item of the test: {}", item_id);

    // The server makes one stream of the whole media, therefore a book of two
    // files gives one track.
    let stream = post_a_stream_session(&api, item_id, None)
        .await
        .expect("the server must give a stream");

    println!(
        "the stream: {} of {:.1} seconds",
        stream.playlist, stream.duration
    );

    assert!(
        stream.playlist.ends_with(".m3u8"),
        "the server must name a playlist: {}",
        stream.playlist
    );

    // The stream holds the whole media. The book holds 30 minutes of MP3 and 30
    // seconds of WMA, therefore the length is more than 30 minutes.
    assert!(
        stream.duration > 1800.0,
        "the stream must hold every file: {} seconds",
        stream.duration
    );

    // **The reader belongs to a thread, and not to a task.** It uses the client
    // of `reqwest` that does not wait, and that client makes a runtime of its
    // own. A runtime inside a task of tokio stops the program when it goes away.
    // The engine of the program is a thread, therefore the real program has no
    // such fault. See the traps of `docs/HANDOVER.md`.
    let playlist = stream.playlist.clone();
    let token_of_the_thread = token.clone();

    let measurement = std::thread::spawn(move || {
        use std::io::Read;

        // The reader asks for the playlist and for the first part. It gives a
        // fault for a form that no decoder of the program reads, therefore this
        // call proves that the audio of the stream is a form that the program
        // plays.
        let mut file = HlsFile::open(SERVER, &token_of_the_thread, &playlist, 0.0)
            .expect("the reader must open the stream");

        let form = file.form();
        let offset = file.offset();

        let mut head = [0u8; 4];
        let count = file.read(&mut head).expect("the reader must give bytes");

        // The place of the media gives the part of the playlist. A reader that
        // starts inside the media names that place, and the loop of the playback
        // adds it.
        let later = HlsFile::open(SERVER, &token_of_the_thread, &playlist, 60.0)
            .expect("the reader must open the stream at a later place");

        (form, offset, head, count, later.offset())
    })
    .join()
    .expect("the thread of the measurement must finish");

    let (form, offset, head, count, offset_of_the_second_reader) = measurement;

    println!("the audio of the stream is {:?}", form);
    assert!(form.a_decoder_of_the_program_reads_it());
    assert_eq!(offset, 0.0);

    // The bytes of the reader are an elementary stream: the sync of a frame of
    // MPEG audio, and no header of a container.
    assert!(count > 0, "the reader must give bytes");
    assert_eq!(head[0], 0xff, "the first byte must be a sync: {:?}", head);
    assert_eq!(head[1] & 0xf0, 0xf0, "the sync needs four bits: {:?}", head);

    println!(
        "the reader of the second 60 starts at {}",
        offset_of_the_second_reader
    );
    assert!(offset_of_the_second_reader > 0.0);
    assert!(offset_of_the_second_reader <= 60.0);

    // The session must not stay open on the server.
    toutui::api::sessions::close_open_session::close_session_without_send_prg_data(
        &api,
        &stream.session_id,
    )
    .await
    .expect("the server must close the session");
}

/// The playlist and the transport stream need no server. This test holds the
/// forms that the program refuses, because a form of LATM comes with xHE-AAC.
#[test]
fn the_program_refuses_a_form_that_no_decoder_reads() {
    assert!(hls::Form::of_the_number(0x03).a_decoder_of_the_program_reads_it());
    assert!(hls::Form::of_the_number(0x0f).a_decoder_of_the_program_reads_it());
    assert!(!hls::Form::of_the_number(0x11).a_decoder_of_the_program_reads_it());
}
