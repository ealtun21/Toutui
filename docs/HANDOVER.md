# The handover of 2026-08-11 (the third session of that day)

This document is for the next session. It says what is done, what is open, and
the traps that cost time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of
each item, and `docs/T-24-coverage.md` for the comparison with the server.

## The state

`main` is clean and pushed. The newest release is **v0.7.7**. Every gate
passes:

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> cargo test    # 661 tests, 28 binaries
cargo tree -i openssl-sys                                # finds nothing
```

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is
in v0.6.7. Do not try to publish v0.6.6.

## What this session closed

**Every item of `docs/T-24-coverage.md` section 5 is done**, except the two
that section 7 of this document names. The releases:

| Release | What | Keys |
|---|---|---|
| v0.7.0 | The statistics of the user; every shelf of Home; the sequence and the filter | `T`, `f` |
| v0.7.1 | Hide from Continue Listening; the list of the chapters | `N`, `C` |
| v0.7.2 | Bookmarks | `b`, `V` |
| v0.7.3 | A timer for sleep | `t` |
| v0.7.4 | Add a podcast | `A` |
| v0.7.5 | The server gets the episodes of a feed | `E` |
| v0.7.6 | A view of the authors | `a` |
| v0.7.7 | The server examines the library again | `L` |

The program now calls 25 paths of the server. It called 15 at the start of the
day.

## The keys of the program

`src/app.rs` holds the key handler, and it is the authority.

| Key | What it does |
|---|---|
| `j`/`↓`, `k`/`↑`, `g`/`Home`, `G`/`End` | Move |
| `J`, `K`, `H` | Scroll the description |
| `l`/`→`/`Enter` | Play, or open the line |
| `h` | Go back one view |
| `Tab` | Home and Library |
| `/` | Search on the server |
| `s`, `a`, `c` | The series, the authors, the lists |
| `f` | The sequence and the filter of the library |
| `e` | Read the EPUB book |
| `D`, `X` | A local copy, and remove it |
| `R`, `L` | Ask the server again, and examine the library |
| `F` | Send the position now |
| `M`, `N` | Mark as finished, and hide from Continue Listening |
| `T` | The time that you listened |
| `C` | The chapters of the media that plays |
| `b`, `V` | Write a bookmark, and show the bookmarks |
| `t` | The timer for sleep |
| `A`, `E` | A new podcast, and the episodes of a feed |
| `S`, `B`, `Q`/`Esc` | Settings, the keys, quit |
| ` `, `p`, `u`, `P`, `U`, `O`, `I`, `o`, `i`, `Y` | The player |

Inside the view of the bookmarks, `X` removes a bookmark. That is the one
place where a key changes its work with the view.

## What is open

### Large, and worth building

1. **The place of the ebook in the form of the web reader (EPUBCFI).** The
   reader writes `toutui:C:L` today, therefore a user who reads on the
   telephone and continues in the terminal finds the chapter and not the line.
   The work is a map from a position in the text to a path in the XHTML, in
   `src/logic/reader/position.rs`. This is the last large item of the
   comparison that gives the user something that no other function gives.

2. **A queue of media.** The client plays one media and it stops. No endpoint:
   the work is in `src/logic/playback` and in the engine.

### Medium

3. **The sessions of the user.** `GET /api/me/listening-sessions` gives
   `total`, `numPages`, `page`, `itemsPerPage`, and `sessions`. The view of
   `T` shows the five last sessions already; this gives the whole history with
   pages.

4. **The statistics of the library and of a year.**
   `GET /api/libraries/:id/stats` gives `totalItems`, `totalSize`,
   `totalDuration`, `largestItems`, and `longestItems`.
   `GET /api/stats/year/2026` gives `topAuthors`, `topNarrators`, `topGenres`,
   and 8 more fields. Both fit in the view of the key `T` as two more groups.

5. **The description of one series.** `GET /api/series/:id` gives
   `description`. The view of the series shows the name and the books only.

### Needs a decision before the work

6. **Live messages.** Audiobookshelf sends the changes of a different client
   over socket.io. The tree holds no client of socket.io in pure Rust,
   therefore this needs a new dependency and an examination against the rule
   of T-20. **Do not add it without that examination.**

7. **The narrators and the tags.** `GET /api/libraries/:id/narrators` and
   `GET /api/tags`. The filter of the key `f` shows the books of one narrator
   and of one tag already, therefore a view of its own gives little.

`docs/T-24-coverage.md` section 6 names every function that the program must
**not** have, with the reason. Read it before you take a row of the table that
still says `No`.

## The traps that cost time

1. **`ALSA_CONFIG_PATH=/dev/null` stops the real program.** It is correct for
   `cargo test`, because no test opens a sound device. A real run writes "The
   pool has 1 address(es)" and then draws nothing. Give it a real file:
   ```
   </usr/share/alsa/alsa.conf>
   pcm.!default { type null }
   ctl.!default { type null }
   ```
2. **A pseudo terminal must answer TWO questions, and not one.** This cost an
   hour: the key `R` looked as if it stopped the program.

   | Question | Answer | Who asks | With no answer |
   |---|---|---|---|
   | `ESC [ 5 n` | `ESC [ 0 n` | `ratatui-image`, at the start | The reader of the crate takes the FIRST KEY of the test |
   | `ESC [ 6 n` | `ESC [ 1 ; 1 R` | crossterm, inside `terminal.clear()` | The program STOPS at the first `R` with "The cursor position could not be read within a normal duration" |

   **Read the raw bytes of the pseudo terminal before you look for a
   deadlock.** That message never reaches the log file of the program.
3. **The device `null` plays a book of 30 minutes in two or three seconds.** A
   test of a view that needs a media that plays must press the key inside that
   time. One write of two keys does the work: `l` starts the playback and the
   space pauses it at once. The position then stops.
4. **`currentTime` comes as a text, and not as a number,** in
   `GET /api/me/progress/:id`.
5. **A test that sets `XDG_CONFIG_HOME` must be alone in its binary.** That
   variable belongs to the process. Put the parts in one test function.
6. **`cargo fmt` before `git commit`, every time.**
7. **The server takes a name of a field that does not exist.**
   `?sort=bogus.field` gives `200` and an unspecified sequence. Measure a field
   before you offer it.
8. **`items` of `GET /api/me/listening-stats` is a map, and not a list.** A map
   has no sequence, therefore the code must make one or the screen moves at
   each frame.
9. **`GET /api/podcasts/:id/checknew` gives an empty list for a podcast that
   came one second before.** It compares with the time of the last
   examination. Read the feed and compare yourself.
10. **`convert_seconds` rounds to the minute.** It is wrong for a place in a
    media: two bookmarks of one minute would show one text. Use `clock` of
    `src/utils/convert_seconds.rs`.

## The shapes that this session made, and that the next work should follow

- **A slot between a task and the screen.** The render is not asynchronous,
  therefore a task asks the server and it puts the answer in a
  `Mutex<State>`, and the render takes it at the next frame.
  `src/logic/stats.rs`, `src/logic/bookmarks.rs`, `src/logic/authors.rs`, and
  `src/logic/new_podcast.rs` all have the same four states: `Nothing`,
  `Waiting`, `Ready`, and `Fault`.
- **A pure function for every decision.** The device `null` and the harness of
  the pseudo terminal make some measurements impossible, therefore the
  decision must live in a function that a test can call with the time and the
  state. `sleep_timer::action_for` is the clearest example.
- **A question before a request that writes.** The key `A` asks the user to
  write `yes`. See section 6 of the comparison.
- **A list that holds a title.** `src/logic/list_moves.rs` gives the four
  moves over a list where some lines are titles. The Home view and the view of
  the sequence both use it.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified
  technical English. Short sentences, active voice, one instruction per
  sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys`
  must find nothing. `libsqlite3-sys` and `ring` are the two known builds of C,
  and they stay. See T-20. This session wrote base64 by hand for that rule.
- No test may need the network. A test that needs the sandbox carries
  `#[ignore]` and says how to run it. Nine such files exist now.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited
  in the README, in the LICENSE, in `Cargo.toml`, and in the settings screen.
- Show a fault before you correct it.
- Tag, push, and go on. Do not wait for continuous integration.

## The prompt for the next session

> Continue the Toutui takeover. Repo: `/home/nyverino/Documents/Toutui`
> (ealtun21/Toutui, branch main). Maintained fork of the archived
> AlbanDAVID/Toutui. Newest release **v0.7.7**; `Cargo.toml` is at 0.7.7, so
> the next release bumps it first — the release workflow refuses a tag that
> disagrees with `Cargo.toml`.
>
> Read `docs/HANDOVER.md` first. It has the state, the open items, and ten
> traps that cost real time. Then `docs/T-24-coverage.md` (the
> function-by-function comparison against Audiobookshelf 2.36.0; **section 6
> names what the program must not have, and why**) and
> `docs/TAKEOVER-BACKLOG.md` (the evidence for every closed item).
>
> Every item of section 5 of the comparison is done. Build these, in this
> order, from the section "What is open" of the handover:
> 1. The place of the ebook in the form of the web reader (EPUBCFI). This is
>    the last large item that gives the user something no other function
>    gives.
> 2. The statistics of the library and of a year, as two more groups of the
>    view of the key `T`.
> 3. The sessions of the user, with pages.
> Do not start socket.io without the examination against the rule of T-20.
>
> Rules that bind every change: run all three gates yourself before each
> commit — `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
> and `cargo test` with `ALSA_CONFIG_PATH` pointing at a real null asound file
> (`/dev/null` hangs the real binary). Baseline: 661 tests, 28 binaries, tree
> clean. All prose and user-facing strings in ASD-STE100 simplified technical
> English. No crate needing a system library; `cargo tree -i openssl-sys` must
> find nothing. No test may need the network — sandbox tests carry
> `#[ignore]`. Never write to AlbanDAVID/Toutui; keep his credit everywhere it
> appears. Show a fault before you fix it, and measure against the sandbox
> (`docs/TEST-SERVER.md`, podman on `:13399`) before you write an endpoint.
> Drive the real program in a pseudo terminal for every view — and answer both
> `ESC [ 5 n` and `ESC [ 6 n`, or the program looks broken when it is not.
> Tag, push, and keep working; don't wait for CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. They gave a temporary account on a server of their own —
> ask before you use it again, and always with an isolated `XDG_CONFIG_HOME`.
> **The address of that server must stay outside this repository.** Ask the
> user for it. Measure against the sandbox of `docs/TEST-SERVER.md` instead.
