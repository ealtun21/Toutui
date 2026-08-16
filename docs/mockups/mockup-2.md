# Toutui redesign — mockup 2: Miller columns

State shown: the Library view, moved into an open series (`The Test Chronicles`),
with a media playing, a download running, and a message on screen.
Size: 45 lines, 160 columns.

---

## (a) Lineage and what was taken

The design is **Miller columns**: the parent on the left, the current list in the
middle, and a live preview on the right. You move horizontally: `h` goes back a
column, `l` goes into a column.

| TUI | What was taken |
|---|---|
| **yazi** | The always-full preview pane, the breadcrumb at the top of the screen, and the tab strip on line 1. From yazi also comes the rule that the preview is never empty — it is the main information surface, not a leftover. |
| **ranger** | The three-column ratio and the "parent stays visible" rule. You always see the list you came from, so the series does not swallow the library. |
| **nnn** | The one-key context switches and the very small, dense status band at the foot of the screen. |
| **lf** | The per-panel bottom hint bar (`h: back  l: open`), which puts the two keys that matter for a panel on that panel. |
| **ncmpcpp / cmus** | The player band at the foot: one line for the media and the seek bar, one optional line for the player keys. |

The identity stays an audiobook player: the preview holds the cover, the writer,
the reader, the length, the chapters, your place, the files, and the sessions —
not file permissions.

---

## (b) Walk-through of every region

Line numbers are the lines of `mockup-2.txt`.

**Line 1 — Tab strip and server bar.**
`🦜 Toutui` then one tab for each library the server has (`Home`,
`Large (book)`, `Podcasts`) and a `[ + ]` button that opens a new tab.
The open tab is marked with the `▎ ▕` bookends. On the right: the account, the
server, an on-line dot, and the version.

**Line 2 — Breadcrumb and modes.**
`📚 Large (book) › Series › The Test Chronicles › Book Two`. Each part is a
button; click one to jump back to that level. Then the three mode controls:
`Sort: Added first ▾`, `Filter: All ▾`, `🔍 Search /`, then `[ ? Keys ]` and the
mouse state.

**Lines 3–38 — The four columns.**

1. **`Views` (26 wide)** — the rail. The libraries at the top, then the filter
   set (with a count for each), then the sort set (with the direction arrow),
   then the groups (Series, Authors, Narrators, Collections, Playlists), then
   the other views (Queue, Downloads, Bookmarks, Sessions, Settings). The last
   two rows are the mouse legend: `[ ]click ▾menu ━drag`.
2. **`Library · 500 of 2056` (34 wide)** — the parent list. Each row is
   `status + title`. The status field holds the percent, a `✓` for finished, or
   `▾ 3` / `▸ 8` for a series with its book count. `▾` means the series is open
   and its books are in the next column; `▸` means it is closed. The selected
   row has a `▌` bar. The right border of the panel is the scrollbar.
3. **`The Test Chronicles · 3` (40 wide)** — the current list: the books inside
   the open series, with the number in the series, the state, and the title.
   Because a series is short, the rest of the panel is **not empty**: it becomes
   a cover grid of the books near you in the library, six covers, each with a
   caption and its state. `g` turns the grid on and off.
4. **`The Test Chronicles 2` (60 wide)** — the preview. One large cover
   (30 × 15 blocks) with the facts beside it: writer, reader, length, series
   number, chapter count, files and size, the date added, genre, rating. Below
   the cover: two rows of buttons, then `Your place` with the seek bar you can
   drag, then the chapter list with the chapter you are in marked `▌ … ▶`, then
   the file and session facts.

**Lines 39–41 — Message box.** A small box above the player. It says how long it
stays and has an `[ x ]` to close it now.

**Line 42 — Download bar.** The file, a bar, the percent, the speed, the time
left, `[ Stop ]` and `[ Hide ]`.

**Line 43 — Player row.** State, media, chapter, the time now, the seek bar with
a `●` handle, the total time, the speed, the volume, the sleep timer.

**Line 44 — Player keys.** Shown only while `B` is on. Every key is a button.

