# The handover of 2026-08-12 (the sixth session of that day)

This document is for the next session. It says what is done, what is open, and the
traps that cost real time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of each
item, and `docs/T-24-coverage.md` for the comparison with the server.

**The newest release is v0.7.67**, and T-124 to T-127 belong to this session.
The items T-122 and T-123 belong to the session before it, T-119 to T-121 to the one
before that, T-112 to T-118 to the one before those, and T-105 to T-111 to the one
before them.

**No row of section 4 of `docs/T-24-coverage.md` says `Half`.**

## The session of the sixth turn of that day: the accounts, and three sweeps

**Three releases: v0.7.65, v0.7.66, and v0.7.67.** The session did the work that
the maintainer named (T-118), and it made the three sweeps that the road held.
**Two of the three sweeps found a fault, and one of those faults stops the
program.**

| Item | What | Keys |
|---|---|---|
| T-124 | **The program holds more than one account.** The key `a` adds one, the key `c` gives it the start, and the mark `▶` says which account starts | `S`, then `a` and `c` |
| T-125 | The search of a library of podcasts said "The server found nothing" for a podcast that the server found | `/` |
| T-126 | **The key `l` on a podcast of a page after the first stopped the program**, and the start of a library of 520 podcasts took 11.9 seconds with a slow server | `l` |
| T-127 | **The start asked one request for the position of each media of the Home view.** One answer holds every position | — |

### T-124, and the shape that T-123 gave it

**The maintainer said "yes, and a later session does the work"**, and the work
needed three keys and no new data: the table `users` holds a row for each account
already, with `is_default_usr`.

**Every key that changes the account starts the program again**, and that is the
answer of T-123: a login screen inside the same process draws a box that goes
away, and every task of the old account holds the old token. `exec` gives the new
program the terminal of the old one.

| The key | What the user sees |
|---|---|
| `a` | the login screen, with the address of the account of now in its field |
| `c` | the Home view of the account of the line. The program asks one time, because a playback stops |
| `l` of the account that starts, with a second account | the Home view of the account that stays |
| `l` of the one account | the login screen |

**A second login writes a second row, and the newest login starts the program**:
`auth_process` gives that row the start now, because two rows with
`is_default_usr = 1` let the rowid decide (T-118).

### T-126, and it is the one to know of this session

**A library of podcasts of more than 500 items met three faults at one time**, and
the first of them stops the program:

1. `App::new` read the episodes of **every** podcast of the first page, one
   request after the other, and it wrote nine lists of 500 rows.
2. The key `l` on the line 519 read `self.all_ids_pod_ep[519]` of a list of 500
   rows: **the program went away**, and tmux said "can't find pane". The panic
   never reached the log of the program.
3. `take_the_next_page_of_the_library` extended seven lists of the library and
   none of the nine lists of the episodes, therefore the podcasts of a later page
   held no episode and the view said "This podcast has no episode".

**The program reads the episodes of one podcast now, when the user opens it.**
That is the shape of T-70: a task asks, `logic::the_episodes` holds the answer,
and the render takes it at the next frame. The first frame of that library takes
**409 ms**, and it took 11.9 seconds with a server of 20 milliseconds.

### T-127, and the measurement of the start

**A proxy of Python gives every request of the sandbox a delay of 500
milliseconds**, and a poll of the box of the start every 50 milliseconds reads the
steps of the start:

| The step | Before | After |
|---|---|---|
| the libraries of the server | 165 ms | 165 ms |
| the shelves of the Home view | 649 ms | 649 ms |
| **the position of each book of that list (29 media)** | 1134 → 3228 ms | — |
| the series, the lists, and every item | 3228 ms | 1134 ms |
| **the first frame** | **3767 ms** | **1725 ms** |

`GET /api/me` holds `mediaProgress` for every media of the account, and the
program asks that endpoint for the permissions already (T-110). **A media that
the answer does not name played never**, therefore a library of books needs no
request of a position at all; a row of a podcast names the episode beside the
media, and such a media keeps its own request.

**Two faults of the program came out of that work.** `ebookProgress` is a
fraction and the program read `i64`: the answer of a book that the user read did
not read at all, and the line said "N/A" for a book of 92 percent. And one row
that does not read took every other row of the answer away (T-41).

### The three sweeps, and what each of them gave

| The sweep | The answer |
|---|---|
| **A media that plays while the program does other work** | **no fault.** The playback went from 2:28 to 9:34 while the reader stood open, the search and four sizes of the terminal changed nothing, and the log holds no line of a fault |
| **A library of podcasts of more than 500 items** | **three faults** (T-125 and T-126) |
| **A book of an EPUB of 100 megabytes, and an EPUB that is not valid** | **no fault.** The reader came after 2 seconds, the program held 55 megabytes, and the three files that are not an EPUB gave "This file is not an EPUB." |
| **A server that answers slowly** (the sweep that the session before this one began) | **one fault** (T-127), and every view of a sweep of nine keys said the truth |

**One measurement of the sweep of the EPUB did not repeat**, and a later session
must look at it: the first attempt of the book of 100 megabytes said "The program
did not get the book: No server address answered", and `curl` sends that book in
0.13 seconds. That moment came after a scan of the library, therefore the socket
of the live messages can have marked the address down (T-107).

### The data of the sandbox that this session made

`docs/TEST-SERVER.md` holds the commands of each: the library **`ManyPods`** of
**520 podcasts** (section 2f), and the four books of an EPUB of section 2g. **A
test that takes "the first item of a kind" breaks when the sandbox gains data**:
`the_place_of_the_ebook_against_the_sandbox` took the first EPUB of the library,
and the books of the sweep stand before Alice in the alphabet. That test names
the book that it needs now.

## The session of the fifth turn of that day: the two faults of a first start

**The newest release is v0.7.64, and T-122 and T-123 belong to it.** The user moved
from the program before this fork to this fork, and the first start gave two faults
one after the other. Each of them said a line of the source of the program, and each
of them drew no view.

| Item | What | The answer |
|---|---|---|
| T-122 | `configuration file … not found` | the binary holds `config.example.toml`, and it writes the file |
| T-123 | `The token is not valid. Log in again.` with no way to log in | the row of the account goes away, and the program starts again at its login screen |

**The trap of T-123, and it cost the design.** The first shape of the work made the
login screen inside the same process, after `restore_terminal` and a second
`ratatui::init`. The screen drew the box of the address one time, the box then went
away, and no key gave a character: **a second start of the terminal inside one
process does not work.** `start_the_program_again` of `src/utils/exit_app.rs` uses
`exec`, therefore the login screen of a first start comes, and every task of the old
token goes away with the process. The loop `'the_session` of `src/main.rs` stays for
a system that has no `exec`.

**A value of the process does not cross `exec`.** The address of the server goes to
the new program in `TOUTUI_THE_ADDRESS_OF_THE_LOGIN`, therefore the user reads their
address in the first field and one press of Enter takes it.

**The measurement, 2026-08-12, with the sandbox and tmux.** A configuration
directory with no file gave `config.toml` and the login screen. A real login of
`toutuitest` wrote the row; the token of that row then became a token that the
server refuses; the next start removed the row, said the reason, held the address,
and gave the Home view again after the name and the password.

## What the session before this one closed (T-119 to T-121)

**That session closed the last row of the table, and it then took a fault of the
user that is worth more than every row of it.**

| Item | What | Keys |
|---|---|---|
| T-119 | **The key `@` sends the ebook of the line to an e-reader.** The devices come from `POST /api/authorize`, and not from the settings of the e-mail | `@` |
| T-120 | **A later file that no decoder reads ended a playback that played.** The book of the user started again from the minute 0 | `l` |
| T-121 | **An account with no permission of a download played no book from its file.** Every media went to a stream of the server | `l` |

**Two releases: v0.7.62 and v0.7.63.**

### T-119, and the measurement that changed the shape of the work

**The road named `GET /api/emails/settings`, and that endpoint cannot do this
work.** Every route of `/api/emails/` holds an `adminMiddleware` of the server,
and it answers `404` for an account that is not an administrator — an account that
**can** send a book. `POST /api/authorize` gives `ereaderDevices` filtered for the
account of the token: one request, and no permission of an administrator.

**One fault of this program came out of that measurement.** The server took
**36.2 seconds** for a book of 479.5 megabytes, and `REQUEST_TIMEOUT` of the
client is **15 seconds**: a book of more than about 200 megabytes stopped at that
limit **while the server sent it**. The send holds a time limit of fifteen minutes
of its own now, and `post_and_read_the_answer` of the client keeps the body of a
refusal — the endpoint answers `404` for three different conditions, and the body
is the one place that tells them apart.

**Every row of section 4 of `docs/T-24-coverage.md` that still says `No` belongs
to an administrator of the server, or to work that the client must not do.** The
issue #24 holds that reading, and it waits for the maintainer to close it.

### T-120, the fault of the user, and it is the one to know

**The user said: "Depthless Hunger, Book 2 always starts from min 0", and the web
page and the client of Android both play it from their place.**

That book holds the same 26 hours **two times**: a file of AAC-LC of 93285
seconds, and the file of xHE-AAC of T-68 of 93278 seconds after it. The place of
the user (2 percent, 3731 seconds) stands inside the first file, and this program
plays that form itself.

**The engine did the right work, and `play_media` threw it away.** The engine
opened the track of the place of the user, it started at 3731 seconds, and it said
"The tracks before it play" for the file that it cannot read (T-48 and T-55).
`play_media` then read the flag of the fault and it asked the server for a stream
of the whole media: the user lost a playback that worked, and the audio began
again.

**The engine writes one flag for two conditions**, and they need two answers: a
track that the playback needs **now** does not open (the playback is dead, and the
stream of T-53 is the answer), or the engine skipped a **later** track (the
playback works). The state tells them apart, because `worker.rs` writes
`playback_id` in the loop that follows a playback that plays and a start that
failed never reaches that loop. `the_stream_must_take_the_playback` is a pure
function that reads "the playback plays" **first**, and two tests of
`tests/a_later_file_with_no_decoder_keeps_the_playback.rs` fail with the old order.

**T-53 does not change.** The book of the sandbox whose **only** file is xHE-AAC
still goes to the stream, and it resumed at the part 70 of that stream.

### T-121, and why no session before this one met it

`HttpFile::open` asked for `/api/items/:id/file/:ino/download`. The server holds
every route of a download behind the permission `download`, and it answers `403`
for an account that does not have it: **no book of such an account played from its
file at all.** The address of a track is the value of `contentUrl`, and it holds
no `/download`.

| The address, with a `Range` | An account `user` | An account `root` |
|---|---|---|
| `GET /api/items/:id/file/:ino` | **`206`** | `206` |
| `GET /api/items/:id/file/:ino/download` | **`403`** | `206` |

**Every measurement of every session before this one used `toutuitest` of the
sandbox, and that account is `root`.** A measurement of a permission needs an
account that does not hold it. `src/logic/download/fetch.rs` keeps the address of
a download, and that is right: the key `D` makes a real download.

### The two questions of the handover, and the answers of the maintainer

- **T-118, more than one account: yes, and a later session does the work.** That
  session is the session of T-124, and the program holds the function now: the
  key `a` adds an account, and the key `c` gives it the start.
- **T-116, the words while a book of a scan opens: leave the text as it is.** A
  book of 502 megabytes is rare, and the work would add a slot, a message of the
  child, and a rule of the render for one condition that few users meet. That row
  is a decision now, and not work that waits.

### The sweep of a server that answers slowly, and what it gave before the fault of the user came

**The sweep is not complete.** The fault of the user came first, and this section
holds what the measurement gave.

A proxy of 60 lines of Python held a port and it gave every request of the sandbox
a delay. The pool of the program takes it with a block `[[servers]]` of
`config.toml` whose endpoints hold the slow address first and the real address
after it.

| The measurement | 500 ms of every request | 2000 ms of every request |
|---|---|---|
| The first frame | (not measured) | **14.6 s**, and it is 0.4 s with the sandbox |
| The header | it says the slow address, therefore `pool.active()` is the truth (T-105) | the same |
| Every view of a sweep of eight keys | each of them drew, and no view said a false reason | — |

