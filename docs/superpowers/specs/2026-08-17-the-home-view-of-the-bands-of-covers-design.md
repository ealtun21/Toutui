# The Home view of the bands of covers

A design of 2026-08-17, for **T-331**. The maintainer asked for this view on
2026-08-16, and `docs/mockups/mockup-6.txt` and `docs/mockups/mockup-6.md` hold
the picture of it and the five decisions of the maintainer. **This file holds
the parts that the mockup does not say**, and no round writes code of T-331
before it reads this file.

**This design must not change a decision of the mockup**: the maintainer
approved that note.

## The problem

The Home view is one table. **The measurement of the real program v0.8.167
inside tmux of 2026-08-17**, of 160 columns and 45 rows, of the library `Large`
of 2056 items:

```text
┌1 Views ────────────────────────┐╔4 Home [20 items] ══════════════════════════════╗ ┌5 Cover ─────────────┐
│➤ Home                       Tab│║    Title                     Author      Time  Done║ │Time      0m         │
│  Library                    Tab│║  ▌ Recently Added                                  ║ │Files     1 file     │
│  Sequence and filter          f│║➤   Large Book 0001                       <1m     -║ │No description       │
│  Authors                      a│║    Large Book 0002                       <1m     -║ │                     │
...
│                                │║  ▌ Discover                                        ║ │                     │
│                                │║    Large Book 1434                       <1m     -║ │                     │
```

The table says the same four columns — the title, the author, the time, and the
mark of the end — for every shelf. The covers of those media stand in the panel
6 of the gallery alone, far from the row of each of them.

## What the server gives

**The measurement of the sandbox of 2026-08-17**,
`GET /api/libraries/<id>/personalized` of the five libraries:

| The library | The shelves, in the sequence of the server |
|---|---|
| `Books` (book) | `continue-listening` (book, 5 of 5), `recently-added` (book, 10 of 22), `recent-series` (series, 3 of 3), `discover` (book, 7 of 7), `listen-again` (book, 10 of 10), `newest-authors` (authors, 9 of 9) |
| `Large` (book) | `recently-added` (book, 10 of 2056), `discover` (book, 10 of 2056) |
| `Podcasts` (podcast) | `continue-listening` (episode, 4 of 4), `newest-episodes` (episode, 10 of 68), `recently-added` (podcast, 2 of 2), `listen-again` (episode, 3 of 3) |
| `ManyPods` (podcast) | `newest-episodes` (episode, 10 of 520), `recently-added` (podcast, 10 of 520), `listen-again` (episode, 2 of 2) |
| `Empty` (book) | no shelf at all |

**Three facts of that measurement decide parts of this design.**

1. **A shelf holds ten entities at the most, and the field `total` of it says a
   number that the program cannot reach.** The shelf `recently-added` of the
   library `Large` holds 10 entities and it says `total: 2056`. **The count in
   the title of a band therefore says the media that the program holds, and
   never the field `total`**: a band that says `6 of 2056` promises 2050 media
   that no key of the user can reach, which is the rule of T-118.
2. **A shelf of the type `authors` gives no media.** `group_home` of
   `src/logic/home_view.rs:69` drops a shelf that gives no line already, and
   the shelf `newest-authors` is that shelf. **The band of the authors stays
   outside this design**, and the reason stands in "What stays outside" below.
3. **A library of no shelf gives no band**, and the words of that view keep the
   rule of T-91.

## The program of today, in the parts that this design touches

- **`src/logic/home_view.rs`** makes one flat list, `Vec<HomeRow>`, of the
  variants `Shelf { label }`, `Media { item }`, and `Series { series }`. The
  cursor of the view is one number, the line of that list, and it lives in
  `App::list_state_cnt_list`. `first_line`, `last_line`, `next_line`, and
  `previous_line` move that number over the lines of a shelf.
