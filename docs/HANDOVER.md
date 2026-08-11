# The handover of 2026-08-11 (the fifth session of that day)

This document is for the next session. It says what is done, what is open, and
the traps that cost time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of
each item, and `docs/T-24-coverage.md` for the comparison with the server.

## The state

`main` is clean and pushed. The newest release is **v0.7.12**. Every gate
passes:

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> cargo test    # 747 tests pass, 15 carry #[ignore], 33 binaries
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
| v0.7.11 | A queue of media | `n`, `q` |
| v0.7.12 | The place of the ebook agrees with `epub.js` | — |
| — | The description of a series: **it was done already** | — |

### v0.7.11, the queue of media

`src/logic/queue.rs` holds the queue. The server gives no endpoint:
Audiobookshelf holds its queue in the web page, and it sends that queue to no
client.

- The key `n` puts the selected media at the end of the queue. It reads
  `selected_download` of `src/app.rs`, therefore it operates in every view that
  holds one media.
- The key `q` shows the queue. `l` starts a media now, and `X` takes one out.
- `follow_playback` gives an `Outcome` now, and `play` reads it. **The queue
  goes on at an end, and at nothing else.** A media that the user stopped
  (`PlayerCommand::Stop` writes `finished = false`), and a media that a
  different playback took away, leave the queue where it is.

The measurement of the work: the queue held Volume 2 and Volume 3 of a series,
the user started a different book, and **the server itself** wrote three lines,
one second apart: `Starting session ... Second Series Volume 1`, `... The Test
Chronicles Volume 2`, `... The Test Chronicles Volume 3`. Read the server with
`podman logs abs-test`.

The queue lives in the memory of the process. A user who stops the program
loses it.

### v0.7.12, the EPUBCFI of `epub.js`

The module wrote the step of a text with the rule of the **specification**: the
text after the element `n` takes the step `2n + 1`. `epub.js`, which the web
reader uses, **counts the texts only**.

Two measurements settle it:

1. The program loaded `epub.js` 0.3.93 in Chromium and asked the library for
   the path of one text of each shape. `<p><b>A</b>: a text.</p>` gives
   `/4/2/1`, and the specification gives `/4/2/3`.
2. The server holds the same rule in the code that it gives to the browser.
   `/app/client/dist/_nuxt/2c96a53.js` of Audiobookshelf 2.36.0 holds
   `position(t){...r=(e=this.textNodes(t.parentNode)).indexOf(t),r}`.

Then a second program gave the XHTML of every chapter of seven books to the
real `epub.js` and compared every path:

| The book | Texts | The old rule disagreed | The new rule disagrees |
|---|---|---|---|
| Frankenstein | 993 | 35 | 0 |
| Moby Dick | 3463 | 6 | 0 |
| Pride and Prejudice | 3768 | 195 | 0 |
| Alice in Wonderland | 1440 | 30 | 0 |
| Three books of a different producer | 19651 | 454 | 0 |
| **In total** | **29315** | **720, therefore 2.46 per cent** | **0** |

`TextPlace` keeps **two** forms of the steps now: `steps` (the form of
`epub.js`, which the program writes) and `steps_of_the_specification`.
`letters_before` reads both, therefore a value that v0.7.8 to v0.7.11 wrote
still gives the exact place.

**The second form is not only for the old values.** The form of `epub.js` does
not grow with the sequence of the document: the text of `<p><b>A</b>: a
text</p>` takes the step 1, and the text of the `<b>` before it takes the steps
2 and 1. Therefore a search that needs the sequence must read the form of the
specification. `letters_before` does this.

### The description of a series was done already

The handover before this one named it as open. A measurement shows that it is
not:

- `GET /api/libraries/:id/series`, which the program asks for already, gives
  `description` for every series of the page. `GET /api/series/:id` gives
  nothing new, and the program must not ask for it.
- `collect_series` reads that field, and `description_for_the_screen` shows it.
  A series with no description shows the description of its first book (T-43).
- The real program shows the text. A run in a pseudo terminal after a `PATCH
  /api/series/:id` gave: `The Test Chronicles - 3 book(s) ... Three books of a
  test. The series has a description, therefore the view can show it.`