**14.6 seconds of a server of 2 seconds means about seven rounds of requests, one
after the other.** The start of the program holds four requests that go together
(T-86), therefore the rest of that time belongs to the work after them. **The
screen holds the box of the start for every second of it**, and that box says the
address and nothing of the wait. No session has measured which requests of the
start are in sequence.

**The key `R` blocks the whole loop while it asks the server again**, and the code
of `main.rs` says it: the program draws one frame with the message "The program
asks the server again…" and then it waits. With a server of 2 seconds the keys of
the user wait behind that work, and they all act when the answers come. The
message is honest, and the length of the wait is not in it.

## What the session before this one closed (T-112 to T-118)

**That session made the four sweeps that the road of the session before it named.
Every one of them found a fault, and one of them found three.**

| Item | What | Keys |
|---|---|---|
| T-112 | **The key `G` goes to the end of the library**, and it went to the end of the page: a user of 2056 items pressed it six times | `G` |
| T-113 | **The search shows the media that the server found.** A book of a page that the program did not read gave no line, and every search of a library of podcasts said that the server found nothing | `/` |
| T-114 | **A text of no letter is not a value.** The line of a book said "Author:  - Year: N/A" | — |
| T-115 | **A box that takes a text follows the size of the terminal**, and the header of a narrow screen keeps every value | `/`, `A`, `c`, `p` |
| T-116 | **The program held a whole book of 502 megabytes in its memory.** The peak was 1007 megabytes, and it is 44 | `e` |
| T-117 | The reader said the identity of the item as the title of a book of the search | `/`, then `e` |
| T-118 | **The text of the accounts promised a function that no key reaches.** The program holds one account | `S` |

**Six releases: v0.7.56 to v0.7.61.**

### The four sweeps, and what each of them found

**1. A library of the size that a user has (2056 items).** The library `Large` of
the sandbox holds 2056 items, and every one of them holds **no tag at all**: that
is a book that a user takes from a disk of their own.

| The work | The measurement |
|---|---|
| The first frame | **609 ms**, with 2056 items in the library |
| The start | **one** request, and the title says "500 items of 2056" |
| 500 presses of `j`, over the end of the page | the page comes, and **no line of the user moves** |
| The key `G` | **the item 500 of 2056**, and six presses for the end (T-112) |
| The search of a book of the page 4 | **"The server found nothing"**, and the server found it (T-113) |
| The line of a book with no tag | **`Author:  - Year: N/A`** (T-114) |
| A page of 500 items, with `curl` | 2 ms and 470 kilobytes |

**2. A terminal that changes its size while the program runs.** The program stands
at **every size from 200x50 to 10x3**, the reader reflows and it keeps its place
(chapter 5 of 14, 16%), and every view draws again. **Two faults of one shape:** a
box of a text read the size one time and it then drew outside the screen (the user
saw **nothing at all**), and the three parts of the header wrote on each other
below 68 columns (T-115).

**3. A book of a scan of 502 megabytes, at `MAX_BOOK_BYTES`.** The child of T-62
does its work: 974 megabytes of the parse stand outside the program of the user,
and that memory goes away with the process. **The program of the user held 1007
megabytes of its own** (T-116), because `download_to_file` read
`response.bytes()`. It holds **44** now. The parse of 150 pages takes **2 minutes
4 seconds**, and the reader said the identity of the item as the title (T-117).

**4. Two accounts of two servers, with a position of a media of each.** **The
condition cannot exist**: the view of the login comes only when the database holds
no account, therefore no key adds a second one. The view of the accounts lists the
account of the start only, every login writes `is_default_usr = true`, and with two
such rows the **rowid** decides. Every value of an account is right for the account
that starts: the two servers gave 50% and 33% of their own media. The text of that
view promised the function, and it says what the program does now (T-118). **The
question of the function itself belongs to the maintainer**, and T-118 of the
backlog holds it.

## What the session before this one closed (T-105 to T-111)

| Item | What | Keys |
|---|---|---|
| T-105 | **The header names the address of the server that answers**, and not the address of the login | — |
| T-106 | The statistics of a library of one media said "1 items" | `T` |
| T-107 | **The program said "Connected" while it knew that the server does not answer** | — |
| T-108 | **The guard of "1 item" read one form of one text, in four files.** It reads every file of `src/ui` and of `src/logic` now | — |
| T-70 | **The program reads the library page by page.** The start made 5 requests of a library of 2056 items, and it makes 1 | — |
| T-62 | **A child process reads a PDF book.** The memory of the parse stays outside the program of the user | `e` |
| T-66 | **Shift+Tab takes the next library of the server** | `S-Tab` |
| T-110 | **The account and its permissions, `?collapseseries=1`, and two endpoints that stay outside** | `S` |
| T-111 | A child read a PDF inside a test, and that child was the test | — |

**Five releases: v0.7.51 to v0.7.55.** Four groups of work, and one release more
for T-111: the run of every test with the sandbox found that fault after the tag
of its group.

**The three sweeps that no session had made each found a fault, again.**

- **A pool of two addresses (T-105).** `socat` held the first address and it sent
  every byte to the sandbox: 9 connections of the start went to
  `127.0.0.1:13456`, and the header said `localhost:13399`. Every view of the
  program works with two addresses, and the change of the address needs no key.
- **A library of one media (T-106).** The view of the statistics said
  "1 items, 57 tracks, 0 authors, 0 genres". **The book of one chapter found no
  fault**: the view of the chapters says "[1 item]".
- **A server that goes away in the middle of a playback (T-107).** The playback
  went on from the bytes that the program held, the position of 1800 seconds
  waited in `pending_progress`, and the header said "Connected as toutuitest"
  for **60 seconds** while the log of the same program said "the server does not
  answer" every six seconds.

**The pool is the truth of the header now.** `render_header` reads
`pool.active()`, and the task of the live messages marks an address down for one
fault only: a connection that no machine takes. A time limit is not evidence
(T-97), and a connection that opened is not evidence either.

**T-70, and the measurement of a mock server.** A library of 2056 items: the
start made **5** requests of `/items`, and it makes **1**. The program asks for
the next page when the user comes near the end of the lines that it holds, and
`ui::keys::the_lines_of_the_library` says "500 items of 2056" while a page stays
outside.

**T-62, and the memory of a book of a scan.** A PDF of 47 megabytes, 60 pages of
1200 by 1600 pixels:

| The moment | With the parse in the program | With a child |
|---|---|---|
| While the parse runs | 101 MB, and it grows to 113 | **39 MB, and it does not move** |
| The book stands on the screen | 113 MB | **53 MB** |

The child took 106 megabytes at its peak, and that memory went away with the
process. **This needed no dependency**: `std::process` spawns the child, and
`pdf_of_a_child` writes the form of the file.

**T-110, and the two measurements that decided a row.** `?collapseseries=1` gives
the **same screen**: 14 items with no parameter and 10 lines with it, the same 10
lines that `group_library` makes. `GET /api/podcasts/:id/checknew` gives **0**
episodes for a podcast that is missing **54**, therefore it stays outside.

## The state

`main` is clean and pushed, and `v0.7.67` is tagged. Every gate passes:

```
nice -n 19 ionice -c 3 cargo clippy --all-targets -j 16 -- -D warnings
nice -n 19 ionice -c 3 cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> nice -n 19 ionice -c 3 cargo nextest run -j 16
    # 979 tests pass in 2.2 s, 25 carry #[ignore], 49 binaries
    # cargo nextest run --run-ignored all gives 1004 of 1004 with the sandbox up,
    # in 16.4 s: one test waits 16 s for the time limit of the send of a book
    # (T-119)
cargo tree -i openssl-sys                # finds nothing
cargo tree -i cc                         # finds libsqlite3-sys and ring only
```

**The sandbox holds more data now, and every test of the sandbox still passes.**
The library `ManyPods` of 520 podcasts and the four books of an EPUB came with
this session (the sections 2f and 2g of `docs/TEST-SERVER.md`), and the library
`Large` of 2056 items and the PDF of 502 megabytes came with a session before it.

**A test that takes "the first item of a kind" breaks when the sandbox gains
data.** `the_place_of_the_ebook_against_the_sandbox` took the first EPUB of the
library, and the three books whose EPUB is not valid stand before Alice in the
alphabet: the run of every test found it, and the fast suite did not.

**`cargo nextest run` gives the same tests in 2.3 seconds**, and `cargo test` gives
them in 8.7. Use nextest: `.config/nextest.toml` stands in the repository, and the
tool is on this machine. See T-74.

**Every test of the sandbox passes too.** One run of
`cargo nextest run --run-ignored all` gives **937 of 937**, and the group
`the-sandbox` of `.config/nextest.toml` runs them one at a time for the rate limit
of the login. With `cargo test`, give `-- --ignored --test-threads=1`.

**The rate limit of the login stopped three runs of the session of T-96, and that
item closed the fault.** Every test of the sandbox takes its token from
`tests/common/mod.rs`, and that module keeps the token in a file of
`CARGO_TARGET_TMPDIR`: one run makes **one** login, and the run after it makes
none. Three runs one after the other now give no line of the rate limiter.

**A run that says "the answer must hold a token" still means the rate limit.**
Read `podman logs abs-test` for "[RateLimiter] Rate limit exceeded", and give the
container a restart. A sweep of the view of the login uses those 40 requests.

**The fault of one run of ten has a name now: T-86.**
`the_four_requests_of_the_start_go_together` failed when the whole start took more
than two seconds, and that is a measurement of the machine as much as of the program:
one run of twelve of that session failed at 4.2 seconds while a build and a program of
tmux ran beside it. The test holds the **time of each request** now, therefore the load
of the machine changes nothing. Twelve runs after that change gave every test each
time.

Two tests read the books of the survey. Those books stand outside the repository,
therefore give their directory in `TOUTUI_SURVEY_BOOKS`. A run with no such variable
reads `tests/data/alice.epub` only, and it passes.

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must. The work of that tag is in
v0.6.7. Do not try to publish v0.6.6.

## What the session before this one closed (T-101 to T-104)

| Item | What | Keys |
|---|---|---|
| T-101 | **The changelog holds every release of this fork**, and a test holds that rule | `S` |
| T-102 | **The sequence of the media inside a collection or a playlist** | `c`, `l`, then `<` and `>` |
| T-103 | **The Home view and the Library view of a library with no media said nothing** | `Tab` |
| T-104 | **The 6 rows of the player go to the view while nothing plays** | — |

**A release holds three files together**: `Cargo.toml`, `Cargo.lock`, and one new
entry at the top of `THE_ENTRIES_OF_THE_FORK` of `src/utils/changelog.rs`. The
gate refuses a release with no entry (T-101).

## What the session before that one closed (T-88 to T-100)

| Item | What | Keys |
|---|---|---|
| T-88 | **A view that makes a collection or a playlist** | `m`, then `c` or `p` |
| T-89 | The box that takes a text left two columns of the view on the screen | `b`, `A`, `c`, `p` |
| T-90 | **Every footer lost its end in a terminal of 80 columns** | — |
| T-91 | The program said that the library holds no list, and the server was down | `c`, `s`, `m` |
| T-92 | **The login said "ERROR: Login failed" for every fault** | — |
| T-93 | **The keys that remove a list and that give it a new name** | `c`, then `r` and `X` |
| T-94 | The row of the item lost its end in 80 columns | — |
| T-95 | "1 items" a second time, in the view of the search | `/` |
| T-96 | **One login for every test of the sandbox**, and a feed that is not a fault | — |
| T-97 | **One request that stopped at its time limit took the server away** | — |
| T-98 | **`CARGO_TARGET_DIR` on a `tmpfs` gives nothing.** A measurement, and no code | — |
| T-99 | A terminal of 18 rows showed one line of the list | — |
| T-100 | **The description of a collection and of a playlist** | `c`, then `D` |

**That session made every sweep that no session had made**: the terminal of 80
columns, the offline mode, the view of the login, and the reader of a book.
**Each of them found a fault**, and T-90, T-91, T-92, T-94, and T-95 are those
faults. The sweep stays the tool that finds what a test does not.

