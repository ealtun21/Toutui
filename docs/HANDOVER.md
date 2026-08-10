# The handover of 2026-08-11 (the second session of that day)

This document is for the next session. It says what is done, what is open, and
the traps that cost time. Read `docs/TAKEOVER-BACKLOG.md` for the evidence of
each item, and `docs/T-24-coverage.md` for the comparison with the server.

## The state

`main` is clean and pushed. The newest release is **v0.7.0**. Every gate
passes:

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
ALSA_CONFIG_PATH=<a real null asound file> cargo test    # 597 tests, 25 binaries
```

**The tag `v0.6.6` has no release.** That tag came before the version of
`Cargo.toml`, and the workflow refused it, as it must: a tag that does not
agree with `Cargo.toml` would give every user an update that never completes.
The work of that tag is in v0.6.7. Do not try to publish v0.6.6.

## What this session closed

The three items that the comparison of T-24 left open. **Every item of
`docs/T-24-coverage.md` section 5 that is not in section 6 is now done.**

| Item | What | Keys |
|---|---|---|
| The statistics of the user | `GET /api/me/listening-stats`, one request, no new crate | `T` |
| The other shelves of Home | Every shelf of `/personalized`, and the request did not change | — |
| The sequence and the filter | `?sort=`, `?desc=`, `?filter=`, and `/filterdata` | `f` |

### The faults that a real measurement found

1. **The Home view of a library of podcasts was empty.** That library gives no
   shelf `continue-listening`, and the program kept that shelf only. The view
   shows "Newest Episodes" and "Listen Again" now.
2. **The six lists of the Home view could move against each other.**
   `collect_ids_cnt_list` pushed a value for every entity, and the five other
   functions pushed for an entity of a media only. One shelf of series would
   have put the title of one book beside the author of a different book. Every
   function walks one iterator now, and a test holds each list to the same
   length.
3. **A date filled the whole name of a line of the statistics.** The name took
   ten columns, and a date of the form `2026-08-10` takes ten columns. The date
   and the bar stood together with no space.

## What is open

`docs/T-24-coverage.md` section 5 names the work that stays. In the sequence
of the value:

1. **Bookmarks** (medium). `POST /api/me/item/:id/bookmark`,
   `DELETE /api/me/item/:id/bookmark/:time`, and the field `bookmarks` of
   `GET /api/me`. A user of a long book needs a place to come back to.
2. **Hide a media from Continue Listening** (small). The field
   `hideFromContinueListening` of `PATCH /api/me/progress/:id`. The same file
   as the key `M`.
3. **Add a podcast** (medium). `GET /api/search/podcast?term=`, then
   `POST /api/podcasts/feed`, then `POST /api/podcasts`. The README names it.
4. **The list of the chapters** (small). The answer of
   `POST /api/items/:id/play` holds them already, and the player shows the name
   of the chapter only.
5. **A timer for sleep** (large). No endpoint. The work is in
   `src/player/engine/worker.rs`.

## The traps that cost time

1. **`ALSA_CONFIG_PATH=/dev/null` stops the real program.** It is correct for
   `cargo test`, because no test opens a sound device. A real run writes "The
   pool has 1 address(es)" and then draws nothing. Give it a real file:
   ```
   </usr/share/alsa/alsa.conf>
   pcm.!default { type null }
   ctl.!default { type null }
   ```
2. **A pseudo terminal must answer two questions, and not one.** This cost an
   hour in this session: the key `R` looked as if it stopped the program.

   | Question | Answer | Who asks |
   |---|---|---|
   | `ESC [ 5 n` | `ESC [ 0 n` | `ratatui-image`, at the start |
   | `ESC [ 6 n` | `ESC [ 1 ; 1 R` | crossterm, inside `terminal.clear()` |

   A harness that answers the first one only lets the program start, and the
   program then stops at the first `R` with "The cursor position could not be
   read within a normal duration". **Read the bytes of the pseudo terminal
   before you look for a deadlock**: the message stands there, and the log of
   the program does not hold it.
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
7. **The server takes a name of a field that does not exist.**
   `?sort=bogus.field` gives `200` and an unspecified sequence. Measure a field
   before you offer it, and refuse a value that the build does not know.
8. **`items` of `GET /api/me/listening-stats` is a map, and not a list.** A map
   has no sequence, therefore the code must make one or the screen moves at
   each frame.

## The rules that do not change

- Every document, comment, and text for the user in ASD-STE100 simplified
  technical English. Short sentences, active voice, one instruction per
  sentence.
- No crate that needs a library of the system. `cargo tree -i openssl-sys`
  must find nothing. `libsqlite3-sys` and `ring` are the two known builds of C,
  and they stay. See T-20. This session wrote base64 by hand for that rule.
- No test may need the network. A test that needs the sandbox carries
  `#[ignore]` and says how to run it. Four such tests exist now.
- Never write to `AlbanDAVID/Toutui`. It is archived. AlbanDAVID stays credited
  in the README, in the LICENSE, in `Cargo.toml`, and in the settings screen.
- Show a fault before you correct it. Three of the faults of this session only
  a real process or a real answer of the server showed.
- Tag, push, and go on. Do not wait for continuous integration.
