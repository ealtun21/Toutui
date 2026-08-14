//! A part of a stream that holds no audio is not the end of the media. See
//! T-195.
//!
//! **T-194 gave the reader the truth of the length of its stream, and one road of
//! a part kept the fault.** `fill_buffer` asked for a part, it read the audio of
//! that part, and a part with no packet of the audio took a line of the log and
//! the part after it. The thread then reached the last part of the playlist, it
//! said that it read every byte, and the engine gave the end of the **whole**
//! media: the program wrote that place to the server with `isFinished`.
//!
//! A measurement of 2026-08-14 against the sandbox with
//! `docs/harness/a_part_that_holds_no_audio.py` and the book of xHE-AAC of ten
//! minutes: the reader held the audio of the part 0 alone (six seconds), the log
//! held 100 lines of "the part output-N.ts holds no audio", and the server then
//! held `currentTime: 600`, `progress: 1`, and `isFinished: true`. The screen
//! said nothing at all, and the book left the shelf Continue Listening.
//!
//! **Such a part comes of a server that started its ffmpeg again.** ffmpeg of
//! Audiobookshelf writes the parts while the client reads them, and the server
//! starts it again with `-c:a aac` when the first try dies (T-68). The identity
//! of the audio of the new parts belongs to the new ffmpeg, and this reader holds
//! the identity of the first part alone.
//!
//! **This test needs no sandbox and no sound device.** A host of a raw socket
//! gives the playlist and the parts, and the part of the fixture is the exact
//! part of an Audiobookshelf 2.36.0 of the measurement of 2026-08-11.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use toutui::player::engine::hls_file::HlsFile;

/// One part of a transport stream of an Audiobookshelf 2.36.0. It holds 143
/// packets of 188 bytes, and its audio is 24033 bytes of MP3.
const PART: &[u8] = include_bytes!("fixtures/audio/transport_stream.ts");

/// The audio of one whole part.
const AUDIO_OF_A_PART: usize = 24033;

/// The length of one packet of a transport stream.
const PACKET: usize = 188;

/// The number of packets of the head of the part of the fixture: the table of
/// the service, the table of the programs, and the table of the map. Those three
/// name the audio, and they hold no byte of it.
const TABLES_OF_THE_HEAD: usize = 3;

/// The playlist of the test: four parts of six seconds, and the end.
const PLAYLIST: &str = "#EXTM3U\n\
    #EXT-X-VERSION:3\n\
    #EXT-X-TARGETDURATION:6\n\
    #EXT-X-MEDIA-SEQUENCE:0\n\
    #EXT-X-PLAYLIST-TYPE:VOD\n\
    #EXTINF:6,\n\
    output-0.ts\n\
    #EXTINF:6,\n\
    output-1.ts\n\
    #EXTINF:6,\n\
    output-2.ts\n\
    #EXTINF:6,\n\
    output-3.ts\n\
    #EXT-X-ENDLIST\n";

/// Gives one packet of padding: the identity 0x1FFF, and no audio.
fn a_packet_of_padding(number: usize) -> Vec<u8> {
    let mut packet = vec![0x47, 0x1F, 0xFF, 0x10 | (number as u8 % 16)];
    packet.resize(PACKET, 0xFF);
    packet
}

/// Gives a part that holds the tables of the container and no audio.
///
/// **This body is a whole body.** It holds 35 packets of 188 bytes, therefore the
/// rule of T-194 (a body of no whole number of packets is a body that stopped)
/// says nothing of it, and the client of `reqwest` reads a clean end of it. The
/// three tables of the head name the audio of the identity 0x100, and no packet
/// of that identity stands in it.
fn a_part_with_no_audio() -> Vec<u8> {
    let mut bytes = PART[..TABLES_OF_THE_HEAD * PACKET].to_vec();

    for number in 0..32 {
        bytes.extend(a_packet_of_padding(number));
    }

    bytes
}

/// A host of a raw socket that gives a stream whose part 1 holds no audio.
///
/// The answers:
///
/// - `output.m3u8`: the playlist of four parts, at every request.
/// - `output-1.ts`: the tables of the container and 32 packets of padding.
/// - every other part: the whole part of the fixture.
fn a_host_of_a_stream_with_a_part_of_no_audio() -> (String, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port of the loopback");
    let address = format!("http://{}", listener.local_addr().expect("the address"));

    let inside = listener.try_clone().expect("a second handle of the port");
    let no_audio = Arc::new(a_part_with_no_audio());

    std::thread::spawn(move || {
        for stream in inside.incoming() {
            let Ok(mut stream) = stream else { return };

            let Some(path) = the_path_of(&stream) else {
                continue;
            };

            let whole = |stream: &mut TcpStream, bytes: &[u8], kind: &str| {
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {}\r\ncontent-length: {}\r\n\
                     connection: close\r\n\r\n",
                    kind,
                    bytes.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(bytes);
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
            };

            if path.ends_with("output.m3u8") {
                whole(&mut stream, PLAYLIST.as_bytes(), "application/x-mpegURL");
                continue;
            }

            if path.ends_with("output-1.ts") {
                whole(&mut stream, &no_audio, "video/MP2T");
                continue;
            }

            whole(&mut stream, PART, "video/MP2T");
        }
    });

    (address, listener)
}