### The items of that session, and what each of them taught

**T-88, and the measurement that decided its shape.** The first idea was a view
that asks for a name and a library. **The server refuses it**: `POST
/api/collections` answers `400` with "No books" for a collection with no book. A
playlist with no item gives `200`, therefore the two requests do not behave in the
same way. **A new collection needs a media**, and the key `m` holds one already:
the work stands in that view, and it needs no new view at all. **Measure the
request before you draw the screen that sends it.**

The server also takes **two lists of one name**, and it gives the second one its
own identity. The measurement made two lines "Measure Collection [1 item]" on the
screen that no key tells apart, therefore the program refuses that name.

**T-89, and the shape of every box of the program.** `ask_for_a_text` makes a
`Terminal` of its own. **ratatui writes the cells that changed only**, and it
compares with the buffer that it holds itself: the terminal of the program knew
nothing of the letters that the box wrote over, therefore they stayed. `Clear`
over the whole rows and one `terminal.clear()` after the box answer it. T-42 held
the same answer for the key `R` already.

**T-90, the first screen of the sweep of 80 columns.** The footer said
"?: every k". **The area of the footer holds two rows**, and the footer wrote on
one of them: a `Paragraph` with no `Wrap` cuts every letter after the width. The
module `ui::keys` said that every footer fits in 80 columns, and no footer of more
than 80 letters did.

**T-91, the sweep of the offline mode.** With `podman stop abs-test`, three views
said something that they cannot know: "This library has no collection and no
playlist", "This library has no series", and "Press c or p to make one". **The
view of the authors held the rule already**: "The server gave no author: the
server does not answer". A view must not give a reason that the program does not
have.

**T-93, the two keys of a list, and the asymmetry of the server.** `PATCH` of a
collection with a name of no letter gives `200` and a collection with **no
name**, and the same request of a playlist keeps the old name. The server
examines the name when it **makes** a list (T-88), and it examines nothing here.
**The rules of a name therefore belong to the program**, and they are the rules
of T-88. The program asks one time before it removes a list, and the question
names the kind: every user of the server sees a collection.

**T-94 and T-95 came from the sweep of 80 columns**, and both are a fault that a
session before this one corrected in a different place. T-94 is T-90 again: the
row of the item holds three rows, it wrote on one, and 21 paragraphs of the views
held that fault. T-95 is T-85 again: the title of the search counted its own
items, and **the test of that title asked for "1 items" and it passed**.

**T-96 and T-97 came from the tests themselves.** The rate limit of the login
stopped three runs, and the answer is one login for a whole run. The measurement
of that work gave the sequence of T-97: a request of a feed stopped at its time
limit, and **every request after it said "No server address answered"** for up to
60 seconds. T-87 corrected that for a status of the server, and this corrects it
for a fault of the transport.

**T-98 is a measurement that says "do not do this work".** Three handovers
carried `CARGO_TARGET_DIR` on a `tmpfs`, and the memory is 16 percent faster than
the ZFS of the maintainer for 3000 small files. A build writes 11 gigabytes, and
that costs about 8 seconds of a disk that gives 1.4 GB/s.

**T-99, the sweep of a small terminal.** A terminal of 100 by 18 gave the Home
view **one line** of 24 media: the row of the item took 3 of the 7 rows of that
area with a fixed length, and 10 rows of the screen were empty.
`the_areas_of_a_list` gives every row to the list when the area holds 12 rows or
fewer, and a terminal of 24 rows and one of 45 rows draw what they drew before.
**T-104 closed the question that this item left.** The 7 rows of the player stayed
empty while nothing played, and the maintainer decided on 2026-08-12: the view
takes 6 of them, and the row of the message keeps the seventh.

**T-100, the description of a list.** The view showed the description of a list
already, and no key made one. The same `PATCH` of T-93 takes it.

**The sweep of two accounts found no fault, and a measurement of this session
said the opposite for some minutes.** The key `l` of the view of the accounts
gives its question at the first press. The wrong measurement read the top of the
screen after the key, and then it read the row of the message 10 seconds later:
**a message of the program lives six seconds.** Read the row that holds the
message, and read it inside that time.

**T-92, the sweep of the view of the login.** Two of the three messages of that
view are good work already, and the third said "ERROR: Login failed" for every
status. **The status of the answer holds the reason**, and 429 is the status that
costs the most time: the rate limit of 40 requests of 600 seconds. Three other
items came out of the same screen: the word "ERROR: " that no other message holds,
an empty username that went forward with no word, and the field of the address
that a wrong password emptied.

## The sessions before those (T-74 to T-87)

| Item | What | Keys |
|---|---|---|
| T-74 | **The run of the tests takes 2.2 seconds**, and it took 18.7 | — |
| T-75 | Two texts of the screen that a sweep of the views found | `v`, `S` |
| T-76 | **The books of a media, when an item holds more than one** | `e` |
| T-77 | **The settings write `config.toml`**, and they keep every comment | `S` |
| T-78 | The message of the program took the letters of the view below it | — |
| T-79 | The key `h` of the view of the search did nothing | `/` |
| T-80 | **The keys of the volume said nothing.** The row of the player names the volume now | `o`, `i` |
| T-81 | **The queue of the episodes that the server downloads**, and the key that empties it | `d`, `X` |
| T-82 | The choice of the library changed nothing on the screen | `S` |
| T-83 | The key `s` of a library of podcasts said nothing | `s` |
| T-84 | **The media of a collection and of a playlist** | `m`, `X` |
| T-85 | "1 items" | — |
| T-86 | The test of the requests of the start measured the machine | — |
| T-87 | **One answer of 400 took the address of the server away** | — |

### The sessions before those two

| Item | What | Keys |
|---|---|---|
| T-66 | **A media that a different client finished leaves Continue Listening**, with no request | `N` |
| T-67 | The cache of the ebooks holds a limit of one gigabyte | `e` |
| T-68 | **The book of xHE-AAC of the user, and four faults of the stream.** T-53 and T-63 close with it | `l` |
| T-69 | **That book plays now**, from a place beside the one that stops ffmpeg of the server | `l` |
| T-70 | A search of the name of an author gives the books of that author | `/` |
| T-71 | The user reads that the cache of the ebooks removed a book | `e` |
| T-72 | `config.toml` holds the limit of that cache, in a block `[reader]` | — |
| T-73 | **The narrators of the library**, in the view of the authors | `v` |
| T-51 | **The decision of the maintainer: Toutui stays GPL.** bookokrat gives ideas only | — |
| T-20 | **The decision of the maintainer: the two builds of C stay.** Both answers need a crate that is not ready | — |

**Six issues of the fork closed with a measurement and no code:** #10 (the reader of
an EPUB book and of a PDF), #17 (Opus), #18 (WMA and AWB through the stream), #22
(the series), #23 (the cover art), and #15 (the login does not fail at the first
attempt). **#24** holds a comment with the state of the comparison, and it stays open
as the umbrella. **#20** stays open, and its comment holds the decision above.
**The items T-27 to T-67 have no issue.**

### The items of the session before this one, and what each of them taught

**T-74, the tests.** `cargo test` was not the slow part: two tests held 12.7 seconds
of the 18.7 of the run. One of them holds a clock of its own now
(`#[tokio::test(start_paused = true)]`), and the other polls for the evidence that
the loop acted. **Both still show the fault**: a build with the correction removed
fails. `cargo nextest` runs every test in one pool of processes, and the run takes
2.2 seconds.

**T-75, the sweep of the views.** Thirteen views in 9.7 seconds with
`docs/harness/drive.sh`. It found two texts that no test held: the footer of the
narrators said "author", and a text of the settings held a run of 22 spaces. A test
now holds every text of a view to one space between two words.

**T-87, and it is the one to know.** `ApiError::is_endpoint_fault` held every status
of `Server` as a fault of the address. One answer of **400** therefore marked the one
address of the pool down, and **every request after it said "No server address
answered"** until the examination of the address ran again. The server answers 400 for
work that a user does every day: a book that stands in a collection already, and a
podcast whose directory exists. A fault of the endpoint is a transport fault or a
status of 500 or more now.

**T-77, the settings that write.** `with_the_value` changes one line of the file
and it keeps every other line, therefore every comment of the user stays. The
write is atomic (a file beside it, and a rename), and the program uses the new
value with no restart.

**T-78 and T-79 came from the sweep of the reader and of the search**, and no
test held either. A `Paragraph` of ratatui gives its style to every cell of its
area and it writes its own text only: the message of the program therefore stood
between the letters of the book. The key `h` of the view of the search moved
nothing, because the handler held no line for that view.

**T-76, the books of a media.** `media.ebookFile` names one book, and `libraryFiles`
holds every file of the item. The key `e` inside the reader gives the list.
**The server holds one place for each media, and not one place for each file**,
therefore the place of a book that is not the book of the server stays on this
machine.

**T-81, the queue of the downloads of the server (`d`).** The key `E` gives the server
the episodes of a feed, and the server does that work alone. The view holds the episode
of now with the mark `▼` and the queue after it, and **it asks the server again at each
message `episode_download_*` of socket.io**: the list moved from 9 lines to 4 while the
screen stood open and the user pressed no key. The key `X` empties the queue of one
podcast, and it asks one time first.

**T-84, the media of a collection and of a playlist (`m` and `X`).** The program read
the lists and it changed none of them. Four requests do that work, and the server
answers **400** for a media that stands in the list already. That answer found T-87.

**T-80, T-82, T-83, and T-85 came from the sweeps too**: the keys of the volume said
nothing, the choice of the library changed no line of the screen, the key `s` of a
library of podcasts said nothing, and every title of a view said "1 items".

### The two items of an older session

**T-66, the shelf of Continue Listening.** T-47 gave the mark of the line the
position of a live message. **The line itself stayed**: a different client finished a
book, and the Home view held its line until the key `R`. `mediaProgress` of
`user_updated` carries the whole account, and two of its values keep a media away
from that shelf: `isFinished` and `hideFromContinueListening`. The program holds
every line of that shelf already, therefore the render makes the lines again and it
asks the server for nothing.

**The fault of the first form of that work, and how the measurement showed it.** The
list held the **identity** of a media. One media stands on two shelves: the book
stood on Continue Listening and on Recently Added together, and both lines went away.
The count of the view fell from 22 lines to 20, and the server gives the second line.
**The list holds the number of the line now.** A pure function and eight tests would
not have found this: the shape of the data of two shelves did.

**T-67, the cache of the ebooks.** The time of the file is the time of the **last
use**, and `std::fs::FileTimes` writes it. Therefore this needed no dependency. The
book that the user reads now never goes away, and the program looks at the limit
after a new book came only.

## The keys of the program

`src/app.rs` holds the key handler, and it is the authority. **`src/ui/keys.rs`
holds the list for the user**, and the key `?` shows it. A new key needs a line in
that file, or the test `every_key_of_the_handler_stands_in_the_list` fails.

## What is open

### 1. The book of xHE-AAC: it plays now (T-68 and T-69)

**T-53 and T-63 close with T-68, and T-69 makes the file play.** The user gave the
real file on 2026-08-11, and a piece of 10 minutes of it stands in the sandbox.

**A wrong answer of that day, and the correction.** The first form of T-68 read the
sentence "Not yet implemented in FFmpeg, patches welcome" and it said that no program
plays that form. The user said "MPV can play xHE-acc but it does it with a lot of
errors so we should be able as well", and they are right: **ffmpeg reads 77 percent of
the frames.** 60 seconds of the file gave 46.2 seconds of audio, and mpv gave 196
lines "Error decoding audio" in 20 seconds. **A sentence of an error is not a
measurement of the whole file.**

What holds now:

- symphonia reads no frame of that form, therefore the program uses the stream of the
  server. That is the shape of T-53, and it needs no decoder.
- ffmpeg of the server cannot **copy** the codec into a transport stream, therefore it
  stops with the code 183 and the server tries again with `-c:a aac` **ten seconds**
  later. No LATM exists at any moment.
- **The place of the media decides if ffmpeg lives.** Two places of the eight of the
  measurement give a frame of NaN to the encoder, ffmpeg stops with the code 234, and
  the server deletes the session. T-69 tries a place beside it, and the file plays.
