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

## 2b. What the sandbox holds now, 2026-08-12

The container lives longer than a session, therefore the books of every session
stay. `podman start abs-test` gives them back.

| The library | What it holds |
|---|---|
| `Books` | 21 items: **two books of eight hours (T-140)**, a book of many files, two series of three books, a book of one chapter (T-106), a PDF of 47 megabytes of a scan of 60 pages (T-62), **a PDF of 502 megabytes of a scan of 150 pages (T-116)**, a PDF that no reader reads (T-62), the book of xHE-AAC of the user, a book with a WMA file, Alice in Wonderland with an EPUB, and a long book of 30 minutes |
| `Podcasts` | **2 podcasts**: a feed of 57 episodes (T-110), and a feed of 27 episodes (T-166) |
| `Empty` | no item (T-103) |
| **`Large`** | **2056 items**, and every one of them holds no tag at all (T-112, T-114) |
| **`ManyPods`** | **520 podcasts** of one episode, for the paging of a library of podcasts (T-125, T-126) |

## 2c. The library of 2056 items, for the paging of T-70

The sweep of a library of the size that a user has needs a library that no session
made: the paging of T-70 came from a mock server only. The library `Large` stands
in the sandbox now, and it holds 2056 items of one MP3 file of 4940 bytes.

**The files stand inside the container**, and not in a volume: the library of the
first sweep needed no volume, and `podman exec` with `tar` writes 2056 directories
in one command.

```bash
cd $(mktemp -d)
ffmpeg -loglevel error -y -f lavfi \
  -i "sine=frequency=440:duration=1:sample_rate=8000" \
  -ac 1 -c:a libmp3lame -b:a 32k seed.mp3

mkdir largebooks && cd largebooks
for i in $(seq -w 1 2056); do
  mkdir -p "Large Book $i"
  cp ../seed.mp3 "Large Book $i/book.mp3"
done

podman exec abs-test mkdir -p /largebooks
tar cf - . | podman exec -i abs-test tar xf - -C /largebooks

curl -X POST http://localhost:13399/api/libraries \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"Large","folders":[{"fullPath":"/largebooks"}],"mediaType":"book"}'
```

**The library that a `POST` makes examines nothing**: `total` stayed 0 for 4
minutes, and one `POST /api/libraries/:id/scan` then read 2056 items in **50
seconds** (about 200 items of every 5 seconds).

**Every item of that library holds no tag**, therefore the server gives
`authorName: ""`, `narratorName: ""`, and `publishedYear: null`. That is the shape
of a book that a user takes from a disk of their own, and it found T-114.

A page of 500 items costs **2 ms** and 470 kilobytes of this server.

## 2d. The book of a scan of 502 megabytes, for T-116

`MAX_BOOK_BYTES` of `logic::reader::pdf` is 512 megabytes. The book "A Huge Book Of
A Scan" holds **502745447 bytes**: 150 pages of a picture of JPEG of 1200 by 1600
pixels of bytes that no algorithm makes smaller.

**A large PDF needs samples that no algorithm makes smaller** (the trap 17 of the
harness), and `img2pdf` takes the same picture more than one time:

```bash
head -c 5760000 /dev/urandom > raw.rgb
magick -depth 8 -size 1200x1600 rgb:raw.rgb page.jpg      # 3.35 megabytes
img2pdf $(for i in $(seq 1 150); do echo page.jpg; done) -o huge.pdf

dir="$ABS/audiobooks/Huge Author/A Huge Book Of A Scan"
mkdir -p "$dir"
cp huge.pdf "$dir/huge.pdf"
ffmpeg -loglevel error -y -f lavfi \
  -i "sine=frequency=330:duration=3:sample_rate=22050" \
  -metadata title="A Huge Book Of A Scan" -b:a 32k "$dir/huge.mp3"
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan" \
  -H "Authorization: Bearer $TOKEN"
```

**The parse of that book takes 2 minutes 4 seconds in the child of T-62**, and the
child holds 974 megabytes at its peak. The program of the user holds 44
megabytes. See T-116.

