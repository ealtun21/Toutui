# Mockup 5 — the cover shelf, driven by a live picker

File: `mockup-5.txt` (exactly 45 lines, every line at most 160 display columns; verified with a
`unicodedata.east_asian_width` counter that gives emoji 2 columns).

---

## (a) Lineage and what was taken

The design is **a cover-art gallery driven by a live fuzzy picker**. The library is a bookshelf, and
books have faces, so the faces get the space. Four named tools gave concrete parts:

| Tool | What was taken |
| --- | --- |
| `television` | The query line is **always** on screen (line 2), never a mode you enter and leave. The match count sits at the right end of that same line (`37 of 2056 match`), so the number is next to the letters that made it. |
| `fzf` | The prompt shape `Search ❯ chr▏` with a real block cursor, and the rule that every keystroke reflows the result set at once — no Enter, no "apply". |
| `helix` pickers | The **preview follows the cursor**. Helix splits the picker and shows the file under the cursor; here the bottom strip (lines 32-37) is that preview, and it repaints on every move of the selection. |
| `superfile` grid | The tile grid itself: fixed-pitch tiles, a bright frame on the one under the cursor, and a scrollbar column that shows how much shelf is left. |

Also taken from `ranger`/`lf`: a series is a *directory*. You open it and the same pane refills with
its contents, and `h` walks back out.

---

## (b) Walk-through of every region

| Lines | Region | What it holds |
| --- | --- | --- |
| 1 | **Status bar** | Version, the account and server, and the current library as a clickable drop-down `[ ▤ Large (book) ▾ ]`. On the right: the item count and the view buttons `[ Home ] [ Library ] [ Series ] [ Authors ] [ Settings ]`. |
| 2 | **Query line** (always there) | `Search ❯ chr▏` on the left, `37 of 2056 match` and `[ x clear ]` on the right. Typing narrows the grid live. This line never goes away, so `/` only moves focus to it. |
| 3 | **Chip bar** | `Sort ▾` with six chips, `Show ▾` with four filter chips, and `[ ▦ Grid ] [ ☰ List ]` for the view mode. The chip in force is drawn in reverse video. |
| 4 | **Grid rule** | Names the geometry in force (`8 tiles across`, `rows 1-2 of 5`) so the user can see why the grid looks the way it does, and reminds them the wheel scrolls. |
| 5-17 | **Tile row 1** | Eight tiles. Tile 1 is a **series** tile (`▤ The Test Chro…`, `3 books`) with a stacked shadow edge down its right side and under its foot. Tile 4 is the **selected** tile: a heavy frame and a `2x-click ▸` hint welded into its bottom border. Tiles that are part-heard replace the last row of the cover with a progress bar (`██████████░░░░░░` = 62%). |
| 18 | Gap row | One blank row between tile rows; the scrollbar column continues through it. |
| 19-31 | **Tile row 2** | Eight more tiles. The scrollbar thumb (`█`) ends part way down, so the shelf visibly goes on. |
| 32 | **Detail rule** | `The tile below the cursor`. |
| 33-37 | **Detail strip** | Title and the facts line (author, year, length, progress, time left); one line of description; a file/chapter/bookmark line with the action buttons `[ l play ] [ n queue ] [ m collect ] [ D download ] [ M finish ]`; the series line, which says what `l` on the series tile will do; and the mouse legend, which states the rule that **every `[ bracket ]` on the screen is a mouse target**. |
| 38-40 | **Message box** | The existing six-second message box, floated over the grid, unchanged in behaviour. |
| 41 | **Download bar** | `Get the file 1 of 3`, the item, a bar, the percent and the rate, and `[ Esc hide ]`. Appears only while something downloads. |
| 42-43 | **Player** | Row 42: state `▸`, title, chapter, then `[ 1.25x ] 🔊 [ 78 ] 💤 [ 25m ]` and `0:03:14 / 0:30:00`. Row 43 is a full-width seek bar: `━` for played, `┃` for the head, `─` for the rest, `┬` for chapter marks. |
| 44 | **Player keys** | The existing `B`-toggled row. |
| 45 | **Footer** | The short key list, plus `mouse: on` at the right end. |

A terminal that **cannot draw images** loses nothing but the picture: the tile rectangle keeps its
frame, its progress bar and its two caption rows, and Toutui fills the rectangle with the shading
blocks shown in the mockup, keyed off the cover's average colour. The grid, the picker, the sort
chips and the mouse all work the same. A user with no image support can also press `[ ☰ List ]` and
get today's list.

---

## (c) How it answers the four problems

### 1. Mouse support everywhere

The screen states its own rule on line 37: **every `[ bracket ]` is a mouse target**, so the user
learns one thing, not twenty. Full map in section (e). The parts that are not brackets are the tiles
themselves, the scrollbar column, and the seek bar, and all three behave the way the shapes suggest:
a tile is a button, a scrollbar is a slider, a bar is a bar.