`tests/the_series_against_the_sandbox.rs` holds the measurement. **That test
writes**: it gives the description itself, because a series of a new sandbox
has none.

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
| `n`, `q` | Put a media in the queue, and show the queue |
| `t` | The timer for sleep |
| `A`, `E` | A new podcast, and the episodes of a feed |
| `S`, `B`, `Q`/`Esc` | Settings, the keys, quit |
| ` `, `p`, `u`, `P`, `U`, `O`, `I`, `o`, `i`, `Y` | The player |

Inside the view of the bookmarks and inside the view of the queue, `X` removes
the selected line. Those are the two places where a key changes its work with
the view.

## What is open

### Needs a decision before the work

1. **Live messages.** Audiobookshelf sends the changes of a different client
   over socket.io. The tree holds no client of socket.io in pure Rust,
   therefore this needs a new dependency and an examination against the rule
   of T-20. **Do not add it without that examination.** This is now the largest
   item of the comparison.

### Small

2. **The narrators and the tags.** `GET /api/libraries/:id/narrators` and
   `GET /api/tags`. The filter of the key `f` shows the books of one narrator
   and of one tag already, therefore a view of its own gives little.

3. **The queue on the disk.** The queue lives in the memory of the process. A
   table of the database would keep it, and it would then need a rule for a
   media that the server does not hold now.

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
4. **A playback of a few seconds leaves no session on the server.** The server
   drops a session whose `timeListening` is 0, and the sync period is ten
   seconds. Therefore `GET /api/me/listening-sessions` shows nothing after a
   test with the books of three seconds. **Read `podman logs abs-test`
   instead:** the server writes one line "Starting session for user ..." for
   each `POST /api/items/:id/play`, and that line names the media.
5. **`currentTime` comes as a text, and not as a number,** in
   `GET /api/me/progress/:id`.
6. **A test that sets `XDG_CONFIG_HOME` must be alone in its binary.** That
   variable belongs to the process. Put the parts in one test function.
7. **`cargo fmt` before `git commit`, every time.**
8. **The server takes a name of a field that does not exist.**
   `?sort=bogus.field` gives `200` and an unspecified sequence. Measure a field
   before you offer it.
9. **`items` of `GET /api/me/listening-stats` is a map, and not a list.** A map
   has no sequence, therefore the code must make one or the screen moves at
   each frame.
10. **`GET /api/podcasts/:id/checknew` gives an empty list for a podcast that
    came one second before.** It compares with the time of the last
    examination. Read the feed and compare yourself.
11. **`convert_seconds` rounds to the minute.** It is wrong for a place in a
    media. Use `clock` of `src/utils/convert_seconds.rs`.
12. **`topGenres` of `GET /api/stats/year/:year` names its value `genre`, and
    `topAuthors` and `topNarrators` name it `name`.** A reader that takes
    `name` only finds nothing there. `TopName` takes both with an alias.
13. **The lists of the narrators and of the genres stay empty until a session
    comes after the metadata.** The server keeps a copy of the metadata inside
    each session, therefore a book that takes a genre today gives nothing for a
    session of yesterday.
14. **The first page of `GET /api/me/listening-sessions` is the page 0**, and a
    page after the last page gives `200` and an empty list, and not an error.
    Therefore the code must look at the list, and never at the status.
15. **`quick-xml` 0.41 gives an entity as its own event `GeneralRef`**, and not
    inside the text. A walk that reads `Event::Text` only loses `&nbsp;` and
    every named entity, and the count of the characters is then wrong. A
    reference does **not** make a new text node of the tree of a web page,
    therefore `cfi::Walk` must not count it as one.
16. **A comparison of two lists of texts must not read the text.** A book holds
    the word "very" two hundred times. The first comparison of the paths of
    `cfi.rs` with the paths of `epub.js` used the text as the key, and it
    reported 44 differences that were not differences. The two lists follow the
    sequence of the document; read them together.
17. **`playwright` wants its own browser.** The measurement with `epub.js`
    needs `play.chromium.launch(executable_path="/usr/bin/chromium")`, or
    playwright looks for a build that the machine does not hold.

## The shapes that this session made, and that the next work should follow

- **A structure first, and a global value after it.** `queue::Queue` holds no
  lock. Every test calls its functions directly, and the tests need no
  sequence. `queue::add`, `queue::take_next`, and `queue::snapshot` are a thin
  box around that structure. `force_sync` has the same shape.