- **Every key of a media of the Home view reads that one number**: the keys
  `D`, `X`, `n`, `m`, `@`, `e`, and `V`, the facts of the panel 5, the place of
  the user, and the download of the media each take the line of the cursor.
- **`src/ui/tui.rs::render_home`** makes `home_table_rows()` of that list and it
  gives them to `render_the_table_of_a_panel`, which the Library view uses too
  (T-321).
- **`src/ui/cover.rs`** holds the store of the covers, keyed by the id of the
  media, of the states `Asked`, `NoCover`, `Fault`, and `Ready`. `request` holds
  one request of one id at a time, and it puts no limit on the number of the
  ids. `CoverArt::picture` decodes one picture one time, and the key `R` alone
  forgets the store (`cover::forget`).
- **`src/ui/the_panel_of_the_gallery.rs`** draws the grid of the panel 6 around
  the cursor of the flat list, with the cells of the widths
  `[8, 10, 14, 20]`, and `the_rows_of_a_picture` takes the rows of a picture of
  the `FontSize` of the picker.
- **The frame of T-320** gives three shapes: three columns at 120 and up, two
  columns from 84 to 119, and one column under 84.

## The idea of this design

**The bands are a shape of the render, and the flat list of the lines stays the
data.** `group_home` keeps its work with no change, the cursor of the view stays
one number of that list, and a new module makes the bands of it:

```rust
/// src/logic/the_bands_of_the_home.rs
pub struct ABand {
    /// The name of the shelf, of `HomeRow::Shelf`.
    pub the_title: String,
    /// The line of the flat list of each cell, in the sequence of the shelf.
    pub the_cells: Vec<usize>,
}

pub fn the_bands(rows: &[HomeRow]) -> Vec<ABand>;
```

**That decision keeps the program of today**: every key of a media, the panel 5
of the facts, the panel 6 of the gallery, the message of a media that went away,
and the mouse of a click each read the line of the flat list, and none of them
changes. **It keeps the 23 tests of `src/logic/home_view.rs` too**, and the
section of the tests below names what stays and what needs a new measurement.

## The parts

### The bands, and the cells of one band

- **One band for one shelf**, in the sequence of the server (the decision 2 of
  the maintainer).
- **One cell for one line of the shelf**: a media, and a series of the shelf
  `recent-series`.
- **The cell of a series draws the cover of the first book of that series**,
  because a series of Audiobookshelf holds no cover of its own. `App::series`
  holds the books of it already (T-22).
- **The band holds an offset of its own**, the number of the first cell that
  the band draws. A band that the user did not move holds the offset 0.

### The layout

The width of a cell is `THE_WIDTHS_OF_A_CELL` of
`src/ui/the_panel_of_the_gallery.rs`, and the cell of the band takes the same
value, because the cell of the gallery and the cell of a band are one picture in
one border. The rows of the picture come of `the_rows_of_a_picture`, of the
`FontSize` of the picker, and never of a number of the mockup.

```text
the cells of a band = (the width inside the panel + 1) / (the width of a cell + 1)
the rows of a band   = 1 (the title) + the rows of a picture + 2 (the border) + 1 (the space)
the bands of a panel = the height inside the panel / the rows of a band
```

**The panel draws whole bands alone**, which is the rule of the panel 6 of
T-327: a part of a band holds rows of the screen that no cell uses.

**A panel that has no room for one whole band draws the table of today.** That
is not a second shape of the design and it is not a setting: it is the rule of
T-321 for a panel that is too narrow for its table, and this fork measures 40
columns as its narrowest screen (T-301). A band needs about 10 columns and about
6 rows, and a screen of 40 columns and 12 rows holds no band at all. **The
decision 5 of the maintainer holds**: no key of the user turns between the two
shapes.

**The narrow screen keeps the frame of T-320**, of 120 and 84 columns, and not
the 100 columns of `mockup-6.md`. **The reason**: the stack of the panels 1 to 3
belongs to the frame of every view, and a Home view that took the stack away at
a width of its own would give two rules of one frame. The count in the title of
the band says how many cells the user sees, therefore the narrow screen says
itself.