- The sound holds a hole at each frame that ffmpeg drops. **No client does better
  today**, and the AAC-LC file of the same book is the better answer for a user.

T-68 and T-69 of `docs/TAKEOVER-BACKLOG.md` hold every measurement and the five
faults of the program that they found.

### 2. The road, and what stays of it

**Every sweep of the road of 2026-08-12 is made now.** The four conditions of the
road of the session of T-112 to T-118 are measured, and the four of the road of
the session after it too (T-124 to T-127): **six sweeps of eight found a fault**,
and the sweep of a library of podcasts found three.

#### What a next session can take

**Every sweep of the road of 2026-08-12 is made now**, and T-118 is done (T-124).
The road of the next session holds the work that those measurements left.

1. **The one measurement that did not repeat, and it is the first work.** The
   first attempt of a book of an EPUB of 100 megabytes said "The program did not
   get the book: **No server address answered**", and the server sends that book
   in 0.13 seconds with `curl`. Four attempts after it gave the book. That moment
   came after a scan of the library: **the socket of the live messages marks an
   address down for a connection that no machine takes** (T-107), and a server
   that is busy can give that fault. A session must make the condition again with
   the log of the program, and it must then read `the_address_is_down`.
2. **The requests of the start that are left.** T-127 took 29 requests away, and
   the start still holds four rounds with a server of 500 ms: the libraries, the
   shelves, the four requests that go together, and the sound device. **The
   shelves and the four do not need each other**, and no measurement says why
   they wait. The first frame is 1.7 seconds with that server.
3. **The row of the table that stays.** *Send an ebook to an e-reader*
   (`GET /api/emails/settings`). It is the one row of section 4 that says `No`
   for a function that a user of a terminal can use. **The issue #24 stays open
   for that row**, and T-119 holds the measurement that says why the endpoint
   cannot give it to a user.
4. **T-116: the words while the user waits.** The maintainer decided that the
   text stays as it is (2026-08-12). The row is a decision, and not work that
   waits.
5. **The sweeps that no session has made.** Every new condition found a fault in
   six sessions of seven. These conditions stay:
   - **A library of books of more than 500 items, with a media that plays**: the
     paging of T-70 and the playback together.
   - **A server that answers slowly while a media plays**: every measurement of a
     slow server of 2026-08-12 played nothing.
   - **A second server of a second account** (T-124): the key `c` while a media
     plays, and the position of that media.
   - **A library whose media the account may not read** (the permissions of
     T-110), with an account of the type `user`.
6. **The fast suite stays at about 2 seconds.** It holds 979 tests in 2.2 s. A
   new test that needs a wait belongs behind `#[ignore]`.
5. **`cargo nextest run --run-ignored all` belongs at the end of a session**, and
   not at the end of one item: it took 18.5 seconds on 2026-08-12, and it found
   two faults of the session before this one that the fast suite did not (T-111).
6. **The words for the user.** Every text in ASD-STE100. A view says why it holds
   no line, and it never says a reason that the program does not have (T-91). **A
   text must not promise a function that the program does not have** (T-118). A
   key that does nothing in one view is a fault of its own (T-79). A message
   lives six seconds.

### 3. The decisions that the maintainer made, and what they bind

**Toutui stays `GPL-3.0-or-later` (T-51).** bookokrat is AGPL. **A person may read
that project for the idea of a function, and they must then write their own code.**
No line, and no near copy: write the code from the rule, and not from the text of the
code. A commit that comes from such an idea **names that project and the idea**. The
rule of T-20 holds too, therefore `mupdf` and every library of the machine stay
outside.

**`libsqlite3-sys` and `ring` stay (T-20).** The binary of the release needs no
library of the system. `ring` carries the TLS of every request and `libsqlite3-sys`
reads the encrypted token, therefore an alpha version and a pre-release are not
acceptable in those two places. Look again when `turso` is a release and
`rustls-rustcrypto` is beta or better.

**The seven decisions of 2026-08-12, and every one of them is done.** The
maintainer answered each of them before the session of that day, therefore no
item of the road needed a question. Section 4 below holds the two answers that a
measurement changed.

1. **The rows of the player reflow while nothing plays** (T-104, done).
2. **T-62, the memory of a PDF: a child process reads the book.** The program
   spawns itself with a hidden flag, it parses the PDF in that process, and it
   writes the text and the pictures to the cache. **The peak of the memory and
   every fault of `lopdf` stay outside the program that the user sees**, and this
   needs no dependency: the rule of T-20 holds, and `mupdf` stays outside.
3. **T-66, Continue Listening of a different library: a key cycles the library in
   the Home view, and that key is Shift+Tab.** `Tab` moves between Home and the
   Library, therefore Shift+Tab reads as its pair, and crossterm gives it as its
   own code (`BackTab`): `handle_key` needs no work for a modifier. **The Home view
   keeps one library**, and no request of the start changes.
4. **T-70: the program reads the library page by page, when the user needs the
   page.** `get_all_books` read every page at the start. The cost of the start
   therefore goes away for a library of every size, and the search of the server
   stays the authority for a title that the program did not load.
5. **The program adopts `?collapseseries=1` and
   `GET /api/podcasts/:id/checknew`.** `GET /api/tags` stays outside: `filterdata`
   gives the tags of the library that the user reads, and that is what the key `f`
   needs. **The reason of that row belongs in section 6 of
   `docs/T-24-coverage.md`.**
6. **The account and its permissions stand in the settings, under "Accounts and
   log out".** The type of the account and the permissions that change the work of
   the program, in the words of a user. A key that a permission refuses keeps its
   own message too.
7. **A measurement of that session plays no sound.** Nobody can answer a question
   while it runs, therefore the sweep of a server that dies in the middle of a
   playback measures the position, the message, the session of the server, and the
   parts of the stream. **No run of that session opens the real sound device.**

### 4. The decisions that this session made

**The maintainer answered no question before this session**, because the road held
sweeps and a sweep needs no decision. This session made four decisions of its own,
and each of them follows a rule of this document.

**1. The key `G` reads the pages that are left, and it does not stop at the page.**
A page of 500 items costs 2 ms and 470 kilobytes of the sandbox, therefore one
press of `G` in a library of 2056 items costs four requests and 2 seconds. **The
user asked for the end**, therefore the cost is theirs; a move of the user ends the
wait at once. A library of 250000 items would need 500 requests for that key, and
that is the same cost that T-70 took away from the **start**. See T-112.

**2. The view of the search shows a media that the program did not read, and a
library of podcasts drops such a line.** The answer of the server carries the whole
media, therefore a book of every page needs no request more. **The lists of the
episodes of a podcast come from the place of the media in the lists of the
library**, and a podcast that the program did not read has no episode at all: a
view that opens with no episode would say a reason that the program does not have
(T-91). One page holds 500 podcasts, therefore no user of the measurement meets
that condition. See T-113.

**3. The header of a narrow terminal takes a short form, and it keeps every
value.** The three parts of the header stand on one row, and every part writes its
own letters only (the trap 32): below 68 columns they met. `👋 toutuitest` and
`🦜 v0.7.58` hold every value with fewer words. **41 columns is the honest limit**
of that form, and a terminal of fewer columns holds one word of a title. See T-115.

**4. The text of the accounts says what the program does, and the function stays
for the maintainer.** The text promised more than one account, and no key of the
program reaches that function. **A text must not promise a function that the
program does not have**, therefore the words changed now. The function needs three
keys, and that decision is not the decision of a session: T-118 of the backlog
holds the question.

### 5. The decisions of the session before this one

The maintainer answered seven questions before that session. **Two of those
answers could not hold as they stood**, and this section holds the change and the
measurement that made it. Every other answer holds as the maintainer gave it.

**1. `GET /api/podcasts/:id/checknew` stays outside.** The maintainer asked for
that endpoint *where it is cheaper than the work of the program*, and with the
rule that *no episode that is missing may go away*. The measurement of
2026-08-12 shows that the two conditions cannot both hold: the podcast of the
sandbox holds **3 episodes of a feed of 57**, and the endpoint gives **0**
episodes in 15 bytes while the feed gives 57 in 27598 bytes. It compares with the
time of the last examination, therefore it says that nothing is new for a podcast
that is missing 54 episodes. **It is cheaper only where it is wrong.** The
nearest answer that keeps the rules: the program keeps its own work, the row of
the table says `Yes` for the function, and section 6 of
`docs/T-24-coverage.md` holds the endpoint with this measurement.

**2. Shift+Tab works in the Home view and in the Library view.** The maintainer
named the Home view. **The two views share one footer**, and a key that a footer
names and that does nothing in that view is a fault of its own (T-79). The
nearest answer that keeps the rules of this document is therefore the key in both
views. `Tab` still moves between them, and the Home view still holds one library.

**3. `group_library` stays, and the program takes `?collapseseries=1`.** The
maintainer said that `group_library` goes away only when the screen of the server
is the same screen. **It is the same screen**, and the function still holds work
that the server does not do: it gives the line of a series the place of that
series in `App::series`, and the view reads the books, the description, and the
cover there. The parameter takes the items of a series out of the answer, and
`total` then counts the lines that the user reads.

**4. The largest footer is 130 characters, and it was 92.** The key of T-66 needs
a place in the footer of the Home view and of the Library view. The area of the
footer holds **two rows**, therefore a terminal of 80 columns holds 160 cells. A
measurement of the real program in a terminal of 80 columns read every word of a
footer of 116 characters, on two rows.

**5. `TOUTUI_AUDIO_DEVICE=null` is the way to measure a playback with no sound.**
`ALSA_CONFIG_PATH` does not silence the real program (the trap 11 of the
harness), and this variable gives the null device of ALSA: the log of the program
then says "[worker] the application uses the sound device alsa:null". **No run of
this session opened the real sound device.**

## The harness: where the time went, with numbers

The maintainer said on 2026-08-11 that the tests take much time and that they make
the machine slow. **This session measured every part of the work.** The answer is not
the one that the session expected: `cargo test` is not the slow part.

| The work | The time | The measurement |
|---|---|---|
| A cold build of every test | **21 s** | `cargo test --no-run -j 16` after `cargo clean --profile dev` |
| A build after **one** edit of the library | **6 s** | `touch src/logic/authors.rs`, then the same command |
| The run of every test | **2.2 s** | `cargo nextest run -j 16` (T-74; it was 18.7 s with `cargo test`, and it is 8.7 s with it now) |
| One sweep of thirteen views in tmux | **9.7 s** | `docs/harness/drive.sh`, with a poll of the screen |
| One restart of the sandbox | 13 s | `podman restart abs-test` |

**T-74 did the work of the three sections below.** They stay here, because the
numbers say why each answer was the right one.

### 1. The first frame comes after 673 milliseconds, and this session waited 17 seconds

Every measurement of this session began with `sleep 17` before the first key. A
measurement of 2026-08-11 polled the screen every 500 milliseconds instead:

```
the Home view came after 673 ms
```

**That is 16 seconds of waiting for each measurement, and this session made about 30
of them: about eight minutes of nothing.** Every `sleep` after a key holds the same
fault.

**The answer stands in the repository now: `docs/harness/drive.sh`.** Give that file
to your shell, and drive the program with it:

```bash
source docs/harness/drive.sh
start_the_program                      # it comes back at the first frame
press a; wait_for "The authors ["
press v; wait_for "The narrators ["
the_screen | head -20
the_log "\\[authors\\]" 3
stop_the_program
```

Every function polls, and every poll holds a timeout of 30 seconds. A poll that gives
up writes the reason and the first twelve lines of the screen, therefore a
measurement that fails says why.

**The measurement of that file, 2026-08-11:**

| The work | With `sleep` | With `drive.sh` |
|---|---|---|
| The start of the program | 17 s | **0.57 s** |
| A sweep of three views | 27 s | **1.6 s** |

`start_the_program "TOUTUI_EBOOK_CACHE_BYTES=200000"` gives more variables of the
environment to the program, and `SESSION`, `COLUMNS_OF_THE_SCREEN`, and `TIMEOUT`
change the harness. `wait_while` waits while a text stands on the screen: a message
of the program lives six seconds, and a measurement of the message after it must wait
for the first one to go.