## 2h. The two books of eight hours, for the sweep of T-140

**The device `null` plays a book of 30 minutes in about 40 seconds** (the trap
14), therefore a sweep of two playbacks at one time needs a longer book: a
measurement of some steps ends with a screen that holds no player at all. A book
of eight hours gives about eight minutes of that speed.

```bash
ffmpeg -f lavfi -i "sine=frequency=200:duration=28800" -ac 1 -c:a libmp3lame \
    -b:a 24k -y many-hours.mp3                      # 82 MB, about 4 minutes
for name in "A Book Of Many Hours" "A Second Book Of Many Hours"; do
    podman exec abs-test mkdir -p "/audiobooks/Many Hours Author/$name"
    podman cp many-hours.mp3 \
        abs-test:"/audiobooks/Many Hours Author/$name/01 - The Whole Book.mp3"
done
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan" \
  -H "Authorization: Bearer $TOKEN"
```

**Two books, because two programs must play a media of their own** at one
moment. The sweep of T-140 gives one of them to each program, therefore the
position of each program stands apart on the server.

## 2g. The books of an EPUB, for the sweep of T-127

The sweep of a book of an EPUB of 100 megabytes and of an EPUB that is not valid
needs four books. A script of Python makes the EPUB: 100 chapters of text and 18
pictures of random bytes give **100.5 megabytes**, and no algorithm makes those
pictures smaller (the trap 23).

| The book | The file |
|---|---|
| `A Very Large Book` | a valid EPUB of 100.5 MB, 100 chapters |
| `A Book Of A Broken Epub` | 200000 bytes of `/dev/urandom` with the name `.epub` |
| `A Book Of An Epub With No Container` | a zip with no `META-INF/container.xml` |
| `A Book Of An Epub That Names Nothing` | a zip whose container names a file that is absent |

Each directory holds one MP3 file beside the book, because a library of books
needs a media. `tar cf - . | podman exec -i abs-test tar xf - -C /audiobooks`
writes them, and one `POST /api/libraries/:id/scan` reads them.

## 2f. The library of 520 podcasts, for the sweep of T-126

A library of podcasts of more than 500 items meets the paging of T-70: the page
holds 500 podcasts, and the podcasts of the second page found three faults
(T-125 and T-126). The library `ManyPods` holds **520 podcasts** of one episode.

```bash
cd $(mktemp -d)
ffmpeg -loglevel error -y -f lavfi \
  -i "sine=frequency=440:duration=1:sample_rate=8000" \
  -ac 1 -c:a libmp3lame -b:a 32k seed.mp3

mkdir manypods && cd manypods
for i in $(seq -w 1 520); do
  mkdir -p "Many Podcast $i"
  cp ../seed.mp3 "Many Podcast $i/episode-1.mp3"
done

podman exec abs-test mkdir -p /manypodcasts
tar cf - . | podman exec -i abs-test tar xf - -C /manypodcasts

curl -X POST http://localhost:13399/api/libraries \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"ManyPods","folders":[{"fullPath":"/manypodcasts"}],"mediaType":"podcast"}'
```

**A library that a `POST` makes examines nothing** (the trap 28): give the scan
yourself, and poll `total`. The scan of 520 podcasts took **20 seconds**.

```bash
curl -X POST "http://localhost:13399/api/libraries/$LIB/scan" \
  -H "Authorization: Bearer $TOKEN"
```

**A server that answers slowly needs no new container.**
`docs/harness/slow.py` gives every request a delay, and a block `[[servers]]` of
`config.toml` puts that address first (the trap 68). That file also **writes the
path and the time of each request**, therefore it gives the rounds of the start
(T-129):

```bash
python3 docs/harness/slow.py 13500 13399 0.5 requests.log &
``` The first frame of `ManyPods` took 11.9 seconds with
20 milliseconds of every request before T-126, and it takes 0.409 seconds now.

## 2e. A second server, for the sweep of two accounts

The sweep of two accounts of two servers needs a second Audiobookshelf. It stands
on the port **13400**, and it holds one account and one book of 30 minutes:

```bash
ABS2=$HOME/.local/share/toutui-abs-test-2
mkdir -p $ABS2/{config,metadata,audiobooks}

podman run -d --name abs-test-2 -p 13400:80 \
  -v $ABS2/config:/config:Z -v $ABS2/metadata:/metadata:Z \
  -v $ABS2/audiobooks:/audiobooks:Z \
  ghcr.io/advplyr/audiobookshelf:latest

curl -X POST http://127.0.0.1:13400/init -H 'Content-Type: application/json' \
  -d '{"newRoot":{"username":"secondtest","password":"secondtest"}}'
```

`podman start abs-test-2` gives it back, and `podman stop abs-test-2` takes it
away. **No test needs it**: T-118 says that the program holds one account, and the
sweep of that condition needs an editor of SQLite.

**A test that needs an EPUB must ask for the form `epub`.** The PDF of 47
megabytes stands first in the alphabet, and a rule that takes "the first item with
an ebook" takes that PDF. See T-111.

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

## 5b. A second podcast of the same library, for the queue of two podcasts

**The queue of the downloads of the server belongs to the library, and not to
one podcast** (`GET /api/libraries/:id/episode-downloads`). A measurement of the
line of that view therefore needs **two** podcasts of one library: the episodes
of the podcast that the user did not choose stand under the episodes of the
podcast that they did choose, and they move up when the server downloads one.
See T-166.

The session of 2026-08-14 added `https://librivox.org/rss/100` — "Narrative of
Arthur Gordon Pym of Nantucket" of 27 episodes — to the library `Podcasts`:

```bash
# The feed. **Not every number of that address answers**: the numbers 1, 5, 10,
# 20, 60, and 61 give "Podcast RSS feed request failed", and 52, 100, and 200
# give a feed.
curl -s -X POST http://localhost:13399/api/podcasts/feed \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"rssFeed":"https://librivox.org/rss/100"}' > feed100.json

# The podcast. `media.metadata` is the metadata of the feed, and `folderId` is
# the folder of the library.
curl -s -X POST http://localhost:13399/api/podcasts \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"path":"/podcasts/Arthur Gordon Pym","folderId":"<the folder>",
       "libraryId":"<the library Podcasts>",
       "media":{"metadata":<the metadata of the feed>,"autoDownloadEpisodes":false},
       "autoDownloadEpisodes":false}'
```

**The server downloads one episode of that feed in about four seconds.** A
measurement that needs a queue of a minute therefore needs about 15 episodes,
and the body of `POST /api/podcasts/:id/download-episodes` is the bare array of
the episodes of the feed (T-154).

**The queue holds the sequence of the requests**, therefore the podcast of the
second request stands at the end of it: a measurement of the line that crosses
the two podcasts asks for the episodes of one podcast, it waits for that block
to go away, and it asks for the episodes of the other one after that. **The
server holds every episode that it downloaded already**, therefore each run
begins with the hard delete of T-154 for both podcasts:
`DELETE /api/podcasts/:id/episode/:episode?hard=1` for every entry of
`media.episodes`. The measurement of T-166 used 48 episodes — 37 of "Letters of
Two Brides" and the 11 of "Narrative of Arthur Gordon Pym" after them — and that
queue held about three minutes.

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

## 6i. Give the book of eight hours three chapters

**A measurement of the view of the chapters while the media that plays changes
needs two media with chapters** (T-162): the media of the user, and the media
that the queue starts after it. "A Book Of Many Hours" held no chapter, and its
three chapters stand far from the three chapters of "A Long Test Book" —
therefore the key `l` of a line that the user did not choose moves the place of
that book by 43 minutes, and `curl` reads it.

```bash
TOKEN=<the token of the login>
ITEM=6ba57b9a-acb5-44f9-b2b6-39ad9107b420   # A Book Of Many Hours

curl -s -X POST "http://127.0.0.1:13399/api/items/$ITEM/chapters" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"chapters":[
        {"id":0,"start":0,"end":10000,"title":"The hours of the start"},
        {"id":1,"start":10000,"end":20000,"title":"The hours of the middle"},
        {"id":2,"start":20000,"end":28800,"title":"The hours of the end"}]}'
```