### The title of a band

```text
 Continue Listening ───────────────────────────────── 6 of 10  ›
```

- The name of the shelf comes of `home_view::the_name_of_the_shelf`, with no
  change.
- The count says **the cells that the band draws** and **the cells that the band
  holds**, of the media of the program (the fact 1 above).
- The arrow `›` stands while the band holds a cell at the right of the last cell
  that it draws, and a `‹` stands while the offset of the band is not 0.

### The keys

| The key | What it does in the Home view |
|---|---|
| `j`, `k` | The band under, and the band above. The cell of the cursor keeps its number in the new band, and the last cell of that band takes it when the band is shorter. The move goes round at the two ends, which is the rule of `next_line` of today. |
| `h`, `l` | The cell at the left, and the cell at the right, of the band of the cursor. **The move stops at the two ends of the band**, and it does not go to the band beside it. |
| `g`, `G` | The first cell and the last cell of the band of the cursor. |
| `Enter` | Plays the media of the cursor, or it opens the series or the podcast of it. |

**The key `l` plays no media in this view.** `Enter` is that key now, and the
footer of the view says the four moves and it says `Enter: play or open`.
**The reason**: a key of two meanings in one view is a fault of its own, and the
user of a band reads `h` and `l` as the two directions of that band. **The key
`Enter` is an alias of `l` in every other view of the program already**
(`src/app.rs:3195` and the four arms beside it), therefore no other view
changes, and a user who presses `Enter` on a row of any list of this program
gets the same work as today.

**The footer of the Home view**, in the place of `FOOTER_OF_A_LIBRARY_OF_BOOKS`
of `src/ui/keys.rs:511`:

```text
j/k: a shelf  h/l: a cover  Enter: play or open  Tab: home/library  S-Tab: the next library  /: search  R: refresh  ?: every key  Q: quit
```

The library of podcasts says `Enter: the episodes` in the place of
`Enter: play or open`, which is the rule of
`FOOTER_OF_A_LIBRARY_OF_PODCASTS` of today.

**The keys of a media do not change**: `D`, `X`, `n`, `m`, `@`, `e`, `V`, and
every key of the player read the line of the cursor, and the cursor is a line of
the flat list.

### The covers: how many, and when

**The rule: the program asks for the cover of a cell that the frame draws, and
of no other cell.** That is the rule of the gallery of today, and it holds the
number down by itself: a band draws about 6 cells, and a panel of 45 rows holds
5 bands, therefore the first frame of the view draws about 30 cells.

**A limit of the new requests of one frame stands beside it**:

```rust
/// The largest number of covers that one frame asks the server for.
pub const THE_NEW_COVERS_OF_A_FRAME: usize = 8;
```

A frame that meets more new ids than that asks for the first eight of them and
it leaves the others for the frame after it. **The reason**: `cover::request`
spawns one task of tokio for each id with no limit at all, therefore a key that
moves the view to 30 media that no frame drew gives 30 requests of one moment,
and the round of T-129 measured that the four requests of the start already
stand before the first frame. **The number 8 is a candidate and not a
measurement**: the round of the covers measures the requests with the log of a
proxy —

```bash
python3 docs/harness/one_path_fails.py 13500 13399 requests.log /a-path-of-no-request
grep -c '/cover' requests.log
```

— and it writes the number of the first frame, of the key `j`, and of the key
`R` in its item of `docs/TAKEOVER-BACKLOG.md`. **A measurement that says the
limit costs the user a band of empty cells for more than one frame takes the
limit up.**

### The terminal that draws no pictures

`TOUTUI_NO_COVERS`, a terminal of no protocol (`cover::asks_the_terminal` gives
the half blocks then), and a media that the server holds with no cover
(`cover::no_picture_comes`): **the cell keeps its border and its place**, and it
holds the title of the media in the rows of the picture, cut to the width inside
the border with the ellipsis of this program.

