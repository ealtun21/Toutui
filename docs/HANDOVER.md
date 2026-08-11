# The handover of 2026-08-11 (the fourth session of that day)

This document is for the next session. It says what is done, what is open, and
the traps that cost time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of
each item, and `docs/T-24-coverage.md` for the comparison with the server.

## The state

`main` is clean and pushed. The newest release is **v0.7.10**. Every gate
passes:

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> cargo test    # 725 tests, 31 binaries
cargo tree -i openssl-sys                                # finds nothing
```

Two tests read the books of the survey. Those books stand outside the
repository, therefore give their directory in `TOUTUI_SURVEY_BOOKS`. A run with
no such variable reads `tests/data/alice.epub` only, and it passes.

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is
in v0.6.7. Do not try to publish v0.6.6.

## What this session closed

**Every item of the section "What is open" of the handover before this one.**

| Release | What | Keys |
|---|---|---|
| v0.7.8 | The place of the ebook in the form of the web reader (EPUBCFI) | — |
| v0.7.9 | The statistics of the library and of the year | `T` |
| v0.7.10 | Every session of the user, with pages | `W` |

The program now calls 28 paths of the server.

### v0.7.8, the EPUBCFI

`src/logic/reader/cfi.rs` walks the tree of the XHTML of one chapter with
`quick-xml`, and it makes the whole path. `rbook` asks for that crate already,
therefore the tree holds no new crate.

**The letter is the common unit of the tree and of the screen.** `html2text`
joins the spaces, it breaks the words at the width, and it writes a mark before
each item of a list, therefore no other unit agrees. A digit stays outside,
because `html2text` writes the number of each item of an ordered list and the
XHTML does not hold that number. A measurement gave a difference of **0 letters
for all 74 chapters of the four books of the survey**.

The measurements that show that the work is correct:

- The real program wrote `epubcfi(/6/8!/4/2/4/3:254)`, the server gave the same
  text back, and `currentTime` did not move.
- A second run of the real program opened the book at the same line.
- A resolution of that path with an independent program, against the raw
  archive, gives the text "boots every Christmas." That text is the first line
  of the screen of the first run.

`chapter_of_epubcfi` of `position.rs` went away. `cfi::parse_epubcfi` gives the
chapter and the whole path. The form `toutui:<spine>:<line>` stays, because a
server holds such a text from an older version and a chapter with no text gives
no EPUBCFI.

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
| `T` | The time that you listened, the library, and the year |
| `W` | Every session that you played, with pages |
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

1. **A queue of media.** The client plays one media and it stops. No endpoint:
   the work is in `src/logic/playback` and in the engine. **This is now the
   largest item of the comparison.**

### Medium

2. **The EPUBCFI against a real web reader.** The module writes the form of the
   specification. `epub.js`, which the web reader of Audiobookshelf uses, gives
   a different step to a text that stands after an element and after no text.
   A measurement of the four books counted **296 such texts of 11343, therefore
   2.61 per cent**. `letters_before` takes the first text that follows a path
   that it does not find, therefore the user loses the place inside the
   paragraph and never the paragraph.

   **A session with a browser must measure this.** Open the web reader of the
   sandbox, put the place in a paragraph of the form `<p><b>A</b>: a text</p>`,
   and read `ebookLocation`. If `epub.js` writes the step 1 and not the step 3,
   change `Walk::text` of `src/logic/reader/cfi.rs` to count the texts only.
   The head of that module says this. This session had no browser.

3. **The description of one series.** `GET /api/series/:id` gives
   `description`. The view of the series shows the name and the books only.

### Needs a decision before the work

4. **Live messages.** Audiobookshelf sends the changes of a different client
   over socket.io. The tree holds no client of socket.io in pure Rust,
   therefore this needs a new dependency and an examination against the rule
   of T-20. **Do not add it without that examination.**

5. **The narrators and the tags.** `GET /api/libraries/:id/narrators` and
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
2. **A pseudo terminal must answer TWO questions, and not one.**

   | Question | Answer | Who asks | With no answer |
   |---|---|---|---|
   | `ESC [ 5 n` | `ESC [ 0 n` | `ratatui-image`, at the start | The reader of the crate takes the FIRST KEY of the test |
   | `ESC [ 6 n` | `ESC [ 1 ; 1 R` | crossterm, inside `terminal.clear()` | The program STOPS at the first `R` with "The cursor position could not be read within a normal duration" |

   **Read the raw bytes of the pseudo terminal before you look for a
   deadlock.** That message never reaches the log file of the program.
3. **The device `null` plays a book of 30 minutes in two or three seconds.** A
   test of a view that needs a media that plays must press the key inside that
   time. One write of two keys does the work: `l` starts the playback and the
   space pauses it at once.
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
    media. Use `clock` of `src/utils/convert_seconds.rs`.
11. **`topGenres` of `GET /api/stats/year/:year` names its value `genre`, and
    `topAuthors` and `topNarrators` name it `name`.** A reader that takes
    `name` only finds nothing there. `TopName` takes both with an alias.
12. **The lists of the narrators and of the genres stay empty until a session
    comes after the metadata.** The server keeps a copy of the metadata inside
    each session, therefore a book that takes a genre today gives nothing for a
    session of yesterday. The measurement of the shape needed a new session.
13. **The first page of `GET /api/me/listening-sessions` is the page 0**, and a
    page after the last page gives `200` and an empty list, and not an error.
    Therefore the code must look at the list, and never at the status.
14. **`quick-xml` 0.41 gives an entity as its own event `GeneralRef`**, and not
    inside the text. A walk that reads `Event::Text` only loses `&nbsp;` and
    every named entity, and the count of the characters is then wrong.

## The shapes that this session made, and that the next work should follow

- **A slot between a task and the screen.** The render is not asynchronous,
  therefore a task asks the server and it puts the answer in a
  `Mutex<State>`, and the render takes it at the next frame.
  `src/logic/stats.rs`, `src/logic/bookmarks.rs`, `src/logic/authors.rs`,
  `src/logic/new_podcast.rs`, and `src/logic/sessions_view.rs` all have the
  same four states: `Nothing`, `Waiting`, `Ready`, and `Fault`.
- **A view with pages reads the next page at the move, and not with a key.**
  `Loaded::wants_the_next_page` is pure, and a test calls it with a line and a
  count. `a_task_asks` gives `true` one time only for one page, therefore a
  user who holds `j` makes one request and not fifty.
- **A request that gives one group of a view may fail alone.** The view of `T`
  sends three requests with `tokio::join!`. A fault of the library or of the
  year takes that group away only, and the user keeps the rest of the view.
- **A pure function for every decision.** The device `null` and the harness of
  the pseudo terminal make some measurements impossible, therefore the
  decision must live in a function that a test can call with the time and the
  state. `sleep_timer::action_for` and `cfi::letters_before` are the clearest
  examples.
- **A measurement of the shape of an answer needs the data to exist.** The
  lists of the narrators and of the genres were empty, and the shape came only
  after the sandbox took a genre, a narrator, and a new session. **Do not write
  a reader for a list that you measured empty.**
- **A measurement of your own work needs a second program.** The round trip of
  the EPUBCFI inside this program shows that the program agrees with itself,
  and it shows nothing more. A resolution of the same path with an independent
  program, against the raw archive, showed that the path is correct.
- **A test must not hold a path of the machine.** `TOUTUI_SURVEY_BOOKS` gives
  the directory of the books of the survey, and a test with no such variable
  passes. `src/logic/reader/book.rs` still holds such a path in a test, and
  that is older work.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified
  technical English. Short sentences, active voice, one instruction per
  sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys`
  must find nothing. `libsqlite3-sys` and `ring` are the two known builds of C,
  and they stay. See T-20.
- No test may need the network. A test that needs the sandbox carries
  `#[ignore]` and says how to run it. Eleven such files exist now.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited
  in the README, in the LICENSE, in `Cargo.toml`, and in the settings screen.
  **`gh` resolves to that repository by default in this clone**, therefore give
  `-R ealtun21/Toutui` to every `gh` command.
- Show a fault before you correct it.
- Tag, push, and go on. Do not wait for continuous integration.
- The address of the server of the user must stay outside this repository.

## The prompt for the next session

> Continue the Toutui takeover. Repo: `/home/nyverino/Documents/Toutui`
> (ealtun21/Toutui, branch main). Maintained fork of the archived
> AlbanDAVID/Toutui. Newest release **v0.7.10**; `Cargo.toml` is at 0.7.10, so
> the next release bumps it first — the release workflow refuses a tag that
> disagrees with `Cargo.toml`.
>
> Read `docs/HANDOVER.md` first. It has the state, the open items, and fourteen
> traps that cost real time. Then `docs/T-24-coverage.md` (the
> function-by-function comparison against Audiobookshelf 2.36.0; **section 6
> names what the program must not have, and why**) and
> `docs/TAKEOVER-BACKLOG.md` (the evidence for every closed item).
>
> Build these, in this order, from the section "What is open":
> 1. A queue of media. This is the largest item that is left, and the work is
>    in `src/logic/playback` and in the engine.
> 2. The EPUBCFI against a real web reader, **if you have a browser**. The head
>    of `src/logic/reader/cfi.rs` says what to measure and what to change.
> 3. The description of one series, in the view of the series.
> Do not start socket.io without the examination against the rule of T-20.
>
> Rules that bind every change: run all three gates yourself before each
> commit — `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
> and `cargo test` with `ALSA_CONFIG_PATH` pointing at a real null asound file
> (`/dev/null` hangs the real binary). Baseline: 725 tests, 31 binaries, tree
> clean. All prose and user-facing strings in ASD-STE100 simplified technical
> English. No crate needing a system library; `cargo tree -i openssl-sys` must
> find nothing. No test may need the network — sandbox tests carry `#[ignore]`
> — and no test may hold a path of the machine. Never write to
> AlbanDAVID/Toutui; keep his credit everywhere it appears, and give
> `-R ealtun21/Toutui` to every `gh` command. Show a fault before you fix it,
> and measure against the sandbox (`docs/TEST-SERVER.md`, podman on `:13399`)
> before you write an endpoint — and make the data exist first, because an
> empty list shows you no shape. Drive the real program in a pseudo terminal
> for every view — and answer both `ESC [ 5 n` and `ESC [ 6 n`, or the program
> looks broken when it is not. Tag, push, and keep working; don't wait for CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. The server of the user is theirs alone: ask before you
> use it, always with an isolated `XDG_CONFIG_HOME`, and never write its
> address into this repository. Measure against the sandbox instead.