- **A decision of the loop belongs in a value, and not in a branch.**
  `follow_playback` gave `()` before, therefore its caller could not know why
  the playback stopped. It gives `Outcome` now, and `the_queue_goes_on` is a
  pure function of one line that a test calls.
- **A measurement of your own work needs a second program.** The queue: the
  log of the **server**, and not the log of the program. The EPUBCFI: the real
  `epub.js` in a real browser, over 29315 texts.
- **Read the code that the server gives to the browser.** The bundle of the web
  client stands in the container, at `/app/client/dist/_nuxt/`. It settled the
  rule of `epub.js` for the version that this server holds, and not for the
  version of a package that a session downloaded.
- **A slot between a task and the screen.** The render is not asynchronous,
  therefore a task asks the server and it puts the answer in a
  `Mutex<State>`, and the render takes it at the next frame.
  `src/logic/stats.rs`, `src/logic/bookmarks.rs`, and `src/logic/authors.rs`
  all have the same four states: `Nothing`, `Waiting`, `Ready`, and `Fault`.
- **A measurement of the shape of an answer needs the data to exist.** The
  description of a series was `null` in the sandbox, and the measurement needed
  a `PATCH` first. **Do not write a reader for a list that you measured
  empty.**
- **Read the endpoint that you call already, before you add one.** `GET
  /api/series/:id` looked necessary for the description of a series. The list
  of the series carries that same field.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified
  technical English. Short sentences, active voice, one instruction per
  sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys`
  must find nothing. `libsqlite3-sys` and `ring` are the two known builds of C,
  and they stay. See T-20.
- No test may need the network. A test that needs the sandbox carries
  `#[ignore]` and says how to run it. Twelve such files exist now.
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
> AlbanDAVID/Toutui. Newest release **v0.7.12**; `Cargo.toml` is at 0.7.12, so
> the next release bumps it first — the release workflow refuses a tag that
> disagrees with `Cargo.toml`.
>
> Read `docs/HANDOVER.md` first. It has the state, the open items, and
> seventeen traps that cost real time. Then `docs/T-24-coverage.md` (the
> function-by-function comparison against Audiobookshelf 2.36.0; **section 6
> names what the program must not have, and why**) and
> `docs/TAKEOVER-BACKLOG.md` (the evidence for every closed item).
>
> The three large items of the old list are done: v0.7.11 the queue of media,
> v0.7.12 the EPUBCFI of `epub.js`, and the description of a series, which was
> done already. What stays is in the section "What is open":
> 1. **Live messages over socket.io.** Do not start it without the examination
>    against the rule of T-20: the tree holds no client of socket.io in pure
>    Rust, therefore this needs a new dependency. Measure what that dependency
>    brings, and write the answer in the backlog before you write code.
> 2. The narrators and the tags, if they are worth a view of their own.
> 3. The queue on the disk, if a queue that a restart loses is not enough.
>
> Rules that bind every change: run all three gates yourself before each
> commit — `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
> and `cargo test` with `ALSA_CONFIG_PATH` pointing at a real null asound file
> (`/dev/null` hangs the real binary). Baseline: 747 tests, 33 binaries, tree
> clean. All prose and user-facing strings in ASD-STE100 simplified technical
> English. No crate needing a system library; `cargo tree -i openssl-sys` must
> find nothing. No test may need the network — sandbox tests carry `#[ignore]`
> — and no test may hold a path of the machine. Never write to
> AlbanDAVID/Toutui; keep his credit everywhere it appears, and give
> `-R ealtun21/Toutui` to every `gh` command. Show a fault before you fix it,
> and measure against the sandbox (`docs/TEST-SERVER.md`, podman on `:13399`)
> before you write an endpoint — and make the data exist first, because an
> empty list shows you no shape. Verify your own work with a second program:
> the log of the server (`podman logs abs-test`), a real browser, or `curl`.
> Drive the real program in a pseudo terminal for every view — and answer both
> `ESC [ 5 n` and `ESC [ 6 n`, or the program looks broken when it is not.
> Tag, push, and keep working; don't wait for CI.
>
> The user tests each release as it lands and does not want to be asked before
> publishing a patch. The server of the user is theirs alone: ask before you
> use it, always with an isolated `XDG_CONFIG_HOME`, and never write its
> address into this repository. Measure against the sandbox instead.