**Line 45 — Footer.** The keys for the view you are in.

---

## (c) How it answers the four problems

### 1. Mouse support everywhere
Every region of the mockup is a target, and the targets are marked in the art:

* `[ ... ]` is a button you can click. They are on line 1 (tabs, new tab),
  line 2 (`? Keys`), in the preview (`Play`, `Queue`, `Done`, `Get file`), on
  the message box (`x`), on the download bar (`Stop`, `Hide`), and on the whole
  player key row.
* `▾` is a drop-down. `Sort: Added first ▾` and `Filter: All ▾` open a menu.
* `━` and `▕██…●───▏` is a bar you can drag: the seek bar in the preview and the
  seek bar in the player.
* `›` in the breadcrumb marks a part you can click to go back.
* The `█` / `│` right border of a list panel is the scrollbar; you can drag it.
* The legend at the foot of the `Views` panel states the three marks, so a new
  user learns them from the screen and not from the manual.

A click on a row selects it; a click on any part of a panel makes that panel the
focus, which is how Miller columns want to work. The wheel scrolls the panel
under the pointer, not the focused one.

### 2. Theme from the terminal
The design uses only default fg, default bg, and the 16 ANSI names — see the
table in (d). Shape carries most of the meaning, so the screen still reads with
no colour at all: the `▌` bar marks the selection, `▸`/`▾` marks open and shut,
`●`/`○` marks the mode that is on, `✓` marks finished, `[ ]` marks a button.
Colour is a second signal, never the only one.

### 3. Filtering, sorting, and open series
The `Views` rail makes the two mode sets permanent, not modal. You see that the
filter is `All` and the sort is `Added first` at all times, and you see the count
of each filter before you use it, so you know if it is worth the click. `f` and
`s` move the focus to the rail.

A series is an ordinary node of the hierarchy. In the `Library` column it is one
row, as today. Press `l` (or click) and it opens into the **next column** as its
single books; the row keeps a `▾` and the book count so you can see it is open.
Press `h` and you are back. Nothing collapses or expands in place, so the library
row numbers never move under you. The same move works for Authors, Narrators,
Collections, and Playlists, which are the same shape of node.

### 4. Covers fill the space
Two rules remove the empty rows:

* **One item selected → one large cover.** The preview holds a 30 × 15 cover with
  the facts beside it, so the widest panel is full from the first row.
* **A short list → a cover grid.** When a list does not fill its panel (a series
  of 3 books, a small collection, a search with few hits), the rest of the panel
  becomes a grid of covers, as large as they can be and still fit. The mockup
  shows 17 × 7 covers, two across in a 40-wide panel; in a wider panel the same
  rule gives three or four across. `g` makes the grid take the whole panel.

Today about 17 rows are blank. Here every row of all four panels holds
something.

---

## (d) Colour plan (16 ANSI + default only)

| Region | Colour | Attribute |
|---|---|---|
| Panel body text | default fg | none |
| Panel background | default bg | none |
| Panel border, not focused | black (bright black where the terminal has it) | dim |
| Panel border, focused | cyan | bold |
| Panel title | cyan | bold |
| Panel bottom hint | bright black | dim |
| Tab strip, tab not open | default fg | dim |
| Tab strip, tab open | bright white on blue | bold |
| Breadcrumb, older parts | bright black | none |
| Breadcrumb, last part | default fg | bold |
| `Sort:` and `Filter:` controls | yellow | none |
| Selected row bar `▌` and its text | bright white on blue | bold |
| Row, finished `✓` | green | none |
| Row, percent 1–99 | yellow | none |
| Row, not started | default fg | dim |
| Series row `▸` / `▾` and count | magenta | none |
| Rail heading (`─ Filter ─`) | bright black | dim |
| Rail mode that is on `●` | green | bold |
| Rail mode that is off `○` | default fg | dim |
| Rail counts | bright black | none |
| Cover blocks | blue (frame), bright blue and cyan for the `░▒▓` bands | none |
| Cover caption text | default fg | bold |
| Preview fact names | bright black | none |
| Preview fact values | default fg | none |
| Rating stars | yellow | none |
| Buttons `[ ... ]` | default fg on bright black | none |
| Button under the pointer | black on cyan | bold |
| Seek bar, part done | green | none |
| Seek bar, part left | bright black | dim |
| Seek bar handle `●` | bright green | bold |
| Chapter you are in | cyan | bold |
| Scrollbar `█` | cyan | none |
| Scrollbar track `│` | bright black | dim |
| Player row, state `▶` | green | bold |
| Player row, media name | default fg | bold |
| Player row, speed and volume | bright black | none |
| Download bar | magenta on default bg | none |
| Message box, note | blue border, default fg text | none |
| Message box, error | red border, red text | bold |
| Footer keys | bright black | none |
| Footer key letters | default fg | bold |