### 2. Theme from the terminal's own colours

Nothing is coloured by hue meaning; colour only marks **role**. The whole screen reads correctly with
default fg on default bg and no colour at all, because every state also has a shape: the selected
tile has a heavy frame, the finished item has `✓`, the series tile has a shadow edge, the chip in
force is reverse video. See the table in (d). Only the 8 ANSI names and their bright variants are
used, so a light terminal theme and a dark one both work.

### 3. Filtering, sorting, and opening a series

Line 3 is the whole control surface: `Sort ▾ [ Added first ] [ Title ] [ Author ] [ Duration ]
[ Progress ]` and `Show ▾ [ All ] [ New ] [ Playing ] [ Finished ]`. Click a chip, or press `s` to
step the sort and `S` to step the filter. The chip in force is reverse video; the sort chip also
carries an arrow for direction on a second click.

A **series is one tile** with a stacked, shadowed edge (tile 1 of row 1) and the caption `3 books`.
Press `l`, or click it twice, and the same grid **refills with that series' 3 books**, in series
order, with the rule on line 4 changing to `The Test Chronicles · 3 books · book order`. The query
line then searches inside the series. `h`, or a click on the rule, walks back out to the full shelf.
Line 36 says this in the mockup so the behaviour is discoverable without a manual. The `[ Series ]`
button on line 1 opens the list of all series as a grid of series tiles.

### 4. Tile arithmetic — 160, 80 and 40 columns

Geometry, all in cells:

* **Large tile**: box 18 wide x 11 tall (16x9 of cover inside the frame), plus 2 caption rows =
  **13 rows**. Horizontal gap 2, vertical gap 1, so the pitch is **20 x 14**.
* **Small tile**: box 14 wide x 9 tall (12x7 inside), plus 2 caption rows = 11 rows. Gap 1 both ways,
  pitch **15 x 12**. The `[ ▦ Grid ]` chip switches size; the default is chosen by width.
* Column 1 is the left margin, the last column is the scrollbar.
* Columns of tiles: `n = floor((cols - 2 + gap) / (tile_w + gap))`, spare cells go to the margins.
* Rows of tiles: `m = floor((grid_rows + 1) / (tile_h + 1))`.

**At 160 x 45** — large tiles, `n = floor((160 - 2 + 2) / 20) = 8`. Used width `8*18 + 7*2 = 158`,
plus 1 margin and 1 scrollbar = 160 exactly, no waste. Chrome is 4 rows on top and 14 at the bottom,
so the grid gets rows 5-31 = 27 rows, and `m = floor(28 / 14) = 2`. **8 x 2 = 16 covers on screen.**
Switch to small tiles and it is `floor(159/15) = 10` across by `floor(28/12) = 2` down = **20 covers**.
Hide the player and the message box and the grid takes 33 rows, still 2 large rows; at 59 lines it
becomes 3.

**At 80 columns** — the width drops below 100, so Toutui picks small tiles: `n = floor((80 - 2 + 1)
/ 15) = 5`. Used width `5*14 + 4*1 = 74`; the 6 spare cells become a 3-cell margin each side. At 45
rows that is `m = 2`, so **5 x 2 = 10 covers**. The detail strip loses its right-hand button block
and keeps the facts line; the chip bar drops to `Sort ▾ [Added] [Title] [Author] …` with short chip
names.

**At 40 columns** — `floor((40 - 2 + 1) / 15) = 2` tiles would fit, but a 14-cell caption cannot hold
a title and the detail strip cannot hold a sentence, so the grid is **not** used. The rule is: the
grid needs **56 columns or more and 24 rows or more**; below either number Toutui falls back to the
**list**, which is today's screen and is known to work at 40 columns:

```
Search ❯ chr▏      37/2056
 62% The Chronicler
 ▤   The Test Chron… [3]
 ✓   A Christmas Carol
```

The query line, the sort order, the filters and the series-open behaviour are all the same in list
mode; only the covers go. The `[ ☰ List ]` chip forces this mode at any width, and `[ ▦ Grid ]` is
greyed out below 56 columns.

---

## (d) Colour plan (default fg/bg and the 16 ANSI names only)