### 2. Two tests held 12.7 seconds of the 18 seconds of the run — done (T-74)

| The test | Before | After | The answer |
|---|---|---|---|
| `the_position_survives_a_playback_that_does_not_start` | 8.01 s | **0.01 s** | A clock of its own, `#[tokio::test(start_paused = true)]` |
| `playback_ownership` (two tests) | 7.72 s | **3.0 s** | A poll of the forced sync, and a poll of `is_finished` |

**Each of the two answers fits its test, and they do not exchange.** The first test
makes no request and opens no socket, therefore a clock of its own is safe. The
second test waits for a server (wiremock): a clock of its own would move to the
timeout of a request while that request is still on its way.

**Both still show the fault.** A build with the correction of T-38 removed fails with
"the position went to 0 seconds", and a build with the guard of the identity of the
playback removed fails after the limit of the poll.

**The rule that this work found, and every new test must follow it.** Some sleeps
wait for a state of the engine, and a poll of that state gives the same measurement at
once. **Others wait for a fault to appear**, and a poll of the value that the test
wants is then a **false pass**.

The sleep of 3500 milliseconds of
`the_position_survives_a_playback_that_does_not_start` is of the second kind. The loop
of the playback reads the state one time each second, and the test asks: does that
loop write the **wrong** position of an engine that did not start? A poll of
"the position is the position of the user" answers `true` **before the loop ever
ran**, because the row of the database holds that value already. The test would then
pass with no measurement at all.

The correct answer for such a wait: poll for the evidence that **the loop acted** —
one write of the loop, one sync, or one line of the log — and then read the value. The
loop acts one time each second, therefore a poll of that evidence gives about 1.2
seconds in place of 3.5. **A test that measures the absence of a fault must give the
fault the time to appear.**

### 3. `cargo test` runs the binaries one after the other — done (T-74)

34 files of `tests/` give 34 binaries, and cargo runs them in sequence: the 18 seconds
are almost the sum of their times. **`cargo-nextest` runs every test of every binary
in one pool of processes.** The run would then take the time of the slowest test, of
about 8 seconds, and 6 seconds after the two sleeps above go away.

`cargo-nextest` is a tool of the machine, and **not** a dependency of the program:
`Cargo.toml` does not change, and the rule of T-20 stays. **It stands on this machine
now**, and `.config/nextest.toml` stands in the repository. The two properties that
needed a measurement:

- nextest gives **one process for each test**. Every test of a global slot of the
  process (`logic::message`, `logic::live`, `logic::authors`, the cache of T-72)
  passes, and ten runs one after the other gave 838 of 838 each time.
- The tests of the sandbox need one thread for the rate limit of the login. The group
  `the-sandbox` of `.config/nextest.toml` holds `max-threads = 1` for every binary
  whose name ends with `_against_the_sandbox`, and one run gave 18 of 18.

### 4. The build is not the slow part, and the disk is

`[profile.dev]` holds `debug = "line-tables-only"` since T-64. The build after one
edit takes 6 seconds, therefore a faster linker gives almost nothing: `ld.lld` stands
on this machine, and a measurement of `-C link-arg=-fuse-ld=lld` gave no useful
change. **Do not put a linker in `.cargo/config.toml` of the repository**: a user who
builds with `cargo install --git` and holds no `lld` would then get an error of the
link.

**The lag of the machine comes from the disk.** `target` grew from 2.1 gigabytes to 11
gigabytes inside this session, three times, and each `cargo clean --profile dev` gave
15 gigabytes back. A build of that size writes much, and the machine of the maintainer
answers slowly while it writes. Two answers to measure:

1. `CARGO_TARGET_DIR` on a different disk, or on a `tmpfs` of the memory for the tests.
   **This is the work that stays**, and no session measured it.
2. Fewer binaries: 35 files of `tests/` hold 300 megabytes each. **Some of those files
   stand alone for a reason** — a test that writes `XDG_CONFIG_HOME` or `XDG_DATA_HOME`
   must be alone in its binary (the trap 8 of the harness). Every other file can join a
   file of a group, and each join takes one binary of 300 megabytes away.
   **`cargo-nextest` gives one process for each test, therefore that reason goes away**
   for a run of nextest. A run of `cargo test` still needs the rule, and the workflow
   of the release uses `cargo test`.

## The traps that cost time

### Of the program and of the server

1. **`ALSA_CONFIG_PATH=/dev/null` stops the real program.** It is correct for
   `cargo test`, because no test opens a sound device. A real run writes "The pool
   has 1 address(es)" and then draws nothing. Give it a real file:
   ```
   </usr/share/alsa/alsa.conf>
   pcm.!default { type null }
   ctl.!default { type null }
   ```
2. **The device `null` plays a book of 30 minutes in two or three seconds.** A test
   of a view that needs a media that plays must press the key inside that time. One
   write of two keys does the work: `l` starts the playback and the space pauses it.
   **A key that comes too early goes to the playback that came before it**: the
   fallback of T-53 starts a second playback about 100 milliseconds after the first
   one.
3. **A playback of a few seconds leaves no session on the server.** The server drops
   a session whose `timeListening` is 0, and the sync period is ten seconds.
   **Read `podman logs abs-test` instead:** the server writes "Starting session for
   user ..." for each `POST /api/items/:id/play`.
4. **`currentTime` comes as a text, and not as a number,** in
   `GET /api/me/progress/:id`.
5. **`PATCH /api/me/progress/:id` with `isFinished` false writes the position back to
   the start, and it ignores every other field of the same request.** A body of
   `{"isFinished":false,"progress":0.5,"currentTime":900}` gives `200`, and the record
   then holds `progress` 0. A media of progress 0 stands on no shelf of Continue
   Listening. **Two requests do the work.** See T-66.
6. **A 404 of a part of a stream is not a media that is absent.** `classify_status`
   gives "The server does not have this item" for a 404, and that sentence is false
   for a part of a stream: ffmpeg writes a part when it made that part. **A stream
   that the server deleted answers 404 for ever**, and one request of the playlist
   tells the two conditions apart. See T-68.
7. **The place of the transcode of the server comes from the position of the user**,
   and **not** from the part that the client asks for. Its log says "Starting Stream
   at startTime 4:52 (User startTime 5:22)", and it takes 30 seconds of that position
   as a pre-roll. A client that moves the part it asks for changes nothing: the
   program must write the position first. See T-69.
8. **A message of the screen lives six seconds** (`message::LIFE`). A capture of the
   screen after that time gives no message, and the program is correct. See T-71.
9. **A sentence of an error of ffmpeg is not a measurement of the whole file.**
   "Not yet implemented in FFmpeg, patches welcome" came 195 times in 20 seconds of a
   file of xHE-AAC, and ffmpeg still gave 46.2 seconds of audio of 60. **Measure the
   output**, and not the lines of the fault: `ffmpeg -i <file> -f wav out.wav` and
   then `ffprobe` of the length. See T-68.
10. **A stream of the server does not open in some milliseconds.** ffmpeg of the
   server makes the first part, and a copy that fails costs ten seconds more. Every
   wait of a stream must hold that time, and `WAIT_FOR_A_FAULT` of 2500 milliseconds
   held it for a **file** only. See T-68.
11. **The row of the message of the screen holds one line.** A message of 200 letters
   loses its end in a terminal of 160 columns. Hold a message at 150 letters or
   fewer. See T-68.
12. **The server takes a name of a field that does not exist.** `?sort=bogus.field`
   gives `200` and an unspecified sequence. Measure a field before you offer it.
13. **`items` of `GET /api/me/listening-stats` is a map, and not a list.**
14. **`GET /api/podcasts/:id/checknew` gives an empty list for a podcast that came one
   second before.** It compares with the time of the last examination.
15. **`convert_seconds` rounds to the minute.** It is wrong for a place in a media.
   Use `clock` of `src/utils/convert_seconds.rs`.
16. **`topGenres` of `GET /api/stats/year/:year` names its value `genre`**, and
    `topAuthors` and `topNarrators` name it `name`.
17. **The lists of the narrators and of the genres stay empty until a session comes
    after the metadata.** The server keeps a copy of the metadata inside each
    session.
18. **The first page of `GET /api/me/listening-sessions` is the page 0**, and a page
    after the last page gives `200` and an empty list.
19. **`quick-xml` 0.41 gives an entity as its own event `GeneralRef`**, and not
    inside the text. A reference makes no text node of the tree of a web page,
    therefore `cfi::Walk` must not count it as one.
20. **A comparison of two lists of texts must not read the text.** A book holds the
    word "very" two hundred times. Read the two lists together, in the sequence of
    the document.
21. **`GET /api/libraries/:id/filterdata` holds no tag.** `GET /api/tags` gives them,
    and a filter of `tags.<base64>` works. A scan of the library changes nothing.
    See T-60.
22. **The rate limit of the login is 40 requests of 600 seconds.** A run of every
    test of the sandbox reaches it, and the test then says "the answer must hold a
    token". Read `podman logs abs-test` for the line of the rate limiter, and run
    those tests with `--test-threads=1`.
23. **A test that changes data must write a value that differs.** The test of the
    live messages wrote the same subtitle at every run: the server saw no change at
    the second run, therefore it sent no message and the test waited for nothing.
24. **A value of the state that belongs to one playback must name that playback.**
    The engine clears the name of a file with no decoder when it starts a playback,
    and the command of the start is not immediate. `playback_of_the_fault` holds the
    identity. See T-53.
25. **`reqwest::blocking` stops the program inside a task of tokio.** That client
    makes a runtime of its own, and a runtime that goes away inside an asynchronous
    context gives "Cannot drop a runtime in a context where blocking is not
    allowed". The engine is a thread, therefore the real program is correct. **A
    test of such a reader must use `std::thread::spawn`.**
26. **A value that comes from the view of the user must come before the view
    changes.** `open_the_ebook` read the title of the media after it set the view to
    the reader, and the answer was then always nothing.
27. **One media stands on more than one shelf of the Home view.** A list that names a
    media by its identity therefore changes every line of that media. **The number of
    the line is the key of a rule of one shelf.** See T-66.
28. **A view key works in the Home view, in the Library view, and in the view of the
    search only.** A sweep that presses `a`, `v`, `c`, `f`, or `s` inside the view of
    the series gets nothing, and the program is correct: `show_the_names` and the keys
    `s` and `c` hold that rule. **A sweep must come back to the Library between two
    views.** See T-75.
29. **`Wrap` of ratatui takes a space away at the start of a line that it makes, and
    it keeps every space that stands inside a line.** A text of the source that an
    old wrap left with a run of 22 spaces reaches the screen as it stands. See T-75.
30. **The server holds one place for each media (`ebookLocation`), and not one place
    for each file.** An item can hold more than one ebook, therefore a reader of a
    second book must neither read that place nor write it. See T-76.
31. **The flag of the forced sync holds one identity for the whole process.** Two
    tests of one binary that ask for a sync at the same time take the flag from each
    other. Give each test its own identity of a playback, and ask again at each step
    of the poll. See T-74.
32. **A `Paragraph` of ratatui gives its style to every cell of its area, and it
    writes its own text only.** Every letter that stood on those cells stays.
    `Clear` takes them away. The row of the message met this in the reader, and
    no list view showed it: that row holds no letter there. See T-78.
33. **A key that does nothing in one view of fifteen is a fault of its own.** The
    user learns `h` in every other view. **Press every key of a view in the
    sweep**, and not the key that opens it only. See T-79.
34. **The key `t` of the timer of sleep is not a view.** It goes to the next value
    (5, 10, 15, 30, 45, and 60 minutes, the end of the chapter, and off) and it
    writes a message. A sweep that waits for a view waits for nothing.
35. **The contents of the reader close with `l` too.** The key `t` opens them and
    it closes them, and `l` goes to the chapter and closes them. A sweep that
    presses `t` after `l` therefore **opens** them again.
36. **A status of 400 is a fault of the request, and not of the address.** The
    server answers 400 for a book that stands in a collection already, for an
    episode that stands in a playlist already, and for a podcast whose directory
    exists. **Read `is_endpoint_fault` before you add a status to it**: the pool
    marks the address down for such a fault, and a pool of one address then has
    no address at all. See T-87.