Rules: no hex, no 256-colour index. Red is only for an error. Green is only for
"done" or "playing". Nothing needs a background other than the default, except
the selected row, the open tab, and the buttons.

---

## (e) Mouse map

| Region | Click | Double click | Wheel | Drag |
|---|---|---|---|---|
| Tab (line 1) | Open that library | — | Move to the next tab | Move the tab |
| `[ + ]` | Open a new tab | — | — | — |
| Breadcrumb part | Go back to that level | — | — | — |
| `Sort: ▾` / `Filter: ▾` | Open the menu | — | Take the next value | — |
| `🔍 Search` | Open the search box | — | — | — |
| Rail row (filter, sort) | Use that mode | — | Scroll the rail | — |
| Rail row (group, view) | Open that view in the columns | — | Scroll the rail | — |
| Library row | Select it and focus the panel | Open it (a book plays, a series opens the next column) | Scroll the list | Move to a playlist |
| Series row `▸` mark | Open the series in the next column | — | — | — |
| Book row in the series | Select it; the preview follows | Play the book | Scroll the list | — |
| Cover in the grid | Select it; the preview follows | Play it | Scroll the grid | — |
| Panel border | Focus that panel | — | — | Change the panel width |
| Scrollbar | Jump to that place | — | Scroll | Scroll |
| Preview cover | Focus the preview | Play the book | Scroll the preview | — |
| Preview button | Do the action | — | — | — |
| Preview seek bar | Move to that place | — | ±10 s | Seek; the time follows the pointer |
| Chapter row | Move to the chapter | Play from the chapter | Scroll the chapters | — |
| Player row, media name | Open the book in the columns | — | — | — |
| Player seek bar | Move to that place | — | ±10 s | Seek |
| Player volume | Set the volume | — | ±5 % | Set the volume |
| Player key button | Do the action | — | — | — |
| Download bar `[ Stop ]` | Stop the download | — | — | — |
| Message `[ x ]` | Close the message | — | — | — |
| Footer key | Do that key | — | — | — |

Right click on a row opens the same menu as `m` (add to a collection, a
playlist, or the queue). The wheel always acts on the panel under the pointer,
and it does not change the focus.

---

## (f) What this design gives up

* **Width.** Four columns need about 120 columns to be good and 160 to be
  comfortable. Below 100 columns the design must drop the rail, then the parent
  column, and become the flat list it is today. That is more layout code than
  the program has now.
* **The long description.** Today the description gets four full-width rows under
  the list. Here it must live inside a 58-wide preview, under the cover and the
  chapters, so you scroll with `J`/`K` to read it. Long descriptions are worse
  off.
* **Real cover images.** The mockup draws covers as blocks. Real sixel or kitty
  images can go in the same boxes, but a grid of many small images is much more
  work for the terminal than the one image the program draws today, so the grid
  may need a cache and a size limit.
* **One list, one place.** A user who knows the current flat list must learn that
  a series now moves right instead of expanding in place. `h` and `l` are the
  same keys as today, but the picture on the screen is new.
* **Screen height.** The player keys row, the download bar, and the message box
  can be on at the same time, as in the mockup. That is 5 rows of the 45. On a
  short terminal (24 rows) the panels get 12 rows, so the cover grid and the
  chapter list must both shrink.
