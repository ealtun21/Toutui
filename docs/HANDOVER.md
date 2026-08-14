# The handover of 2026-08-14

This document is for the next session. It says what is done, what is open, and the
traps that cost real time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of each
item, and `docs/T-24-coverage.md` for the comparison with the server.

**The newest release is v0.8.9.** The items T-177 and T-178 belong to this
session. The items T-175 and T-176 belong to the session before it.
The items T-173 and T-174 belong to the session before that one.
The items T-171 and T-172 belong to the session before that one. The
items T-169 and T-170 belong to the session before that one. The item T-168 belongs to
the session before that one, and the item
T-167 to the session before that one.
The correction of T-166 belongs to the session before that one, and the
measurement of T-166 to the session before that one.
The item T-165 belongs to the session before that one.
The items T-163 and T-164 belong to the session before it, the item
T-162 to the one before that, the item
T-161 to the one before that, the item
T-160 to the one before that, and the items T-158 and T-159 to the one
before that, the items T-155, T-156, and T-157 to the one before that, the item
T-154 to the one before that, T-152 and T-153 to the one before that, T-150 and
T-151 to the one before those, T-148 and T-149 to the one before them, T-146 and
T-147 to the one before those, T-145 to the one before that, T-142 to T-144 to
the one before it, and T-140 and T-141 to the one before those.

**No row of section 4 of `docs/T-24-coverage.md` says `Half`.**

## The session of the nineteenth turn of 2026-08-14: the place of a book that the program did not read

**One release: v0.8.9.** The session before this one left no item open, and it
named two roads: **a key that reads a state of the server and that then writes
it** (the shape of T-175), and **`src/api/me/get_media_progress.rs`, the one
structure of `src/api/` that asks for every field of an answer**. This session
took both, and **each of them held a fault**. The second road came with a
decision of the session before it — "that structure stays as it is" — and **the
measurement says that the decision was wrong**.

| Item | What | Where |
|---|---|---|
| T-178 | **The reader wrote a place of a book that it did not read.** `place_of_the_server` held `.ok()?`, therefore a `GET /api/me/progress/:id` that came back with the status 500 gave the reader the first page of the book, and the send of the key `h` then wrote that first page to the server: `ebookProgress` went from `0.6` to `0.0041`, and **the user lost their place in a book of 14 chapters on every machine of the account**. No word of the reader said why. The reader sends no such place now, and it says what the server said | `src/logic/reader/session.rs`, `src/app.rs` |
| T-177 | **Every position of the account went away for two fields that the program does not read.** `mediaItemId` and `mediaItemType` came to `mediaProgress` with Audiobookshelf 2.5.0, and `Root` asked for both: a server before that version gave the log 20 lines of "a position of the account does not read", and the Home view then held no percent, no mark of a book that is finished, and `Progress:  N/A%,   N/A`. **No word said why.** `libraryItemId` stays now, and every other field takes a default | `src/api/me/get_media_progress.rs`, `src/api/me/permissions.rs` |

The evidence stands in `docs/TAKEOVER-BACKLOG.md` under T-177 and T-178. Five
things are worth the room here:

1. **T-178 is the larger of the two, and it takes work of the user away.** The
   other faults of this class write a wrong word or they hide a value; this one
   **wrote over a place that the user made**, on the server, for every machine
   of that account. `.ok()?` and `Err(_)` are the same line of thought, and a
   sweep of this class must read every `.ok()` of a read that stands before a
   write.
2. **A decision of a session before can be wrong, and the measurement says so.**
   T-176 left `get_media_progress.rs` with a reason: "a default of a field of a
   position is a state, and a state that the program did not read is T-175."
   **The two are not the same thing.** T-175 is a **read that came back with a
   fault**, and a field that an answer does not hold is **a server of another
   version**. The paragraph of T-176 in the backlog holds that correction now.
3. **A harness of a body of a file cannot reach `GET /api/me`.** That answer
   holds the id of the account, the id of every media, and the position of each
   of them — values that the sandbox made at the moment of the request.
   **`docs/harness/a_field_of_the_answer_goes_away.py`** forwards the request
   and it takes the named fields out of the answer, at every depth of the body.
   It works for every path and every field, therefore the next sweep of a
   server of another version needs no new file.
4. **The reader holds a reason now, and not a boolean.** `sends_the_place: bool`
   became `ThePlaceOfTheBook` of three values, because **the two roads that send
   no place are two different things** (T-91): a book of another file of the
   item (T-76) is a book of this machine, and a place that the server did not
   give is a place that the program must ask for again.
5. **The sweep of the shape of T-175 is closed for the keys.** Every key that
   writes to the server was asked what the program read before that write: the
   key `b` writes the place of the playback of this program, the keys `<` and
   `>` read the list of this program (T-165), and the key `X` and the key `r`
   write with no read. **The keys `M`, `N`, and `e` were the three of this
   shape, and each of them held a fault.**

**The condition that this session leaves open.** None of its own. The road of
the next session stands in the prompt at the end of this file.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1107 of 1107** in 2.3 seconds |
| `cargo nextest run --run-ignored all` | **1132 of 1132** in 23.8 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | three runs, and every run passed |
| `cargo tree -i openssl-sys` | no package |
| `cargo tree -i cc` | `libsqlite3-sys` and `ring` only |

## The session of the eighteenth turn of 2026-08-14: the answer of the server that the program did not read

**One release: v0.8.8.** The session before this one left no item open, and it
named two parts of the program that no measurement had reached: **the keys that
write to the server**, and **the answer of `GET /api/libraries` after the
login**. This session took one of each, and **each of them held a fault**.

| Item | What | Where |
|---|---|---|
| T-175 | **The keys `M` and `N` wrote a state that the program did not read.** Each key reads the progress of the media and it then writes the opposite of it, and `Err(_)` of that read held every fault: a server that answered `500` to `GET /api/me/progress/:id` gave both keys the value "not finished" and "not away from the shelf" at every press. The user pressed `M` on a book that the server holds as finished, the program wrote `isFinished: true` one more time, and it said `The media is finished now.` The two keys write nothing now, and each of them says what the server said | `src/app.rs` |
| T-176 | **One field of a library that the program does not read stopped the program.** `Library` asked for every field of the answer of Audiobookshelf 2.36.0, and a body with no `icon`, or with no `settings.autoScanCronExpression`, gave `Toutui stops: it cannot read the lists of the server.` Three fields stay now — the id, the name, and the media type — and the words of a body that the program cannot read name the field | `src/api/libraries/get_all_libraries.rs`, `src/api/client/mod.rs` |

The evidence stands in `docs/TAKEOVER-BACKLOG.md` under T-175 and T-176. Five
things are worth the room here:

1. **A key that reads a state and that then writes it is a shape of its own**,
   and no harness of this repository reached it: the read and the write of these
   two keys stand on **one** path, therefore `one_path_fails.py` fails both of
   them and the words of the write hide the fault of the read.
   **`docs/harness/one_method_fails.py`** fails one method of a path, and it
   forwards the other.
2. **The comment of the old code was true of one status alone.** "A media that
   never played has no progress, and the server gives an error" — the sandbox
   answers `404` for such a media, and `Err(_)` held the status 500, the limit
   of time, and a body that does not read beside it. **`ApiError` holds that
   difference already** (T-172 gave it every category), and the correction is
   one `match` of three arms.
3. **The words of a decode named nothing at all.** `error decoding response
   body` of `reqwest::Response::json` stood for a field that the program does
   not need, for a field that it needs, and for a body of no JSON at all.
   `serde_json` names the field and the place, and the body of the answer stays
   in the memory of that one function: **a body can hold a token.**
4. **The structures of the media hold the rule of T-176 already**, and one
   structure is left with a reason: `src/api/me/get_media_progress.rs`. A field
   of a position that the program does not read is a **state**, and a default of
   a state is the fault of T-175. A row of that answer that does not read takes
   a line of the log and no more, therefore it stops no program.
5. **A library whose name holds no character is measured, and its words stay
   open.** The program starts, the header says `📖  (book)`, and the view of the
   key `S` holds a line of no character. No server of a measurement gives such a
   name, therefore a text for that line would name a condition that no
   measurement has reached (T-91 and T-118).

**The condition that this session leaves open.** None of its own. The road of
the next session stands in the prompt at the end of this file.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1102 of 1102** in 2.3 seconds |
| `cargo nextest run --run-ignored all` | **1127 of 1127** in 18.4 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | three runs, and every run passed |
| `cargo tree -i openssl-sys` | no package |
| `cargo tree -i cc` | `libsqlite3-sys` and `ring` only |

## The session of the seventeenth turn of 2026-08-14: the login of no library

**One release: v0.8.7.** The session before this one left no item open, and it
named the parts that a server of a fault has not reached. **The login screen is
the first of them.** `POST /login` holds five sentences already (T-92),
therefore this session took the request after it — `GET /api/libraries` — and
`docs/harness/no_library.py` gave a server that answers the login and that
holds no library. **The program did not say a wrong word: it said nothing at
all, and it stood for ever with a screen of no character.**

| Item | What | Where |
|---|---|---|
| T-173 | **A login of an account that reaches no library took the program away.** The row of the account holds the name and the id of the library of the start, and `library_names[0]` of a list of nothing stopped the thread of the login. The login says `The server gave no library for this account. Ask an administrator of the server for a library.` now, and it writes no row | `src/api/server/auth_process.rs` |
| T-174 | **A fault of one thread stopped the program with a screen of no character.** The screen of the login, the screen of the search, and the box that asks for a text each held `io::stdout().lock()` for their whole life, and the hook of a panic of `ratatui::init` waited for that lock. The three screens take `the_backend_of_a_field` now, and it holds no lock | `src/ui/text_field.rs`, `src/logic/auth/auth_input.rs`, `src/logic/search/search_active.rs`, `src/logic/prompt.rs` |

The evidence stands in `docs/TAKEOVER-BACKLOG.md` under T-173 and T-174. Five
things are worth the room here:

1. **The larger of the two items is the second one.** T-173 is one line of an
   index, and every user of a new server meets it; T-174 is the reason that the
   user of T-173 read no word of it. **A program that says nothing at all is
   worse than a program that says the wrong thing**, and no sweep of the words
   for the user finds it.
2. **`strace` found the cause, and two other roads lied.** A mark of the log
   inside the hook said nothing, and a mark of `std::fs::write` said the same:
   the hook did not run. `gdb` and `eu-stack` give `Operation not permitted` for
   a process that they did not start. **`strace -f -tt -o` of the whole program
   gave the answer in one line**: the thread of the login gave the raw mode back
   and it then waited for a mutex, and no function of this repository called
   `disable_raw_mode`. The hook of `ratatui::init` did.
3. **T-133 named one half of this road in 2026-08-12, and it took the smaller
   half.** That item took a `println!` out of `auth_process`, and the comment of
   that function says "No line of this function writes to the terminal". **A
   panic is such a line, and no function can promise that it holds none.** This
   session took the cause away instead.
4. **The first correction of T-174 saved nobody.** A copy of the descriptor of
   the standard output gives the hook of this program a road that waits for no
   lock — and the hook of `ratatui::init` stands before it. A measurement of the
   real program said so, and that correction went away.
5. **The three screens of a field are one function now.** `the_backend_of_a_field`
   of `src/ui/text_field.rs` holds the decision, therefore a fourth screen of a
   field takes it too.

**The condition that this session leaves open.** None of its own. The road of
the next session stands in the prompt at the end of this file.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1100 of 1100** in 2.4 seconds |
| `cargo nextest run --run-ignored all` | **1125 of 1125** in 18.3 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | six runs, and every run passed |
| `cargo tree -i openssl-sys` | no package |
| `cargo tree -i cc` | `libsqlite3-sys` and `ring` only |

## The session of the sixteenth turn of 2026-08-14: the header of a server that answers

**Two releases: v0.8.5 and v0.8.6.** The session before this one left no item
open, and it named a class of conditions: **every view of a request of its own holds the
shape of T-170**. This session took `docs/harness/one_path_fails.py` to the
bookmarks, the chapters, the sessions, the statistics, the authors and the
narrators, the devices of an e-reader, and the downloads of the server. **Six of
those views hold the rule already** — each of them holds a `State::Fault` and
each of them says what the server said. **The fault stood two rows above every
one of them.**

| Item | What | Where |
|---|---|---|
| T-171 | **The header said that the server does not answer, for a server that answers.** One request that came back with the status 500 took the one address of the pool away, therefore `active()` gave nothing and the two lines at the top said `⚠ toutuitest: the server does not answer` and `🔗 127.0.0.1:13500 does not answer`, with the notice `R: the media of the disk`. The state `Down` holds a reason now, and the header says `the server reports a fault` with the notice `R: ask the server again` | `src/api/client/endpoint.rs`, `src/api/client/mod.rs`, `src/api/live.rs`, `src/ui/keys.rs`, `src/ui/tui.rs` |
| T-172 | **The program stopped with a line of its own source.** A server that answers 500 to `GET /api/libraries` gave the user `Error: The server reported a fault. Status 500.` with `Location: src/app.rs:644:44`, and no screen of the program came. T-123 closed that road for a token that the server refused, and every other fault of the first request kept it. The program says what the server said now, it names the account and the server, and it says that it changed nothing | `src/api/client/error.rs`, `src/main.rs` |

The evidence stands in `docs/TAKEOVER-BACKLOG.md` under T-171. Five things are
worth the room here:

1. **The class that the road named is closed for the views, and it was open one
   row higher.** The key `a`, the key `v`, the key `T`, and the key `d` each said
   `The server reported a fault. Status 500.` in the words of their own view, and
   the header of all four of them said that the server is away. **A sweep of a
   class must read the whole screen, and not the panel of the view alone.**
2. **The false header stood 10.5 seconds**, until the probe task ran, and
   `PROBE_INTERVAL` is 60 seconds: it can stand for a whole minute. In the middle
   of it `curl` got an answer of that same address in **1.4 milliseconds**, and
   the key `W` of that same program gave the 114 sessions of the account.
3. **The decision of T-87 stays.** A status of 500 is a fault of the endpoint,
   and the address goes down, because a second address of the same server can
   answer that request (T-97). This item changes the words for the user alone:
   `Health::Down` holds a `WhyDown` now, and
   `EndpointPool::every_address_answers_with_a_fault` says that every address of
   the pool answered with a fault.
4. **`mark_down` gives up its work for an address that stands down already**, and
   that guard keeps the log short. The first build of the correction therefore
   kept the words of a fault when the server went away. `mark_down` writes the
   new cause now, and it writes no line of the log for it. **A guard that stops a
   log must not stop a state.**
5. **The header says what the program measured last.** A second press of the key
   `a` makes no request — `logic::authors` keeps its state, and the key `R` alone
   forgets it — therefore the header of that moment names the request before it.
   The measurement of the two roads needed a key of a fresh request (`W`).

**The decision of T-172, with its reason.** The offline mode of T-25 is **not**
the road of a server that reports a fault at the start. That mode is made for a
server that gives no answer, and its words say that the server does not answer:
a server that answers `500` answers, and those words are a reason that the
program does not have (T-91 and T-171). `is_offline` of `App` reaches more than
thirty texts of the views, therefore that road would put the words of T-25 in
every one of them — the sweep of T-168, of T-169, and of T-170 again. **The
program must not promise a function that it does not have** (T-118): it cannot
read the lists of that server, and it says so.

**The condition that this session leaves open.** None of its own. The road of
the next session stands in the prompt at the end of this file.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1098 of 1098** in 2.3 seconds |
| `cargo nextest run --run-ignored all` | **1123 of 1123** with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | six runs, and every run passed |
| `cargo tree -i openssl-sys` | no package |
| `cargo tree -i cc` | `libsqlite3-sys` and `ring` only |

## The session of the fifteenth turn of 2026-08-14: the server that fails one request

**Two releases: v0.8.3 and v0.8.4.** The session before this one left one
condition open, and it named it: **a server that answers some requests and that
fails others**. No harness of this repository gave that condition, therefore
this session wrote one — `docs/harness/one_path_fails.py` — and **the four
requests of the start of a library held five faults**.

| Item | What | Where |
|---|---|---|
| T-169 | **The two views of the lists said a reason that the program does not have.** The server answered the libraries and it failed the collections and the playlists: the key `c` said `This library has no collection and no playlist.` for a library that holds both, and the key `m` asked the user to make a list of a server that the program did not read. Each of them says what the server said now | `src/logic/the_lists.rs`, `src/app.rs`, `src/ui/tui.rs` |
| T-170 | **The three other requests of the start held the same fault.** The Library view said `This library holds no media. Press L to tell the server to examine the library.` for a library of 17 books, the view of the series said `This library has no series.` for a library of three, and the Home view said `The server gave no shelf for this library.` Each of them says what the server said now, and the key `R` asks the server again | `src/logic/the_requests_of_the_start.rs`, `src/ui/keys.rs`, `src/app.rs`, `src/ui/tui.rs` |

The evidence and the fifteen measurements stand in `docs/TAKEOVER-BACKLOG.md`
under T-169 and T-170. Five things are worth the room here:

1. **The cause is the cause of T-168**: `is_offline` of `App` holds the offline
   mode of the start (T-25). Every text that reads that one value says the words
   of a server that answers, and the server can fail one request of that user.
   **T-168 and T-169 close the four hits of that sweep**: the episodes of a
   podcast, the pages of the library, the search, and the lists.
2. **The box holds the library of the request.** The key `S` gives the program a
   new library, and the fault of the library before it is not the truth of this
   one: the measurement took the key `S` to the library `Empty` while the fault
   of `Books` stood, and that view said the truth of `Empty`.
3. **The condition of the request that runs does not exist for this view.**
   `App::new` waits for the four requests before the first frame (T-129),
   therefore no frame stands between the key of the user and the answer of the
   server. `the_reason_of_no_list` holds three conditions, and
   `the_reason_of_no_episode` of T-168 holds four. **A measurement said this**:
   a proxy of a delay of 1.5 seconds gave the lists at the first frame already.
4. **The sentence of a fault is longer than the sentence before it.** The first
   measurement of the correction read `The server reported a fault. Status` and
   the number stood outside the panel of 95 columns: that paragraph takes `Wrap`
   now. **A view that names what the server said must wrap its text.**
5. **A sentence of a fault must name a key that does the work of that fault.**
   The Library view of T-170 said `Press L to tell the server to examine the
   library.` for a request that came back with a fault, and the media of that
   library stand on the server already (T-118). The three sentences of T-170
   name the key `R` and the key `h`, and the sequence of the conditions of that
   view holds a rule of its own: **a filter says nothing of a list that never
   came**, therefore the fault stands above the filter and under the offline
   mode of T-25.

**The condition that this session leaves open.** None of its own. **The four
requests of the start each say why now**, and no `unwrap_or_else` of `App::new`
is left with no word for the user. The road of the next session stands in the
prompt at the end of this file.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1095 of 1095** in 2.3 seconds |
| `cargo nextest run --run-ignored all` | **1120 of 1120** in 18.1 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | six runs, and every run passed |

## The session of the fourteenth turn of 2026-08-14: the request that did not come back

**One release: v0.8.2.** No item of the road stayed open, therefore this session
took the sweep that T-167 named — **the keys that say nothing** — and it drove
the real program against the sandbox for every hit of that sweep that a key of
the user makes. **Two keys held a silence, and the first of them is not a
silence at all: it is a promise of a work that the program stopped.**

| Item | What | Where |
|---|---|---|
| T-168 | **A request of the server that did not come back said nothing at all.** The view of the episodes of a podcast said `The program gets the episodes of this podcast…` for ever, and the key `G` of a library of 2056 items said no word. Each of them says what the server said now | `src/logic/the_episodes.rs`, `src/logic/library_pages.rs`, `src/app.rs`, `src/ui/tui.rs` |