37. **The queue of the downloads of the server does not fill at once.**
    `POST /api/podcasts/:id/download-episodes` answers `200`, and a read two
    seconds later gave an empty queue. Poll it. See T-81.
38. **`GET /api/podcasts/:id/clear-queue` does not stop the episode that
    downloads now.** The server holds that episode outside the queue, and
    `currentDownload` names it. See T-81.
39. **A message of the program hides the last row of the view below it.** The row
    of the message stands above the footer, and `Clear` takes it for six seconds
    (T-78). A list that fills the whole view therefore loses its last line while
    a message lives. The line comes back with no key.
40. **A key that writes a value of the server must ask for the lines after that
    write, and not beside it.** A question that goes with the write gives the
    state of the moment before the key. See T-84.
41. **The reader keeps the book of the session while the user reads it.** The key `h`
    and a second `e` give the book with no call of `get_the_ebook`, therefore the
    time of the file does not change inside one run of the program. A measurement of
    the cache needs a second **run**. See T-67.
42. **`POST /api/collections` refuses a collection with no book.** It answers
    `400` with "Invalid collection data. No books". `POST /api/playlists` with no
    item gives `200` and an empty playlist. **The two lists do not behave in the
    same way**, therefore a view that makes a collection must hold a media. Both
    refuse a name of no letter with `400`. See T-88.
43. **The server takes two lists of one name.** It answers `200` and it gives the
    second list its own identity. The user then reads two lines that say the same
    words, and no key tells them apart. **The program refuses that name before the
    request.** See T-88.
44. **A box that makes a `Terminal` of its own writes on the cells of the view,
    and the terminal of the program does not know it.** ratatui writes the cells
    that changed only, and it compares with its own buffer. The letters of the
    view stay on the screen until a key makes the program write those rows again.
    **`Clear` over the whole rows, and then `terminal.clear()` of the loop.** See
    T-89, and T-42 for the key `R`.
45. **The area of the footer holds two rows, and a `Paragraph` with no `Wrap`
    writes one.** Every footer of more than 80 letters lost its end in a terminal
    of 80 columns, and the keys `?` and `Q` stand at the end of every one of
    them. See T-90.
46. **A view must not give a reason that the program does not have.** With no
    answer from the server, three views said "This library has no …". No request
    gave an answer, therefore the program knows nothing of that library.
    **`App::is_offline` says which of the two sentences the view takes.** See
    T-91.
47. **The status of a login that failed holds the reason.** The old code made one
    error for every status. **429 is the status that costs the most time**: the
    rate limit of 40 requests of 600 seconds. See T-92 and the trap 22.
48. **`PATCH` of a list does not examine the name, and `POST` of a list does.**
    A `PATCH` of a collection with a name of no letter gives `200` and a
    collection with no name; the same request of a playlist keeps the old name.
    **A rule of a name belongs to the program.** See T-93.
49. **`DELETE` of a list that went away gives `404`.** A second key of the user
    and a different client both make that condition. See T-93.
50. **A `Paragraph` with no `Wrap` writes one row for each line of its text.**
    The row of the item holds three rows, and 21 paragraphs of the views wrote
    on one of them: a terminal of 80 columns lost the end of every long line.
    **Every text of a view that can be long needs `Wrap`.** See T-94 and T-90.
51. **A test that holds the answer of the code keeps the fault of the code.**
    The test of the title of the search asked for "Search result [1 items]" and
    it passed for two years of releases. **Write the rule of the user in the
    test**, and not the words that the program gives today. See T-95.
52. **A request that stops at its time limit is not evidence that the address is
    down.** The server reads a web site for a feed and every file for a scan.
    Two such requests, one after the other, give the state `Down` now, and one
    answer of the address forgets them. See T-97.
53. **`ApiError::Timeout` reaches the caller only when a second address exists.**
    The old `send` gave `Unreachable` for a pool of one address, therefore the
    user read "No server address answered" for a server that answered slowly.
    See T-97.
54. **The key Esc quits the program from every view.** The key `h` goes back. A
    sweep that presses Esc to leave a view kills the program, and the next
    `wait_for` then says "no server running on /tmp/tmux-1000/default".
55. **A `Paragraph` of ratatui breaks a line that is too long, and it never
    joins two lines.** A text that holds the wrap of its source therefore gives
    a narrow column in a wide terminal. **One item of a list is one line**, and
    `Wrap { trim: true }` gives it the width of the panel. See T-101.
56. **A text that names the version of the build is not a text that names the
    release.** The newest entry of the changelog took `CARGO_PKG_VERSION`, and
    the screen said "Changelog Toutui v0.7.46" above the words of v0.6.9: the
    reader of that screen therefore had no way to see that 38 releases were
    absent. See T-101.
57. **`PATCH` of a list with a body that is not complete.** `items` of a playlist
    that holds one media fewer gives `400` and "Invalid playlist items. Length
    mismatch"; `books` of a collection gives `200`, **and the book that the body
    does not name goes to the first line**. A body of the sequence therefore
    holds every media. See T-102.
58. **`handle_key` of `src/app.rs` reads `key.code` only.** A key of Ctrl or of
    Alt reaches the handler as the letter itself, therefore a new key with a
    modifier needs work in that function first. The keys `<` and `>` of T-102
    need none.
59. **A key that writes a sequence must move the screen at once.** A user presses
    the key more than one time, and a screen that waits for the answer of the
    server between two keys moves the wrong line. See T-102 and the trap 40.
60. **A title of "[0 items]" is not a sentence.** The Home view and the Library
    view of a library with no media drew an empty list and no word. **Every view
    that can hold no line needs its own sentence**, and
    `App::render_the_reason` of `src/ui/tui.rs` holds the shape. See T-103.
61. **The program starts in the Library view when the Home view holds no line**
    (`src/app.rs`, near the line 939). A measurement of the Home view of an empty
    library must press `Tab` first. See T-103.
62. **A rule of a split is a rule of the screen, and not of an area.** The area of
    the work of a view grows by the 6 rows of the player while nothing plays
    (T-104), and the threshold of 12 rows of T-99 then gave a half of that area to
    the description: the list of a screen of 20 rows lost one line. Take the rows
    that the player left away before you compare. See T-104.

63. **The header of the screen named the address of the **login**, and the pool
    moves between the addresses at every request.** A sweep of two addresses read
    `localhost:13399` in the header while every request went to
    `127.0.0.1:13456`. `render_header` reads `pool.active()` now. See T-105.
64. **`App::is_offline` comes from `App::new` only.** The start of the program
    and the key `R` are the two moments that give it a value, therefore the
    screen said "Connected" for as long as the user pressed no key. **The pool
    knows more, and it knows it earlier.** See T-107.
65. **No request of the program fails while the user presses no key.** The pool
    therefore learned nothing of a server that went away. The task of the live
    messages tries a connection every few seconds, and it marks the address down
    for one fault only: a connection that no machine takes. See T-107.
66. **A rule of a text that reads one form finds one fault of many.** The guard
    of "1 item" read `{} items` of four files, and it did not find `{} item(s)`,
    `{} sessions`, `{} book(s)`, or `{count} files`. A guard reads **every** file
    of `src/ui` and of `src/logic` now. See T-108.
67. **`total` of the library changes with `?collapseseries=1`.** With no
    parameter it counts the items (14 in the sandbox), and with the parameter it
    counts the **lines** (10). A measurement of the paging that reads `total`
    must therefore come after the decision of that parameter. See T-70 and T-110.
68. **`GET /api/podcasts/:id/checknew` compares with the time of the last
    examination, and not with the episodes that the server holds.** A podcast of
    3 episodes of a feed of 57 gives **0**. It is cheaper only where it is wrong.
    See T-110 and the trap 14.
69. **`std::env::current_exe()` is the binary of a test inside a test.** A
    program that spawns itself must know that the program of the user runs:
    `main` writes that value, and a test and every program that takes this
    library read the book in their own process. See T-62 and T-111.
70. **A `Paragraph` of ratatui needs an area, and an area of no work is an area
    that a session forgets.** The view of the accounts held `_item_area` and it
    drew nothing there for a year of releases. Look for a name that starts with
    an underscore before you say that a view has no room. See T-110.
71. **The end of the lines that the program holds is not the end of the
    library.** The program reads one page of 500 items (T-70), therefore a key
    that means "the end" must read the pages that are left. A user of a library
    of 2056 items pressed `G` **six** times, and each press asked for one page.
    See T-112.
72. **A view that reads the lists of the library shows the media of the pages
    that came, and no other media.** The view of the search mapped every
    identity of the answer to a place in `ids_library`: the server found a book
    of the page 4, and the screen said "The server found nothing". **The answer
    of the server carries the whole media**, therefore such a view needs no
    request more. See T-113.
73. **A library of podcasts answers a search with the group `podcast`**, and a
    library of books with `book`, `series`, `authors`, and `narrators`. A program
    that reads the groups of the books only says that the server found nothing
    for **every** search of a library of podcasts. The group `episodes` exists
    too, and no measurement of the sandbox gave one hit of it. See T-113.
74. **An answer that came stops the work that the program does itself.** The view
    of the search shows the titles of the program while it waits, and it shows
    the answer of the server when that answer comes: an answer of nothing
    therefore takes the lines of the program away too. Measure both moments.
    See T-113.
75. **A text of no letter is not a value.** The server gives `authorName: ""` for
    a book with no tag of an author, and `publishedYear: null` for the year of
    the same book: the line of the Library view said `Author:  - Year: N/A`.
    **`null` is not the only shape of a value that is absent**, and
    `utils::values_of_the_server` holds the rule for every collector. See T-114.
76. **A box that makes a `Terminal` of its own must read the size at each turn of
    its loop.** `search_active` and `ask_for_a_text` read `term.size()` before
    the loop: a terminal that became 80 by 24 while the box stood at the row 41
    gave the user **an empty screen**, and every letter that they wrote went to a
    box that they could not see. ratatui writes no cell of an area that stands
    outside its buffer. See T-115.
77. **Three paragraphs of one area write on each other.** The header holds the
    account at the left, the library in the middle, and the name of the program
    at the right, over **one** area: 60 columns gave
    "👋 Connected as toutuitestBooks (book)". A part that is too long meets its
    neighbour, and no letter of the two goes away. See T-115 and the trap 32.
78. **`response.bytes()` holds the whole answer in the memory of the program.**
    A book of a scan of 502 megabytes gave the program of the user a peak of
    **1007** megabytes, because the buffer grows by a copy of itself. The loop of
    `response.chunk()` gives 44 megabytes, and `logic::download::fetch` of a media
    of the disk held that shape already. **T-62 moved the parse of a PDF into a
    child for this reason, and the download stayed inside.** See T-116.
79. **A PDF holds no title, therefore the reader takes the title of the view.**
    `selected_item_title` gave nothing for the view of the search, and the reader
    then said the identity of the item: `27c55369-… — page 1 of 150 — 0%`. A view
    that gains a list of the titles must give it to that function. See T-117 and
    T-54.
80. **The program holds one account, and the words of three places said more.**
    The view of the login comes only when the database holds no default account
    (`main.rs`), the view of the accounts lists `database.default_usr` alone, and
    every login writes `is_default_usr = true`: with two such rows the **rowid**
    decides which account starts. **A text must not promise a function that no
    key reaches.** See T-118.

### The traps of the session of the accounts and of the three sweeps

81. **A library of podcasts costs one request for each podcast, and the program
    held nine lists of one row for each of them.** A page after the first gave
    those lists no row at all: `self.all_ids_pod_ep[index]` of the line 519 of a
    list of 500 rows **stopped the program**, and the panic never reached the
    log. Every list of that view takes `get` now (T-41), and the program reads
    the episodes of one podcast when the user opens it. See T-126.
82. **`GET /api/me` holds `mediaProgress` for every media of the account.** The
    start asked `GET /api/me/progress/:id` for each media of the Home view: 29
    requests of a server of 500 milliseconds cost 2.1 seconds of a start of 3.8.
    **A media that the answer does not name played never**, therefore it needs no
    request at all. See T-127.