| Region | Colour | Attribute |
| --- | --- | --- |
| Page background | default bg | — |
| Body text, tile captions, description | default fg | — |
| Status bar (line 1) | default fg on default bg | dim, except the library name in **bright cyan** bold |
| Query prompt `Search ❯` | bright blue | bold |
| Query text the user typed | default fg | bold |
| Query cursor `▏` | bright blue | reverse (blinks with the terminal cursor) |
| Match count `37 of 2056` | bright black | — |
| Matched letters inside a tile caption | bright yellow | bold |
| Chip, not in force | default fg | dim |
| Chip, in force | bright white on blue | reverse, bold |
| Chip under the mouse | default fg | underline |
| Rules (lines 4, 32) and their text | bright black | — |
| Tile frame, not selected | bright black | — |
| Tile frame, selected | bright yellow | bold |
| Selected tile caption | bright yellow | bold |
| Series tile frame and shadow edge | magenta | — |
| Cover shading blocks (no image support) | white | dim |
| Progress bar inside a tile, played part | green | — |
| Progress bar inside a tile, rest | bright black | — |
| `✓` finished mark | bright green | bold |
| `new` mark | bright black | — |
| Scrollbar thumb `█` | bright black | — |
| Scrollbar trough `│` | black | — |
| Detail strip title | bright white | bold |
| Detail strip facts line | cyan | — |
| Detail strip action buttons | default fg | dim, the key letter in bright yellow |
| Mouse legend (line 37) | bright black | italic if the terminal has it, else dim |
| Message box frame | bright blue | — |
| Message box text, normal | default fg | — |
| Message box text, error | bright red | bold |
| Download bar, done part | cyan | — |
| Download bar, rest | bright black | — |
| Player state `▸` / `⏸` | bright green | bold |
| Player title and chapter | default fg | bold |
| Player speed / volume / timer chips | yellow | — |
| Seek bar, played `━` | bright magenta | — |
| Seek bar, head `┃` | bright white | bold |
| Seek bar, rest `─` | bright black | — |
| Seek bar chapter marks `┬` | bright black | bold |
| Player key row (line 44) | bright black | — |
| Footer (line 45) | bright black | key letters in bright yellow |

Rules the palette follows: bright yellow means "this is the thing you are on", green means "you have
heard this", magenta means "there is more than one thing here" (series, and the played part of the
seek bar), bright black is furniture. No two states use the same colour with the same shape.

---

## (e) Mouse map

| Region | Click | Double click | Wheel | Drag |
| --- | --- | --- | --- | --- |
| Cover tile | Select it, the detail strip repaints | Play the book, or open the series into the grid | Scroll the grid by one tile row | Drag onto the queue button to queue it |
| Series tile | Select it | Open the series into the grid | as above | as above |
| Progress bar inside a tile | Select the tile | Play from that point | Scroll the grid | Seek in that book |
| Query line | Put the cursor in the text at that character | Select a word | — | Select a range of text |
| `[ x clear ]` | Empty the query | — | — | — |
| Sort chip | Use that sort order | Turn the order around | Step to the next sort order | — |
| Filter chip | Use that filter | — | Step to the next filter | — |
| `[ ▦ Grid ] [ ☰ List ]` | Change the view mode | Change the tile size | Step the tile size | — |
| Library name on line 1 | Open the library menu | — | Go to the next library | — |
| View buttons on line 1 | Go to that view | — | — | — |
| Grid rule (line 4) | Go back out of a series | — | Scroll the grid | — |
| Scrollbar column | Jump to that part of the shelf | — | Scroll the grid | Drag the thumb |
| Detail strip action buttons | Do that action | — | — | — |
| Message box | Close the message | — | — | — |
| Download bar | Open the downloads view | — | — | — |
| Player title | Open the chapters view | Show the item in the grid | — | — |
| `[ 1.25x ]` `🔊 [ 78 ]` `💤 [ 25m ]` | Open that control | Set it back to the default | Change the value up or down | — |
| Seek bar | Seek to that point | — | Move ±10 s | Scrub, and seek when the button goes up |
| Player key row / footer | Do that key | — | — | — |

Every mouse action has a key that does the same thing, so the mouse is never the only way in. A user
can turn the mouse off with a setting, and the `mouse: on` mark at the right of line 45 says which
state it is in, so a text selection with the terminal's own mouse handling is one click away.

---

## (f) What this design gives up

* **Density.** Today's list shows about 30 titles in the same space; this grid shows 16. A user who
  reads titles faster than covers must press `[ ☰ List ]`. The grid is the default because the
  common job is "find the book I want to hear now", not "read 500 titles".
* **The long description.** Today the description can take four or five lines and `J`/`K` scroll it.
  Here it is one line, clipped. The full text needs a key (`K` opens the detail strip to half the
  screen, over the grid).
* **Home view sections.** Today "Recently Added" and "Discover" are headed groups in one list. A
  grid has no room for section headers between tile rows without breaking the pitch, so Home becomes
  a set of shelves the user pages through with the `[ Home ]` button, one section at a time, with
  the section name in the rule on line 4.
* **Series in place.** Today a series row can, in principle, expand where it sits. Here opening a
  series replaces the grid, so the user loses their place in the shelf. The fix is that `h` returns
  to the exact scroll offset and the same selected tile.
* **Very small terminals.** Below 56 columns the design is simply not itself; it hands the screen
  back to the list. That is a deliberate surrender, not a fallback that half works.
* **Cost.** Sixteen sixel or kitty images per screen, redrawn on every keystroke of the query, is far
  more terminal traffic than 30 lines of text. The grid needs an image cache keyed by item id and
  tile size, and it must not redraw a tile whose rectangle and item did not change.
