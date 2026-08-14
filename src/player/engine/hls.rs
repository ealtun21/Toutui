//! The stream of the server, for a file that no decoder of the program reads.
//! See T-53.
//!
//! Audiobookshelf gives every media in two ways:
//!
//! 1. **The file itself.** `GET /api/items/:id/file/:ino`. The program reads
//!    the file with its own decoders. This way is the first choice: it needs no
//!    work of the server, and it holds the exact bytes of the book.
//! 2. **A stream of the server.** `POST /api/items/:id/play` with
//!    `forceTranscode` gives `playMethod: 2` and one address of HLS for the
//!    whole media. ffmpeg of the server makes that stream, therefore **every
//!    codec that ffmpeg reads becomes a codec that Toutui plays.**
//!
//! The second way needs no new dependency of this program, and it holds the
//! rule of T-20. The work is a reader of two forms of text and of one container:
//!
//! - **The playlist** of HLS is a list of lines. `parse_playlist` reads it.
//! - **A segment** is an MPEG transport stream: packets of 188 bytes. The audio
//!   inside it is an elementary stream of MP3 or of ADTS AAC, and both are
//!   self-framing. Therefore the program takes the payload of the packets of one
//!   identity and gives those bytes to symphonia, which reads both forms
//!   already.
//!
//! A measurement on 2026-08-11 against an Audiobookshelf 2.36.0:
//! `output-0.ts` of 26884 bytes holds 143 packets, and the identity 256 gives
//! 24033 bytes that start with `ff f3`, the sync of an MP3 frame. The whole
//! segment stands in `tests/fixtures/audio/transport_stream.ts`, and the tests
//! of this module read it.
//!
//! Every function here is pure. Therefore a test needs no server and no sound
//! card.

/// The size of one packet of a transport stream.
const PACKET: usize = 188;

/// The first byte of every packet.
const SYNC: u8 = 0x47;

/// The identity of the table that names the programs.
const PAT_PID: u16 = 0;

/// One part of the media, as the playlist of HLS names it.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// The name of the file, as the playlist gives it. An example is
    /// `output-12.ts`.
    pub name: String,
    /// The time of this part, in seconds.
    pub seconds: f64,
}

/// The form of the audio inside a transport stream.
///
/// The value comes from the table of the programs, and the numbers are the
/// numbers of the standard of MPEG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// `0x03` and `0x04`. An elementary stream of MP3.
    Mp3,
    /// `0x0f`. AAC with a header of ADTS before each frame.
    AdtsAac,
    /// `0x11`. AAC inside LATM. **symphonia does not read this form**, and
    /// xHE-AAC of a transport stream comes in it. The program must then ask the
    /// server for AAC of the old form.
    LatmAac,
    /// A form that this program does not know.
    Other(u8),
}

impl Form {
    /// Makes a form of the number of the table of the programs.
    pub fn of_the_number(value: u8) -> Form {
        match value {
            0x03 | 0x04 => Form::Mp3,
            0x0f => Form::AdtsAac,
            0x11 => Form::LatmAac,
            other => Form::Other(other),
        }
    }

    /// Tells if symphonia reads this form.
    ///
    /// MP3 and ADTS are self-framing: the bytes of the payload are a file that
    /// a reader opens. LATM needs the description of the stream of the
    /// container, and symphonia has no reader of it.
    pub fn a_decoder_of_the_program_reads_it(&self) -> bool {
        matches!(self, Form::Mp3 | Form::AdtsAac)
    }
}

/// The audio stream of a transport stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioStream {
    /// The identity of the packets of this stream.
    pub pid: u16,
    /// The form of the audio.
    pub form: Form,
}