The evidence and the three tables of the measurement stand in
`docs/TAKEOVER-BACKLOG.md` under T-168. Four things are worth the room here:

1. **`is_offline` of `App` holds the offline mode of the start** (T-25). A
   program that started with a server that answers holds `false` for ever,
   therefore every text that reads that value says the words of a server that
   answers — and the server went away. **That one value is the cause of both
   forms.** The header of the program knows the truth (T-128 gives it the state
   `Down`), and the view of the user did not.
2. **T-91 gave that view three conditions in 2026-08-12, and a fourth exists**:
   the request went, and it did not come back. A text that promises a work that
   the program stopped is the fault of T-118 from the other side.
3. **A box of a fault belongs to the podcast of its own request.** A user who
   opens a second podcast must not read the fault of the first one: the box
   holds the place, the answer that comes takes the fault away, and a new
   request takes it away too. The measurement holds all three.
4. **The key `G` took a state of the program with it.** A page that did not come
   left `reads_every_page_of_the_library` at `true`: the key did nothing, it
   said nothing, and its work stood open until the next move of a line. The
   render stops that work with the sentence now.

**The condition that this session leaves open.** The fourth hit of the sweep is
the collections and the playlists of the start (the two `unwrap_or_else` of
`App::new`). A server that answers the libraries and that fails those two
requests gives the view of the lists a reason that the program does not have
(T-91), and **a measurement of it needs a server that answers some requests and
fails others** — `docs/harness/slow.py` is the shape of it. **The session after
this one closed that condition with T-169**, and the harness of it is
`docs/harness/one_path_fails.py`.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1090 of 1090** |
| `cargo nextest run --run-ignored all` | **1115 of 1115** in 19.8 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | six runs, and every run passed |

## The session of the thirteenth turn of 2026-08-14: a playback that did not start

**One release: v0.8.1.** No item of the road stayed open, therefore this session
took one of the parts of the program that the road named and that no measurement
had reached: **the view of the episodes of a podcast while a second window
changes that podcast**. The measurement found one fault, and **that fault does
not belong to that view**: it belongs to the playback, and every view that plays
a media meets it.

| Item | What | Where |
|---|---|---|
| T-167 | **A playback that did not start said nothing at all.** The user read `Loading the media...`, the six seconds of it went by, the row of the message became empty, and no media played. The program says one sentence for each of the three faults of `play_media` now | `src/logic/the_playback.rs`, `src/logic/playback/mod.rs` |

The evidence and the two tables of the measurement stand in
`docs/TAKEOVER-BACKLOG.md` under T-167. Four things are worth the room here:

1. **The program reads the episodes of a podcast one time.**
   `the_episodes_that_came` of `App` goes to `true` and it never goes back to
   `false`: the list of that view is a photograph of the moment of the open, and
   an episode that a second program takes away stays on the screen until the key
   `R`. **T-47 says this well** — the row 2 of the header held
   `R: the server has newer data` at the frame of the change — therefore the
   fault is not the old list. The fault is the key that acts on it.
2. **The fault belongs to the playback, and to no view.** The key `l` of the
   view of the episodes and the key `l` of the Home view gave the same silence
   for the same episode. Therefore the correction stands in `play_media`, and
   the message uses `say` and not `say_in` (T-164: the answer of a key stands
   above every view).
3. **These are three of the five places of T-35.** That item gave the value
   `is_loop_break` to all five of them in 2026-08-10, and it left the words
   outside. The two places that are left go to the offline mode of T-25, and
   that road says its own words already.
4. **The text names no media.** The title comes from the answer of the session,
   and that answer is the thing that did not come (T-91).

**The trap that this measurement adds.** A test of this fault must **not** use
the port that no program holds (`NO_SERVER` of T-146): that road is the offline
mode of T-25, and it keeps the copy of the disk. A server that **answers** and
that holds nothing is a different road, and a host of a raw socket that answers
`404` gives it with no network and no sandbox.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1089 of 1089** in 2.3 seconds |
| `cargo nextest run --run-ignored all` | **1114 of 1114** in 24.4 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | three runs, and every run passed |

## The session of the twelfth turn of 2026-08-14: the correction of the line of the downloads

**One release: v0.8.0.** This session wrote the correction that the session
before it named, and it measured every part of it against the sandbox. **The
rule of the line of the view is made for six views now.**

| Item | What | Where |
|---|---|---|
| T-166 | **The line of the view of the downloads holds an episode of a podcast, and not a number of a line.** An episode that leaves the queue of the server takes the line to nobody with a message that names it, the keys `j` and `k` give a line again, and the key `X` on a line of nobody says one sentence and it changes no queue | `src/logic/the_downloads.rs`, `src/api/podcasts/the_downloads.rs`, `src/app.rs`, `src/main.rs` |

The evidence, the table of the correction, and the ten measurements of it stand
in `docs/TAKEOVER-BACKLOG.md` under T-166. Three things are worth the room here:

1. **The name of an episode leaves the field `now` outside.** An episode that
   becomes the download of this moment is the same episode: it moves from
   `queue` to `currentDownload` of the answer of the server, and the cursor of
   the measurement went with `Letter 27` into the row `▼ Letter 27`.
2. **The two keys that a message names must give a line again.**
   `ListState::select_previous` of ratatui gives `usize::MAX` to a line of
   nobody, and the rule of the line then takes that line to nobody one more
   time: the view would hold no line for ever, and the message would name two
   keys that do nothing (T-143). `the_line_of_the_move` is the rule of the keys
   `j` and `k` of that view now, and it is pure.
3. **The view opens with no line, and the first list of the server gives it.**
   This is the answer to the open question of the session before this one: the
   answer of the server does not stand at the frame of the open, therefore a
   line of the open stands on nothing at all. `the_downloads_gave_the_first_line`
   of `App` holds that one difference. A line that went to nobody after the
   first list stays with nobody.

**The decision of the version.** The series 0.7.x came to `.99`. `0.7.100` is
right for cargo and wrong for a person who reads the releases, therefore the
next release is **v0.8.0**. No function of the program changed its shape with
that number: it is the release of one correction, as every release of this fork
before it.

### The gates of this session

| The gate | The answer |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | no word |
| `cargo fmt --check` | no word |
| `cargo nextest run` | **1087 of 1087** in 2.3 seconds |
| `cargo nextest run --run-ignored all` | **1112 of 1112** in 18.1 seconds, with the sandbox up |
| `cargo test -j 16 --no-fail-fast` | four runs, and every run passed |

## The session of the eleventh turn of 2026-08-14: the downloads of the server that move under the cursor

**No release.** This session took one of the parts of the program that the road
named and that no measurement had reached — **the view of the downloads of the
server (T-81) while a download ends** — and **it holds one fault of two forms**.
**The session stopped after the measurement**, therefore the correction stands
open. **No file of `src/` changed**, and the three documents hold the
measurement.

| Item | What | Keys |
|---|---|---|
| T-166 | **The key `X` emptied the queue of a podcast that the user never chose.** The user stood on an episode of one podcast, two episodes came to their end, and the two presses of `X` took eight episodes of the other podcast out of the queue of the server | `d`, then `X` |

### The fault, and the measurement of it

**This view is the second list of the program that moves with no key of any user
at all.** The queue of the media (T-161) is the first. The server takes an
episode out of the queue when it downloaded it, and it sends a message of that
change: `logic::the_downloads::the_view_must_ask` then gives `true`,
`render_the_downloads` asks the server again, and the new list stands on the
screen at that frame. **The line of the user keeps its number of a line.**

| The moment | The view of the downloads of the user |
|---|---|
| The keys `d` and five times `j` | `The downloads of the server [14 items]`, and the cursor stands on `Chapter 10 — Narrative of Arthur Gordon Pym` |
| **Two episodes come to their end** | `[12 items]`, and the cursor stands on the same line 5: **`Letter 12 — Letters of Two Brides`** |
| The message row of that frame | **empty** |
| **The key `X`** | `Press X again to empty the queue of "Letters of Two Brides…"` — **the podcast that the user never chose** |
| The key `X` again | `The queue of "Letters of Two Brides…" is empty now.` **Eight episodes went away, and the queue of the podcast of the user stayed** |

**The queue of the downloads belongs to the library, and it holds the work of
the server for every user of that server.** The cost of this key is therefore
larger than a view of one user.

**The second form needs no second podcast.** The queue emptied itself while the
cursor stood on its last line: the line of the user then stands **past the end**
of the list, `all.get(line)` gives `None`, and **the key `X` returns with no
word at all** (T-79). The footer promises `X: empty the queue of this podcast`
(T-143).

**The confirmation of two presses does not close this**, and the measurement
says why: the first press names the podcast of the **new** list, and not the
podcast of the frame that the user read. A user who presses `X` and then presses
`X` again — the sentence of the program tells them to — empties a queue that
they never chose.

### The correction that the next session writes — **it is written, in v0.8.0**

**It is the rule of T-147, of T-160, of T-161, of T-162, of T-163, and of T-165
for a sixth view: the line holds an episode of a podcast, and not a number of a
line.** The shape of T-161 fits this view best, because the list moves with no
key of the user:

1. **`src/logic/the_downloads.rs` holds the rule, and it is pure.**
   `what_the_line_of_the_downloads_holds(the_episode_of_the_line, all)` in the
   shape of `what_the_line_of_the_lists_holds`. **The identity of a line is the
   podcast and the title of the episode** (`item_id` and `title` of
   `OneDownload`) — and **not the field `now`**: an episode that becomes the
   download of this moment stays the same episode, and it moves from `queue` to
   `currentDownload` of the answer of the server.
2. **`App` holds the entry of the line of the frame before**, and the rule runs
   in `render_the_downloads` after `ask_for_the_downloads`, in the shape of
   `take_the_lists`. A line that the user moved with `j` or `k` gives the
   episode of the new line (the rule of `what_the_line_of_the_user_holds` of
   `src/logic/queue.rs`: the line of the user is the truth of the choice).
3. **An episode that left the queue takes the line to nobody**, with a message
   that names it. The program cannot say **why** it left — the server downloaded
   it, or a different program emptied the queue — therefore the text says what
   the program knows (T-91): `The episode "Chapter 10" of "…" is not in the
   queue of the server now. No line is selected: the keys j and k select one.`
4. **The message belongs to the view of the downloads** (T-164): the rule writes
   it with no key of the user, therefore `say_in(AppView::Downloads, …)` and not
   `say`.
5. **The key `X` on a line of nobody says one sentence** (T-79):
   `No episode is selected.` It returns with no word today.
6. **The mark of the confirmation must go away with the line.**
   `confirm_the_empty_queue` holds the identity of a podcast, and a line that
   goes to nobody must take that mark with it.

**One question that the next session must answer with a measurement**: the view
opens with `select(Some(0))` on a queue that can be empty, therefore the line of
nobody exists at the first frame already. The rule must give `None` for an empty
list, and the key must say its sentence.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `X`, after two episodes of the queue came to their end** | **one fault** (T-166): the program named a podcast that the user never chose, and the two presses emptied its queue |
| The queue after it, of `curl` | two lines, and both of them belong to the podcast of the line of the user |
| **The key `X` on a line past the end of the list** | **one fault of the words**: no word at all, and the footer promises the key |
| The time of one download of the server | **about four seconds** for an episode of LibriVox |
| The correction, and the tests of it | **the session of the twelfth turn wrote them, in v0.8.0** |

## The session of the tenth turn of 2026-08-14: the lists of two windows

**One release: v0.7.99.** No condition of the road stayed, therefore this
session took one of the parts of the program that the road named and that no
measurement had reached: **the lists (the collections and the playlists) with
two windows of one account**. It is the shape that found a fault in fourteen
sessions — a state of one process that a second program cannot see — and **it
held one fault of two forms**. The key of that view removes a whole collection
or a whole playlist, and every user of the server loses a collection.

| Item | What | Keys |
|---|---|---|
| T-165 | **The key `X` took a media out of a playlist that the user never opened.** A second window removed the collection of the line above, and the view of the user became another list with no word at all. The second form of it left the user in a view of no name, no line, and a footer of five keys that do nothing | `c`, `l`, then `X` |

### T-165, and it is the one to know of this session

**`take_the_lists` of `src/app.rs` is the one door**: it is the one function
that changes `self.lists`, the render calls it at each frame, and it held the
rule of T-41 for the media of a list alone. It clamped the line of the media to
the number of the media, and **it never asked whether the list of the line
stayed**: `selected_list()` reads a **number of a line**.

| The moment | The window A | The window B |
|---|---|---|
| A opens the media of `A Test Playlist` | `A Test Playlist [4 items]` | — |
| B removes the collection of the line above it | the same screen | `The collection "A Test Collection" is not on the server now.` |
| **A presses `X`** | **`Z Second Playlist [2 items]`**, and the message names the media of `A Test Playlist` | — |
| A presses `X` again | **`"The Test Chronicles Volume 3" is not in the playlist "Z Second Playlist" now.`** | — |

**The first key of A was right** (the rule of T-147: a key acts on the media of
its own line), and the refresh that came with it moved every list one place up.
**The second form is sharper**: A stood in the media of the last list, B removed
that list, and the screen of A then held **no title, no line, and no text at
all** — with a footer of five keys that do nothing (T-143).

**The correction is the rule of T-147, of T-160, of T-161, of T-162, and of
T-163 for a fifth view: the line holds a list, and not a number of a line.**
`what_the_line_of_the_lists_holds` of `src/logic/the_lists.rs` reads the
identity of the list of the line before the write: the same list takes the line
with it, a list that went away takes the line to nobody with a message that
names it, and **a user who stands in the media of a list that went away gets the
view of the lists again** — that view holds nothing at all without its list.

**The message belongs to the view of the lists, and `say_in` writes it**
(T-164): the rule runs in the render with no key of the user. The answer of
their key stands above it for six seconds, and the sentence of the view comes
after it: both sentences reached the user in the measurement.

**The key of this window that removes a list says nothing of this rule, and that
is a decision.** `remove_the_list_of_the_line` moves the line to the list below
the one that goes away, or to the list above it when that one is the last:
`take_the_lists` then follows a list that stays, and the answer of the key names
the list that went away already.

**The text of the correction found a fault of its own.** It says that the keys
`j` and `k` select a line, and the key `j` gave **no line at all** for a line of
nobody: `if let Some(selected)` of `src/app.rs`. The key gives the first line
now, in the view of the lists and in the media of a list. **A text that promises
a key needs a measurement of that key** (T-118 and T-143).

**The view `PutInAList` keeps its number of a line, and that is a decision.**
That view opens at the line 0 with each key `m`, it stands open for some
seconds, and its keys `c` and `p` say nothing for a line of nobody: a line of
nobody there would need a sentence of its own for no measured fault. **It is a
part of the program that no measurement has reached**, and it holds the same
cause.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `X` of A, after B removed the collection of the line above** | **one fault** (T-165): the view became `Z Second Playlist` with no word, and the next key took a media out of it |
| The same condition, after the correction | `A Test Playlist [2 items]`, and the line stays on the media of the user |
| **The key `X` of A, after B removed the list that A opened** | **one fault**: no title, no line, and five keys of the footer that do nothing |
| The same condition, after the correction | the view of the lists, a line of nobody, and the sentence that names the playlist |
| The two sentences of that moment | the answer of the key for six seconds, and the sentence of the view after it (T-164) |
| **The key `j` after the line went to nobody** | **one fault of the words**: no line came, and the text promised it. The first line comes after the correction |
| The key `X` of B on the list of its own line | **no fault**: the line goes to the list that follows, and B reads one sentence |
| **`cargo test`, the command of CI, three runs** | **no fault** |
| **`cargo nextest run --run-ignored all`** | **1109 of 1109**, with the sandbox up |

## The session of the ninth turn of 2026-08-14: the bookmarks of another media, and the message of another view

**Two releases: v0.7.97 and v0.7.98.** This session took the last part of the
program that the road named — **the view of the bookmarks while the media that
plays changes** — and then **the one condition that T-162 named and did not
close**: a message belongs to the view of the user. **Each of them held a
fault.**

| Item | What | Keys |
|---|---|---|
| T-163 | **The key `b` wrote a bookmark of a book that the user did not choose.** The user read the bookmarks of a book of 30 minutes, and the key wrote a place of a book of eight hours at 5:25:30 | `V`, then `b` |
| T-164 | **The user read the message of a view that they were not in, and the message of their own view never came.** Three rules of the loop wrote to one slot, and the rule of the Home view won every time | `q` |

### T-163, and it is the first of this session

**The list of this view is not the fault**: `bookmarks_of` holds the media of the
user already, no line moves under the cursor, the key `X` names the media of its
own bookmark, and the key `l` refuses a media that does not play. **The key `b`
is the one door**: it read the media of the **engine**.

| The moment | The view of the bookmarks of the user |
|---|---|
| The user plays `A Long Test Book` of 30 minutes, and the queue holds `A Book Of Many Hours` | — |
| The key `V` | **`The bookmarks [1 item]`**, and the line is `A place of the long book (00:10)` |
| **The book comes to its end, 22 seconds later** | **the same title, the same line**, and the message row is **empty** |
| The key `b`, and a name | `The bookmark "…" is on the server.`, and **the list still holds one line** |
| `GET /api/me` of `curl` | **the bookmark stands on `A Book Of Many Hours` at 19530 seconds** |

**The view of the user contradicts the message of the program**, and no text of
the screen names the media of either side: the title said `The bookmarks` alone.

**The correction is the rule of T-160, of T-161, and of T-162 for a fourth
view.** `what_the_media_of_the_bookmarks_is` of `src/logic/bookmarks.rs` compares
the media of the view with the media of the engine: another media, or no
playback at all, writes nothing and says
`The media "A Long Test Book" does not play now, and this key writes a place of
it. The key V shows the bookmarks of the media that plays.` **The title names the
media now**, in the shape of the view of the chapters.

**The rule stands in the key, and not in the loop of `src/main.rs`. That is a
decision**, and it is the difference from the three views before it: **no line of
this view moves under the cursor**, therefore the user reads the truth of their
own view until they press `b`.

### T-164, and it is the one to know of this session

**The condition that T-162 named**: three rules of the loop write a message with
**no key of the user** — the shelf Continue Listening (T-160), the line of the
queue (T-161), and the media of the chapters (T-162) — and each of them wrote to
**the one slot**. The rule of the Home view stands in the render of **every**
view and it runs last: **it won every time.**

| The moment | The view of the queue of the user |
|---|---|
| Before | `The queue [2 items]`, and the cursor stands on `A Book Of Many Hours` |
| **The book comes to its end** | the cursor stands on **nobody** (T-161 works) |
| The message row, every 0.2 seconds for 6.2 seconds | **`The media "A Long Test Book" is not on the shelf Continue Listening now.`** |
| The sentence of T-161 | **it never reached the screen** |

**The correction of T-161 is destroyed for the one user that it was written
for.** The correction: `say_in(view, text)` writes a message that belongs to one
view, `for_the_screen(self.view_state)` names the view of the user, and **the
life of such a message starts at the frame that shows it**. The user of the queue
reads `Loading the media…` and then the sentence of their own view, and the
sentence of the Home view stands on the screen when they press `h` — 14 seconds
later in the measurement. **Both sentences reach the user, and neither of them is
lost.**

