# A sandbox server for the tests

Date: 2026-08-10

The tests must not use the server of a user. This document gives a server in a
container. The server holds a podcast that is in the public domain, and a book
that has many audio files.

## 1. Why this is necessary

Two paths of the application had no test with real data:

- One episode of a podcast.
- A book that has many audio files.

A test with a real book of three files found two faults that no unit test
found. See T-2 and the commit `345682d`.

## 2. Start the server

The commands use `podman`. `docker` accepts the same commands.

```bash
ABS=$HOME/.local/share/toutui-abs-test
mkdir -p $ABS/{config,metadata,audiobooks,podcasts}

podman run -d --name abs-test -p 13399:80 \
  -v $ABS/config:/config:Z -v $ABS/metadata:/metadata:Z \
  -v $ABS/audiobooks:/audiobooks:Z -v $ABS/podcasts:/podcasts:Z \
  ghcr.io/advplyr/audiobookshelf:latest
```

The server answers at `http://localhost:13399`. The port is not 13378, thus
the server of the user stays free.

## 3. Make the first user

```bash
curl -X POST http://localhost:13399/init \
  -H "Content-Type: application/json" \
  -d '{"newRoot":{"username":"toutuitest","password":"toutuitest"}}'
```

Then get a token:

```bash
curl -X POST http://localhost:13399/login \
  -H "Content-Type: application/json" \
  -d '{"username":"toutuitest","password":"toutuitest"}'
```

## 4. Make the libraries

```bash
# A library for the podcasts.
curl -X POST http://localhost:13399/api/libraries \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"Podcasts","folders":[{"fullPath":"/podcasts"}],"mediaType":"podcast"}'

# A library for the books.
curl -X POST http://localhost:13399/api/libraries \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"Books","folders":[{"fullPath":"/audiobooks"}],"mediaType":"book"}'
```

## 5. Add a podcast that has no copyright

LibriVox gives its recordings to the public domain. Therefore a test can use
them with no permission.

```bash
# Read the feed.
curl -X POST http://localhost:13399/api/podcasts/feed \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"rssFeed":"https://librivox.org/rss/52"}'
```

The feed `https://librivox.org/rss/52` gives 57 episodes of "Letters of Two
Brides" by Balzac. Then `POST /api/podcasts` makes the podcast, and
`POST /api/podcasts/:id/download-episodes` gets the audio files.

## 6. Make a book that has many audio files

`ffmpeg` makes the files. The book is short, thus a test is quick.

```bash
BOOK="$ABS/audiobooks/Test Author/Multi File Test Book"
mkdir -p "$BOOK"
for i in 1 2 3; do
  ffmpeg -f lavfi -i "sine=frequency=$((300+i*110)):duration=20:sample_rate=44100" \
    -ac 1 -c:a libmp3lame -b:a 64k \
    -metadata title="Part $i" -metadata album="Multi File Test Book" \
    "$BOOK/0$i - Part $i.mp3"
done
```

## 6b. Make books that are in a series

The test of the series (T-22) needs books with a series. `ffmpeg` writes the
tags `series` and `series-part`, and Audiobookshelf reads them.

```bash
for series in "The Test Chronicles" "Second Series"; do
  for i in 1 2 3; do
    dir="$ABS/audiobooks/Series Author/$series/Book $i"
    mkdir -p "$dir"
    ffmpeg -loglevel error -y -f lavfi \
      -i "sine=frequency=$((300 + i * 100)):duration=3:sample_rate=44100" \
      -ac 1 -c:a libmp3lame -b:a 64k \
      -metadata title="$series $i" -metadata album="$series $i" \
      -metadata artist="Series Author" \
      -metadata series="$series" -metadata series-part="$i" \
      "$dir/book.mp3"
  done
done
```

The scan gives the title from the name of the directory, and not from the tag
`album`. Therefore the three books of a series have the same title. A `PATCH`
on `/api/items/:id/media` gives each book its own title:

```bash
curl -X PATCH "http://localhost:13399/api/items/$ITEM_ID/media" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"metadata":{"title":"The Test Chronicles Volume 1"}}'
```

**A warning about the endpoint of the series.** `GET
/api/libraries/:id/series?limit=0` gives an empty list, and not every series.
The endpoint of the items gives every item for the same value. Therefore the
application always asks for a page of a known size.