/// Reads the playlist of HLS.
///
/// The playlist holds one line `#EXTINF:<seconds>,` before the name of each
/// part. A line that starts with `#` and that is not `#EXTINF` says nothing
/// about a part, therefore the function goes over it.
///
/// A part with no time before it takes the time 0. The program then plays it,
/// and the time of the whole media comes from the media and not from the
/// playlist.
pub fn parse_playlist(text: &str) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut seconds = 0.0;

    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            let value = rest.trim_end_matches(',').trim();
            seconds = value.split(',').next().unwrap_or("").parse().unwrap_or(0.0);
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        segments.push(Segment {
            name: line.to_string(),
            seconds,
        });

        seconds = 0.0;
    }

    segments
}

/// Tells if the body of the playlist is the whole playlist. See T-193.
///
/// **A playlist that stops in the middle names fewer parts, and it gives no
/// fault of its own.** A body with no `Content-Length` ends at the close of the
/// connection (RFC 9112, section 6.3), therefore a proxy in front of the server
/// that loses its own connection gives a clean end of a body that holds a part
/// of the playlist. The parts that the body does not hold then belong to no
/// playback: the book of 30 minutes ends after five of them, and the program
/// tells the server that the user listened to the whole book.
///
/// **A playlist of the type `VOD` names its own end**, with the line
/// `#EXT-X-ENDLIST`. A measurement of an Audiobookshelf 2.36.0 of 2026-08-14:
/// `GET /hls/:session/output.m3u8` gives `#EXT-X-PLAYLIST-TYPE:VOD` with the
/// 300 parts of a book of 30 minutes and that line at its end, two seconds
/// after the start of the session.
///
/// A playlist that names no type is not a playlist of this rule: a server that
/// makes the parts while the client reads them holds no end yet, and this
/// function gives `true` for it.
pub fn the_playlist_is_whole(text: &str) -> bool {
    let mut of_the_type_vod = false;

    for line in text.lines() {
        let line = line.trim();

        if line == "#EXT-X-ENDLIST" {
            return true;
        }

        if line.eq_ignore_ascii_case("#EXT-X-PLAYLIST-TYPE:VOD") {
            of_the_type_vod = true;
        }
    }

    !of_the_type_vod
}

/// The sentence of a playlist that stops in the middle.
pub fn the_sentence_of_a_playlist_that_stopped() -> String {
    "The playlist of the server stopped in the middle. Press the key again.".to_string()
}

/// Tells if the body of one part is the whole part. See T-194.
///
/// **A part of a transport stream holds packets of 188 bytes, and nothing
/// else.** ffmpeg of the server writes a whole number of them, therefore a body
/// whose length is not a multiple of 188 is a body that stopped in the middle.
/// A body with no `Content-Length` ends at the close of the connection (RFC
/// 9112, section 6.3), therefore `reqwest` reads a **clean** end of such a body
/// and the client sees no fault at all: `packets` then drops the packet that
/// the body cut, and the sound of the user holds a hole of some seconds with no
/// word of it.
///
/// **This is the truth of the length of a part.** A part names a time and no
/// number of bytes, therefore the playlist gives no length; the container gives
/// it. A body of no byte is no part either.
pub fn the_part_is_whole(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.len().is_multiple_of(PACKET)
}

/// The sentence of a part whose body stops in the middle.
pub fn the_sentence_of_a_part_that_stopped(name: &str) -> String {
    format!(
        "The body of the part {} of the stream stopped in the middle.",
        name
    )
}

/// The sentence of a part of the stream that holds no packet of the audio.
///
/// **A part with no audio is not a part of silence** (T-195). The tables of such
/// a part name an audio, and the body of it holds no byte of that audio: a
/// reader that goes to the part after it gives a book of silence, and the
/// program then tells the server that the user finished the media. The first
/// part of a stream gives this sentence to the user, because the playback of it
/// does not start.
///
/// **The caller of the open adds what the user can do**
/// (`the_message_of_a_stream_that_did_not_play`), therefore this sentence names
/// the fault alone and it stays short: the message stands in one row of the
/// screen (the trap of the message of T-68).
pub fn the_sentence_of_a_part_with_no_audio(name: &str) -> String {
    format!(
        "The part {} of the stream of the server holds no audio.",
        name
    )
}