**A measurement of that condition takes 22 seconds of real time.** The device
`null` plays the 30 minutes of "A Long Test Book" in that time, and the section
15 gives the book its place 0 again: `PATCH /api/me/progress/:id` with
`{"isFinished": false}`.

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
# The program makes this file itself since T-133. A key of the measurement keeps
# the tokens of this directory readable between two builds.
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

## An item with two ebooks

T-76 needs an item that holds more than one ebook. Put an EPUB book beside the
PDF book of the item of the section above, and tell the server to examine the
library again:

```
podman cp tests/data/alice.epub \
  abs-test:"/audiobooks/Decoder Test/One File With No Decoder/A Second Book.epub"
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan?force=1" \
  -H "Authorization: Bearer $TOKEN"
```

A measurement of 2026-08-11 then gave this:

```
ebookFile: A Book Of The Test.pdf
 file: A Book Of The Test.pdf  ebook  ino 6121534
 file: A Second Book.epub      ebook  ino 94488

GET /api/items/:id/ebook           200  53688 bytes  application/pdf
GET /api/items/:id/ebook/6121534   200  53688 bytes  application/pdf
GET /api/items/:id/ebook/94488     200 136761 bytes  application/epub+zip
```

**`media.ebookFile` names one book, and the item holds two.** The key `e` opens
the book of the server, and the key `e` inside the reader gives the list.
`tests/the_ebooks_of_an_item_against_the_sandbox.rs` reads this item. A sandbox
with no such item gives a line of text, and the test does not fail.

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

## The queue of the episodes that the server downloads

T-81 needs a queue that holds lines. The feed of the section 5 gives 57 episodes,
and the server holds three of them:

```bash
# The episodes of the feed that the server does not hold.
curl -s -X POST http://localhost:13399/api/podcasts/feed \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"rssFeed":"https://librivox.org/rss/52"}' > feed.json

# Ten of them go in the queue. The server takes some seconds to fill it.
curl -X POST "http://localhost:13399/api/podcasts/$POD_ID/download-episodes" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d @ten-episodes.json

curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:13399/api/libraries/$POD_LIB_ID/episode-downloads"

# The queue of one podcast goes away. The episode that downloads now goes on.
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:13399/api/podcasts/$POD_ID/clear-queue"
```

**The queue does not fill at once.** A measurement of 2026-08-11 read an empty
queue two seconds after the request, and the clear three seconds later removed
nine episodes. **Poll that endpoint**, and do not sleep.

`tests/the_downloads_against_the_sandbox.rs` does this work with three episodes,
and it empties the queue at the end. The episode that the server downloaded stays
in the library.

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

## 13. The e-mail of the server, and the devices of an e-reader, for T-119

**The server sends a book with SMTP**, therefore the measurement of T-119 needs an
SMTP server. A server of the internet is not acceptable: the book of the
measurement then goes to a real address.

**The SMTP server of the measurement stands on the machine of the maintainer**, and
the container reaches it at `host.containers.internal`. Any small SMTP server does
the work: it must answer `220`, `250` for `EHLO`, `MAIL FROM`, and `RCPT TO`, `354`
for `DATA`, and `250` after the body. A server of about 60 lines of Python holds
that, and it writes the size of the body to a file.

```bash
# The port of the measurement, on the machine of the maintainer.
python3 <the smtp server of the measurement> 1025
podman exec abs-test sh -c 'nc -z -w2 host.containers.internal 1025' \
  && echo "the container reaches it"
```

**The settings of the e-mail.** The port is not 465, therefore `secure` is false:
`getTransportObject` of the server sets `secure` to false for every other port, and
a `true` there gives a fault of TLS.

```bash
curl -s -X PATCH http://127.0.0.1:13399/api/emails/settings \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"host":"host.containers.internal","port":1025,"secure":false,
       "rejectUnauthorized":false,"fromAddress":"toutui@example.invalid",
       "testAddress":"kobo@example.invalid"}'
```