**A series of a new sandbox holds no description.** The list of the series
gives the field `description`, and its value is then `null`. A measurement of
that field needs the text to exist. `PATCH /api/series/:id` writes it, and it
gives `200` with the whole series:

```bash
curl -s -X PATCH "http://127.0.0.1:13399/api/series/$SERIES_ID" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"description":"Three books of a test."}'
```

`GET /api/libraries/:id/series` then carries that same text, therefore the
program needs no request for one series. `tests/the_series_against_the_sandbox.rs`
holds this measurement, and that test writes the description itself.

Then scan the library:

```bash
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan" \
  -H "Authorization: Bearer $TOKEN"
```

This gives a book of three files and 60 seconds. A test with this book is
quick, and it still proves that the engine plays every file.

## 6e. Make a book of thirty minutes

The device `null` accepts samples with no clock. A book of 60 seconds therefore
plays in one second, and the state of the engine is `Stopped` again before a
test can read the screen. A test of the player needs a book that is long enough
to stay in the engine. This one takes about thirty seconds of real time.

```bash
dir="$ABS/audiobooks/Long Author/A Long Test Book"
mkdir -p "$dir"
ffmpeg -loglevel error -y -f lavfi \
  -i "sine=frequency=220:duration=1800:sample_rate=22050" \
  -metadata title="A Long Test Book" -metadata artist="Long Author" \
  -b:a 32k "$dir/long.mp3"
```

## 6h. Give the long book three chapters

The view of the chapters (the key `C`) needs a media with chapters. "A Long
Test Book" has none, and `POST /api/items/:id/chapters` gives it three. The
answer is `{"success":true,"updated":true}`.

```bash
TOKEN=<the token of the login>
ITEM=<the identity of "A Long Test Book">

curl -s -X POST "http://127.0.0.1:13399/api/items/$ITEM/chapters" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"chapters":[
        {"id":0,"start":0,"end":600,"title":"The first part"},
        {"id":1,"start":600,"end":1200,"title":"The second part"},
        {"id":2,"start":1200,"end":1800,"title":"The third part"}]}'
```

`POST /api/items/:id/play` then gives those three chapters.

**The device `null` plays that book in two seconds.** A test of the view of
the chapters must therefore press the keys inside that time. One write of two
keys does the work: `l` starts the playback and the space pauses it at once.
The position then stops, and the view stays open.

## 6f. Give each book its own cover

The test of the cover art (T-23) needs a cover that a test can name. One colour
for each book is enough: a model of the terminal reads the colour of a cell,
and the test then knows which cover the screen shows. Audiobookshelf makes a
WebP file of 400 by 400 from every file that it receives.

```bash
for i in 1 2 3 4 5 6 7; do
  colour=$(echo "red green blue yellow magenta cyan orange" | cut -d' ' -f$i)
  ffmpeg -loglevel error -y -f lavfi -i "color=c=$colour:s=600x600" \
    -frames:v 1 "/tmp/cover-$i.jpg"
done

curl -X POST "http://localhost:13399/api/items/$ITEM_ID/cover" \
  -H "Authorization: Bearer $TOKEN" -F "cover=@/tmp/cover-1.jpg"
```

## 6g. Add an EPUB book

The reader of T-10 needs an ebook. Project Gutenberg gives books of the public
domain. Put the EPUB beside an audio file, in the directory of one book, and
scan the library.

```bash
dir="$ABS/audiobooks/Lewis Carroll/Alice in Wonderland"
mkdir -p "$dir"
curl -L -o "$dir/alice.epub" https://www.gutenberg.org/ebooks/11.epub3.images
ffmpeg -loglevel error -y -f lavfi \
  -i "sine=frequency=440:duration=5:sample_rate=22050" \
  -metadata title="Alice in Wonderland" -metadata artist="Lewis Carroll" \
  -b:a 32k "$dir/alice.mp3"
```

The item then has `media.ebookFormat` `"epub"`. A measurement on 2026-08-10
gives these answers:

| Request | Answer |
|---|---|
| `GET /api/items/:id/ebook` | `200`, `application/epub+zip`, the whole file |
| The same with `Range: bytes=0-99` | `206`, `Accept-Ranges: bytes` |
| `PATCH /api/me/progress/:id` with `ebookLocation` and `ebookProgress` | `200`. The two fields come back, and `currentTime` does not change |

## 6d. Make a collection and two playlists

The test of T-9 needs a collection and a playlist. A playlist can also hold
episodes of a podcast.

```bash
# A collection of books.
curl -X POST http://localhost:13399/api/collections \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"libraryId":"'$BOOK_LIB_ID'","name":"A Test Collection",
       "description":"Three books for a test.","books":["'$ID1'","'$ID2'"]}'

# A playlist of books.
curl -X POST http://localhost:13399/api/playlists \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"libraryId":"'$BOOK_LIB_ID'","name":"A Test Playlist",
       "items":[{"libraryItemId":"'$ID1'"}]}'

# A playlist of episodes. The entry names the podcast and the episode.
curl -X POST http://localhost:13399/api/playlists \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"libraryId":"'$POD_LIB_ID'","name":"A Podcast Playlist",
       "items":[{"libraryItemId":"'$POD_ID'","episodeId":"'$EP_ID'"}]}'
```

An entry of a playlist gives `libraryItem` and, for an episode, `episode`. The
episode gives its own title and its own length. A podcast gives the author in
the field `author`, and a book gives it in the field `authorName`.

## 6c. Make a library with no item

An empty library tests the condition that has no data: an empty list, and no
series.

```bash
podman exec abs-test mkdir -p /emptybooks
curl -X POST http://localhost:13399/api/libraries \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"Empty","folders":[{"fullPath":"/emptybooks"}],"mediaType":"book"}'
```

A test with this library found a fault: the key `G` in an empty list made the
calculation `len() - 1`, and the application stopped.

## 7. Give the application its own configuration

`db_path()` follows `XDG_CONFIG_HOME`. Therefore a test gets its own database
and its own configuration file, and it does not touch the files of the user.

```bash
TESTCFG=$HOME/.local/share/toutui-abs-test/toutui-config
mkdir -p $TESTCFG/toutui
echo 'TOUTUI_SECRET_KEY=testkey123' > $TESTCFG/toutui/.env
cp config.example.toml $TESTCFG/toutui/config.toml

XDG_CONFIG_HOME=$TESTCFG TOUTUI_AUDIO_DEVICE=null ./target/release/toutui
```

`TOUTUI_AUDIO_DEVICE=null` sends the sound to nothing. The computer of the
user then stays silent.

`ALSA_CONFIG_PATH` gives ALSA a different configuration, and it makes the
silence sure. **The value must be a real file.** `ALSA_CONFIG_PATH=/dev/null`
is correct for `cargo test`, because no test opens a sound device. The real
program stops for ever with that value: it writes "The pool has 1 address(es)"
in the log and it draws nothing. Write this file, and give its path:

```
</usr/share/alsa/alsa.conf>
pcm.!default { type null }
ctl.!default { type null }
```

## 8. A warning about the device `null`

The ALSA device `null` accepts samples as fast as the processor gives them.
Therefore the playback does not follow a clock, and a book of 60 seconds
finishes in a few milliseconds.

This is good for a quick test, and it is bad for a test of time. A test of the
position or of the speed needs a real device, or it needs a different method.

The same property found a real fault: the queue became empty between two
tracks, and the engine reported the end. A real device hides that fault,
because a track lasts many seconds. See T-2.

## 9. Stop the server

```bash
podman stop abs-test
podman rm abs-test
```

## A book of two files, and one file with no decoder

T-53 needs a media that the program cannot read by itself. The program plays no
WMA file (T-18), therefore a book of one MP3 file and one WMA file gives the
exact shape of a book of a user of 2026-08-11: that book held the same audio as
AAC-LC and as xHE-AAC, and symphonia reads AAC-LC only.

```
dir="$HOME/.local/share/toutui-abs-test/audiobooks/Decoder Test/One File With No Decoder"
mkdir -p "$dir"
ffmpeg -f lavfi -i "sine=frequency=440:duration=1800" -c:a libmp3lame -y "$dir/01 - Part 1.mp3"
ffmpeg -f lavfi -i "sine=frequency=660:duration=30"   -c:a wmav2     -y "$dir/02 - Part 2.wma"
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan" \
  -H "Authorization: Bearer $TOKEN"
```