/// The sentence for the user, for a stream that did not reach its last part.
///
/// **A stream that stops is not the end of the media** (T-194). The engine held
/// the end of the reader for the end of the book: the program then wrote the
/// whole place of the media, it told the server that the user finished the
/// book, and the screen said nothing at all.
///
/// The message lives on one line of the screen, therefore it stays under 150
/// letters (the trap of the message of T-68).
pub fn the_sentence_of_a_stream_that_stopped() -> String {
    "The stream of the server stopped before the end of this media. Press the \
     key of the media again to go on."
        .to_string()
}

/// Gives the time of every part before the part of the number `index`.
///
/// The engine needs it to know the place of the playback inside the media when
/// it starts at a part that is not the first one.
pub fn seconds_before(segments: &[Segment], index: usize) -> f64 {
    segments.iter().take(index).map(|part| part.seconds).sum()
}

/// Gives the number of the part that holds one place of the media, and the
/// place inside that part.
pub fn place_in_the_playlist(segments: &[Segment], seconds: f64) -> Option<(usize, f64)> {
    if segments.is_empty() {
        return None;
    }

    if seconds <= 0.0 {
        return Some((0, 0.0));
    }

    let mut before = 0.0;

    for (index, part) in segments.iter().enumerate() {
        let after = before + part.seconds;

        if seconds < after || index + 1 == segments.len() {
            return Some((index, (seconds - before).max(0.0)));
        }

        before = after;
    }

    None
}

/// Tells if the bytes are a transport stream.
pub fn looks_like_a_transport_stream(bytes: &[u8]) -> bool {
    bytes.len() >= PACKET && bytes[0] == SYNC
}

/// Gives the identity and the form of the audio of a segment.
///
/// The function reads the table that names the programs (`PAT`), and then the
/// table of the first program (`PMT`). That second table names the identity of
/// each stream and the form of each stream.
pub fn audio_stream_of(bytes: &[u8]) -> Option<AudioStream> {
    let pmt_pid = pmt_pid_of(bytes)?;
    let table = table_of_the_pid(bytes, pmt_pid)?;

    // The table of a program: 12 bytes of header, and then one description of
    // five bytes or more for each stream.
    if table.len() < 12 {
        return None;
    }

    let program_info_length = (usize::from(table[10] & 0x0f) << 8) | usize::from(table[11]);
    let mut place = 12 + program_info_length;

    while place + 5 <= table.len() {
        let form = Form::of_the_number(table[place]);
        let pid = ((u16::from(table[place + 1]) & 0x1f) << 8) | u16::from(table[place + 2]);
        let info_length =
            (usize::from(table[place + 3] & 0x0f) << 8) | usize::from(table[place + 4]);

        // A stream of a form that this program does not know can be a stream of
        // pictures. The program takes the first stream of audio.
        if !matches!(form, Form::Other(_)) {
            return Some(AudioStream { pid, form });
        }

        place += 5 + info_length;
    }

    None
}

/// Gives the identity of the table of the first program.
fn pmt_pid_of(bytes: &[u8]) -> Option<u16> {
    let table = table_of_the_pid(bytes, PAT_PID)?;

    // The table that names the programs: 8 bytes of header, and then four bytes
    // for each program. The last four bytes are the sum of control.
    let mut place = 8;

    while place + 4 <= table.len().saturating_sub(4) {
        let number = (u16::from(table[place]) << 8) | u16::from(table[place + 1]);
        let pid = ((u16::from(table[place + 2]) & 0x1f) << 8) | u16::from(table[place + 3]);

        // The number 0 names the table of the network, and not a program.
        if number != 0 {
            return Some(pid);
        }

        place += 4;
    }

    None
}