**The three devices of the sandbox.** One request writes the whole list: the
endpoint takes an array, and it replaces every device. The measurement of T-119
needs one device of each condition of `availabilityOption`.

```bash
curl -s -X POST http://127.0.0.1:13399/api/emails/ereader-devices \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"ereaderDevices":[
        {"name":"Kobo of the measurement","email":"kobo@example.invalid",
         "availabilityOption":"adminOrUp","users":[]},
        {"name":"The Kindle of the plain user","email":"kindle@example.invalid",
         "availabilityOption":"specificUsers","users":["<the identity of toutuiplain>"]},
        {"name":"A device of every user","email":"all@example.invalid",
         "availabilityOption":"guestOrUp","users":[]}]}'
```

**A second account, of the type `user`.** The measurement of T-119 needs an account
that is not an administrator: every endpoint of `/api/emails/` answers `404` for
such an account, and `POST /api/authorize` still gives its devices.

```bash
curl -s -X POST http://127.0.0.1:13399/api/users \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"username":"toutuiplain","password":"toutuiplain","type":"user"}'
```

**`POST /api/users` makes an account that is not active**, and the login of that
account then answers `401` with "User is not active" in `podman logs abs-test`. One
request more gives the account its work:

```bash
curl -s -X PATCH "http://127.0.0.1:13399/api/users/<the identity>" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"isActive":true}'
```

**The token of the list of the users is not a token of a request.** `GET /api/users`
holds a field `token` for each account, and every request with it answers `401`.
Take the token from `POST /login` of that account.

**The books of the measurement.** The send needs a book with an ebook, and the time
of the request comes from its size:

| The item | The ebook | The time of one send |
|---|---|---|
| A Book That No Reader Reads | 0.1 MB | 0.007 s |
| A Big Book Of A Scan | 45.2 MB | 3.6 s |
| A Huge Book Of A Scan | 479.5 MB | 36.2 s |

## 14. An account that reads one library only, for T-136

The sandbox holds `toutuilimited` / `toutuilimited`: an account of the type
`user`, with `download` false, and with one library of the five. **Reuse it**, so
that the rate limit of the login of section "The rate limit of the login" stays
free.

```bash
# The account, if it is not there. `isActive` needs a request of its own: the
# request that makes an account gives `isActive: false`.
curl -X POST http://localhost:13399/api/users \
  -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' \
  -d '{"username":"toutuilimited","password":"toutuilimited","type":"user",
       "permissions":{"download":false,"accessAllLibraries":false,
                      "accessAllTags":true,"accessExplicitContent":true},
       "librariesAccessible":["<the identity of Books>"]}'

curl -X PATCH "http://localhost:13399/api/users/$ID" \
  -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' \
  -d '{"isActive":true}'
```

**A `PATCH` takes `librariesAccessible` inside `permissions` only.** The same name
beside `permissions` gives `200`, and it changes nothing:

```bash
curl -X PATCH "http://localhost:13399/api/users/$ID" \
  -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' \
  -d '{"permissions":{"download":false,"accessAllLibraries":false,
                      "accessAllTags":true,"accessExplicitContent":true,
                      "librariesAccessible":["<the identity of Podcasts>"]}}'
```

**An empty `librariesAccessible` is every library**, and not no library:
Audiobookshelf 2.36.0 reads it that way, therefore an account that reads nothing
at all does not exist. Read the account again after each request.

## 15. The place of a media, when a measurement writes it

A sweep of the place of a user needs a "different client" that writes a new
place. `PATCH /api/me/progress/:id` is that client, and it holds one trap:

```bash
curl -X PATCH "http://localhost:13399/api/me/progress/$ITEM" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"currentTime":500}'                       # t=500, and progress stays 0
```

**`isFinished: false` sets the position to 0.** The request
`{"currentTime":500,"isFinished":false}` gives **0 seconds** and not 500: the
server reads that name as the command "the user did not read this media", and
that command takes the place away. Give `currentTime` alone, and give `progress`
beside it when a view of the program must show a percent (a measurement of
2026-08-13 for T-141).