The first file is 30 minutes long, because the device `null` plays 30 seconds in
two seconds and no view of the player then stays on the screen.

`tests/the_stream_against_the_sandbox.rs` reads this book. A sandbox with no such
book gives a line of text, and the test does not fail.

## A PDF book with pictures

T-54 needs a media whose ebook is a PDF. A media with an EPUB book keeps that
book, therefore the PDF must stand beside a media that holds none.

```
groff -Tpdf -ms <<'END' > text.pdf
.TL
A Book Of The Test
.PP
This is the first paragraph of the book.
END
magick -size 400x300 gradient:blue-yellow pic1.jpg
magick -size 300x300 plasma:red-blue pic2.jpg
img2pdf pic1.jpg pic2.jpg -o pictures.pdf
pdfunite text.pdf pictures.pdf book.pdf

podman cp book.pdf \
  abs-test:"/audiobooks/Decoder Test/One File With No Decoder/A Book Of The Test.pdf"
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan?force=1" \
  -H "Authorization: Bearer $TOKEN"
```

The book then holds three pages: one page of text, and two pages of a picture of
JPEG. The key `e` opens it.

## The rate limit of the login

The server permits **40 requests of `POST /login` in 600 seconds** for one address:
the answer of the header is `RateLimit-Policy: 40;w=600`.

Every test of the sandbox logs in one time. The whole list of the tests of the
sandbox is 17 files today, and a run of `cargo test -- --ignored` beside some
requests of `curl` therefore reaches that limit. The server then answers `429`, and
a test that reads `answer["user"]["token"]` gives:

```
panicked at tests/force_sync_against_the_sandbox.rs:64:10:
the answer must hold a token
```

**That message names the token, and the cause is the limit.** Read
`podman logs abs-test`: it holds one line "[RateLimiter] Rate limit exceeded -
Endpoint: POST /login" for each such request.

The limit gives its own time back: the header `Retry-After` says how many seconds
are left. Wait for that time, and run the tests again.

## 11. The book of xHE-AAC, for T-68

The sandbox holds a book "Depthless Hunger, Book 2" of one file:

```
audiobooks/Sarah Lin/Depthless Hunger xHE-AAC/01 - xHE-AAC.m4b     8.8 MB
```

That file is a piece of 10 minutes of a real book of a user. **No program of this
machine writes a file of xHE-AAC**, therefore no command of this document makes it
again: `ffmpeg` reads no such file, and it encodes none. A person who needs a new
piece must take it from a file of that form with a copy of the codec, because a copy
keeps the form:

```bash
ffmpeg -i <a file of xHE-AAC> -t 600 -map 0:a -c copy piece.m4b
ffprobe -v error -show_entries stream=codec_name,profile piece.m4b
# codec_name=aac
# profile=xHE-AAC
```

**Why the sandbox holds it.** T-68 measured every step of a media that no program
plays, and that measurement needs a file of that form. Keep this book: a session that
changes `src/player/engine/hls*.rs` can measure the whole path with it, and the
measurement needs no sound at all.

**The state of the server after such a media.** ffmpeg of the server stops with the
code 234, and the server then deletes the session of the stream. Every new session of
that media answers "No Segments", and the log holds "Failed checking files" every two
seconds for ever. **`podman restart abs-test` gives a server that works**, and a
measurement of that media must start from a server that came up now.

## 12. Two books with an ebook, for T-71

The limit of the cache of the ebooks needs **two** books with an ebook. The sandbox
holds the EPUB book of "Alice in Wonderland" and a copy of it in "A Long Test Book":

```bash
cp tests/data/alice.epub \
  "$ABS/audiobooks/Long Author/A Long Test Book/book.epub"
curl -s -X POST "http://localhost:13399/api/libraries/$LIB/scan" \
  -H "Authorization: Bearer $TOKEN"
```

Each file holds 136761 bytes. A run of the program with
`TOUTUI_EBOOK_CACHE_BYTES=200000` therefore removes one book when the user opens the
second one, and the row of the message says it. **A message lives six seconds**:
capture the screen inside that time.