83. **`ebookProgress` of the server is a fraction, and not a whole number.** The
    program read `i64`, therefore the whole answer of a book that the user read
    did not read and the line said "N/A" for a book of 92 percent. **A row that
    the program cannot read must not take the other rows away**: each row of the
    account reads by itself now. See T-127 and T-41.
84. **A second start of the terminal inside one process does not work** (T-123),
    therefore **every key that changes the account starts the program again**.
    `start_the_program_again_with` gives the new program the variables that the
    key needs: the request of the login screen, the address of the login, or
    nothing at all. See T-124.
85. **A test that takes "the first item of a kind" breaks when the sandbox gains
    data.** `the_place_of_the_ebook_against_the_sandbox` took the first EPUB of
    the library, and the three books whose EPUB is not valid of the sweep of this
    session stand before Alice in the alphabet. **A test names the data that it
    needs.** See T-127.
86. **`pkill -f <a text>` kills the shell of the harness** (the trap 15 below),
    and this session met it again: read the identity of the process with `pgrep`,
    and give that number to `kill`.
87. **The key `BTab` of tmux is Shift+Tab.** `send-keys S-Tab` writes the letters
    of that text, and the library of the program then does not change: a sweep
    that presses Shift+Tab must use `press BTab`.
88. **A proxy of Python of 60 lines is the whole measurement of a slow server.**
    `python3 slow.py <the port of the proxy> <the port of the server> <the delay
    in ms>`, and a block `[[servers]]` of `config.toml` puts that address first
    (the trap 68). **The box of the start says the step of the start**, therefore
    a poll of the screen every 50 milliseconds gives the sequence of the requests
    with no line of code.

### Of the harness and of the machine

1. **A fixed `sleep` is the largest waste of a session.** The first frame of the
   program comes after **673 milliseconds**, and this session slept 17 seconds before
   each of about 30 measurements. **Poll for a marker of the screen, with a timeout.**
   See the section of the harness.
2. **A screen of your own is not a terminal. Use tmux.** A hand-written model of the
    screen mangled the line that the user selected, and it kept the old text of a
    line that the program wrote again. Both times the program was correct.
    ```
    tmux new-session -d -s check -x 160 -y 45 "<the program>"
    sleep 9; tmux capture-pane -p -t check        # -e keeps the colours
    ```
3. **A frame of the program comes by itself.** A measurement of a live message needs
    **no key at all**: the three frames of T-66 came with no key press. A key press
    inside the measurement moves the selection, and it then looks like a fault of the
    rule of the selection.
4. **A pseudo terminal must answer TWO questions**, and not one: `ESC [ 5 n` with
    `ESC [ 0 n` for `ratatui-image`, and `ESC [ 6 n` with `ESC [ 1 ; 1 R` for
    crossterm inside `terminal.clear()`. With no answer the program stops with "The
    cursor position could not be read within a normal duration", and that message
    never reaches the log file.
5. **ratatui writes the cells that changed only.** A change of one number therefore
    gives two bytes in the stream of the terminal. A move of the list (`j` and then
    `k`) makes the program write whole lines again.
6. **A key of a login that comes too early goes to the application.** The login
    examines the address with a request, therefore the next field is not ready at
    once: the password `claudetmp` went to the application as the keys `c`, `l`,
    `a`, `u`, `d`, `e`, and `e` opened the reader. **Log in one time, and then reuse
    the database of that isolated `XDG_CONFIG_HOME`.**
7. **An isolated `XDG_CONFIG_HOME` needs two files before the program starts.**
    `config.toml`, and `.env` with `TOUTUI_SECRET_KEY=<something>`.
8. **A test that sets `XDG_CONFIG_HOME` or `XDG_DATA_HOME` must be alone in its
    binary**, and every test of a global slot must stand in one function.
    `logic::message` gave a fault of one run of three before its two tests became
    one, and `tests/the_cache_of_the_ebooks.rs` holds one function for that reason.
9. **`target/debug` grew to 221 gigabytes and the disk of the maintainer became
    full.** A test binary holds 300 megabytes, `cargo test` makes 37 of them, and
    cargo keeps the binary of every build that came before. Look at `du -sh target`,
    and run `cargo clean --profile dev` at the end of a session. See T-64.
10. **The container of the sandbox lives longer than the session.** `podman ps -a`
    with `| head` did not show it, and `podman run` then said "the container name is
    already in use". **`podman start abs-test` gives the server back with every book
    of the session before**, therefore no session needs to make the data again.
11. **`ALSA_CONFIG_PATH` with a null device does not silence the real program.** It
    is correct for `cargo test`, because no test opens a sound device. A run of
    2026-08-11 played real sound through the sound card of the maintainer with that
    variable set, and the sound was **not smooth**: a half of a configuration of ALSA
    is worse than none. **Ask the user before a measurement that plays**, and use the
    real device: a null device hides the faults of the real path.
12. **A transcode of the server that dies leaves the server in a bad state.** Every
    new session of that media then answers "No Segments", and the log holds "Failed
    checking files" every two seconds for ever. `podman restart abs-test` is the
    answer, and a measurement of such a media must start from a server that came up
    now. See T-68.
13. **`TOUTUI_AUDIO_DEVICE=null` gives the null device of ALSA**, and the log of
    the program then says "[worker] the application uses the sound device
    alsa:null". **This is the way to measure a playback with no sound**, and
    `ALSA_CONFIG_PATH` is not (the trap 11). `docs/TEST-SERVER.md` names it.
14. **The null device plays fast, therefore a media of 30 minutes comes to its
    end in about 50 seconds.** A measurement of a playback that must live longer
    needs a pause, and a measurement of the end of a media needs that time.
15. **`pkill -f <a text>` kills the shell of the harness.** The command line of
    that shell holds the text of the script, therefore the pattern finds it.
    **Write the identity of the process in a file** and give that identity to
    `kill`.
16. **`pgrep -f "target/debug/toutui"` finds the client of tmux too.** That
    process holds the whole command of `tmux new-session`. Take
    `pgrep -n -f` for the newest process, and read
    `/proc/<the process>/cmdline` before you measure its memory: a measurement of
    the wrong process gave 5.9 megabytes for a program that held 128.
17. **A large PDF for a measurement needs samples that no algorithm makes
    smaller.** `convert -size 1200x1600 -depth 8 rgb:<random bytes> big.jpg` and
    then `img2pdf` of 60 copies give a file of 47 megabytes.
18. **`/usr/bin/time` stands on no machine of this session.** Read
    `resource.getrusage(RUSAGE_CHILDREN)` of Python for the peak of a child, and
    `/proc/<the process>/status` (`VmHWM` and `VmRSS`) for a program that runs.
19. **`playwright` wants its own browser:**
    `play.chromium.launch(executable_path="/usr/bin/chromium")`.
20. **The sweep of the view of the login uses the rate limit of the login.** Each
    attempt is one `POST /login` of the 40 of 600 seconds, and the tests of the
    sandbox need that limit too. **Run the sweep, and then `podman restart
    abs-test` before the tests.** See T-92.
21. **The offline mode needs no new configuration**: `podman stop abs-test`, and
    the program starts in that mode with the media of the disk. The header says
    "Offline". `podman start abs-test` gives the server back. See T-91.
22. **A sweep of the login needs a `XDG_CONFIG_HOME` with no database.** Two
    files make it (the trap 7), and the program then draws the field of the
    address. A second sweep needs the database of that directory removed again.
23. **A measurement of a disk must write data that no algorithm makes smaller.**
    `dd if=/dev/zero` gave 7.4 GB/s on the ZFS of the maintainer, because
    `compression=on` makes a file of zeros almost nothing. The same measurement
    with a file of `/dev/urandom` gave 1.4 GB/s. See T-98.
24. **A second `XDG_DATA_HOME` gives a cache of the ebooks that is empty.**
    `start_the_program "XDG_DATA_HOME=<a new directory>"` gives the variable a
    second time, and the last value wins. A measurement of a book that the cache
    holds already needs it.
25. **The disk of the maintainer is not the slow part of a build.** ZFS gives
    1.4 GB/s of data that no algorithm makes smaller, and 3000 small files take
    1140 ms against 953 ms of the memory. **A `tmpfs` for `target` gives some
    seconds and it takes 11 gigabytes of the memory.** The processor makes the
    machine slow, and `nice -n 19 ionice -c 3` is the answer. See T-98.
26. **`tmux resize-window` changes the size of the terminal of a program that
    runs.** `tmux resize-window -t <the session> -x 80 -y 24` is the whole
    measurement of a resize, and the program answers it with no key. See T-115.
27. **A measurement of the memory of a client needs a server outside the
    process.** A mock server of `wiremock` makes its answer in the memory of the
    test: a download of 96 megabytes gave 192 megabytes of `VmHWM` with the loop
    of the parts **and** with `response.bytes()`. The memory of the answer of the
    server hides the memory of the client. **Read the real program**, and hold
    the rule of the code with a test that reads the source. See T-116.
28. **A library that a `POST` makes examines nothing.** `total` of the library of
    2056 items stayed 0 for four minutes, and one
    `POST /api/libraries/:id/scan` then read every item in 50 seconds. Poll
    `total`, and give the scan yourself. See T-112.
29. **`tar` through `podman exec` writes many files in one command.**
    2056 directories of the library of the measurement:
    `tar cf - . | podman exec -i abs-test tar xf - -C /largebooks`. A volume of
    its own would need a new container. See T-112.
30. **The view of the login of a fresh `XDG_CONFIG_HOME` draws its box at once**,
    and the box stands at the row 22 of a screen of 45. The three fields come one
    after the other, and each of them holds its own marker for a poll:
    "Server address", "Username", "Password". **The login examines the address
    with a request**, therefore a key that comes too early goes to the
    application (the trap 6). See T-118.
31. **A second account of a second server needed an editor of SQLite before
    T-124.** The program showed the view of the login only when the database held
    no default account. **The key `a` of the view of the accounts makes that
    account now**, and the key `c` gives the start to a different one: a sweep of
    two accounts needs no SQL at all. See T-124 and T-118.

### The traps of the session of the e-reader (T-119 to T-121)

60. **Every route of `/api/emails/` of the server holds an `adminMiddleware`.**
    `GET /api/emails/settings` answers **`404`** for an account that is not an
    administrator, and that account **can** send a book to an e-reader.
    `POST /api/authorize` gives `ereaderDevices` filtered for the account of the
    token, and the login gives the same payload. See T-119.
61. **`POST /api/emails/send-ebook-to-device` answers `404` for three different
    conditions**, and the body of the answer is the one place that tells them
    apart: "Ereader device not found", "Library item not found", and "Ebook file
    not found". Every body is plain text, and not JSON. See T-119.
62. **The send of a book is slow work of the server, and `REQUEST_TIMEOUT` is 15
    seconds.** The server took 36.2 seconds for a book of 479.5 megabytes, about
    13 megabytes each second. A book of more than about 200 megabytes therefore
    stopped at the time limit **while the server sent it**, and a second such
    request marks the address down (T-97). See T-119.
63. **The engine writes one flag for two conditions of a decoder.** A track that
    the playback needs **now** does not open, and the playback is then dead; or
    the engine skipped a **later** track, and the playback plays. **Read
    `state.playback_id` before you read the flag**: `worker.rs` writes that
    identity in the loop that follows a playback that plays, and a start that
    failed never reaches that loop. See T-120.
64. **`/api/items/:id/file/:ino/download` needs the permission `download`, and
    `/api/items/:id/file/:ino` does not.** The second address is the value of
    `contentUrl` that the server gives for each track. A measurement with an
    account of the type `user`: `206` for the one, and `403` for the other. See
    T-121.
65. **A measurement of a permission needs an account that does not hold it.**
    Every session before 2026-08-12 used `toutuitest` of the sandbox, and that
    account is `root`: T-121 lived in the program for as long as the fork, and no
    test of the sandbox met it. `docs/TEST-SERVER.md` holds the commands of an
    account of the type `user`.
