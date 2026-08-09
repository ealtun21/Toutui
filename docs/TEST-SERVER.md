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

Then scan the library:

```bash
curl -X POST "http://localhost:13399/api/libraries/$BOOK_LIB_ID/scan" \
  -H "Authorization: Bearer $TOKEN"
```

This gives a book of three files and 60 seconds. A test with this book is
quick, and it still proves that the engine plays every file.

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