**The band does not become a table**, because the keys of a view must not change
with the terminal of the user.

### The panels 5 and 6

- **The panel 5 of the cover says the facts of the media of the cursor**, with
  no change at all: it reads the line of the flat list (T-325 and T-326).
- **The panel 6 of the gallery draws the whole shelf of the cursor**, and not
  the media of the whole Home view. `App::the_media_of_the_gallery`
  (`src/app.rs:2697`) takes the cells of the band of the cursor for the Home
  view, and it keeps the rows of the Library view with no change.

### The mouse

The map of `mockup-6.md`, in the words of `src/ui/the_mouse.rs`:

| The report | The target | What the program does |
|---|---|---|
| A click of a cell | `TheTarget::TheListOfTheView { the_line }` | The cursor takes that line. **The target of today**, therefore `App::handle_the_mouse` keeps its arm. |
| Two clicks of a cell | The same target, of two reports inside the time of a double click | Plays or opens that media, which is the work of `Enter`. |
| The wheel over a band | A new target, `TheTarget::TheBandOfAShelf { the_band }` | Moves that band to the left or to the right by one cell. |
| A click of the title of a band | The same new target | The cursor takes the first cell that the band draws. |

**The rule of T-316 holds**: one click of a media takes that media and it opens
nothing. The second click opens it, and the mockup asks for that.

## The decisions of this design, and the reason of each

1. **The flat list of the lines stays, and the bands are a shape of the
   render.** It keeps every key of a media, the panel 5, the message of a media
   that went away, the mouse of a click, and 23 tests of `home_view.rs`.
2. **`Enter` plays or opens, and `l` moves to the right.** A key of two meanings
   in one view is a fault of its own, and `Enter` is an alias of `l` in every
   other view already.
3. **`h` and `l` stop at the two ends of a band, and `j` and `k` go round at the
   two ends of the view.** The move of the bands keeps the rule of `next_line`
   of today; the move inside a band is a move of a picture, and a jump to the
   other end of a shelf of covers says nothing to the user.
4. **The count of the title says the media that the program holds**, and never
   the field `total` of the server (T-118).
5. **A panel that has no room for one whole band draws the table of today**, as
   a narrow panel draws the list of today (T-321). No key of the user turns
   between the two shapes.
6. **The frame keeps the 120 and the 84 columns of T-320**, and not the 100
   columns of the mockup.
7. **The cell of a series draws the cover of the first book of it**, because a
   series holds no cover of its own.
8. **One frame asks for eight new covers at the most**, and the round of the
   covers measures that number against the sandbox.

## What stays outside

- **The band of the shelf `newest-authors`.** An author holds no media, no
  cover of the store of this program, and no key of a view. The picture of an
  author stands at `GET /api/authors/:id/image`, which is a path that this
  program never asked for, and a band of authors therefore needs a store, a
  view of the books of an author, and a measurement of its own. `group_home`
  drops that shelf today, and it keeps that work.
- **A setting of `config.toml`** of the shape of a band, of the width of a cell,
  or of the shape of today. The decision 5 of the maintainer says one Home view.
- **The drag of the mouse** over a band. The drag stands open for the bar of the
  player already (T-322), and the two belong to one round.
- **The Library view.** It keeps the table of T-321.

## The road, in rounds

**One round is one item of `docs/TAKEOVER-BACKLOG.md`, one commit, and one
release.** Each round drives the real program inside tmux before its change and
after it, and it puts the two screens in its item (the rule of the items of the
maintainer).