**A message of a view waits with no limit of time, and that is a decision**: each
of the three sentences names a state that stays until the user presses `j` or `k`
in that view. **The answer of a key stands above them all**, and that is a
decision too: the user pressed that key.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `b` after the queue started the media of its front** | **one fault** (T-163): a bookmark of a book of eight hours at 19530 s, and a second run gave 22806 s |
| The same condition, after the correction | the key writes nothing and it names the media, and `curl` holds the one bookmark of the user |
| The title of the view of the bookmarks | `The bookmarks [1 item]` before, and `The bookmarks of "A Long Test Book" [1 item]` after |
| **The message of a user of the view of the queue, at the frame of the change** | **one fault** (T-164): six seconds of the sentence of the Home view, and no sentence of their own |
| The same condition, after the correction | `Loading the media…` for six seconds, and the sentence of T-161 for the six after them |
| The key `h` to the Home view, 14 seconds after the change | the sentence of T-160 stands there: the fault that T-160 closed stays closed |
| **`cargo test`, the command of CI, nine runs** | **one fault of a test of this session**: two test functions of `src/logic/message.rs` fought for the slot of the process (the shape of T-144). They are one function now, and five runs after it gave no fault |
| **`cargo nextest run --run-ignored all`** | **1107 of 1107**, with the sandbox up |

## The session of the eighth turn of 2026-08-14: the chapters of another media

**One release: v0.7.96.** No condition of the road stayed, therefore this
session took the one part of the program that the road named and that no
measurement had reached: **the view of the chapters while the media that plays
changes**. The queue starts the next media with no key of the user (T-24), and
the view draws the chapters of the media that plays at each frame. **It held one
fault, one window makes it, and the fault reaches the server.**

| Item | What | Keys |
|---|---|---|
| T-162 | **The key `l` moved the place of a book that the user did not choose.** The user chose a chapter of a book of 30 minutes, and the key moved a book of eight hours 43 minutes forward | `C`, `G`, then `l` |

### T-162, and it is the one to know of this session

**The list of the chapters holds no media**: it is the list of the media that
**plays**, therefore the rule of T-161 for the queue does not reach it.

| The moment | The view of the chapters |
|---|---|
| The user plays `A Long Test Book` of 30 minutes, and the queue holds `A Book Of Many Hours` at 4:50:35 | — |
| The keys `C` and `G` | `The chapters of "A Long Test Book"`, and the cursor stands on `The third part` (20:00) |
| **The book comes to its end, 22 seconds later** | `The chapters of "A Book Of Many Hours"`, and the cursor stands on the line 3: **`The hours of the end` (5:33:20)** |
| The key `l` | **the playback went from 4:50:35 to 5:34:44**, and `curl` then read `currentTime: 21036` |

**The correction is the rule of T-160 and of T-161 for a third view: the line
holds a media, and not a number of a line.**
`what_the_media_of_the_chapters_is` of `src/logic/chapters.rs` reads the
`playback_id` of the frame before: the same playback keeps the line, another
playback (or no playback at all) takes the line to nobody with a message that
names the media, and a view that holds no playback reads the media that plays.
**The loop of `src/main.rs` holds the rule at each frame**, beside the rule of
the queue: the media changes with no key of this user.

**The identity is the playback, and not the item.** A user who plays the same
book again gives a new playback, and a new answer of `POST /api/items/:id/play`
gives its chapters.

**The key `l` says `No line is selected.` now** (T-79).

**The message of the Home view stands above the message of this view 0.8 seconds
later, and that is the condition for the next session.** The media leaves the
shelf Continue Listening too, and the rule of T-160 says its text from the
render of **any** view. Both sentences are true, and a change that holds the
text of T-160 for the moment of the Home view would take the reason away from a
user who comes back to that view later — the fault that T-160 closed. **A
message belongs to the view of the user**, and no session has measured that.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `l` of the view of the chapters after the queue started the media of its front** | **one fault** (T-162): the place of a book of eight hours went 43 minutes forward, and the server took it |
| The same condition, after the correction | `No line is selected.`, and `curl` holds the place of the playback of the queue |
| The message row at the frame of the change | **it named the Home view only** before, and it names `A Long Test Book` after |
| The keys `j` and `l` after it | the cursor stands on a line again, and the playback goes to the chapter that the user chose |
| The text of this view against the text of the Home view | **0.8 seconds**, of a message that lives six |
| **`cargo test`, the command of CI, four runs** | **no fault** |

## The session of the seventh turn of 2026-08-14: the queue that moves under the cursor

**One release: v0.7.95.** No condition of the road stayed, therefore this
session named one of its own, and it took the shape that T-160 left open: **a
line of a view that is not the Home view and that goes away under the cursor**.
The road named three such lists — the queue, the downloads, and the episodes of
a podcast — and **the queue is the one of the three that moves with no key of
any user at all**. **It held one fault, and one window makes it.**

| Item | What | Keys |
|---|---|---|
| T-161 | **The key of the user took a media of the queue that the user did not choose.** The key `X` took it out, and the key `l` played it and stopped the media that the queue had started | `q`, then `X` or `l` |

### T-161, and it is the one to know of this session

**The queue takes the media of its front away when the media that plays comes
to its end** (T-24), and the view of the queue draws the queue of this process
at each frame. **The lines keep the number of the line**, therefore the media
below moves under the cursor with no word at all:

| The moment | The view of the queue |
|---|---|
| The user plays a book of 30 minutes, and the queue holds three media | `The queue [3 items]` |
| The keys `q` and `j` | the cursor stands on the line 2, `A Second Book Of Many Hours` |
| **The book comes to its end, 22 seconds later** | `The queue [2 items]`, and the cursor stands on the line 2: **`Multi File Test Book`** |
| The message row of that frame | **empty** |
| The key `X` | **`Multi File Test Book` went out**, and the media of the user stays |

**The key `l` is the sharp form of it.** The same condition with `l`: the key
played `One Chapter Book`, it **stopped the media that the queue had started
one second before**, and it took that media out of the queue. **Two media of
the user go the wrong way with one key.**

**The rule of T-147 does not reach this.** `take_the_media` reads the disk and
it takes the media **of the line** (T-147), and the sentence of T-151 names
that media: both of them read the line of the user, and the fault is that the
media of that line is not the media that the user chose. **The queue that moved
is the queue of this process**, therefore no read of the disk sees it.

**The correction is the rule of T-160 for a second view: the cursor holds a
media, and not a number of a line.** `what_the_line_of_the_user_holds` reads the
line of the frame before and the media of that line: a media that stands in the
queue takes the cursor with it, a media that left the queue takes the line to
nobody with a message that names it, and a line that the user moved gives the
media of the new line. **The loop of `src/main.rs` holds the rule at each
frame**, beside the timer for sleep: the queue moves with no key of this user,
therefore no key handler can hold it.

**The keys `X` and `l` say `No media is selected.` now** (T-79). They returned
with no word at all for a line of nobody, and a line of nobody comes with no key
of the user since this correction.

**A second window that takes a media out gives no such fault, and that is a
decision.** A key of this window reads the disk (T-147), it finds the media of
its line in no row, and it says the sentence of T-151. The view of this window
keeps the old line until that key, and **a read of the disk at each frame would
change the decision of T-147 for no measured fault**.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `X` after the media of the queue moved under the cursor** | **one fault** (T-161): the key took `Multi File Test Book` out, and the media of the user stayed |
| The same condition, after the correction | the cursor stands on the media of the user, and that media goes out |
| **The key `l` in the same condition** | **one fault**: it played the media below, and it stopped the media that the queue started |
| The media of the line of the user comes to the front and plays | **the message row was empty** before, and the text names `A Book Of Many Hours` after |
| The key `X` on a line of nobody | `No media is selected.`, and the queue keeps its media |
| The key `j` after it | the cursor stands on the first media again |
| **`cargo test`, the command of CI, four runs** | **no fault** |

## The session of the sixth turn of 2026-08-14: the line that goes away under the cursor

**One release: v0.7.94.** No condition of the road stayed, therefore this
session named one of its own: **a line of the shelf Continue Listening that goes
away while the cursor of the user stands on it**. The road named two shapes of a
fault, and this condition holds both of them at one time — "a key of a view that
the user presses two times" (the shape of T-154) and "a state of one process
that a second program cannot see". **It held one fault, and one window makes it
as well as two.**

| Item | What | Keys |
|---|---|---|
| T-160 | **The key of the user reached a media that the user did not choose.** Three presses of `M` marked three books, and a second window did the same to the line of the first one | `M`, and `M` again |

### T-160, and it is the one to know of this session

**A media that the user marks as finished goes away from the shelf Continue
Listening** (T-66), and the lines of the Home view come again. **The lines keep
the number of the line**, therefore the media below moves under the cursor with
no word at all:

| The press of the key `M` | The line of the user | The media that the server marked |
|---|---|---|
| before | `A Long Test Book` | — |
| the first press | `A Book Of Many Hours` | `A Long Test Book` |
| the second press | `A Second Book Of Many Hours` | **`A Book Of Many Hours`** |
| the third press | `Depthless Hunger, Book 2` | **`A Second Book Of Many Hours`** |

**The message of each press is the same text, and it names no media**: "The
media is finished now." A user who marks a book by mistake presses the key again
to take the mark back, and that press marks the next book of the shelf. **The
mark that goes back costs more than a mark**: `{"isFinished": false}` writes
`currentTime: 0` on the server (the section 15 of `docs/TEST-SERVER.md`).

**The second form needs no key of this user at all.** Two windows of one
account: the window B pressed `M` on the media of the line of the window A, the
line of A went away, the cursor of A stood on the next media, and **the message
row of A was empty**. The next `M` of A then marked that media.

**The correction: the program cannot know which media the user wants now,
therefore it takes the line away and it says what happened.** No key of the
selection can then reach a media that the user did not choose — the next press
of `M` says "No media is selected." — and the keys `j` and `k` give a line
again. The text names the media that went away, and it promises no other key
(T-118 and T-143).

**Why not the line above, and why not the line below.** Both of them put a
media of the shelf under the cursor of a user who chose no media, therefore both
of them keep the fault. A line of nobody is the one answer that changes no byte
of the server.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **Two presses of the key `M` on the shelf Continue Listening** | **one fault** (T-160): two media of `isFinished: true`, and one message |
| Three presses of it | three media, and the user read one message |
| The same, after the correction | the second press says "No media is selected.", and `curl` holds `isFinished: false` |
| **The key `M` of a second window on the media of the line of this window** | the line of this window went away with no word, and its next key marked another media |
| The same, after the correction | the message names `A Long Test Book`, and the key `M` changes no media |
| The key `j` after the line went away | the cursor stands on the first media again |
| `PATCH /api/me/progress/:id` with `{"isFinished": false}` of `curl` | `currentTime` went from **500 to 0** |
| **`cargo test`, the command of CI** | **no fault** |

## The session of the fifth turn of 2026-08-14: the window that stays after a log out

**Two releases: v0.7.92 and v0.7.93.** No condition of the road stayed for the
account of two windows, therefore this session named one of its own: **a window
of an account that stands open while a second window logs out of that account**.
The road named the shape of T-155 as a sweep of its own — a write of a state
that names a row of the database — and this condition is that shape from the
side of the caller. **It held two faults, and the first of them stops the whole
program.**

| Item | What | Keys |
|---|---|---|
| T-158 | **The media never played, and the program then answered no key at all.** The key `Q` did not stop it | `S`, `l`, then `l` |
| T-159 | **A log out left a program of that account, and that program named nobody.** It held the token of the account, and it said that it kept a choice that no row holds | `R`, or any key |

### T-158, and it is the one to know of this session

**The wait of a playback waits while `is_loop_break` is not `1`**, and the two
reads of that wait gave the text `No db found` for an account that stands in no
row. **No loop of a playback of a row that does not exist can ever write that
value.**

| The moment | The window A |
|---|---|
| The window B logs out with `l` | the screen of A does not change, and `users` holds **0 rows** |
| The key `l` of A on a media | `Syncing your last listening session. Please wait...` |
| **78 seconds later** | the same message, and **no media** |
| 40 presses of `l` after it | **the key `j` moves no line**, and **the key `Q` leaves the program standing** |

**The freeze comes of the shape of the wait.** The key gives its work to
`tokio::spawn`, and the wait holds `std::thread::sleep`: each press takes one
worker of the runtime for ever, and the machine of the measurement holds 32 of
them. The user of that terminal must take the program away with a signal.

**The correction is the rule of the fork for a read of the disk: a read must say
when it found no row.** `get_is_loop_break` and `get_has_played_before` give
`Option<String>` now, and `None` says that the account holds no row: **no loop
stands before that playback**, therefore it starts at once. The podcast played
at the second 0.07 after the correction.

**No wait of a playback stands longer than 30 seconds, and that is a decision.**
A program that dies inside the loop of its playback writes the end of that loop
never, therefore a wait with no limit can come back with a cause that no session
has met. 30 seconds is the time of this fork for a program that stood still
(T-140, T-148, and T-153).

**A test must not call a function that may never come back.** The first form of
the test held the gate of the machine for ever. `tests/a_playback_waits_for_no_account_that_is_gone.rs`
gives the wait a thread of its own and it reads the end of that thread with a
limit of time: the two tests **fail** with the correction removed, and they do
not hang.

### T-159, and the second fault of the same condition

**Every key that refreshes the screen makes a new application** (T-131), and
that application read the disk of an account that the disk does not hold:

| The measurement | The window A after the log out of B |
|---|---|
| The header | **`👋 Connected as `** — the program named nobody |
| The token | `Failed to decrypt the token`, and **every request answered**: the client of the start holds the token (T-131) |
| The key `S` on the line `Books` | `The library has been updated.` for a write of **0 rows**, and the header said `📖 Podcasts` after it |

**A log out that leaves a program of that account is no log out.** The
correction is the rule of T-142 and of T-155: the disk is the truth, and the
program reads it at the moment of the use. **A key of the user is that moment**:
the loop of `src/main.rs` reads `select_every_usr` after every key, and a
program whose account stands in no row of that list starts again with the
request of T-139 — the engine stops, the place of the playback reaches the
server, and `exec` gives the new program the terminal. The key `j` at the minute
14:44 of an episode gave `Item 9fa45bd1… closed at 884s`, and `GET /api/me` of
`curl` holds **884**. **The login screen of the new program says which account
went away**, and that sentence promises no key (T-118 and T-143).

**A read of the database for each key is the cost, and that is a decision.** A
key is an event of a person, and a view of this program reads `get_download` of
the database at every **frame** already (T-148).

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `l` of a media, after a second window logged out of the account** | **one fault** (T-158): no media for 78 seconds, and no key of the program worked after 40 presses |
| The same condition, after the correction | the media plays at the second 0.07, and the log names the account of no row |
| The wait of a playback whose loop wrote no end | **for ever** before, and the limit of 30 seconds after |
| **The key `R` after that log out** | **one fault** (T-159): a program of no name that holds the token of the account |
| The same condition, after the correction | the login screen, with the address of the server and the reason |
| **A key while a media plays, after that log out** | the place of the playback reaches the server: 884 s of `curl`, the same second |
| The rows of the queue and of the downloads of that account | **no fault**: they hold the name, and a login of that name finds them again (T-123) |
| **`cargo test`, the command of CI, three runs** | **no fault** |

## The session of the fourth turn of 2026-08-14: the accounts of two windows, and the key `X` of a playback

**Two releases: v0.7.90 and v0.7.91.** No condition of the road stayed,
therefore this session named two of its own, and **each of them held a fault**:
**the view of the accounts (T-124) with two windows of one account**, and **the
key `X` of a media that plays in the other window**. The prompt of the session
named both as parts that no measurement had reached.

| Item | What | Keys |
|---|---|---|
| T-155 | **One key took the start of the program from every account and gave it to nobody.** The login screen then came at every start, and the account of the user stood on the disk with its token | `S`, then `c` and `l` |
| T-156 | **The key `X` of one window removed the book that the other window played of the disk**, and the server was away: no key gave that book back | `l`, then `X` |
| T-157 | **The gate of CI failed one run of six**, and the gate of the machine passed: two tests of one binary shared the boxes of the authors | — |

### T-156, and the second condition of this session

**The condition needs the server away**: `play` reads the files of a download
only when no address answers. The window A played the book of 115200330 bytes in
the offline mode, and the key `X` of the window B removed the file and the row
at the minute 34 of that playback. **The playback of A went on** — the engine
holds the file open — and the key `l` of A on that media then said
`The server does not answer, and the disk has no copy of this media.`

**The key asked two questions and it asked nothing of a playback** (T-150). An
offline playback opens no session on the server (T-152), therefore
`listening_session` holds no row of it: that is the shape of T-142, of T-147, of
T-148, and of T-153. **The correction is the rule of the cache of the ebooks for
the audio** — "the book that the user reads now never goes away" (T-65 and
T-153) — and the heartbeat of it exists already: the loop of the offline
playback keeps the place of the user in `pending_progress` **at each second**
since T-152. A place that moved inside 30 seconds belongs to a playback that
runs, and **a mark of a playback is not for ever**: 35 seconds after the window
A went away, the key took the disk.

**The sentence names no program**, and that is a decision: no column of that
table holds a process, therefore a sentence of "a different program" would name
a program that this program does not know (T-154 and T-91).

### T-155, and it is the one to know of this session

**The list of the accounts comes of `App::new` alone.** The view drew that list
at every frame after it, therefore a window that stands open shows an account
that a second window removed, and it hides an account that a second window
added. That is the shape of T-142, of T-147, of T-148, and of T-153.

| The moment | The window A | The disk |
|---|---|---|
| Before | `▶ toutuitest`, `  toutuilimited` | both, and `toutuitest` holds the mark |
| B logs out of `toutuilimited` | **the two lines stay** | `toutuitest` alone |
| The key `c` of A on that line | `Press c again to start with the account "toutuilimited"` | — |
| **The second press** | **the login screen**: a server, a name, and a password | **`toutuitest`, and its `is_default_usr` is `0`** |
| A new window after it | **the login screen** | the same |

**One write of two lines is the cause.** `make_this_account_the_default` takes
the mark from every account and it then gives that mark to the account of the
name: a name of no row gives **0 rows** of the second write, and the transaction
commits all the same. `select_default_usr` reads `WHERE is_default_usr = 1
LIMIT 1`, therefore `src/main.rs` draws the login screen. The log said
`the account toutuilimited starts the program`, and no such account existed.

**The user needs the password of that server to come back**, and no key of the
program gives the mark back: that is the shape of T-136. The rows of the queue,
of the downloads, and of the positions hold the name of the account, therefore a
login of the same name finds them again.

**The correction is the rule of T-142: the disk is the truth, and the program
reads it at the moment of the use.** The view reads `select_every_usr` when it
opens, and the keys `c` and `l` read it before they act; **a key acts on the name
of its own line** and not on the place of that line (T-147). The write gives the
database back as it was when its name holds no row, and its caller starts the
program again no more. **A database that met the fault already must find its
account again**, and no key can do that work:
`an_account_takes_the_start_when_none_holds_it` gives the start to the first
account, and `src/main.rs` calls it before `Database::new`.

**Two faults of the words stood beside it.** The key `l` on such a line removed
0 rows and said **nothing at all** (T-79), and the view hid an account that a
second window added with the key `a`. Both keys say
`A different program of this account removed the account "…".` now, and that
sentence promises no key (T-118 and T-143).

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `c` of one window on an account that the other window removed** | **one fault** (T-155), and the program locks the user out of the account that stays |
| The same condition, after the correction | the key says the reason, the view holds the account of the disk, and the account of the start stays |
| The key `l` on such a line | **one fault of the words** before the correction: no row went away, and the program said nothing |
| A window that opens the view after the key `a` of a second window | **the fault before the correction** (the account did not stand in the list), and both accounts and the right mark after it |
| The start of a program whose database holds no account of a start | **the login screen** before the correction, and the Home view of `toutuitest` after it |
| **The key `X` of one window while the other window plays that media of the disk** | **one fault** (T-156), and the book of the user goes away while they listen to it |
| The same condition, after the correction | the key says `A program of this account plays "…" from the disk now.`, and the 115200330 bytes stay |
| The key `X`, 35 seconds after that window went away | **no fault**: the mark of a playback is not for ever, and the key takes the disk |
| **`cargo test`, the command of CI, eight runs** | **one fault** (T-157): one run of six failed, and `cargo nextest run` passed every time |

