# The handover of 2026-08-11

This document is for the next session. It says what is done, what is open, and
the traps that cost time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of
each item, and `docs/T-24-coverage.md` for the comparison with the server.

## The state

`main` is clean and pushed. The newest release is **v0.6.9**. Every gate passes:

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> cargo test    # 523 tests, 23 binaries
```

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must: a tag that does not
agree with `Cargo.toml` would give every user an update that never completes.
The work of that tag is in v0.6.7. Do not try to publish v0.6.6.

## What this session closed

| Item | What |
|---|---|
| T-33 | ratatui 0.30, crossterm 0.29, `tui-input` in place of `tui-textarea` |
| T-23 | The cover art, beside the description, with a shelf for a series |
| T-22 | A series takes one line of the Library view |
| T-32 | The key `F` sends the position at once |
| T-10 | The reader of EPUB: the key `e`, the place of the user, the EPUBCFI of the web reader |
| T-31 | macOS has a way to remove the program with no binary |
| T-24 | `docs/T-24-coverage.md`, and three of its five items |
| T-36 to T-46 | The ten items of the report of the user of 2026-08-10 |

## What is open, in the order of the value

The comparison names five items. Two are complete. These three stay:

### 1. The statistics of the user (small)

`GET /api/me/listening-stats` gives `totalTime`, `today`, `days`,
`dayOfWeek`, `items`, and `recentSessions`, and every value is ready to draw.
The README names "stats" as a function that the program must have.

The work: a new `src/api/me/listening_stats.rs`, a new `AppView`, and a key.
No new crate. Follow `src/api/libraries/search_library.rs` for the shape of a
new endpoint, and `src/ui/reader_tui.rs` for a view with pure functions.

### 2. The other five shelves of the Home view (medium)

The program asks for `/api/libraries/:id/personalized` and it keeps one shelf
of six. The other five are "Recently Added", "Discover", "Listen Again",
"Recent Series", and, for a podcast library, "Newest Episodes". The data comes
already, and nothing more goes to the server.

The work: `get_library_perso_view.rs` keeps the shelf `continue-listening`
only, in `is_the_shelf_of_continue_listening`. The Home view then needs more
than one list, and that is the largest part.

### 3. The sort and the filter of a library (medium)

`?sort=media.metadata.title&desc=1` works, and it is measured.
`GET /api/libraries/:id/filterdata` gives what a filter can hold, and the
filter itself is `?filter=<type>.<base64>`. `get_all_books.rs` holds a field
`sort_by` that it never sends.

A library of 2056 items needs this more than a small one.

## The traps that cost time in this session

1. **`ALSA_CONFIG_PATH=/dev/null` stops the real program.** It is correct for
   `cargo test`, because no test opens a sound device. A real run writes "The
   pool has 1 address(es)" and then draws nothing. Give it a real file:
   ```
   </usr/share/alsa/alsa.conf>
   pcm.!default { type null }
   ctl.!default { type null }
   ```
2. **A pseudo terminal must answer the Device Status Report.** `ratatui-image`
   asks the terminal what it can do, and the last question is `ESC [ 5 n`. A
   harness that answers nothing leaves the reader of the crate inside `read`,
   and that reader takes the **first key of the test**. Answer `ESC [ 0 n`. A
   comparison with an older build tells a fault of the harness from a fault of
   the program.
3. **The device `null` plays a book of 60 seconds in one second.** A test of
   the player needs "A Long Test Book", which is thirty minutes.
   `docs/TEST-SERVER.md` section 6e makes it.
4. **`currentTime` comes as a text, and not as a number,** in
   `GET /api/me/progress/:id`.
5. **A test that sets `XDG_CONFIG_HOME` must be alone in its binary.** That
   variable belongs to the process, therefore two tests of one file share one
   database and they fight. Put the parts in one test function.
6. **`cargo fmt` before `git commit`, every time.** The gate refuses a file
   that `rustfmt` would change.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified
  technical English. Short sentences, active voice, one instruction per
  sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys`
  must find nothing. `libsqlite3-sys` and `ring` are the two known builds of C,
  and they stay. See T-20.
- No test may need the network. A test that needs the sandbox carries
  `#[ignore]` and says how to run it.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited
  in the README, in the LICENSE, in `Cargo.toml`, and in the settings screen.
- Show a fault before you correct it. This session found three faults that only
  a real process showed: a deadlock of a lock, a terminal that lost its raw
  mode, and a position of 0 that reached the server.
- Tag, push, and go on. Do not wait for continuous integration.