/// Gives the bytes of one table of one identity.
///
/// A table stands in one packet in every stream that ffmpeg makes. Therefore
/// this function reads the first packet of that identity, and it does not join
/// the parts of a table of many packets.
fn table_of_the_pid(bytes: &[u8], wanted: u16) -> Option<Vec<u8>> {
    for packet in packets(bytes) {
        if pid_of(packet) != wanted || !starts_a_unit(packet) {
            continue;
        }

        let body = body_of(packet)?;

        // The first byte of a table of a packet that starts a unit gives the
        // number of bytes to go over.
        let pointer = usize::from(*body.first()?);
        let start = 1 + pointer;

        if start >= body.len() {
            return None;
        }

        return Some(body[start..].to_vec());
    }

    None
}

/// Gives the bytes of the audio of one segment, with no header of the
/// container.
///
/// The payload of the packets of one identity is a stream of PES. This function
/// removes the header of each part of PES, therefore the answer is an
/// elementary stream: a file of MP3 or a file of ADTS.
pub fn audio_payload(bytes: &[u8], pid: u16) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());

    for packet in packets(bytes) {
        if pid_of(packet) != pid {
            continue;
        }

        let Some(body) = body_of(packet) else {
            continue;
        };

        // A part of PES starts with `00 00 01`. The byte 8 gives the length of
        // the rest of the header of PES.
        if starts_a_unit(packet) && body.len() > 9 && body[0..3] == [0x00, 0x00, 0x01] {
            let header = usize::from(body[8]);
            let start = 9 + header;

            if start < body.len() {
                out.extend_from_slice(&body[start..]);
            }

            continue;
        }

        out.extend_from_slice(body);
    }

    out
}

/// Gives the audio of one segment, or the reason why the program cannot read
/// it.
pub fn audio_of_the_segment(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if !looks_like_a_transport_stream(bytes) {
        return Err("The part of the stream is not a transport stream.".to_string());
    }

    let stream = audio_stream_of(bytes)
        .ok_or_else(|| "The part of the stream holds no audio.".to_string())?;

    if !stream.form.a_decoder_of_the_program_reads_it() {
        return Err(format!(
            "The stream of the server holds the audio in the form {:?}, and no \
             decoder of the program reads it.",
            stream.form
        ));
    }

    let audio = audio_payload(bytes, stream.pid);

    if audio.is_empty() {
        return Err("The part of the stream holds no audio.".to_string());
    }

    Ok(audio)
}

/// Gives every packet of the bytes.
fn packets(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes
        .chunks_exact(PACKET)
        .take_while(|packet| packet[0] == SYNC)
}

/// Gives the identity of one packet.
fn pid_of(packet: &[u8]) -> u16 {
    ((u16::from(packet[1]) & 0x1f) << 8) | u16::from(packet[2])
}

/// Tells if the packet starts a new unit: a table, or a part of PES.
fn starts_a_unit(packet: &[u8]) -> bool {
    packet[1] & 0x40 != 0
}