## The session of the third turn of 2026-08-14: the two downloads of one podcast

**One release: v0.7.89.** The road held one condition that no session had
measured — **the key `D` of this program and the queue of the downloads of the
server (T-81), in one library at one time** — and this session took it. **That
condition holds no fault of the data**, and a sweep of the keys inside it found
one fault of a key.

| Item | What | Keys |
|---|---|---|
| T-154 | **The second press of `D` on one media took the bar of that download off the screen**, and it said that a different program of the account downloads it. 58 seconds of a download with no sign of it | `D` |

### The condition of the road, and its answer

**The two sides write two files.** The server writes the audio of a new episode
in the directory of the library, and the program writes
`downloads/<user>/<episode>/`. The program downloaded "Letter 13" of 10050287
bytes while the server downloaded Letters 51 to 57, and it downloaded "Letter
15" of 10041092 bytes while the server downloaded Letters 40 to 49: **the file
of the disk and the file of the server give one sum of MD5 in both
measurements.**

**A new episode of the server stands at the end of `media.episodes`**, therefore
no line of the list of the program moves and the key `D` of a line takes the
media of that line. The header of the program says `R: the server has newer
data` while the episodes come, and the view of the key `d` shows the queue of
the server beside the bar of the download of the program.

**Two facts of the sandbox that the condition needs.** The server holds the 57
episodes of the feed already, therefore
`DELETE /api/podcasts/:id/episode/:episode?hard=1` on ten of them makes the work
of the server exist again; and the body of
`POST /api/podcasts/:id/download-episodes` is the **bare array** of the episodes
of the feed — an object of one field gives `400`.

### T-154, and it is the one to know of this session

**The map of the progress is global and its key is the media.** It must be
global: a refresh with the key `R` makes a new `App`, and a map inside `App`
would lose a download that runs (T-131). **The second task of the key `D`
therefore writes on the row of the first task.** `fetch_item` writes
`bytes_done = 0` and `Running` at its head, it then finds the lock of T-148 in
the hand of the first task, and it writes `Failed` on that row.
`render_downloads` draws a bar for each row of the state `Running` alone.

| The moment | The screen |
|---|---|
| the first press of `D` | `⬇ Letter 15  0.0 MB / 9.6 MB` |
| **the second press** | **no bar at all**, and `A different program of this account downloads "Letter 15" now.` |
| 58 seconds later | no bar, and no line of the screen names that work |

**The bytes are safe, and the user cannot see them.** The lock of T-148 keeps
one writer, and the file of the disk is the file of the server. A book of 700
megabytes gives an hour of a download with no sign of it.

**The words were wrong beside it.** The key `X` holds two sentences since T-150,
one of this window and one of the other window, and the key `D` held the
sentence of the other window alone: the program named a different program of the
account, and no such program existed.

**The correction is the rule of T-148 and of T-150: the map of the progress of
this process says which download this program runs.** `claim_the_download` reads
and writes that map under one lock, therefore two presses of one moment give one
claim. A row of the state `Running` gives `ThisProgramDownloadsIt`, the key
changes no field of that row, and the program says `This program downloads "…"
now.` **The claim stands before the request of the item**, therefore the bar
comes with the key and not with the first byte, and every road out of the
download gives the place back with `release_the_download`.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **A download of an episode while the same podcast downloads on the server** | **no fault of the data**, and one sum of MD5 on both sides in two runs |
| The list of the episodes while the server adds eight of them | the lines stay, and the header asks for the key `R` |
| **The key `D` two times on one media** | **one fault** (T-154): the bar of the download went away for 58 seconds, and the words named a program that did not exist |
| The same condition, after the correction | the bar stays and it grows, and the sentence names this program |

## The session of the second turn of 2026-08-14: the playback with no server, and the ebook of two windows

**Two releases: v0.7.87 and v0.7.88.** The road held three conditions that no
session had measured, and this session took **two** of them. **Both held a
fault, and each of them destroys work of the user.**

| Item | What | Keys |
|---|---|---|
| T-152 | **An offline playback that a program does not end lost every second of itself.** The server kept the place of the day before, and the next playback wrote that old place over the place of the disk | `l` |
| T-153 | **The window that gets an ebook took the book that the other window reads of the disk** — 545898521 bytes of one key | `e` |

### T-152, and it is the one to know of this session

**The sharp form of T-145.** The session of T-145 measured a program that dies
while the server **answers**: the row of `listening_session` then holds the
position, and the next program of the account sends it. **An offline playback has
no such row at all.** `play_offline` opens no session on the server, and no
request of that playback ever reaches the server: **the row of the disk is the
one copy of the whole playback.**

`follow_playback_offline` wrote the place of the user to the row of the download
at each second, and it kept that place for the server — the table
`pending_progress` — **at the end of the loop only**. A program that dies reaches
no end.

| The moment | The row of the download | `pending_progress` | The server |
|---|---|---|---|
| Before the playback | 100 s | no row | 100 s |
| The offline playback runs | 1731 s | **no row** | 100 s |
| The terminal goes away | 1731 s | **no row** | 100 s |
| The server answers again, and the program starts again | 1731 s | no row | **100 s** |

**27 minutes went away, and the one copy went away with them.** The user then
played that book with the server up: the program took the 100 seconds of the
server, and the loop wrote 100 over the 1731 of the row of the download.

**The correction is the rule of the loop of the online playback**: "Write the
position for each second. A crash must not lose it." The loop of the offline
playback keeps the place for the server at each second now. `INSERT OR REPLACE`
gives one row of a media whatever the number of the calls.
`logic::offline::keep_progress` writes the row and **says no line of the log**:
one line for each second gives 28800 lines for a book of eight hours.

The same measurement after the correction gives **1154 s on the server**, and the
log of the start says `[offline] the server took the position 1154s of 6ba57b9a…`.

### T-153, and the second condition of the road

**`keep` of the removal of the cache of the ebooks is a fact of the process**,
and one account holds more than one program (T-140): the window A read a book of
a scan of 502745447 bytes, the window B pressed `e` on a book of its own, and the
log of B says `the cache of the ebooks gave 545898521 bytes of 2 book(s) back` —
the book of A and the 43016313 bytes of its pages of T-62, **under the reader of
A**. That is the shape of T-148 and of T-150, and **the module of the cache says
the rule that it broke**: "The book that the user reads now never goes away."

**The user loses no line and no place**: the reader of A holds the book in the
memory of its process. **The user loses the bytes of the disk**: the next start
of A asks the server for 502 megabytes again and it waits 131 seconds for the
child of T-62, and a user with no server has no book at all.

**The correction is the rule of the fork: the disk is the truth.** The reader
writes the time of its file every 15 seconds, and the removal keeps every book of
a time inside 30 seconds. **The time of the file is the one word that two
programs of one account share here**, and `the_book_is_in_use` wrote that word at
the open of a book since T-67 already: the correction gives it a heartbeat, and
that is the rule of T-140 and of the lock of T-148. **It needs no new file, no
call of the system, and no dependency.**

**A mark of a reader is not for ever**: a window that goes away writes no more
marks, and the next removal takes its book 30 seconds later.

**The cache can stand above its limit while the user reads**, by one book for
each program that holds a reader open. **That is a decision of this session**,
and the head of the module says it: the module said the same of `keep` already —
one book of 500 megabytes is a correct cache of one book — and this is that rule
for an account of two windows.

**The first form of that condition gives no fault of the data.** The two windows
opened the **same** book: the two children of T-62 each read the 150 pages in 131
seconds, the second one renamed its file over the file of the first one, and the
bytes of the disk are the bytes of the server. **The cost is two**, and a machine
of less memory would meet the two peaks of about a gigabyte at one time.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **A program that dies while the server does not answer** | **one fault** (T-152), and the whole playback goes away |
| The same condition, after the correction | the server holds the place of the playback at the next start |
| **Two windows that open one ebook at one time** | **no fault of the data**, and a cost of two children of T-62 |
| **A window that gets an ebook while the other window reads one** | **one fault** (T-153), and it takes 545898521 bytes of the disk |
| The same condition, after the correction | the removal takes nothing, and the time of the file moves every 15 seconds |

## The session of 2026-08-14: the key `X`, the downloads, and the queue

**Two releases: v0.7.85 and v0.7.86.** The road held three conditions that no
session had measured, and this session took **two** of them.

| Item | What | Keys |
|---|---|---|
| T-150 | **The key `X` said "holds no local copy" for 115 megabytes on their way, and it left the bytes of a download that stopped on the disk for ever.** No key of the program removed them | `D`, `X` |
| T-151 | The key `X` of the view of the queue said nothing at all when a different program took the media of that line out first | `q`, `X` |

### T-150, and it is the one to know of this session

**The session of T-148 could not measure this condition**: a download of 115
megabytes over the loopback ends in less than one second (the trap 111).
`docs/harness/slow_body.py` gives the delay of the **body** of the answer, and the
key `D` takes the address of the pool since T-149: a proxy of 0.05 seconds for
each block of 64 kilobytes in the block `[[servers]]` of `config.toml` makes that
download take about 90 seconds, and the two keys then meet.

**`remove_download` read the database, and the database holds a row after the last
byte of the last file.** A download that runs and a download that stopped
therefore stand in no row at all:

| The measurement | Before | After |
|---|---|---|
| The window B presses `X` while A downloads | `"…" holds no local copy and no ebook.`, and the 115200330 bytes came 60 seconds later | `A different program of this account downloads "…" now.`, and the `.part` file stays |
| The window A presses `X` while A downloads | the same sentence of no copy | `This program downloads "…" now.` |
| The key `X` on the 7713867 bytes of a program that died | **the bytes stay for ever**, and no key of the program removes them | `Removed 7 MB of a download of "…" that did not come to its end.` |

**The rule is the rule of T-142, of T-147, and of T-148: the disk is the truth.**
`remove_the_directory_of_the_download` takes every file of the directory of the
download — the audio, the `.part` file, and the lock — whatever the database
holds. **A download that runs holds its files, and the key takes none of them**: a
removal under a writer gives that writer the `cannot rename` of T-148 from the
other side. The lock of T-148 says that a program writes them, and the map of the
progress of the process says which program that is.

**No key of this program stops a download that runs, and that is a decision.** A
key that stops it needs a map of the handles of the tasks of this process, and it
reaches the download of the other window never. The sentence therefore promises no
key that the program does not hold (T-118 and T-143).

### T-151, and the first condition of the road that found no fault of the data

**The queue holds.** The window B took the next media of the queue out with `X`
while A played, and A then started the media that B left: `the queue starts "A
Book Of Many Hours", and 0 media wait`. A key of a view that is older than the
disk took the media of its own line, and not the media of that place (T-147).

**One fault of the words stands beside it.** The key `X` of A on a line whose
media B took out **returned with no message**, and the list of that view lost the
line all the same: the user reads one media less and no word, and they cannot tell
the key that worked from the key that did nothing (T-79). `text_of_the_key_that_takes`
gives one sentence for both roads, and it names no program — this program cannot
say which program took that media out (T-91).

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The key `X` of one window while the other window downloads** | **two faults of one cause** (T-150), and one of them keeps the disk of the user for ever |
| The key `X` after that download came to its end | **no fault**: the directory and the row both go away |
| **A queue of media that a second program plays** | **no fault of the data**, and one fault of the words (T-151) |
| The key `X` of a view of the queue that is older than the disk | **no fault**: the media of the line goes, and the media beside it stays |

## The session of the sixth turn of 2026-08-13: the downloads of two programs

**Two releases: v0.7.83 and v0.7.84.** The road held three conditions that no
session had measured, and this session took the first of them: **the downloads,
with two programs of one account**. That is the third state of the shape of T-142
and of T-147, and the two states before it each held a fault. **This one holds a
fault too, and it destroys a file of the user.** The measurement of that fault
then found a second one in the same key.

| Item | What | Keys |
|---|---|---|
| T-148 | **Two windows wrote one file of a download at one time, and the book of the user held audio that no decoder reads.** The program said that the media is available offline | `D` |
| T-149 | **The download went to the address of the login, and it waited for ever.** Every other request of the program takes the address that answers | `D` |

### T-148, and it is the one to know of this session

The key `D` spawns a task, and **the map of the progress is a map of the
process**: no program can see the download of another program. The two writers
then meet in one directory, and the second one adds its bytes to the **end** of
the file of the first one (`resume_from` gives the bytes of the disk, and the
answer of a `Range` goes to that file with `append`).

Each writer counts its own bytes, therefore the guard `written != file.size` of
`fetch_one` passes for both of them.

The measurement, on a book of eight hours whose file holds 115200330 bytes:

| The measurement | The answer |
|---|---|
| The bytes on the disk, four runs of two windows | **116576586, 115200330, 117316426, 117123402** |
| The first 115200330 bytes of the file of 116576586 | the file of the server, `ef133993…` |
| The audio, with `ffmpeg -f null -` | **8:07:24**, and `Header missing` with `Invalid data found` |
| The audio of the file of the server | 8:00:00, and no line of a fault |
| The two screens | **`[Downloaded]`**, and one of them said "is now available offline" |

**The offline mode plays that file when the server is away**: this is the copy
that the user has at the moment that they have nothing else.

**The correction is the rule of T-142 and of T-147 for a file: the disk is the
truth.** `logic::download::lock` makes the file `.the-program-of-the-download`
inside the directory of the download with `create_new`, before the first byte,
and the lock goes away with its value at every return of the task.

**A program that died leaves its lock, therefore a lock is not for ever.** The
rule is the heartbeat of T-140: the time of the lock **and the time of every
`.part` file of that directory** say when that program last worked, and a
download that stood still for 30 seconds belongs to a program that is gone. A
download of an hour holds its lock for that hour, because its file grows at each
block. **This needs no call of the system and no dependency.**

Four runs after the correction give 115200330 bytes and the sum of the server,
and the second window says "A different program of this account downloads … now."
**That sentence must not say "failed"**: the download of the user is on its way.

**Two presses of the key `D` of one program held the same fault**, and the log of
that measurement says `cannot rename` for a download that worked.

### T-149, and the fault that the harness of T-148 found

The proxy of the measurement held the address of the login, and **the header of
the program said a different address**: `pool` decides the address of every
request of the program (T-105 and T-128), and the key `D` held
`self.server_address`.

| The measurement | Before | After |
|---|---|---|
| The requests of the key `D`, with a proxy on the address of the login | `GET /api/items/:id` and the file: **the address of the login** | **no request at all** |
| The key `D` toward an address that accepts a connection and answers nothing | **no message, no line of the log, and no bar, for ever** | `Download failed for …: the request failed`, at the second **15** |

`reqwest::Client::new()` of `logic::download` held **no limit of time at all**.
The client holds 3 seconds for the connection and **30 seconds with no byte** of
the answer now, and the request of the list of the audio files takes the 15
seconds of every other request. **A download must hold no limit of its whole
time** (T-119: 36 seconds for a book of 479 megabytes).

**A port that no program holds refuses a connection at once**, and it therefore
says nothing of a limit of time: the measurement needs a port that accepts the
connection and answers nothing (the trap 112).

### The measurements of this session

| The measurement | The answer |
|---|---|
| **The downloads, with two programs of one account** | **two faults** (T-148 and T-149), and T-148 destroys a file of the user |
| The two programs after the correction | one file, and it is the file of the server. The second window says which program holds the download |
| The key `X` of one window while the other window downloads | **no measurement**: the download of 115 megabytes over the loopback ends in less than one second, and the two keys took their sequence in every run |
| The `[Downloaded]` mark of a second window | **no fault**: every view reads `get_download` of the database at the frame that it draws |

## The session of the fifth turn of 2026-08-13: the queue of the media

**Two releases: v0.7.81 and v0.7.82.** The road held two conditions that no
session had measured, and this session took **both of them**, and both of them
hold the same list: **the queue of the media**. Each of them found a fault, and
each fault takes a media of the user away for ever.

| Item | What | Keys |
|---|---|---|
| T-146 | **A server that went away in the middle of a queue took one media of the queue with it.** The queue took the media out before the playback started, and the playback did not start | `n`, `l` |
| T-147 | **A second window wrote its own queue over the media of the first window.** Each screen said "The queue [1 item]" with its own media, and the disk held one of the two | `n`, `q` |

### T-146, and it is the one to know of this session

The queue removes the media **before** the playback of that media starts, and a
playback that did not start left nothing at all: the media stood in no list, and
no key gave it back.

The measurement, with a queue of two books and `podman stop -t 0 abs-test` in the
middle of a book of 30 minutes:

| The moment | The queue |
|---|---|
| Before the playback | **2 items**: One Chapter Book, Alice in Wonderland |
| The book comes to its end | the log says `the queue starts "One Chapter Book", and 1 media wait` |
| The server does not answer, and the disk holds no copy | the message says so, and the queue **stops** |
| The view of the queue, and the disk after it | **1 item**. **One Chapter Book is gone** |

**The media goes back to the front of the queue now**, and the queue stops there.
`the_media_goes_back_to_the_queue` reads the outcome `Fault` — the outcome that
says that no audio played at all.