/// Gives the path of the request of a connection.
fn the_path_of(stream: &TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut first = String::new();

    if reader.read_line(&mut first).ok()? == 0 {
        return None;
    }

    let path = first.split_whitespace().nth(1)?.to_string();

    // The rest of the head goes away, therefore the client sees no fault of a
    // request that this test did not read.
    loop {
        let mut line = String::new();

        if reader.read_line(&mut line).ok()? == 0 || line.trim_end().is_empty() {
            return Some(path);
        }
    }
}

/// Reads a stream of the host, in a thread of its own.
///
/// `reqwest::blocking` stops the program inside a task of tokio, therefore the
/// reader runs in a thread of its own (the trap 25). **A test must not call a
/// function that may never come back**, therefore the caller reads the end of
/// that thread with a limit of time.
type TheAnswerOfTheReader = Result<
    (
        usize,
        Option<toutui::player::engine::hls_file::TheStreamStopped>,
    ),
    String,
>;

fn the_stream_of(address: &str, seconds: f64) -> TheAnswerOfTheReader {
    let (send, take) = std::sync::mpsc::channel();
    let address = address.to_string();

    std::thread::spawn(move || {
        let answer = (|| {
            let mut file = HlsFile::open(&address, "test-token", "/hls/x/output.m3u8", seconds)?;
            let report = file.report();
            let mut got = Vec::new();
            file.read_to_end(&mut got)
                .map_err(|error| error.to_string())?;
            Ok((got.len(), report.the_stream_stopped()))
        })();

        let _ = send.send(answer);
    });

    take.recv_timeout(std::time::Duration::from_secs(30))
        .expect("the reader must come back")
}

/// **The parts of this test stay in one function.** The host belongs to one
/// thread, and a second function of this module would take the slot of that
/// thread. See the shape of T-144 and of T-157.
///
/// The fault, before the correction:
///
/// 1. The reader gave the audio of the parts 0, 2, and 3, and the report of the
///    stream held nothing: the engine then gave the end of the whole media for a
///    book of which the user heard 18 seconds of 24, and the program wrote that
///    place to the server with `isFinished`.
/// 2. The open of the stream at the part that holds no audio gave a reader of no
///    byte at all. The decoder read the end of the book at its first read, and
///    the same fault came with no line of the log of the parts.
#[test]
fn a_part_that_holds_no_audio_is_not_the_end_of_the_media() {
    let (address, keep) = a_host_of_a_stream_with_a_part_of_no_audio();

    // **The road of the thread of the buffer.** The reader holds the audio of the
    // part 0, and the part 1 holds no audio: the stream stops there.
    let (bytes, stopped) = the_stream_of(&address, 0.0).expect("the reader must open the stream");

    assert_eq!(
        bytes, AUDIO_OF_A_PART,
        "the reader must give the audio of the part 0 alone: a part that holds no \
         audio stops the stream, and the parts after it belong to no playback"
    );

    let stopped = stopped.expect(
        "a stream that met a part with no audio must say so: the engine reads \
         this, and a media that stops in the middle is not a media that the user \
         finished",
    );

    assert_eq!(
        stopped.seconds, 6.0,
        "the parts before the part that holds no audio hold six seconds of the \
         media"
    );
    assert!(
        stopped.why.contains("stopped"),
        "the user must read why the media stopped: {}",
        stopped.why
    );

    // **The road of the open.** A playback that starts at the part 1 meets that
    // part at its first request: the playback does not start, and the user reads
    // why. A reader of no byte is the same fault of the user, and it says
    // nothing.
    let fault = the_stream_of(&address, 6.0).expect_err(
        "an open whose first part holds no audio must give a fault: a reader of \
         no byte gives the end of the media at its first read",
    );

    assert!(
        fault.contains("output-1.ts") && fault.contains("no audio"),
        "the fault of the open must name the part and its cause: {}",
        fault
    );

    drop(keep);
}