/// Gives the bytes of one packet after the header and after the field of
/// adaptation.
fn body_of(packet: &[u8]) -> Option<&[u8]> {
    let control = (packet[3] >> 4) & 0x03;

    // The value 0 holds no payload, and the value 2 holds the field of
    // adaptation only.
    if control == 0 || control == 2 {
        return None;
    }

    let mut start = 4;

    if control == 3 {
        let length = usize::from(packet[4]);
        start = 5 + length;
    }

    if start >= PACKET {
        return None;
    }

    Some(&packet[start..])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact playlist of an Audiobookshelf 2.36.0, of the measurement of
    /// 2026-08-11.
    const PLAYLIST: &str = "#EXTM3U\n\
        #EXT-X-VERSION:3\n\
        #EXT-X-ALLOW-CACHE:NO\n\
        #EXT-X-TARGETDURATION:6\n\
        #EXT-X-MEDIA-SEQUENCE:0\n\
        #EXT-X-PLAYLIST-TYPE:VOD\n\
        #EXTINF:6,\n\
        output-0.ts\n\
        #EXTINF:6,\n\
        output-1.ts\n\
        #EXTINF:5.999000000000024,\n\
        output-2.ts\n\
        #EXT-X-ENDLIST\n";

    /// One segment of the same measurement. It holds 143 packets, and the
    /// audio of the identity 256 is an elementary stream of MP3.
    const SEGMENT: &[u8] = include_bytes!("../../../tests/fixtures/audio/transport_stream.ts");

    #[test]
    fn the_playlist_gives_every_part_and_its_time() {
        let parts = parse_playlist(PLAYLIST);

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].name, "output-0.ts");
        assert_eq!(parts[0].seconds, 6.0);
        assert_eq!(parts[2].name, "output-2.ts");
        // The time of the last part is not a whole number of seconds.
        assert!((parts[2].seconds - 5.999).abs() < 0.001);
    }

    /// A body of a playlist that stops in the middle names fewer parts, and it
    /// gives no fault of its own: the book of 30 minutes ends after five of
    /// them, and the program tells the server that the user listened to the
    /// whole book. See T-193.
    #[test]
    fn a_playlist_that_stops_in_the_middle_is_no_playlist() {
        // The playlist of the measurement of the sandbox, whole.
        assert!(the_playlist_is_whole(PLAYLIST));

        // The same playlist, of which the body holds the first part alone. The
        // parse gives one part of the three, and it gives no fault.
        let stopped = "#EXTM3U\n\
            #EXT-X-VERSION:3\n\
            #EXT-X-ALLOW-CACHE:NO\n\
            #EXT-X-TARGETDURATION:6\n\
            #EXT-X-MEDIA-SEQUENCE:0\n\
            #EXT-X-PLAYLIST-TYPE:VOD\n\
            #EXTINF:6,\n\
            output-0.ts\n";

        assert_eq!(parse_playlist(stopped).len(), 1);
        assert!(!the_playlist_is_whole(stopped));

        // A body that stops inside the head of the playlist holds no part and
        // no end.
        assert!(!the_playlist_is_whole(
            "#EXTM3U\n#EXT-X-PLAYLIST-TYPE:VOD\n#EXT-X-TARG"
        ));

        // **A playlist that names no type is not a playlist of this rule**: a
        // server that makes the parts while the client reads them holds no end
        // yet.
        assert!(the_playlist_is_whole("#EXTM3U\n#EXTINF:6,\noutput-0.ts\n"));
        assert!(the_playlist_is_whole(""));

        assert!(the_sentence_of_a_playlist_that_stopped().contains("stopped in the middle"));
    }

    /// A part of a transport stream holds packets of 188 bytes, and nothing
    /// else. A body that stops in the middle of a packet gives no fault of its
    /// own, therefore the length of the container is the one truth of it. See
    /// T-194.
    #[test]
    fn a_part_that_stops_in_the_middle_is_no_whole_part() {
        // The part of the measurement of the sandbox, whole.
        assert!(the_part_is_whole(SEGMENT));
        assert_eq!(SEGMENT.len() % PACKET, 0);

        // The same part, of which the body holds 20000 bytes: the harness
        // `a_body_that_ends_early_and_looks_whole.py` of the measurement of
        // 2026-08-14 gives that body, and it holds no fault at all.
        assert!(!the_part_is_whole(&SEGMENT[..20000]));

        // A body of no byte is no part.
        assert!(!the_part_is_whole(&[]));

        // A body that stops at the end of a packet keeps the rule of the
        // packets, and the program then reads it as a whole part. The playlist
        // holds the number of the parts, and `TheStreamStopped` holds the rest
        // of the answer.
        assert!(the_part_is_whole(&SEGMENT[..PACKET * 3]));

        assert!(the_sentence_of_a_part_that_stopped("output-7.ts").contains("output-7.ts"));
        assert!(
            the_sentence_of_a_part_that_stopped("output-7.ts").contains("stopped in the middle")
        );
    }

    /// The sentence of a stream that stopped must name the media and a key that
    /// does the work of that fault (T-91 and T-170), and it must stand on one
    /// line of the screen (the trap of the message of T-68). See T-194.
    #[test]
    fn the_sentence_of_a_stream_that_stopped_names_a_key_of_the_user() {
        let sentence = the_sentence_of_a_stream_that_stopped();

        assert!(sentence.contains("stopped"), "{}", sentence);
        assert!(sentence.contains("Press the key"), "{}", sentence);
        assert!(
            !sentence.contains("finished"),
            "the media did not come to its end: {}",
            sentence
        );
        assert!(sentence.len() <= 150, "{} letters", sentence.len());
    }

    #[test]
    fn a_playlist_with_no_part_gives_no_part() {
        assert!(parse_playlist("").is_empty());
        assert!(parse_playlist("#EXTM3U\n#EXT-X-ENDLIST\n").is_empty());
    }

    #[test]
    fn the_place_of_the_media_gives_the_part_and_the_place_inside_it() {
        let parts = parse_playlist(PLAYLIST);

        assert_eq!(place_in_the_playlist(&parts, 0.0), Some((0, 0.0)));
        assert_eq!(place_in_the_playlist(&parts, 3.0), Some((0, 3.0)));
        assert_eq!(place_in_the_playlist(&parts, 6.0), Some((1, 0.0)));
        assert_eq!(place_in_the_playlist(&parts, 8.5), Some((1, 2.5)));

        // A place after the end gives the last part. The playback then comes to
        // the end at once, and it does not stop the program.
        assert_eq!(place_in_the_playlist(&parts, 10_000.0), Some((2, 9988.0)));

        assert_eq!(place_in_the_playlist(&[], 5.0), None);

        assert_eq!(seconds_before(&parts, 0), 0.0);
        assert_eq!(seconds_before(&parts, 2), 12.0);
    }

    /// The segment of the measurement holds the audio of the identity 256, and
    /// that audio starts with the sync of an MP3 frame.
    #[test]
    fn the_segment_of_the_server_gives_an_elementary_stream_of_mp3() {
        assert!(looks_like_a_transport_stream(SEGMENT));
        assert_eq!(SEGMENT.len() % PACKET, 0);

        let stream = audio_stream_of(SEGMENT).expect("the segment must name its audio");
        assert_eq!(stream.pid, 256);
        assert_eq!(stream.form, Form::Mp3);
        assert!(stream.form.a_decoder_of_the_program_reads_it());

        let audio = audio_of_the_segment(SEGMENT).expect("the segment must give audio");

        // The measurement of 2026-08-11 gave 24033 bytes.
        assert_eq!(audio.len(), 24033);

        // `ff f3` is the sync of a frame of MPEG audio.
        assert_eq!(audio[0], 0xff);
        assert_eq!(audio[1] & 0xf0, 0xf0);
    }

    #[test]
    fn bytes_that_are_not_a_transport_stream_give_a_fault_and_not_a_stop() {
        assert!(audio_of_the_segment(&[]).is_err());
        assert!(audio_of_the_segment(b"Not Found").is_err());
        assert!(audio_of_the_segment(&[0x47; 10]).is_err());

        // A stream of packets with no table gives no audio, and it stops
        // nothing.
        let mut packet = [0u8; PACKET];
        packet[0] = SYNC;
        assert!(audio_of_the_segment(&packet).is_err());
    }

    /// xHE-AAC of a transport stream comes in LATM, and symphonia does not read
    /// that form. The program must know it, therefore it asks the server for
    /// the old form of AAC. See T-53.
    #[test]
    fn the_program_knows_the_forms_that_it_cannot_read() {
        assert!(Form::of_the_number(0x03).a_decoder_of_the_program_reads_it());
        assert!(Form::of_the_number(0x04).a_decoder_of_the_program_reads_it());
        assert!(Form::of_the_number(0x0f).a_decoder_of_the_program_reads_it());

        assert_eq!(Form::of_the_number(0x11), Form::LatmAac);
        assert!(!Form::of_the_number(0x11).a_decoder_of_the_program_reads_it());
        assert!(!Form::of_the_number(0x1b).a_decoder_of_the_program_reads_it());
    }
}