66. **`POST /api/users` makes an account that is not active.** The login then
    answers `401`, and `podman logs abs-test` says "User is not active". One
    `PATCH /api/users/:id` with `{"isActive":true}` gives the account its work.
    **The field `token` of `GET /api/users` is not a token of a request** either:
    every request with it answers `401`. Take the token from `POST /login`.
67. **A null device of audio plays about 60 seconds of a book in one second.** A
    measurement of a position must read the log of the program, and not the row
    of the player after a wait: the row raced from 1:02 to 1:43 in twelve
    seconds. **The local sessions of the program carry that race to the server**
    at the next start, therefore a clean measurement of a resume removes the rows
    of `listening_session` and of `pending_progress` of the database of the
    program first. See T-120.
68. **The pool of the program takes a slow address with a block `[[servers]]` of
    `config.toml`.** The block holds for a server whose endpoints hold the stored
    address, therefore the slow address stands first and the real address after
    it. A proxy of 60 lines of Python gives every request a delay, and the header
    of the program then says the slow address.

## The shapes that the next work should follow

- **A slot between the work and the screen.** The render is not asynchronous.
  `logic::live`, `logic::stats`, `logic::bookmarks`, `logic::authors`, and
  `logic::message` all hold that shape: a task or a key writes, and the render takes
  it at the next frame. A function that needs no `&mut App` reaches every caller.
- **The render may change the state, and it must do that work one time.**
  `take_the_media_that_left_away` of T-66 compares two small lists at every frame,
  and it makes the lines again at a change only. `Widget for &mut App` gives the
  frame that permission.
- **A rule that needs no request is better than a request.** T-66 holds every line of
  the shelf already, therefore the program never asks the server after a live
  message. The sync of the playback of the program makes one such message every ten
  seconds, and each costs one comparison.
- **A view says why it holds no line. A message of one row says nothing.** The view
  of the bookmarks held the right shape, and the view of the chapters holds it now.
- **A rule of a loop belongs in a pure function with a name.**
  `the_fault_stops_the_playback`, `the_queue_goes_on`, `wait_after_the_faults`,
  `is_for_the_screen`, `without_the_media_that_left`, and `the_ebooks_that_must_go`
  are small and pure, and a test holds each of them to the measurement that made it.
- **A pure function with a test can still hold a wrong key.** Eight tests of
  `without_the_media_that_left` passed while the caller named the media by its
  identity. **The shape of the real data found it.** Make the data of two shelves
  exist, and then look at the screen.
- **A test may read the source of the program.**
  `every_key_of_the_handler_stands_in_the_list` reads `src/app.rs` with
  `include_str!` and it finds every key of the handler in the list of the keys.
- **Measure the dependency before you accept that you need one.** socket.io needed
  none: the second transport of that protocol is plain HTTP. Every codec of the
  server needed none: the stream of HLS is a playlist of text and packets of 188
  bytes. The time of the last use of a file needed none: `std::fs::FileTimes`.
- **Show the fault with the old binary.** `git checkout -- <the file>` and a build in
  a different `CARGO_TARGET_DIR` give the old program beside the new one.
- **Make the data of the fault in the sandbox.** A book of one MP3 file and one WMA
  file gives the shape of a book that no decoder reads. `docs/TEST-SERVER.md` holds
  the commands of that book and of a PDF with pictures.
- **A message of the server can hold a secret.** `user_updated` carries a new token
  of the user. The log holds the name of a message, and never the body.
- **A sweep of every view finds what a test does not.** One run in tmux pressed the
  key of each view and counted the lines of text of each screen. Two views of fifteen
  answered with nothing, and one of them was a real fault. The sweep of 2026-08-11
  read the **text** of each screen too, and it found two texts that were wrong (T-75).
- **A test may hold every text of the views to one rule.**
  `a_text_of_a_view_holds_no_run_of_spaces` reads a list of constants, and a new text
  of a view joins that list. A rule with no list is a rule that the next text breaks.
  **A rule that no list can hold reads the source**:
  `a_text_of_a_view_names_a_key_with_no_quotation_mark` reads `src/ui/tui.rs` with
  `include_str!` and it finds every text that says "Press '". See T-91.
- **Measure the request before you draw the screen that sends it.** The first
  shape of T-88 was a view that asks for a name and a library, and the server
  refuses a collection with no book. Four minutes of `curl` moved the whole work
  into a view that exists already.
- **A test of a view must hold the two conditions of the server.** The test of the
  view of T-88 makes an `App` with an address that answers nothing, therefore that
  application is in the **offline mode**. The screen of that mode says a different
  sentence (T-91), and the test measures the two conditions one after the other.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified technical
  English. Short sentences, active voice, one instruction per sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys` must find
  nothing. `libsqlite3-sys` and `ring` are the two known builds of C, and the
  decision of 2026-08-11 keeps them.
- No test may need the network. A test that needs the sandbox carries `#[ignore]`
  and says how to run it. Seventeen such files exist now, and no test holds a path of
  this machine.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited in
  the README, in the LICENSE, in `Cargo.toml`, and in the settings screen. **`gh`
  resolves to that repository by default in this clone**, therefore give
  `-R ealtun21/Toutui` to every `gh` command.
- Show a fault before you correct it.
- Tag, push, and go on. Do not wait for continuous integration.
- **The release builds `--locked`.** A bump of the version must hold the new
  `Cargo.lock` in the same commit: run `cargo build` after the bump, and see that
  `git status` is clean before the commit and the tag. The tag of v0.7.22 held
  `Cargo.toml` 0.7.22 and `Cargo.lock` 0.7.21, and it had to move.
- **A release holds three files: `Cargo.toml`, `Cargo.lock`, and the entry of the
  changelog.** `THE_ENTRIES_OF_THE_FORK` of `src/utils/changelog.rs` takes the
  new entry at the top, in the words of a user. The gate fails without it, and
  that is the rule of T-101.
- **Run every cargo command under `nice -n 19 ionice -c 3` with `-j 16`.** The
  machine has 32 cores, and the user tests the program while the tests build.
- The address of the server of the user must stay outside this repository, and the
  account of the user too.

## The prompt for the next session

**Every sweep of the road of 2026-08-12 is made**, and T-118 is done. This prompt
asks for the measurement that did not repeat, the requests of the start that are
left, and the sweeps that no session has made.

> Continue the Toutui takeover. Repo: `/home/nyverino/Documents/Toutui`
> (ealtun21/Toutui, branch main). Maintained fork of the archived
> AlbanDAVID/Toutui. Newest release **v0.7.67**; `Cargo.toml` is at 0.7.67. The
> workflow refuses a tag that disagrees with `Cargo.toml`, **and it builds
> `--locked`**. **A release holds three files together**: `Cargo.toml`,
> `Cargo.lock`, and one new entry at the top of `THE_ENTRIES_OF_THE_FORK` of
> `src/utils/changelog.rs`. The gate fails without that entry (T-101).
>
> **Read before you touch code:** `docs/HANDOVER.md` (the state, the decisions,
> the road, and the traps that cost real time), `docs/TAKEOVER-BACKLOG.md` (the
> evidence of every item; **T-87, T-97, T-107, T-120, and T-126 are the five to
> know**, and T-124 to T-127 are the newest), and `docs/T-24-coverage.md` (**no
> row of section 4 says `Half`, and every row that says `No` belongs to an
> administrator of the server**, and **section 6 names what the program must not
> have, with the reason**).
>
> **The way of working, for every item.** Show the fault before you correct it,
> and let a test find it: a build with the correction removed must fail. Make the
> data of the fault exist in the sandbox (`docs/TEST-SERVER.md`, podman on
> `:13399`; `podman start abs-test` gives the server back with every book of the
> sessions before, and it holds a library of 2056 items, a library of **520
> podcasts**, a PDF of 502 megabytes, and an **EPUB of 100 megabytes** now).
> **Drive the real program inside tmux** with `docs/harness/drive.sh`; a screen of
> your own writing lies to you. Verify with a second program: `curl`,
> `podman logs abs-test`, or a browser. Write the measurement in
> `docs/TAKEOVER-BACKLOG.md` under a new item (T-128 and up), and name that item
> in the commit.
>
> **The gates, before each commit**, under `nice -n 19 ionice -c 3` with `-j 16`:
> `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and
> `cargo nextest run` with `ALSA_CONFIG_PATH` pointing at a real null asound file.
> Baseline: **979 tests in 2.2 seconds**, and `cargo nextest run --run-ignored all`
> gives **1004 of 1004** with the sandbox up, in 16.4 seconds. **Run that second
> command at the end of the session too**: it found the fault of a test of
> 2026-08-12 that the fast suite did not.
>
> **No run opens the real sound device.** `TOUTUI_AUDIO_DEVICE=null` gives the
> null device of ALSA, and the log then says "the application uses the sound
> device alsa:null". `ALSA_CONFIG_PATH` does **not** silence the real program.
>
> ### The work, in the sequence of its value
>
> 1. **The measurement that did not repeat.** The first attempt of the book of an
>    EPUB of 100 megabytes said "The program did not get the book: **No server
>    address answered**", and `curl` sends that book in 0.13 seconds. Four
>    attempts after it gave the book. That moment came after a scan of the
>    library: the socket of the live messages marks an address down for a
>    connection that no machine takes (T-107). **Make the condition again with the
>    log of the program**, and read `the_address_is_down`. A user who meets it
>    reads a reason that is false.
> 2. **The requests of the start that are left.** T-127 took 29 requests away and
>    the first frame with a server of 500 ms went from 3.8 s to 1.7 s. Four rounds
>    stay: the libraries, the shelves, the four requests that go together, and the
>    sound device. **The shelves and the four do not need each other.** The proxy
>    of `slow.py` and a poll of the box of the start give the sequence with no
>    line of code (the trap 88).
> 3. **The sweeps that no session has made.**
>    - **A library of books of more than 500 items, with a media that plays.**
>    - **A server that answers slowly while a media plays**: every measurement of
>      a slow server played nothing.
>    - **A second account of a second server** (T-124): the key `c` while a media
>      plays, and the position of that media.
>    - **A library whose media the account may not read**, with an account of the
>      type `user` (T-121 holds the commands of such an account).
> 4. **The words for the user.** Every text in ASD-STE100. A view says why it
>    holds no line, and it never says a reason that the program does not have
>    (T-91). **A text must not promise a function that the program does not
>    have** (T-118). A key that does nothing in one view is a fault of its own
>    (T-79). A message lives six seconds.
>
> ### The two issues of the fork
>
> Give `-R ealtun21/Toutui` to every `gh` command. **#24** holds the state of the
> table of 2026-08-12, and it stays open for the row of the e-reader. **#20**
> stays open with its decision (`libsqlite3-sys` and `ring` stay until `turso` is
> a release and `rustls-rustcrypto` is beta or better). Never write to
> AlbanDAVID/Toutui: it is archived, and his credit stays in the README, the
> LICENSE, `Cargo.toml`, and the settings screen.
>
> **Do not open these again.** The book of xHE-AAC plays (T-68 and T-69). Toutui
> stays GPL (T-51). `libsqlite3-sys` and `ring` stay (T-20).
> `GET /api/podcasts/:id/checknew` and `GET /api/tags` stay outside, and section 6
> of `docs/T-24-coverage.md` holds the measurement of each (T-110). The group
> `episodes` of the search stays outside until a measurement gives one hit of it
> (T-113). The words of a book of a scan of T-116 stay as they are: the maintainer
> decided that on 2026-08-12. **The list of the devices of an e-reader comes from
> `POST /api/authorize`**, and `GET /api/emails/settings` can never give it to a
> user (T-119). **The program holds more than one account now** (T-124), and the
> episodes of a podcast come when the user opens that podcast (T-126).
>
> All prose and user-facing strings in ASD-STE100 simplified technical English. No
> crate that needs a library of the system: `cargo tree -i openssl-sys` must find
> nothing, and `cargo tree -i cc` must find `libsqlite3-sys` and `ring` only. No
> test may need the network — the tests of the sandbox carry `#[ignore]` and run
> one at a time. The server of the maintainer is theirs alone: **never use it**,
> and never write its address or its account into this repository. Measure against
> the sandbox.