**The queue must not go on to the media after the fault, and that is a decision.**
A server that does not answer gives the same fault to every media of the queue,
therefore a queue that goes on empties itself in one second. The head of
`crate::logic::queue` promised the opposite ("the queue then goes on to the media
after it"), and no line of the code ever said it: the text is right now.

**The key `l` of the view of the queue is the second door of the same rule**, and
it took the media with `take_at` before it played it.
`play_the_media_of_the_queue` takes the whole entry for that key, and `play`
takes the target alone.

**A media of the queue that the disk holds still plays while the server is
down**, and the queue goes on to it: "the offline mode plays Multi File Test Book
at 60 seconds with 3 track(s)", and the queue then stops at the media after it
that the disk does not hold.

### T-147, and the second state of the road

**The road named three states that a second program cannot see** (the queue, the
cache of the ebooks, and the downloads). The cache holds the rule of T-142
already, and **the queue held the fault**.

`write_the_queue` holds **every** row (T-56), and the queue of the process stood
beside it: the write of one program took the media of the other program away.

| The moment | The window A | The window B | The disk |
|---|---|---|---|
| A presses `n` on "One Chapter Book" | 1 item: One Chapter Book | — | One Chapter Book |
| B presses `n` on "Multi File Test Book" | 1 item: One Chapter Book | 1 item: Multi File Test Book | **Multi File Test Book alone** |

**The book of A is gone, and the screen of A still names it.** The correction is
the rule of T-142: the disk is the truth, and the program reads it at the moment
of the use. Every function that changes the queue reads the disk first, and the
view of the queue reads it when it opens.

**A key of a view that is older than the disk takes the media of its own line.**
`the_place_of_the_media` gives the place of the line when the media of that place
agrees, and the first media of that identity otherwise.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **A queue of media while the server goes away in the middle of it** | **one fault** (T-146), and it takes one media of the queue away for ever |
| A media of the queue that the disk holds, while the server is down | **no fault**: the offline mode plays it, and the queue goes on |
| The key `n` while the server does not answer | **no fault**: the media goes in the queue, and the disk holds it |
| **Two programs of one account, and each of them puts a media in the queue** | **one fault** (T-147), and the disk held one media of the two |
| The two programs after the correction | both windows and the disk hold both media, in the sequence of the user |
| The key `X` of one window, and the view of the other window after it | the two windows agree |

## The session of the fourth turn of 2026-08-13: the terminal that goes away, and the machine that sleeps

**One release: v0.7.80.** The road held three conditions that no session had
measured, and this session took the first of them: **a media that plays while
the terminal goes away (`SIGHUP`), and while the machine sleeps**. The sleep of
the machine gives no fault. **The death of the terminal takes the place of the
user away, and the disk held that place the whole time.**

| Item | What | Keys |
|---|---|---|
| T-145 | **The terminal went away with 1026 seconds of a book on the disk, and the next program removed that row with no request.** The server stayed at 872 seconds for ever | `l`, then `Q` |

### T-145, and it is the one to know of this session

The loop of the playback writes the row of `listening_session` **every second**,
and the line of the source says why: "Write the position for each second. A
crash must not lose it." **The crash loses it.**

`sync_session_from_database` held two rules that do not agree: it closes **one**
session (`LIMIT 1`), and it removed **every** row that this program may take. A
user who starts the program again **inside 30 seconds** meets both, and that is
what a user does when the terminal of their program went away:

| The moment | The row of the disk | The server |
|---|---|---|
| The terminal goes away | **1026 s** | 872 s (the sync of ten seconds) |
| The user starts the program again at once, and plays a second book | the row stays, and the log says "The database holds no session to close" | 872 s |
| The key `Q` | **the table holds no row at all** | **872 s**, for ever |

The row of the program that died is **too young** for the rule of T-140 at the
moment of the key `l`, and it is **old enough** for the removal at the moment of
the key `Q`.

**The program closes every row that it may take now**, one after the other, and
it removes a row **after** that row reached the server. The rows of a program
that died go first, and the row of this program goes last: two rows of one media
then leave the newest position on the server. `delete_listening_session` has no
caller left, and it went away: **a removal of every row of an account is the
fault itself**.

The same measurement after the correction: `Item 6ba57b9a… closed at 1026s` and
`Item e2b76945… closed at 1771s`, and the server holds 1026 for the book of the
program that died.

**What stays, and it is a decision.** The row of a program that died is hidden
for 30 seconds (T-140), therefore a user who plays **the same book** again at
once hears the ten seconds of the sync a second time. The position is safe, and
the last ten seconds need the program to know that the process of `owner` does
not live: **that needs a call of the system for each program**, and the decision
of T-140 keeps it outside.

### The machine that sleeps, and it gives no fault

**`SIGSTOP` does not reach a program of tmux from a session of this harness**
(the trap 101). The freezer of the cgroup does, and it is the better model of a
suspend: `echo 1 > /sys/fs/cgroup<the scope of the program>/cgroup.freeze`.

| The measurement, with 120 seconds of sleep | The answer |
|---|---|
| The position, while the program sleeps | 536 s, and it does not move: **no clock of the wall stands in the loop of the playback** |
| The playback after the wake | it goes on, and the sync reaches the server |
| The connection of the live messages | it ends, and it is open again **10 seconds** later with no key of the user |
| The row of the database, and the key `Q` after the wake | the same row and the same owner, and the session closes at 1331 s |

**The limit of that measurement:** the device `null` is not a real sound device.
A suspend that takes an ALSA device away needs the sound of the machine of a
user, and no run of a session plays sound.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **A media that plays while the terminal goes away (`SIGHUP`)** | **one fault** (T-145), and the place of the user of that playback goes away for ever |
| The same condition, after the correction | the place of the program that died reaches the server at the next close |
| **A media that plays while the machine sleeps** (120 s of a freezer of the cgroup) | **no fault** of the position, of the playback, of the row, or of the live connection |
| The open sessions of the server, after a program died | **the server keeps them**: seven sessions of one book stood open after the sweeps of this session. The correction closes the newest of them, and the sessions of the days before it stay |

## The session of the third turn of 2026-08-13: a setting of two programs

**One release: v0.7.79.** The road held three conditions that no session had
measured, and this session took the first of them: **two programs of one account,
and one of them changes a setting with the key `S`**. The road expected a fault of
the file, and the file holds every line. **The fault takes the books of the user
of the disk.**

| Item | What | Keys |
|---|---|---|
| T-142 | **A second window removed two books of 105 MB while its own screen said the value that keeps them.** The limit of the cache came of the moment of the start | `S`, `e` |
| T-143 | The key `h` did nothing in the view of the cache of the ebooks, and the footer promised it | `S`, then `h` |
| T-144 | **The gate of the machine passed and the gate of CI failed.** Three binaries of the tests share a database, and nextest gives each test a process of its own | — |

### T-142, and it is the one to know of this session

**The key `S` writes one value of `config.toml`**: the limit of the cache of the
ebooks (T-77). `write_the_value` reads the file and it changes one line,
therefore **no program loses the line of another program** — that is the fault
that the road expected, and it does not exist.

**The limit stood in three places of one program**, and the third one never moved:

| The place | Who writes it | What it does |
|---|---|---|
| `config.toml` | every program of the account | the value of the user |
| `self.config` | `App::new`, therefore the start **and every refresh** | the title and the mark `✓` of the view |
| a slot of `logic::reader::cache` | `src/main.rs`, **one time** (T-72) | the removal of a book of the disk |

The measurement, with a cache of 447 megabytes on the disk:

| The moment | Before | After |
|---|---|---|
| A takes 4096 MB with `S`, and B reads its own view | B says **"512 MB now"**, and the file says 4096 | B says "4096 MB now" |
| B gets one book with `e` | B **removed two books of 105386785 bytes** at 512 MB | no book goes away |

**The screen of B promised 4096 MB at that moment, and B removed the books at
512.** The rule: the file is the truth, and the program reads it at the three
moments that it needs it — a new application, the view of the settings, and
**before a removal takes a book of the disk**. A removal comes with no key of
that window, therefore the two first moments are not enough.

### T-143, and the first key of that measurement found it

The footer of the view of the cache says `h: back`, and three presses of `h`
moved nothing: `AppView::SettingsReader` came with T-77 with an arm of
`toggle_view` and no arm of the key `h`. The next key of the measurement was
`Esc`, and **that key stops the program** (the trap 69): the window of the
measurement went away.

`tests/the_key_h_leaves_every_view.rs` reads the source, it names every view of
`AppView`, and it holds each of them to an arm of that handler. Three views stand
outside the rule (Home, Library, and the reader of an ebook), and the test named
`SettingsReader` and no other view.

### The measurements of this session

| The measurement | The answer |
|---|---|
| **Two programs of one account, and one of them changes a setting with `S`** | **two faults** (T-142 and T-143), and T-142 removes the books of the user |
| The lines of `config.toml` after the two programs wrote it | **every line stays.** `write_the_value` reads the file, and it changes one line (T-77) |
| The row of the account of the database, with two programs | **no fault of a lost value**: every write of `users` names its own column, and the speed rate is a relative write |
| The two programs after the correction | one value of the limit, in the file, on the screen of both windows, and in the removal |
| **`cargo test`, the command of CI** | **six tests of three binaries failed** (T-144), and `cargo nextest run` gives every one of them |

## The session of the second turn of 2026-08-13: two programs of one account

**Two releases: v0.7.77 and v0.7.78.** The road held no new condition after
T-138 and T-139 — it said that a next session must name one of its own. This
session named **two programs of one account, on one database, while a media
plays**: a user starts the program in two terminals, and every measurement of
every session before this one ran one program.

**The condition found two faults, and each of them takes the place of a user
away.**

| Item | What | Keys |
|---|---|---|
| T-140 | **Two programs of one account destroyed the place of both users.** One row of the database stood for one account, and the two programs shared it | `l`, `Q` |
| T-141 | **A media that came to its end left its row**, and the key `Q` sent that end again over a place that a different client wrote later | `Q` |

### T-140, and it is the one to know of this session

The measurement, with a book of eight hours in each program:

| The moment | Before | After |
|---|---|---|
| B plays its own book, while A plays | B closes the **live session of A** on the server, and it removes the row of A | B leaves the row of A |
| The key `Q` of A, while B plays | A sends the position of **the book of B**, and it removes the row of B | A closes its own session |
| The key `Q` of B | "The database holds no session to close" | B closes its own session |
| The server | the book of A: **73 s** of 114, the book of B: **0 s** of 116 | 107 s and 108 s |

**The cause is the rule of T-4.** `play_media` closes the session that the
database holds before it opens its own, because a program that stopped without a
correct exit leaves a row. **That rule cannot tell the row of a program that died
from the row of a program that lives**, and the answer is therefore the identity
of the program: `owner` holds the process, `heartbeat` holds the moment of the
last second of that playback (the version 9 of the schema), and a program takes a
row of its own or a row that stood still for 30 seconds. **This needs no
dependency**, and the rule of T-20 holds.

**The key `Space` shared that row too**: `handle_key_player` reads the session of
the account, therefore the key of one program wrote the mark of the pause of the
other one. The rule of the owner corrects that key with no line of its own.

### T-141, and the fault that stood beside it

A book played to its end and the program stayed open: the row held `t=28800` and
`finished=1`, **and the server held the same values**. The position was safe, and
the row stayed. A different client then marked that book "not finished", and the
key `Q` sent 28800 seconds and "finished" again — **the newer place of the user
went away.** The loop removes the row of its own playback now, and a server that
refused the position keeps it (T-25).

### The measurements of this session

| The measurement | The answer |
|---|---|
| **Two programs of one account, one database, while a media plays** | **two faults** (T-140 and T-141), and each of them takes the place of a user away |
| The two programs after the correction | each program holds its own session, and both places reach the server |
| A media that comes to its end, and a place that a different client writes after it | the place of the other client stays |

## The session of 2026-08-13: two faults of the maintainer, the road, and the last two sweeps

**Five releases: v0.7.72 to v0.7.76.** The maintainer named two faults, and both
of them stopped a user of a first start. The session then took the two items of
the road that a measurement could reach, and it made **the last two sweeps of the
road**: a library that the account may not read (T-136), and two accounts of two
servers while a media plays (T-138 and T-139). The maintainer also named a fault
of the covers of a refresh; it did not repeat, and the tool that a next session
needs for it stands in `docs/harness` now (T-137).

| Item | What | Keys |
|---|---|---|
| T-133 | **A program that a user builds keeps no token, and it stops with a screen of no character.** `install.sh` made the secret key, and no other way to the program did | the login |
| T-134 | **The cursor of the terminal stood six rows below the field of the user.** The message of the login stood outside the frame of ratatui | the login |
| T-135 | The key `R` took the timer for sleep of the user away, and the media played on | `R`, `S-Tab` |
| T-136 | **An account that loses a library could not use the program again**, in any view and after every start | every key |
| T-137 | **A harness that reads the covers of the graphics protocol.** The fault of the covers that the maintainer named did not repeat, and no session could see a cover before this tool | `R` |
| T-138 | **The place of one account went to the server of another account, and the program then destroyed it** | `S`, then `c` |
| T-139 | The place of a playback did not reach the server before a key started the program again | `S`, then `a` and `c` |

### T-133, and it is the one to know of this session

**The maintainer met it, and no session had met it**: every measurement of every
session before this one wrote `.env` by hand, because `docs/TEST-SERVER.md` said
so. A user who builds the program with `cargo`, with `nix`, or with a package of
their system has no such file, therefore **`encrypt_token` failed for every one
of them**.

The screen of no character came of a `println!` of the thread of the login: the
login screen holds the lock of the standard output while it waits for that thread
with `join`, therefore the two threads waited for each other for ever. **A
`println!` of a thread of the login stops the program**, and no line of
`auth_process` writes to the terminal now.

The program makes the key itself at the start, when the machine has none. It
reads `.env` first, therefore it never makes a second key: a new key makes every
token of every account unreadable.

### T-136, and the sweep that the road named in three sessions

The account `toutuilimited` of the sandbox reads one library of the five. **An
administrator can take that library away while the program holds it**, and the
program then held a library that answers `403`: the header said `📖  ()`, every
view said "This library holds no media", the key `S-Tab` moved to nothing, and a
new start gave the same screen. **No key gave the user the library that they may
read.**

The start is the place of the answer: `App::new` takes the first library of the
account when the library of the database is not one of them, and it says so. The
key `S-Tab` keeps its rule, because a key must not guess.

Two traps of the API came with the sweep, and `docs/TEST-SERVER.md` holds them: a
`PATCH` of an account takes `librariesAccessible` **inside `permissions`** only,
and an empty list of libraries is **every** library.

### T-137, and the tool that a next session needs for a cover

**The maintainer named a fault of the covers of a refresh, and neither the
maintainer nor the measurement could repeat it.** The tool is what stays: **a
screen of tmux holds no byte of a cover**, therefore every session before this
one measured blocks of Unicode and never the protocol of the terminal of a user.

`docs/harness/kitty.sh` drives the program in a real window of kitty, and
`docs/harness/covers.py` reads the identity and the place of every picture of the
screen out of the **unicode placeholders** of that protocol. It says the one
thing that names the fault: **a placeholder of a picture that the program did not
send**.

Nine measurements of that day gave no fault: three refreshes one after the other,
a refresh 150 milliseconds after a key that asks for a new cover, four pictures at
one time with a media that plays, and a refresh in that state. **kitty takes a
picture away with its placeholders**, and its memory did not move for 27 pictures.
T-137 of `docs/TAKEOVER-BACKLOG.md` holds the table, and it holds the three
questions for the maintainer when the fault comes again.

### T-138 and T-139, and the last sweep of the road

**The sweep of two accounts of two servers while a media plays is made now**, and
it is the last sweep of the road of 2026-08-12. **It found two faults, and both
of them take the place of a media away.**

- **T-138 destroys the place of the user.** One row of `listening_session` stood
  for the whole program, and it held no account: the next media of a second
  account sent the place of the first account to **its own** server, the server
  answered "The server does not have this item", and the program then removed the
  row. The row holds the account and the server now (the version 8 of the
  schema), as the queue of the version 7 does.
- **T-139: a key of the accounts started the program again with no word to the
  server.** The key `a` at the minute 13:31 of a book left the server at 13:23.
  `exec` takes every task away, therefore **the loop of `src/main.rs` closes the
  session before it starts the program again**, and a key handler only writes the
  request in `the_program_starts_again`.

The measurement after the correction: the key `a` at 13:43 gives the server 823
seconds, the same second. A session that waits stays while another account plays,
and it reaches its own server when its own account plays again.

### The measurements of this session

| The measurement | The answer |
|---|---|
| A first start with a configuration directory of no file | **one fault** (T-133), and it stops the program |
| The cursor of the terminal of the login screen | **one fault** (T-134), at every moment of that screen and not only with a message |
| The key `t` and then the key `R` | **one fault** (T-135): `💤 4:58` before, and no timer after |
| An account of the type `user` that reads one library of five | **one fault** (T-136), and it locks the account out for ever |
| The covers of a refresh, in a real window of kitty | **no fault** in nine measurements (T-137), and the tool of that measurement stays |
| **Two accounts of two servers while a media plays** | **two faults** (T-138 and T-139), and T-138 destroys the place of the user |

## The session of the seventh turn of that day: the address, the rounds of the start, and the playback

**Four releases: v0.7.68 to v0.7.71.** The session took the first three items of
the road, and **the sweep of the third one found the fault of this session**: a
refresh of the screen took the playback of the user away from every key.

| Item | What | Keys |
|---|---|---|
| T-128 | **The program said "No server address answered" for a server that answers.** One connection that no machine took kept the address away for up to 60 seconds | any |
| T-129 | The four requests of the start waited for the shelves of the Home view, and they need that answer for nothing | — |
| T-130 | **The program sent the position of a playback as a text.** The server kept that text, and the row of the answer then did not read at all | — |
| T-131 | **The key `R` took the playback away from every key of the player.** The row of the player went away, and the book played to its end | `R`, `S-Tab` |
| T-132 | The test of the live messages measured the data of the sandbox | — |

### T-128, and the answer that a request must try

**The road of the session before this one named this measurement first**, and it
did not repeat for that session. **It repeats every time now**: the server goes
away for 25 seconds, the live task cannot connect, and the one address of the
pool takes the state `Down` (T-107). The server then answers `curl` in 1.5
milliseconds, and **16 presses of the key `e` in the 31.6 seconds after it all
said "No server address answered"**. The probe task sleeps 60 seconds, therefore
the limit of that wait is a minute.

**A request must try an address before the program says that no address
answered**, and an address that answered holds the state `Up`. The first press of
the key gives the book now, 120 milliseconds after the server answered.

**`mark_down` wrote no line of the log at all**, therefore the measurement had to
read the fault of the live task and make a guess. The log holds that moment now,
with the reason.

### T-131, and it is the one to know of this session

**Every key that refreshes the screen makes a new application**: the key `R`, the
key that takes the next library (T-66), and the keys of the sequence of the
library. **`App::new` starts a new engine of the sound**, therefore the old engine
kept the playback and the new application knew nothing of it: the row of the
player went away while the media played, and **the keys `Space` and `Y` acted on
an engine of no playback**. The measurement pressed `R` at the minute 2 of a book
of 30 minutes, and the book played to its end while the user held no key of it.

`App::new_with_the_engine` takes the engine of a program that plays already.
**The rule of the loop of `src/main.rs` needs a test that reads the source**,
because no unit test reaches that loop.

### T-129, and the way to measure the rounds of the start

**A proxy of 70 lines of Python answers a question that no session had answered.**
It holds a port, it gives every request a delay of 500 milliseconds, and **it
writes the path and the time of each request**: the rounds of the start then stand
in a file, and no line of the program changes. The start held three rounds, and it
holds two:

| The round | Before | After |
|---|---|---|
| 1 | `GET /api/libraries` | `GET /api/libraries` |
| 2 | the shelves, and `GET /api/me` | the shelves, the account, the series, the lists, and the items |
| 3 | the series, the lists, and the items | — |
| **the first frame** | **2.03 s** | **1.56 s** |

**The first round stays**: every request of the second round needs the identity of
the library, and the first round gives it.

### T-130, and the fault that a sweep found in the log

**The sweep of a media that plays with a slow server found one line of the log**,
and that line named a fault of five sessions: `POST /api/session/:id/sync` sent
`{"currentTime": "714"}`, **two numbers as a text**. The server keeps the form
that a client gives it, therefore `GET /api/me` answered a text and the row did
not read. The line of the Home view of a book at the minute 11 of 30:
`➤ 40% A Long Test Book` with the correction, and `➤     A Long Test Book`
without it.

**The program reads a number that comes as a text now too.** The rows of a text
stand in the database of every server that this program wrote to, therefore the
correction of the request alone gives those users nothing.

### The sweeps of this session, and what each of them gave

| The sweep | The answer |
|---|---|
| **A server that answers slowly while a media plays** | **one fault** (T-130), and the playback itself held: the book played from the minute 20 to its end with a proxy of 500 ms |
| **A library of more than 500 items, with a media that plays** | **one fault** (T-131). The paging of T-70 read the four pages of 2056 items while the media played, and the key `G` went to the last item |
| **A server that goes away and answers again** | **one fault** (T-128) |

**Two sweeps of the road stay**: a second account of a second server while a
media plays (T-124), and a library whose media the account may not read, with an
account of the type `user` (T-121 holds the commands of such an account).

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

`main` is clean and pushed, and `v0.8.0` is tagged. Every gate passes:

```
nice -n 19 ionice -c 3 cargo clippy --all-targets -j 16 -- -D warnings
nice -n 19 ionice -c 3 cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> nice -n 19 ionice -c 3 cargo nextest run -j 16
    # 1087 of 1087 in 2.3 s, and cargo nextest run --run-ignored all gives 1112
    # of 1112 with the sandbox up, in 18.1 s of wall clock: one test waits 16 s
    # for the time limit of the send of a book (T-119), and one waits 15 s for
    # the time limit of a request
ALSA_CONFIG_PATH=<the same file> nice -n 19 ionice -c 3 cargo test -j 16 --no-fail-fast
    # **CI runs this command**, and it is not the same run as nextest: it passed
    # three times of three. nextest gives each test a process of its own, therefore it
    # hides a test that shares a database with another test of its binary. Six
    # tests of three binaries failed on CI while nextest passed (T-144), and
    # `--no-fail-fast` says every binary that fails. **It found a fault of the
    # session of T-164 at the run 1 of 4**: two test functions of `src/logic/message.rs`
    # fought for the slot of the process of the message, and the module of that
    # slot says the rule in its own comment — "the parts of this test stay in one
    # function". They are one function now, and five runs after it gave no fault.
    # **The file of `ALSA_CONFIG_PATH` needs two lines only**: `pcm.!default {
    # type null }` and `ctl.!default { type null }`. No session before this one
    # said where that file stands, and each of them made it again.
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

**The first three items of the road of the session before this one are done**
(T-128, T-129, and T-131 with T-130). The road holds the work that the
measurements of this session left.

1. **Every sweep of the road is made now, and a session names its own
   condition.** Every new condition found a fault in twelve sessions of
   thirteen:
   - ~~**Two programs of one account, on one database, while a media plays**~~:
     **made on 2026-08-13 (the second session), and it found T-140 and T-141.**
     The session named that condition itself, because the table held none.
   - ~~**Two programs of one account, and one of them changes a setting with the
     key `S`**~~: **made on 2026-08-13 (the third session), and it found T-142 and
     T-143.** The file loses no line, and the second window removed the books of
     the user with the limit of its own start.
   - ~~**A media that plays while the machine sleeps, or while the terminal goes
     away (`SIGHUP`)**~~: **made on 2026-08-13 (the fourth session), and it found
     T-145.** The sleep of the machine gives no fault, and the death of the
     terminal took the place of that playback away: the disk held it, and the
     next program removed that row with no request.
   - ~~**A queue of media while the server goes away in the middle of it**~~:
     **made on 2026-08-13 (the fifth session), and it found T-146.** The queue
     took the media out before the playback started, and the media of a playback
     that did not start went away for ever.
   - ~~**A state of the program that a second program cannot see**~~: **made on
     2026-08-13 (the fifth session) for the queue, and it found T-147.** Two
     windows each put one book in the queue, and the disk held one of them.
     **The three states are measured now**: the cache of the ebooks holds the
     rule of T-142, the queue holds it since T-147, and the downloads hold it
     since T-148.
   - ~~**The downloads, with two programs of one account**~~: **made on
     2026-08-13 (the sixth session), and it found T-148 and T-149.** The two
     windows wrote one file at one time, and the book of the user held audio
     that no decoder reads. The map of the progress is a map of the process,
     therefore no program saw the download of the other one.
   - ~~**The key `X` of one window while the other window downloads**~~: **made
     on 2026-08-14, and it found T-150.** `docs/harness/slow_body.py` gives the
     delay of the body of the answer, and the key `D` takes the address of the
     pool since T-149: the two keys then meet. The key `X` read the database,
     and the database holds a row after the last byte of the last file.
   - ~~**A queue of media that a second program plays**~~: **made on 2026-08-14,
     and it found no fault of the data** — the first condition of the road that
     found none. The rule of T-147 does the work. One fault of the words stands
     beside it (T-151).
   - ~~**A program that dies while the server does not answer**~~ (the sharp
     form of T-145): **made on 2026-08-14 (the second session), and it found
     T-152.** `play_offline` opens no session on the server, therefore the row
     of the disk is the one copy of the whole playback, and the loop kept that
     place for the server at its end only: a program that dies reaches no end.
     27 minutes of a book went away, and the playback after it wrote the old
     place of the server over the place of the disk.
   - ~~**The view of the chapters while the media that plays changes**~~: **made
     on 2026-08-14 (the eighth session), and it found T-162.** The key `l` moved
     a book of eight hours 43 minutes forward.
   - ~~**The view of the bookmarks while the media that plays changes**~~:
     **made on 2026-08-14 (the ninth session), and it found T-163.** The list of
     that view holds the media of the user already, therefore no line moves under
     the cursor: **the key `b` is the one door**, and it read the media of the
     engine. The title of that view named no media at all.
   - ~~**A message belongs to the view of the user**~~ (the condition that T-162
     named and did not close): **made on 2026-08-14 (the ninth session), and it
     found T-164.** Three rules of the loop wrote to one slot, and the rule of
     the Home view runs last in the render of every view: a user of the view of
     the queue read the sentence of the Home view for six seconds, and the
     sentence of T-161 never came.
   - ~~**Two programs of one account that read one ebook at one time**~~: **made
     on 2026-08-14 (the second session), and it found T-153.** The two windows
     that open the **same** book give no fault of the data, and a cost of two
     children of T-62. The window that gets a **different** book took the book
     of the other window of the disk: 545898521 bytes of one key, because `keep`
     is a fact of the process.
   - ~~**A download of a podcast episode while the same podcast downloads on the
     server**~~ (the key `D` of the program and the queue of the server of
     T-81): **made on 2026-08-14 (the third session), and the condition holds no
     fault of the data.** The two sides write two files, and a new episode of
     the server stands at the end of `media.episodes`, therefore no line of the
     list moves. **A sweep of the keys inside that condition found T-154**: the
     second press of `D` on one media took the bar of that download off the
     screen for the whole of it.
   - ~~**A line of the Home view that goes away while the cursor of the user
     stands on it**~~: **made on 2026-08-14 (the sixth session), and it found
     T-160.** The condition holds the two shapes at one time: the key `M` of two
     presses marked two media of one window, and a second window took the line
     of the first one with no word. The line goes to nobody now.
   - ~~**A line of a view that is not the Home view and that goes away under
     the cursor**~~: **made on 2026-08-14 (the seventh session) for the queue,
     and it found T-161.** The queue takes the media of its front away when the
     media that plays comes to its end, and the lines keep the number of the
     line: the key `X` then took a media of the queue that the user did not
     choose out, and the key `l` played it and stopped the media that the queue
     had started. **One window makes that fault, and the user presses no key
     while it happens.**
   - ~~**The view of the episodes of a podcast while a second window changes
     that podcast**~~: **made on 2026-08-14 (the thirteenth session), and it
     found T-167.** The old list of that view is not the fault — T-47 says
     `R: the server has newer data` at the frame of the change — and **the key
     that acts on it is**: the key `l` on an episode that the server lost said
     `Loading the media...` and then nothing at all. The same key of the Home
     view gave the same silence, therefore the fault belongs to the playback and
     to no view.
   - **No condition of the road stays. A next session must name a condition of
     its own.** The shapes that found faults before: a state of one process that
     a second program cannot see (T-142, T-147, T-148, T-150, T-153, T-154,
     T-155, T-156, T-158, T-159, T-160, T-161, T-167), a program that dies in the
     middle of work (T-145, T-152), and a server that does not answer (T-146,
     T-149, T-152, T-156). **The parts of the program that no such measurement
     has reached**: the search of a library that a second window changes, the
     bookmarks and the lists of two windows, **the writes of the sequence, of
     the speed, of the key bindings, and of the rows of a session
     (`id_session`), which say nothing when their name holds no row** (the
     sweep of T-155 and of T-159), and **the view of the chapters while the
     media that plays changes**: the queue starts the next media with no key of
     the user, the list of the chapters is then the list of another media, and
     the key of that view seeks in the media that the user did not choose. The
     rule of T-160 and of T-161 reaches the Home view and the queue alone.
   - ~~**A second account of a second server while a media plays**~~ (T-124):
     **made on 2026-08-13, and it found T-138 and T-139** — the place of one
     account went to the server of another account, and no key sent the place of
     a playback before the program started again.
   - ~~**A library whose media the account may not read**~~, with an account of
     the type `user`: **made on 2026-08-13, and it found T-136** — an account
     that loses a library could not use the program again, in any view and after
     every start.
2. **What a refresh of the screen still loses: nothing that a measurement
   names** (T-135, done). The key `R` took the timer for sleep of the user away,
   and the new application takes it now. **The queue needs no such work**:
   `crate::logic::queue` holds it in a slot of the module, no line of `App::new`
   touches it, and every change of it reads the disk first (T-147).
3. **The rounds of the start that stay.** The start holds two rounds now, and the
   first is `GET /api/libraries`: every request of the second needs the identity
   of the library. **A start of one round needs the program to trust the identity
   that its database holds**, and to correct itself when the answer of the
   libraries comes. That is a design, and not a measurement.
4. **The row of the table that stays.** *Send an ebook to an e-reader*
   (`GET /api/emails/settings`). It is the one row of section 4 that says `No`
   for a function that a user of a terminal can use. **The issue #24 stays open
   for that row**, and T-119 holds the measurement that says why the endpoint
   cannot give it to a user.
5. **T-116: the words while the user waits.** The maintainer decided that the
   text stays as it is (2026-08-12). The row is a decision, and not work that
   waits.
6. **The fast suite stays at about 2 seconds.** It holds 991 tests in 2.2 s. A
   new test that needs a wait belongs behind `#[ignore]`.
7. **`cargo nextest run --run-ignored all` belongs at the end of a session**, and
   not at the end of one item: it took 27.6 seconds on 2026-08-12, and it found a
   fault of a test of this session that the fast suite did not (T-132), as it
   found T-111 before.
8. **The words for the user.** Every text in ASD-STE100. A view says why it holds
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

### 3b. The decisions of the session of the key `X` (T-150 and T-151)

- **No key of this program stops a download that runs.** The key `D` spawns a
  task, and a key that stops it needs a map of the handles of the tasks of the
  process — and it reaches the download of the other window never, because that
  download belongs to a different process. The key `X` says which window
  downloads the media, and the user presses it again when that download ends. The
  sentence promises no key that the program does not hold (T-118 and T-143).
- **The key `X` removes nothing at all while a download of that media runs**, and
  that holds for the ebook of the reader too. The lock of T-148 guards the whole
  key: a rule of two halves gives two sentences for one press, and the user reads
  one message at a time.
- **The sentence of the key `X` of the view of the queue names no program.** This
  program cannot say which program took the media of that line out — its own
  playback can take it too (T-146) — therefore the sentence says what happened to
  the media and not who did it (T-91).

### 4. The decisions of the session of the queue (T-146 and T-147)

**The maintainer answered no question before this session**, because the road
asked for a condition. The session made three decisions of its own.

**1. The queue stops at a media that it cannot play, and it does not go on to the
media after it.** A server that does not answer gives the same fault to every
media of the queue, therefore a queue that goes on empties itself in one second
and the user then has no list at all. The media stands at the front of the queue,
and the key `l` of the view starts it again. See T-146.

**2. The words for the user stay as they are.** The screen says the reason ("The
server does not answer, and the disk has no copy of this media."), and the
message before it named the media ("The queue starts \"One Chapter Book\"."). The
row of the message holds one message, therefore a second message about the state
of the queue would take the reason away. **The view of the queue is the answer**:
the media stands at its front, and the key `q` shows it. See T-146.

**3. The view of the queue reads the disk when it opens, and not at every
frame.** The render reads the queue of the process at each frame, and a read of
the database there would pay for a change that comes some times in a day. A view
that stands open while a second program changes the queue is therefore older than
the disk until the user leaves it and opens it again, and a key of that view takes
the media of its own line (`the_place_of_the_media`). See T-147.

### 5. The decisions of the session of the terminal that goes away (T-145)

**The maintainer answered no question before this session**, because the road
asked for a condition. The session made two decisions of its own.

**1. A row that no request carried must not go away.** The close of a session
removes the row of **that** session now, and not every row of the account. The
other answer — one close that carries every row of the account together — needs
one request of the server for each row anyway, and it hides the row of a program
that died inside the work of another program. See T-145.

**2. The 30 seconds of T-140 stay, and the last ten seconds of a playback that
died come back at the next close and not at once.** A program could ask the
system if the process of `owner` lives, and that answer needs a call of the
system for each program: the decision of T-140 keeps it outside, and the cost is
that a user who plays the same book again inside 30 seconds hears the ten
seconds of the sync a second time. **The position itself is never lost**, because
the row stays until a request carries it. See T-145.

### 6. The decisions of the session of a setting of two programs (T-142 and T-143)

**The maintainer answered no question before this session**, because the road
asked for a condition. The session made two decisions of its own, and both of
them follow the decision of T-140 (a second program is a condition of a user).

**1. The file is the truth of a value that two programs write, and the program
reads it at the moment of the use.** Two other answers exist, and each of them
costs more than it gives: a **watch of the file** needs a dependency of the
system (the rule of T-20), and a **lock of the file** would take a window of the
user away for the value of a setting. `config.toml` is 2 kilobytes, and the
program reads it at three moments only: a new application, the view that shows
the value, and a removal of a book. See T-142.

**2. A removal of a book reads the file, and a message of the program is not
enough.** The program could say "the value changed, press R" instead. **The
removal takes a file of the disk away**, and no message gives that book back:
the moment of a removal is the moment that needs the truth. See T-142.

### 7. The decisions of the session of two programs (T-140 and T-141)

**The maintainer answered no question before this session**, because the road
asked for a condition and a condition needs no decision. The session made two
decisions of its own.

**1. Two programs of one account are a condition of a user, and not a fault of
the user.** The answer could refuse the second program with a lock of the
database. **The server of Audiobookshelf holds a session for each client
already** — the web page of two tabs does the same — therefore a lock would take
a function away that the server gives. The program keeps every window, and the
row of the database says which window owns it. See T-140.

**2. A row of the database that stands still for 30 seconds belongs to a program
that died.** The identity of the process alone cannot answer that question
without a call of the system for each program, and the loop of the playback
writes the position of every second already. **The limit is longer than one
second**, because the loop writes nothing while the engine seeks to the place of
the user (T-38). The cost: a program that stopped by force inside those 30
seconds keeps its row until the next play or the next key `Q` of that account,
and the position of the user then reaches the server later. See T-140.

### The decisions of the session of the two faults of a first start

**The maintainer answered no question before this session**, because the road held
three measurements and two sweeps. This session made two decisions of its own.

**1. A request tries an address that holds the state `Down` when no address holds
the state `Up`.** The state `Down` is the answer of an attempt that came before,
and a key of the user is a new question: a program that says "No server address
answered" with no attempt says a reason that it does not have (T-91). The cost is
one connect timeout of 3 seconds for a program that is truly offline, and the
offline mode holds already. See T-128.

**2. The engine of the sound belongs to the playback, and not to the
application.** A refresh makes a new application, and T-131 gives it the engine
that plays. **The timer for sleep and the queue stay with the application**, and a
refresh loses them: that is the road of the next session, and it needs a
measurement of a user first.

### The decisions of the session before this one

**The maintainer answered no question before that session** either, because the
road held sweeps and a sweep needs no decision. That session made four decisions
of its own, and each of them follows a rule of this document.

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

### 8. The decisions of the session of T-112 to T-118

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

### The traps of the session of two programs (T-140 and T-141)

89. **A second program of one account needs no second configuration.** One
    `XDG_CONFIG_HOME` and two sessions of tmux give the whole condition, and the
    two programs then share one database, one `config.toml`, and **one file of
    the log**: the lines of the two programs stand between each other, and the
    time of the line is the only way to tell them apart. `start_the_program` of
    `docs/harness/drive.sh` empties that file, therefore the second program needs
    `tmux new-session` of its own.
90. **A book of 30 minutes is too short for a sweep of some steps.** The device
    `null` played it to its end in about 40 seconds (the trap 14), and the screen
    then held no player row at all. **Two books of eight hours** stand in the
    sandbox now, and section 2h of `docs/TEST-SERVER.md` holds the command that
    makes them. One book for each program: the position of each program then
    stands apart on the server.
91. **`wait_for` of the harness reads the box of the search too.** A poll for the
    title that the measurement typed comes back at once, because that text stands
    in the field of the search, and the key `l` then acts on the line before the
    answer of the server. **Poll for the line of the selection**
    (`capture-pane | grep '^➤'`), and press `j` until that line holds the title.
92. **`PATCH /api/me/progress/:id` with `isFinished: false` sets the position to
    0.** A measurement that writes "the place of a different client" gives
    `currentTime` alone, and `progress` beside it when a view must show a
    percent. See section 15 of `docs/TEST-SERVER.md`.
93. **The row of `listening_session` says which program wrote it** since T-140:
    `owner` is the identity of the process and `heartbeat` is the moment of the
    last second of the playback. A measurement that writes a row by hand must
    give those two columns, or the program takes that row for the row of a
    program that died.

### The traps of the session of a setting of two programs (T-142 and T-143)

94. **The sequence of the two programs decides the whole condition.** A program
    reads `config.toml` at its start, therefore the second program must **stand
    already** when the first one writes the value: write the value of the start in
    the file, start the window B, and then press the keys of the window A. A
    window that starts after the change holds the new value, and the measurement
    then says nothing.
95. **The removal of a book needs more books than the sandbox holds.** The cache
    of the sandbox holds 146 megabytes, and the smallest value of the view is 256.
    Five copies of the book of 100 megabytes in the directory of the downloads
    make the condition (`copy-of-a-user-N.epub`), and the cache counts them: the
    filesystem of this machine holds one copy of the bytes for all of them, and
    `du` therefore says 164 megabytes for six files of 105. **`metadata.len()` is
    what the program reads**, and it says 105386785 for each of them.
96. **The removal comes after a download of a book, and at no other moment.** A
    book that the cache holds already gives no removal, therefore a measurement of
    the removal must take a book that the directory does not hold: move it away
    first, and the key `e` gets it again.
97. **A test that writes `XDG_CONFIG_HOME` and then reads the configuration makes
    a file.** `load_config` writes `config.toml` when the file is absent (T-122),
    therefore that test needs a `tempfile::tempdir` and it must never run with the
    variable of the user. nextest gives each test its own process, and `cargo
    test` does not (the trap 29).
98. **`write_the_value` is the way to make the condition of a second program.** It
    reads the file and it changes one line, therefore a test of two programs needs
    no second process at all: the test writes the file as the other window does,
    and it then makes a new application.
99. **`cargo nextest run` is not the gate of CI.** The workflow runs
    `cargo test --verbose`: nextest gives each test a process of its own, and
    `cargo test` runs the tests of one binary in threads of **one** process. A
    test that writes `XDG_CONFIG_HOME` or that reads every row of a table
    therefore passes with nextest and fails on CI (T-144). **Run `cargo test`
    before the last commit of a session**, and `--no-fail-fast` for every binary
    that fails: `cargo test` alone stops at the first one.
100. **A test that stops the whole binary with a lock that is poisoned costs a
    session more than the fault.** `THE_TURN.lock().unwrap_or_else(|of_a_test|
    of_a_test.into_inner())` gives the turn to the test after a test that
    panicked, and the report then names the one test of the fault.

### The traps of the session of the terminal that goes away (T-145)

101. **`SIGSTOP` does not reach a program of tmux from a session of this
    harness.** `kill -STOP` on the process of the program gave no fault and no
    stop: the state stayed `S`, and the row of the database kept moving. A child
    of the session itself stops with the same command. **The freezer of the
    cgroup does the work**, and it is the better model of a suspend of the
    machine:
    ```bash
    CG="/sys/fs/cgroup$(grep '^0::' /proc/$PID/cgroup | cut -d: -f3)"
    echo 1 > "$CG/cgroup.freeze"   # the machine sleeps
    echo 0 > "$CG/cgroup.freeze"   # the machine wakes up
    ```
    systemd gives the program of tmux a scope of its own, therefore that file
    holds the program alone and not the server of tmux. `cgroup.events` says
    `frozen 1`, and the state of `/proc/PID/status` stays `S`: **read the row of
    the database to see that the program stands still**, and not the state.
102. **`pgrep -f 'target/debug/toutui$'` gives two numbers**, and the second one
    is the client of tmux (the trap 16 in a new place). A `kill` of that text
    says "not a pid", and a measurement that reads the first number signals the
    wrong process. **`pgrep -x toutui` gives the program alone.**
103. **`tmux kill-session` is the `SIGHUP` of a terminal that goes away.** The
    program dies at once, and no line of the exit runs: that is the condition of
    T-145, and it needs no other tool.
104. **The server keeps an open session of a program that died.** Seven sessions
    of one book stood open in `GET /api/users/online` after the sweeps of one
    session. A measurement of "the program closed its session" must read the
    **new** identity of the session, and not the count of the open sessions.
105. **A sweep of two books needs the sequence of the lines of Continue
    Listening**, and that sequence follows the newest place: the book of the
    measurement moves while the measurement runs. Poll the line of the selection
    (`capture-pane | grep '^➤'`) and press `j`, and hold two titles apart when one
    of them holds the other ("A Book Of Many Hours" stands inside "A Second Book
    Of Many Hours").

### The traps of the session of the queue (T-146 and T-147)

106. **`podman stop -t 0 abs-test` takes the server away in 50 milliseconds**,
    and `podman stop` alone waits for the `SIGTERM` of the container. A
    measurement that must take the server away **inside** a playback needs the
    first form: the book of 30 minutes plays in about 25 seconds with the device
    `null`, and 10 seconds of a stop is half of that.
107. **The device `null` gives the measurement of the queue for free.** A book of
    30 minutes at the speed 1.30 comes to its end in about 24 seconds of real
    time, therefore one measurement of "the media comes to its end while the
    server is down" costs half a minute and no wait of the wall.
108. **The queue of the process crosses a test of a binary.** `crate::logic::queue`
    holds a slot of the module, as the database does (T-144): two test functions
    of one binary fight for it. The measurement of T-147 therefore stands in
    **one** test function, and it ends with `queue::clear()` and
    `queue::forget_the_account()`.
109. **A book of the queue that the disk holds plays while the server is down**,
    and that hides the fault of T-146. "Multi File Test Book" of the sandbox has
    3 of 3 tracks on the disk, and "One Chapter Book" has none: **read the log
    line "the download … gives N of M track(s) from the disk"** before you choose
    the media of a measurement of the offline mode.
110. **`sqlite3 $ABS/toutui-config/toutui/db.sqlite3 "select title from queue"`
    is the truth of a measurement of two windows.** The two screens each say
    their own list, and neither of them says which list the disk holds.

### The traps of the session of the downloads (T-148 and T-149)

111. **A download of 115 megabytes over the loopback ends in less than one
    second.** The two keys of a measurement of two writers therefore go one after
    the other with no sleep between them (`tmux send-keys -t A D; tmux send-keys
    -t B D`), and the fault of T-148 came in three runs of four. **The number of
    bytes on the disk is the truth**, and not the screen: the two windows both
    said `[Downloaded]` for a file that no decoder reads.
112. **A port that no program holds refuses a connection at once**, therefore it
    measures no limit of time at all. A measurement of "the program waits for
    ever" needs a port that **accepts** the connection and answers nothing: 20
    lines of `asyncio.start_server` with a sleep of an hour. A port of a
    container that stands and that answers nothing (13401 of this machine) gives
    the same condition, and no session can be sure of it.
113. **The header of the program says `pool.active()` (T-105), and that is no
    evidence of the address that a task uses.** The proxy of `docs/harness/slow.py`
    on the address of the login is the evidence: it writes the path of each
    request, and a task that goes to a different address leaves it empty.
114. **`pkill -f <a text>` of a shell of this harness kills that shell**, when
    the text stands in the command line of the shell itself: the command gives
    the code 144, and every line after it does not run. It cost this session two
    restores of the sandbox. **Give the number of the process** (`ps -eo
    pid,args | grep`), or `pgrep -x`.
115. **`: > <the log>` of a file that a program holds open leaves the bytes of
    zero of the old length**, and `grep` then says nothing at all for a file that
    holds the lines. Give `grep -a`, or make a new file.

### The traps of the session of the key `X` (T-150 and T-151)

116. **A measurement inside a download needs `docs/harness/slow_body.py`**, and
    not `slow.py`: the second one delays the **request**, therefore a download of
    115 megabytes of the loopback still ends in less than one second (the trap
    111). A delay of 0.05 seconds for each block of 64 kilobytes makes that
    download take about 90 seconds, and every other request of the program stays
    fast because a small answer holds one block. **The proxy stands in the block
    `[[servers]]` of `config.toml`** with the slow address first: the key `D`
    takes the address of the pool since T-149.
117. **`start_the_program` of the harness truncates the log**, therefore the
    second window of a measurement of two programs takes the log of the first one
    away. Start the two windows first, and read the log after both of them stand.
118. **The null device plays a book of 30 minutes in about 12 seconds when that
    book is at its half.** A measurement that needs a window of some seconds must
    set the place of that media to 0 first
    (`PATCH /api/me/progress/:id` with `{"currentTime":0,"isFinished":false}`),
    or the media comes to its end while the harness still presses its keys.
119. **`podman stop -t 0 abs-test` gives a server that is away, and the program
    then starts in the offline mode**: the header says
    `📴 Offline: the media on the disk`, and the view holds the media of the
    disk alone. The key `l` of that view plays the local copy, and the loop of
    that playback is `follow_playback_offline` — a different loop, with no
    session on the server (T-152). **A measurement of the offline mode needs a
    media that the key `D` downloaded before**, therefore make the download while
    the server answers.
120. **The child of T-62 reads the PDF of 502 megabytes in 131 seconds, and it
    writes the file of the pages in one call at the end of that time.** A
    measurement of two readers of one book therefore has a window of two minutes
    for its second key, and the two writes of the file of the pages do not meet.
    The file of the pages holds 43016313 bytes, and the memory of the child holds
    about a gigabyte at its peak (T-153).
121. **The removal of the cache of the ebooks runs after a new book came, and at
    no other moment.** A measurement of that removal therefore needs a book that
    the disk does **not** hold: a book of the cache gives no request and no
    removal. `ls` of `<XDG_DATA_HOME>/toutui/downloads/<the user>` says which
    books the cache holds (T-153).
122. **The server holds the 57 episodes of its feed already, therefore
    `POST /api/podcasts/:id/download-episodes` gives it no work.** A measurement
    of the queue of the downloads of the server needs
    `DELETE /api/podcasts/:id/episode/:episode?hard=1` on some episodes first,
    and `POST /api/podcasts/feed` with `https://librivox.org/rss/52` gives the
    lines of the feed again. **The body of `download-episodes` is the bare array
    of those lines**: an object of one field gives `400 Bad Request` and the
    queue stays empty. A hard delete and a new download can leave the file
    `Letter 49 (<uuid>).mp3` beside `Letter 49.mp3`, and the library then holds
    58 episodes of a feed of 57: that is the work of Audiobookshelf, and a
    second hard delete takes it away (T-154).
123. **A media that came to its end stays away from the shelf Continue
    Listening.** A second run of a measurement of the queue therefore needs
    `PATCH /api/me/progress/:id` with `{"isFinished": false}` first, and the
    place after it: `{"isFinished": false}` writes `currentTime: 0` (the section
    15 of `docs/TEST-SERVER.md`). A measurement that gives no media back walks
    a shelf that lost its lines, and the cursor of the harness then finds no
    title at all (T-163).
124. **The log of the queue says `The queue starts` with a capital T.** A poll
    of `grep -q "the queue starts"` therefore never comes back, and the
    measurement waits its whole timeout and reads the screen many seconds after
    the frame that it wanted. Give `grep -i`, or name the text of the log
    exactly (T-164).
125. **The row of the message stands above the footer, and its number of the
    row moves with the view.** A poll of a fixed row gives the footer of one
    view and the message of another. `tmux capture-pane -p | grep -B1
    "j/k: move" | head -1` gives that row in every view of a list (T-164).
126. **A window reads the lists of the server at a key of its own alone.** The
    view of the collections and of the playlists draws `app.lists`, and one of
    the keys of that window writes that field: a change of a second window
    therefore reaches the screen at the next key, and not at the next frame. A
    measurement of two windows needs one key of the first window after the key
    of the second one (T-165).
127. **The sandbox holds one collection and two playlists**, and a measurement
    of a line that moves needs a second list of the library `Books` (the
    section 6d of `docs/TEST-SERVER.md` makes them). The lists stand in the
    sequence of the collections and then of the playlists, therefore a
    collection that goes away moves every playlist one line up. The measurement
    of T-165 made `Z Second Playlist`, and it took it away at its end: the
    library holds `A Test Collection` and `A Test Playlist` again (T-165).
126. **A cursor of the harness that walks with `j` walks in one direction
    only.** A helper that presses `j` until a title stands under `➤` must take
    the cursor to the top of the view first, or the second call of it walks past
    its title and off the end of the list. The measurement then presses no key
    at all, and its `&&` hides that (T-164).

### The traps of the session of the server that fails one request (T-169)

128. **A server that answers some requests and that fails others is
    `docs/harness/one_path_fails.py`.** It answers the status `500` to every
    path that holds a part of its command line, and it forwards every other
    request to the sandbox:
    ```bash
    python3 docs/harness/one_path_fails.py 13500 13399 requests.log \
        <the library>/collections <the library>/playlists
    ```
    A part that holds the library gives the fault to one library alone, and the
    key `S` then measures the other libraries beside it (T-169).
129. **A pool of two addresses hides a fault of the status 500.** That status is
    the fault of one machine, therefore `send` tries the address after it
    (T-97), and the second address of a block `[[servers]]` is the sandbox
    itself: the request comes back with the answer of the sandbox, and the
    measurement says nothing. **The pool of such a measurement needs one
    address**: write the address of the proxy in `users.server_address` of
    `db.sqlite3`, keep a copy of that file, and give the account its own address
    again at the end (T-169).
130. **The answer `500` keeps the state `Up`** (T-128), therefore every request
    of the program stays with the proxy. This is the difference from a port that
    no program holds: that road is the offline mode of T-25 (T-167).
131. **`pkill -f one_path_fails.py` killed the shell of this harness** (the trap
    114 again, and it cost this session a minute). Take the process of a port
    with `ss -lptnH 'sport = :13500'` instead.
132. **A `cd` of one command and a `&` of that same command take the program of
    the background to the new directory.** The proxy of this session then found
    no file of `docs/harness`, and the port stood empty with no word: give the
    absolute path of the file, and start the proxy in a command of its own
    (T-170).
133. **The four requests of the start come before the first frame** (T-129,
    and `App::new` awaits the task of them). A view of the start therefore holds
    no condition of "the request runs", and a proxy of a delay of 1.5 seconds
    gave the collections at the first frame already (T-169).

### The traps of the session of the login of no library (T-173 and T-174)

134. **A server that answers the login and that gives no library is
    `docs/harness/no_library.py`.** It forwards every request to the sandbox,
    and it answers `GET /api/libraries` with the status 200 and the body
    `{"libraries": []}`:
    ```bash
    python3 docs/harness/no_library.py 13501 13399 requests.log
    ```
    The login of the sandbox works through it, therefore the token is a real
    token (T-173).
135. **The login screen needs a `XDG_CONFIG_HOME` that holds no account.** Make
    a directory of nothing, and give it to the program with `XDG_DATA_HOME`
    beside it: the program then makes its key, its `config.toml`, and its
    database, and the first screen is the login screen. **The address of the
    login goes in the first field**, and the pool takes that address alone when
    `config.toml` holds no block of it (T-173).
136. **`gdb` and `eu-stack` say `Operation not permitted` for a program that
    they did not start.** `ptrace_scope` of this machine permits a child alone.
    **`strace -f -tt -o <file> <the program>` inside `tmux` gives the answer**,
    and it gave the cause of T-174 in one line: the last three system calls of
    the thread that stopped.
137. **A mark of the log inside a hook of a panic says nothing at all when the
    hook does not run.** A mark of `std::fs::write` says the same, and a mark
    before the line of the panic says that the mark works. **Three marks that
    say nothing are the measurement**: the hook that ran belongs to another
    crate (T-174).
138. **A screen of this program must take no lock of the standard output.** The
    login screen, the search, and the box of a text held
    `io::stdout().lock()` for their whole life, and a panic of another thread
    then held the program for ever. `the_backend_of_a_field` of
    `src/ui/text_field.rs` is the one place of that decision now (T-174).
139. **A key that reads a state of the server and that then writes it needs a
    proxy of one method.** The read and the write of the keys `M` and `N` stand
    on **one** path, therefore `one_path_fails.py` fails both of them and the
    fault of the read never reaches the screen.
    **`docs/harness/one_method_fails.py`** takes rules of the shape
    `METHOD:part-of-a-path`:
    ```bash
    python3 docs/harness/one_method_fails.py 13500 13399 requests.log \
        GET:/api/me/progress
    ```
    The log of that proxy says whether a write left the program (T-175).
140. **A body of the libraries of your own is
    `docs/harness/another_body_of_the_libraries.py`.** It forwards every request
    to the sandbox, and it answers `GET /api/libraries` with the body of a file:
    ```bash
    python3 docs/harness/another_body_of_the_libraries.py 13502 13399 \
        requests.log /the/absolute/path/of/the/body.json
    ```
    Take the body of the sandbox with `curl`, change one field of it with
    `python3`, and the program of the sandbox then meets a server of another
    version (T-176).
141. **`pkill -f` of a program of a harness kills the shell of that harness**
    (the trap 114), and a `for` loop of this shell holds the name of the
    program in its own command line. **The process of a port comes of
    `ss -lptnH 'sport = :13502'`** (the trap 131), and one line of a function
    stops it:
    ```bash
    pid=$(ss -lptnH "sport = :13502" | grep -o 'pid=[0-9]*' | head -1 | cut -d= -f2)
    [ -n "$pid" ] && kill "$pid"
    ```

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
7. **An isolated `XDG_CONFIG_HOME` needs no file before the program starts.**
    The program makes `config.toml`, and it makes `.env` with a key of its own
    since T-133. A `.env` of the measurement keeps the tokens of that directory
    readable between two builds, therefore a harness that gives
    `TOUTUI_SECRET_KEY` still works.
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

### The traps of the session of the address and of the playback (T-128 to T-132)

69. **`Esc` stops the whole program.** The list of every key says "Q / Esc: Quit",
    and no footer of a list names it: a measurement that presses `Esc` to leave a
    view kills the program, and the tmux server goes with it ("no server running
    on /tmp/tmux-1000/default"). **The key `h` goes back**, and `Esc` goes back
    inside the reader (T-52) and inside the list of the keys only.
70. **`tmux send-keys` takes `BTab` for Shift+Tab**, and not `S-Tab`. A
    measurement with the wrong name pressed nothing at all, and the library of the
    screen never changed.
71. **`docs/harness/slow.py` stands in the repository now**, and it gives the
    rounds of the start with no line of code of the program: it holds a port, it
    forwards every byte to the sandbox, it sleeps before each request, and it
    writes the path and the time of each request. Two sessions wrote that proxy
    and neither kept it. See T-129.
72. **The null device plays about 30 minutes of a book in 25 seconds** (the trap
    14 says 50 seconds for 30 minutes; the machine of this session is faster). **A
    measurement of a playback that must live longer than that needs a position of
    0 first**: `PATCH /api/me/progress/:id` with `currentTime: 0` gives the whole
    media, and a book that stands at the minute 20 comes to its end in 10 seconds.
73. **A measurement that writes the data of the sandbox breaks a test of the
    sandbox.** This session wrote the position of one book with `curl`, and
    `a_change_of_a_different_client_comes_to_the_screen` then failed: the test
    measured the data and not the program (T-132). **Put the data back**, and run
    `cargo nextest run --run-ignored all` at the end of the session.
74. **`PATCH /api/me/progress/:id` keeps the `progress` of the body**, and a body
    that holds `isFinished` beside it gives the fraction of the position that
    stood there before. **Send the mark in a request of its own.** See T-132.
75. **The server keeps the form of a number that a client gives it.** One
    `POST /api/session/:id/sync` with `"currentTime": "714"` makes `GET /api/me`
    answer `"currentTime": "714"` for ever, until a client writes a number. **A
    measurement of a JSON body must look at the type of every value.** See T-130.
76. **`mark_down` of the pool writes a line of the log now**, therefore a report of
    "No server address answered" has evidence: read
    "[api] The program does not use the address" of the log of the user. See
    T-128.

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
- **The sessions can run in a loop.** The driver is `~/.local/bin/toutui-loop`,
  outside this repository, and its design is
  `docs/superpowers/specs/2026-08-13-session-loop-design.md`. It reads the block of
  the quote of `## The prompt for the next session`, therefore **that block must stay
  the last part of this file, and every line of it must start with `> `**. A session
  that writes a handover with no such block stops the loop. **The name of that
  heading stands in this bullet too**, therefore a tool that splits this file on
  the first hit of that text cuts the bullet and not the heading: split on the
  **last** hit of it.

## The prompt for the next session

**This session took the two roads that the session before it named**: a key
that reads a state of the server and that then writes it, and the one structure
of `src/api/` that asks for every field of an answer. **The reader wrote the
first page of a book over the place of the user** when the read of that place
came back with a fault (T-178), and **two fields that the program never reads
took every position of the account away** (T-177). The session left no item
open, and it corrected a decision of the session before it. **The next session
must name a condition of its own.** This prompt names the state of the program
on 2026-08-14.

> Continue the Toutui takeover. Repo: `/home/nyverino/Documents/Toutui`
> (ealtun21/Toutui, branch main). Maintained fork of the archived
> AlbanDAVID/Toutui. Newest release **v0.8.9**; `Cargo.toml` is at 0.8.9. The
> workflow refuses a tag that disagrees with `Cargo.toml`, **and it builds
> `--locked`**. **A release holds three files together**: `Cargo.toml`,
> `Cargo.lock`, and one new entry at the top of `THE_ENTRIES_OF_THE_FORK` of
> `src/utils/changelog.rs`. The gate fails without that entry (T-101).
>
> **Read before you touch code:** `docs/HANDOVER.md` (the state, the decisions,
> the road, and the traps that cost real time), `docs/TAKEOVER-BACKLOG.md` (the
> evidence of every item; **T-87, T-107, T-128, T-131, T-140, T-142, T-145, and
> T-148 are the eight to know**, and T-142 to T-178 are the newest), and
> `docs/T-24-coverage.md`
> (**no row of section 4 says `Half`, and every row that says `No` belongs to an
> administrator of the server**, and **section 6 names what the program must not
> have, with the reason**).
>
> **The way of working, for every item.** Show the fault before you correct it,
> and let a test find it: a build with the correction removed must fail. Make the
> data of the fault exist in the sandbox (`docs/TEST-SERVER.md`, podman on
> `:13399`; `podman start abs-test` gives the server back with a library of 2056
> items, a library of **520 podcasts**, a PDF of 502 megabytes, an **EPUB of 100
> megabytes**, and **two books of eight hours**). **Drive the real program inside
> tmux** with `docs/harness/drive.sh`; a screen of your own writing lies to you.
> **The key `h` goes back, and `Esc` stops the program** (the trap 69), and `tmux
> send-keys` takes `BTab` for Shift+Tab (the trap 70). **A second program of one
> account needs one `XDG_CONFIG_HOME` and a second `tmux new-session`** (the trap
> 89), and the sequence of the two starts decides the condition (the trap 94).
> **`tmux kill-session` is the `SIGHUP` of a terminal that goes away** (the trap
> 103), and a program of tmux takes no `SIGSTOP` of this harness: the freezer of
> the cgroup gives the sleep of the machine (the trap 101). **A port that no
> program holds refuses a connection at once**, therefore a measurement of a
> limit of time needs a port that accepts and answers nothing (the trap 112).
> **`pkill -f` and `pgrep -f` of a shell of this harness kill that shell** (the
> trap 114), and the process of a port comes of
> `ss -lptnH 'sport = :13500'` (the trap 131). **A download of the loopback ends
> in less than one second**, therefore a measurement inside a download needs
> `docs/harness/slow_body.py` (the trap 116). **`podman stop -t 0 abs-test` takes
> the server away**, and the program then starts in the offline mode with the
> media of the disk (T-152).
>
> **A server that answers some requests and that fails others is
> `docs/harness/one_path_fails.py`** (T-169 and T-170). It answers the status
> `500` to every path that holds a part of its command line, and it forwards
> every other request to the sandbox:
>
> ```bash
> python3 docs/harness/one_path_fails.py 13500 13399 requests.log \
>     <the library>/series <the library>/items
> ```
>
> **That road holds four traps.** A pool of two addresses hides a fault of the
> status 500: `send` tries the address after it (T-97), therefore the pool of
> such a measurement needs **one** address — write `http://127.0.0.1:13500` in
> `users.server_address` of `db.sqlite3` of the sandbox, keep a copy of that
> file, and give the account its own address again at the end (the trap 129).
> The answer `500` keeps the state `Up` (T-128), therefore every request stays
> with the proxy (the trap 130). A `cd` of one command and a `&` of that same
> command take the proxy to the new directory, and it then finds no file: give
> the absolute path (the trap 132). **The four requests of the start come before
> the first frame** (T-129), therefore no view of the start holds a condition of
> "the request runs" (the trap 133).
>
> **The child of T-62 reads the PDF of 502 megabytes in 131 seconds**, therefore
> a measurement of two readers has a window of two minutes (T-153). **The server
> holds the 57 episodes of its feed already**, therefore a measurement of the
> downloads of the server needs
> `DELETE /api/podcasts/:id/episode/:episode?hard=1` first, and the body of
> `POST /api/podcasts/:id/download-episodes` is the bare array of the episodes of
> the feed (T-154). **A playback reads the files of a download only when no
> address answers**, therefore a measurement of the media of the disk needs the
> server away (T-156). **The sandbox holds a second account for the keys of the
> view of the accounts**: `toutuilimited` / `toutuilimited` (section 14 of
> `docs/TEST-SERVER.md`), and the key `a` of that view takes it through the
> login screen of a second window (T-155). **The key `l` of that view needs two
> presses**, and a window whose account went away gives the login screen: the
> account of the sandbox comes back with the address, `toutuitest`, and
> `toutuitest` (T-158). **`PATCH /api/me/progress/:id` with
> `{"isFinished": false}` writes `currentTime: 0`**, therefore a measurement
> that gives a media a place again needs that request first and the place after
> it (section 15 of `docs/TEST-SERVER.md`, and T-160). **A media of the sandbox
> that ends while the user looks at a view** comes of `A Long Test Book` of 30
> minutes: the null device plays it in 22 seconds, and the two books of eight
> hours then hold the queue open (T-161). **A measurement of two lists of
> chapters needs the three chapters that a session gave the book of eight
> hours** (the section 6i of `docs/TEST-SERVER.md`), because the queue takes 22
> seconds to start that book (T-162). **A media that came to its end stays away
> from the shelf Continue Listening**, therefore a second run of such a
> measurement needs `PATCH /api/me/progress/:id` with `{"isFinished": false}`
> first and the place after it (T-163). **The log of the queue says `The queue
> starts` with a capital T**, therefore a poll of `grep` for that line needs
> `-i` (T-164). **The row of the message stands above the footer**, and a poll
> of it reads the line above `j/k: move` and not a number of a row (T-164).
> **A window reads the lists of the server at a key of its own alone**, therefore
> a measurement of two windows needs one key of the first window after the key of
> the second one (T-165). **The queue of the downloads of the server belongs to
> the library**, therefore a measurement of the line of that view needs the two
> podcasts of the library `Podcasts` (the section 5b of `docs/TEST-SERVER.md`),
> and 48 episodes give about three minutes of downloads at four seconds each
> (T-166). **The message of a view lives six seconds, and a download of the
> sandbox ends in four**: a poll of that message needs a step of 0.25 seconds and
> a `grep` of its words, and not a number of a row (T-166). **The program reads
> the episodes of a podcast one time** (`the_episodes_that_came` of `App` never
> goes back to `false`), therefore a measurement of an old list of that view
> needs one window and a second program alone, and a fresh list needs the key `R`
> (T-167). **A test of a server that holds nothing must not use the port that no
> program holds**: that road is the offline mode of T-25, and it keeps the copy
> of the disk. A host of a raw socket that answers `404` or `500` gives a server
> that answers with no network and no sandbox (T-167, T-169, T-170, and the three
> files `tests/a_playback_that_did_not_start_says_why.rs`,
> `tests/the_lists_that_did_not_come_say_why.rs`, and
> `tests/the_requests_of_the_start_that_failed_say_why.rs`).
> **A server that answers the login and that holds no library is
> `docs/harness/no_library.py`** (T-173). It forwards every request to the
> sandbox, and it answers `GET /api/libraries` with the status 200 and the body
> `{"libraries": []}`:
>
> ```bash
> python3 docs/harness/no_library.py 13501 13399 requests.log
> ```
>
> **A key that reads a state of the server and that then writes it needs
> `docs/harness/one_method_fails.py`** (T-175): the read and the write of the
> keys `M` and `N` stand on one path, therefore a proxy that fails the whole
> path hides the fault of the read. It takes rules of the shape
> `METHOD:part-of-a-path`, and its log says whether a write left the program:
>
> ```bash
> python3 docs/harness/one_method_fails.py 13500 13399 requests.log \
>     GET:/api/me/progress
> ```
>
> **A body of the libraries of your own is
> `docs/harness/another_body_of_the_libraries.py`** (T-176). It forwards every
> request to the sandbox, and it answers `GET /api/libraries` with the body of a
> file: take the body of the sandbox with `curl`, change one field of it with
> `python3`, and the program then meets a server of another version:
>
> ```bash
> python3 docs/harness/another_body_of_the_libraries.py 13502 13399 \
>     requests.log /the/absolute/path/of/the/body.json
> ```
>
> **A server of another version is
> `docs/harness/a_field_of_the_answer_goes_away.py`** (T-177). It forwards every
> request to the sandbox, and it takes the named fields out of the answer of one
> path, at every depth of the body. A body of a file cannot do this work for
> `GET /api/me`: that answer holds the id of the account, the id of every media,
> and the position of each of them, and the sandbox makes those values at the
> moment of the request:
>
> ```bash
> python3 docs/harness/a_field_of_the_answer_goes_away.py 13503 13399 \
>     requests.log /api/me mediaItemId mediaItemType
> ```
>
> **`pkill -f` of a proxy kills the shell of this harness** (the trap 114): the
> command line of a `for` loop of the shell holds that name too. The process of
> a port comes of `ss -lptnH 'sport = :13502'` (the trap 131 and the trap 141).
>
> **The login screen needs a `XDG_CONFIG_HOME` that holds no account** (the trap
> 135): make a directory of nothing, give it and a `XDG_DATA_HOME` of nothing to
> the program, and write the address of the proxy in the first field.
> **`gdb` and `eu-stack` say `Operation not permitted` for a program that they
> did not start**, therefore a program that stands needs
> `strace -f -tt -o <file>` inside tmux (the trap 136). **A mark of a hook of a
> panic that says nothing at all is the measurement** (the trap 137), and **a
> screen of this program must take no lock of the standard output** (the trap
> 138).
>
> **A sweep of a class must read the whole screen, and not the panel of the view
> alone**: the six views of T-171 each said what the server said, and the header
> two rows above them said that the server is away (T-171). **The header says
> what the program measured last**, and a view that holds its answer already
> makes no request: a measurement of two roads of the header needs a key of a
> fresh request, and the key `R` alone forgets the state of a view.
> Verify with a second program: `curl`, `podman logs abs-test`, or a browser.
> Write the measurement in `docs/TAKEOVER-BACKLOG.md` under a new item (T-179 and
> up), and name that item in the commit.
>
> **The gates, before each commit**, under `nice -n 19 ionice -c 3` with `-j 16`:
> `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and
> `cargo nextest run` with `ALSA_CONFIG_PATH` pointing at a null asound file of
> two lines (`pcm.!default { type null }` and `ctl.!default { type null }`).
> Baseline: **1107 tests in 2.3 seconds**, and `cargo nextest run --run-ignored
> all` gives **1132 of 1132** with the sandbox up, in about 24 seconds. **Run that
> second command at the end of the session too**: it found T-132 and T-111.
>
> **A box of the process needs one test function.** Two test functions of one
> module fight for the slot of that module, and `cargo test` found such a fault
> at the run 1 of 4 while nextest passed every time (the shape of T-144 and of
> T-157). The head of such a test says the rule already: "the parts of this test
> stay in one function".
>
> **A test must not call a function that may never come back.** The wait of the
> playback of T-158 held the gate of the machine for ever, and a test of that
> shape says nothing at all: give such a function a thread of its own, and read
> the end of that thread with a limit of time. **`wait_prev_session_finished`
> blocks the thread that calls it**, therefore a limit of time on the future
> alone says nothing (T-167).
>
> **`cargo test -j 16 --no-fail-fast` is the gate of CI, and it is a different
> run.** nextest gives each test a process of its own, therefore it hides a test
> that shares a database or `XDG_CONFIG_HOME` with another test of its binary:
> six such tests failed on CI while nextest passed (T-144), and **a box of the
> process gave the same fault of one run of six** (T-157). **Run it before the
> last commit of the session, and run it more than one time**: the tests of one
> binary take a thread each, therefore a fault of that kind comes of the
> sequence of the moment.
>
> **No run opens the real sound device.** `TOUTUI_AUDIO_DEVICE=null` gives the
> null device of ALSA, and the log then says "the application uses the sound
> device alsa:null". `ALSA_CONFIG_PATH` does **not** silence the real program.
> **The null device plays 30 minutes of a book in 25 seconds** (the trap 72), and
> a sweep of some steps therefore needs a book of eight hours (the trap 90).
>
> ### The work, in the sequence of its value
>
> 1. **A condition of the program that no measurement has reached.** A sweep of
>    this shape found a fault in thirty-one sessions of thirty-two. **No
>    condition of the road stays**: the session of the nineteenth turn took the
>    two roads of the session before it, and it found a reader that writes the
>    first page of a book over the place of the user (T-178) and an answer of one
>    version of the server that takes every position of the account away (T-177).
>    It wrote the correction of each, and it left no item open.
>    - **The three shapes that found a fault before:** **a state of one process
>      that a second program cannot see** (T-142, T-147, T-148, T-150, T-153 to
>      T-167), **a program that dies in the middle of work** (T-145, T-152), and
>      **a server that does not answer, that answers with a fault, or that
>      answers with another body** (T-146, T-149, T-152, T-156, T-167 to T-178).
>      **A fourth shape came of T-175**: a key that **reads** a state of the
>      server and that then writes it. A proxy of one path hides that fault, and
>      `docs/harness/one_method_fails.py` gives it. **That shape is closed for
>      the keys** (T-178): the keys `M`, `N`, and `e` were the three of it, and
>      each of them held a fault. **A fifth shape came of T-177**: an answer of
>      a server of another version, which holds one field fewer, and
>      `docs/harness/a_field_of_the_answer_goes_away.py` gives it.
>    - **The class of the views of `one_path_fails.py` is closed.** The bookmarks,
>      the sessions, the statistics, the authors and the narrators, the devices of
>      an e-reader, and the downloads of the server each hold a `State::Fault`,
>      and each of them says what the server said: the measurement of T-171 read
>      the four of them that a key of the sandbox reaches. **The view of the
>      chapters holds no request of its own** — the chapters come of
>      `POST /api/items/:id/play`, and T-167 holds that road. **The fault of that
>      sweep stood in the header** (T-171): a sweep of a class must read the whole
>      screen, and not the panel of the view alone.
>    - **The parts of the program that a server of a fault has not reached**: the
>      keys `F`, `b`, `n`, `m`, `r`, `D`, and `X`, the send of an ebook to an
>      e-reader, and the stream of the audio. **The keys `M`, `N`, and `e` are
>      closed** (T-175 and T-178), **the first request of the program is closed**
>      (T-172), and **the login screen is closed for the status of `POST /login`
>      (T-92), for a server that gives no library (T-173), and for a body of the
>      libraries that the program cannot read (T-176)**. **The send of an ebook
>      to an e-reader is the one to take**: `POST /api/emails/send-ebook-to-device`
>      writes, and the list of the devices comes of the payload of the login
>      (T-119) — the sweep must ask what the program does with a device that the
>      server no longer holds.
>    - **The shape of T-177 is the answer of a server of another version**, and
>      **no structure of `src/api/` asks for a field that the program does not
>      read now**: `get_all_books.rs`, `sessions.rs`, `bookmarks.rs`,
>      `get_authors.rs`, `stats/mod.rs`, `get_all_libraries.rs` (T-176), and
>      `get_media_progress.rs` (T-177) each give every such field a default.
>      **The parts that no measurement of that shape has reached**: the answer
>      of `POST /api/items/:id/play` (the chapters and the parts of the stream),
>      the answer of `GET /api/items/:id`, and the answer of the socket. The
>      harness is `docs/harness/a_field_of_the_answer_goes_away.py`, and the
>      question of every sweep of it is **which field does this program read**.
>    - **A library whose name holds no character is measured** (T-176), and the
>      words of it stay open by a decision: the program starts, the header says
>      `📖  (book)`, and the view of the key `S` holds a line of no character. No
>      server of a measurement gives such a name.
>    - **A program that says nothing at all is the shape of T-174**, and no sweep
>      of the words for the user finds it. **The parts of the program that no
>      measurement of that shape has reached**: a panic of a thread while a view
>      of the application stands (the screens of `src/ui/tui.rs` take no lock,
>      therefore the words of that panic must come to the terminal), and a panic
>      of the thread of the playback.
>    - **The rule of the line of the view is made for six views** (T-160 the Home
>      view, T-161 the queue, T-162 the chapters, T-163 the bookmarks, T-165 the
>      collections and the playlists, and T-166 the downloads of the server), and
>      **the message of each of them belongs to its own view** (T-164). **The
>      view of the episodes of a podcast needs no such rule** (T-167): its list
>      is a photograph of the moment of the open. **The parts of the program that
>      no such measurement has reached**: the search of a library that a second
>      window changes, **the key `S` of the library of the start with two windows
>      that both hold their account**, and **the view `PutInAList` of the key
>      `m`, which keeps its number of a line by the decision of T-165**.
>    - **The messages of the other views are not measured**: the view of the
>      accounts. **The reader holds one message of `say_in` now** (T-178), and
>      the rule of T-164 gave it that road: a message that a task writes with no
>      key of the user names its view, and a message of a key names no view.
>    - **The shape of T-155 is a sweep that a session began and did not
>      finish**: a write of a state that names a row of the database. T-159 gave
>      the number of the rows to the caller of the library of the account, and
>      **the writes of the sequence, of the speed, of the key bindings, and of
>      the rows of a session (`id_session`) still say nothing** when their name
>      holds no row. **T-159 may have closed the road of a key to them**: a
>      program whose account stands in no row starts again after every key,
>      therefore a measurement of that sweep needs a write with no key — the
>      loop of a playback.
> 2. **The words for the user.** Every text in ASD-STE100. A view says why it
>    holds no line, and it never says a reason that the program does not have
>    (T-91), **and the header of the screen holds that same rule** (T-171). **A
>    text must not promise a function that the program does not
>    have** (T-118), and **a footer must not promise a key that the view does not
>    hold** (T-143). **A sentence of a fault must name a key that does the work
>    of that fault** (T-170: the Library view named the key `L` of a scan of the
>    server for a request that came back with a fault). A key that does nothing
>    in one view is a fault of its own (T-79), **and a key that does nothing in
>    every view is T-167**. A message lives six seconds.
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
> (T-113). The words of a book of a scan of T-116 stay as they are. **The list of
> the devices of an e-reader comes from `POST /api/authorize`** (T-119). The
> program holds more than one account (T-124), the episodes of a podcast come
> when the user opens that podcast (T-126), **a request tries an address that
> holds the state `Down`** (T-128), **the refresh keeps the engine of the
> playback** (T-131), **a listening session belongs to one program** (T-140),
> **the limit of the cache of the ebooks comes of the file at the moment of the
> use** (T-142), **a close removes the row of that session alone** (T-145), **a
> media of a playback that did not start goes back to the front of the queue and
> the queue stops there** (T-146), **the disk is the truth of the queue**
> (T-147), **one program writes the files of one download** (T-148), **a
> download takes the address of the pool and it holds a limit of time** (T-149),
> **the key `X` takes the disk of a download and it takes no file of a download
> that runs** (T-150), **the key `X` of the queue says one sentence on both of
> its roads** (T-151), **an offline playback keeps its place for the server at
> each second** (T-152), **the cache of the ebooks keeps the book that a second
> window reads, and the cache may then stand above its limit** (T-153), and **the
> key `D` on a media that this program downloads already changes no byte of that
> download and it names this program** (T-154), and **the view of the accounts
> reads the disk at the moment of the use, and the mark of the account of the
> start never stands on nobody** (T-155), and **the key `X` removes no file of a
> media that a program of this account plays from the disk** (T-156), and **a
> playback of an account that stands in no row of the disk starts at once, and no
> wait of a playback stands longer than 30 seconds** (T-158), and **the program
> reads the accounts of the disk after every key, and a program whose account
> stands in no row starts again** (T-159), and **a line of the Home view whose
> media leaves the shelf Continue Listening goes to nobody, and the message
> names that media** (T-160), and **the cursor of the view of the queue holds a
> media and not a number of a line** (T-161), and **the line of the view of the
> chapters holds the playback that the user opened** (T-162), and **the key `b`
> of the view of the bookmarks writes a place of the media of that view alone**
> (T-163), and **a message that a rule of the loop writes belongs to its own
> view, and the answer of a key belongs to no view and it stands above them all**
> (T-164), and **the line of the view of the lists holds a collection or a
> playlist and not a number of a line** (T-165), and **the line of the view of
> the downloads of the server holds an episode of a podcast and not a number of a
> line** (T-166), and **a playback that did not start says why: the three faults
> of `play_media` each say one sentence, and that sentence names what the server
> said** (T-167), and **a request of a key that did not come back says why: the
> box of that fault belongs to the podcast or to the page of its own request**
> (T-168), and **the two views of the collections and of the playlists say what
> the server said, and the box of that fault belongs to one library** (T-169),
> and **the Home view, the view of the series, and the Library view each say what
> the server said of their own request of the start, and the sentence of a fault
> names a key that does the work of that fault** (T-170), and **an address that
> answers with a fault keeps the state `Down`, because a second address of the
> same server can answer that request, and the header of the program says that
> the server reports a fault and not that it does not answer** (T-171), and **a
> server that reports a fault at the first request of the program does not start
> the offline mode of T-25: the program says what the server said, and it stops
> with no line of its own source** (T-172), and **a login of an account that
> reaches no library writes no row of the database and it says that the server
> gave no library** (T-173), and **no screen of a field holds the lock of the
> standard output, because a hook of a panic of another thread writes to it**
> (T-174), and **a key that reads a state of the server writes nothing when that
> read came back with a fault: the status 404 is the answer of a media that
> never played, and every other fault stops the write** (T-175), and **the
> program reads the id, the name, and the media type of a library, and every
> other field of that answer takes a default** (T-176), and **the program reads
> the media of a position of `mediaProgress`, and every other field of that row
> takes a default: a row that names no media belongs to no line of any view,
> therefore it takes a line of the log and no word for the user** (T-177), and
> **the reader writes no place of a book whose place the server did not give:
> the status 404 is the answer of a book that the user never opened, and every
> other fault stops the send and says so** (T-178).
>
> All prose and user-facing strings in ASD-STE100 simplified technical English. No
> crate that needs a library of the system: `cargo tree -i openssl-sys` must find
> nothing, and `cargo tree -i cc` must find `libsqlite3-sys` and `ring` only. No
> test may need the network — the tests of the sandbox carry `#[ignore]` and run
> one at a time. The server of the maintainer is theirs alone: **never use it**,
> and never write its address or its account into this repository. Measure against
> the sandbox.