| The round | What it gives | The gate |
|---|---|---|
| 1 | `src/logic/the_bands_of_the_home.rs`: the bands of the flat list, the moves of `h`, `l`, `j`, `k`, `g`, and `G`, and the count of a title. No screen changes. | A test file of the pure functions, of a shelf of no media, of a band of one cell, and of the move at the two ends. |
| 2 | The render of the bands in the panel 4 of the Home view, the keys, and the footer. The table of today stays for a panel of no room. | A test that draws the bands into a `Buffer` and reads the cells, the title, and the arrows, and a test of the footer (T-143). |
| 3 | The mouse: the click of a cell, the two clicks, the wheel over a band, and the click of a title. | A test of `the_target_of_a_point` of the new areas, and a test of the arm of `App::handle_the_mouse`. |
| 4 | The covers: the limit of the new requests of one frame, with the measurement of the log of the proxy. | A test of the limit, and the numbers of the measurement in the item. |
| 5 | The panel 6 of the gallery of the shelf of the cursor, and the terminal that draws no pictures. | A test of `the_media_of_the_gallery` of the Home view, and a test of the cell of a title. |

**The round 2 is the round that changes the screen of the user**, therefore the
release of it holds the two screens of the item and the entry of the changelog
says the new keys.

## The tests that measure the table of today

**The sweep of 2026-08-17.** The tests below measure the Home view, and the
round that gives them a new measurement names them in its item.

**What keeps standing with no change** (the flat list stays):

- The 23 tests of `mod tests` of `src/logic/home_view.rs`, and
  `tests/a_shelf_of_no_name_keeps_the_home_view.rs`.
- `tests/the_keys_of_a_line_of_more_than_one_media_read_that_line.rs`,
  `tests/the_place_of_an_episode_of_a_podcast_holds_that_episode.rs`, and
  `tests/the_bookmarks_of_an_episode_of_a_podcast_hold_that_podcast.rs`: each of
  them selects a line of `home_rows` with `list_state_cnt_list`, and that is the
  cursor of the design.
- The five tests of `src/api/libraries/get_library_perso_view.rs`, and the
  message tests of `AppView::Home`.
- `tests/the_frame_of_the_panels_holds_its_three_shapes.rs` and
  `tests/the_screen_survives_a_short_list.rs`.
- `tests/the_click_of_a_chapter_plays_that_chapter.rs`: the Home view opens no
  row at one click, and the decision of the mouse above keeps that rule.

**What needs a new measurement**:

| The file | Why |
|---|---|
| `tests/the_mouse_of_the_program_reaches_its_panels.rs` | The head of it measures the Home view of the library `Large` as one list of rows, and the click of a cell of a band takes the arithmetic of a cell and not of a row. **The tests of the Library view of that file keep standing.** |
| `tests/the_panel_of_the_gallery_shows_the_media_of_the_list.rs` | The grid of the panel 6 comes of the shelf of the cursor in the Home view now (the round 5). |
| `tests/the_table_of_the_panel_4_holds_its_columns.rs` | The head of it says that the Home view and the Library view are the two views of that table. The Home view keeps it for a panel of no room alone, and the words of the head must say so. |
| `tests/the_panel_of_the_cover_of_the_home_view_says_the_facts.rs` | The facts keep the sequence of the flat list, therefore the tests stand; the head of the file names the table, and the words of it change. |
| The footer of `src/ui/keys.rs` | `FOOTER_OF_A_LIBRARY_OF_BOOKS` is the footer of the Home view **and** of the Library view. The Home view takes a footer of its own, and the Library view keeps this one. |

## The traps that this design sees already

- **A band that draws a cell of a picture that the store does not hold yet must
  keep the place of that cell**, because a cell that comes and goes moves every
  cell beside it at each frame.
- **`cover::forget` of the key `R`** takes every picture away, therefore the
  first frame after that key meets the limit of the new covers of a frame. The
  round 4 measures that key.
- **The cursor of the flat list can stand on a line of a shelf** (`HomeRow::
  Shelf`) after a refresh: `home_view::first_line` gives the first line of a
  media, and the bands must never give a cell of a title.
- **A screen of the mockup is not a screen of the program**: the boxes of
  `mockup-6.txt` hold three rows of a picture, and the rows of a picture come of
  the `FontSize` of the picker of the terminal of the user.
