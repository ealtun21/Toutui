# Mockup 4 — the sortable data grid with a command line

File: `mockup-4.txt` (45 lines, every line exactly 160 display columns).
State shown: the Library table, sorted by ADDED (newest first), two filters on
(`unfinished`, `format: book`), the series **The Test Chronicles** opened into its
three books, and a media playing.

## (a) Lineage and what was taken

| Source | What was taken |
| --- | --- |
| **k9s** | The context block at the top left (`Context / View / Playing` key-value pairs), the key hints at the top right, the box title that counts what is shown (`26 rows of 500 shown · 2 filters on`), and the `:` command line at the bottom that accepts `:sort added` style words. |
| **btop** | The dense fixed grid with no wasted rows, and the small inline bars (`▓▓░░ 62%`) that put a number and a picture in the same 9 columns. |
| **lazydocker** | Two panels side by side, where the narrow right panel always shows the detail of the row that the left panel selects. |
| **ncmpcpp / cmus** | The audiobook identity: a media row at the bottom with a seek bar, and a row of player keys that you can hide. |

The columns are audiobook columns only: title, author, narrator, series, duration,
progress, added, flags. There is no Kubernetes word in the design.

## (b) Walk-through of every region

| Line(s) | Region | What it holds |
| --- | --- | --- |
| 1-3 | **Context block** (left) | `Context` = the user and the server. `View` = the view and the two view modes (`t` table, `g` gallery). `Playing` = the media that plays now. `Library`, `Sort` and `Time` are the second key column. |
| 1-3 | **Key hints** (right) | The name and version, then the six keys that a new user needs. |
| 4 | **Filter and sort bar** | `FILTERS²` and the chips that are on, an `[ + add a filter ]` chip, then `SORT¹` with the column and the direction, and one line of help. |
| 5 | **Table frame title** | The library name, the count of the rows shown of the total, and the count of the filters. `³` marks the rows as a mouse target, `⁵` marks the cover panel. |
| 6 | **Column names** | `TITLE AUTHOR NARRATOR SERIES DUR PROGRESS ADDED ▼ FLG`. The `▼` shows the sort column and the direction. |
| 7 | **Rule** | Holds the names apart from the rows. |
| 8-33 | **Rows** | One row for one media. `▾` = a series that is open, `▸` = a series that is shut, `└─ n` = a book inside an open series. `➤` marks the row that is selected. |
| 8-33 col 115 | **Scroll bar** | `█` thumb, `╎` track. |
| 5-34 right | **Cover panel** | The cover of the selected row as block art, then the title, the author, the narrator, the length, the progress and the time that is left. Below it `NEXT COVERS` shows six small covers — a sample of the gallery mode that `g` opens on the full width. |
| 35 | **Detail line** | The full metadata of the selected row. |
| 36 | **Description** | One line of the description, with the keys that scroll it. |
| 37 | **Player row** | State, title, chapter, the time now, the seek bar with the head `●`, the total time, the speed, the volume and the sleep timer. |
| 38 | **Player keys** | The row that `B` shows or hides. |
| 39-41 | **Message box** | The box above the footer. It lives six seconds. |
| 42 | **Download bar** | The download that runs now. |
| 43 | **Command line** | The `:` prompt with the text that the user types, and examples on the right. |
| 44 | **Mouse map** | The five mouse targets, with the marks that appear in the mockup. |
| 45 | **Footer** | The key list. |

## (c) How it answers the four problems

**1. Mouse support everywhere.** Five targets carry a mark in the mockup and a word
on line 44:
`¹` a column name — click sorts by it, click again reverses it, and the `▼`/`▲` moves
to that name; `²` a chip — click removes that filter, click `[ + add a filter ]` opens
the filter list; `³` a row — click selects, double click plays or opens, and a click on
the `▾`/`▸` opens or shuts the series; `⁴` the seek bar — press and drag moves the
position; `⁵` a cover — click selects that media, and a click on `g gallery` in the
panel title changes the mode. The wheel scrolls the table, and the wheel on the cover
panel moves through the gallery. The scroll bar takes a drag too.

**2. The terminal's own colours.** The whole design works with default fg on default
bg. Colour only adds rank: it never carries information alone. Every state also has a
glyph (`▓░` for progress, `↓ ★ ≡ ▸` for the flags, `▼` for the sort), so the design is
still correct in a terminal with one colour. See the table in (d).

**3. Filtering, sorting and the series.** The sort column shows `▼` in its own name, so
you always see what the order is. The chips on line 4 show every filter that is on and
each one has an `×`. The command line takes `:sort added`, `:sort title`,
`:filter unfinished`, `:series expand`, `:gallery` — the same actions that the mouse
does, for a user who prefers to type. `The Test Chronicles` is open: the parent row
keeps the totals (`3 books`, `24h57`, the mean progress) and the three books are below
it, indented, each with its own number in the series. `The Old Sea Cycle` and
`The Small Hours` stay shut, so the two states are visible together.

**4. Covers fill the space.** Today about 17 rows are empty. Here the table takes 117
columns and a cover rail takes the other 43, so no row is empty. The rail shows one
large cover for the selected row, its metadata, and six small covers of the rows near
it. `g` changes the mode: the table goes away and the covers fill all 160 columns as a
grid of large covers with the title under each one; `t` brings the table back. `:cover
off` gives all 160 columns to the table for a user who wants more columns. This is the
answer to the tension: the two modes are a toggle, not a compromise, and the rail is
the middle step between them.

## (d) Colour plan

Only default fg, default bg and the 16 ANSI names.

| Region | Colour | Attribute |
| --- | --- | --- |
| Page background | default bg | — |
| Body text, row text | default fg | — |
| Key names in the context block (`Context`, `View`, `Playing`) | cyan | bold |
| Values in the context block | default fg | — |
| Server name and user | green | — |
| Key hints at the top right | default fg | dim |
| Program name and version | yellow | bold |
| `FILTERS` and `SORT` labels | cyan | bold |
| Filter chip text | black on cyan | — |
| Filter chip `×` | black on cyan | bold |
| `[ + add a filter ]` chip | default fg | dim |
| Box borders and rules | blue | — |
| Box titles (`LIBRARY …`, `COVER …`) | blue | bold |
| Column names | default fg | bold + underline |
| The sorted column name and its `▼` | yellow | bold + underline |
| Row that is selected (`➤` and the whole row) | default bg on white | bold |
| Row of the media that plays | green | — |
| Series parent row (`▾`, `▸`) | magenta | bold |
| Books inside an open series (`└─`) | default fg | — |
| The `└─` guide marks | blue | dim |
| Title cell | default fg | — |
| Author and narrator cells | default fg | dim |
| Series cell | magenta | — |
| Duration cell | default fg | dim |
| Progress bar, part that is done | green | — |
| Progress bar, part that is left | default fg | dim |
| Progress that is 0% | default fg | dim |
| Progress that is 90% or more | bright green | — |
| Added cell | default fg | dim |
| Flag `↓` (the copy is on disk) | cyan | — |
| Flag `★` (a bookmark) | yellow | — |
| Flag `≡` (in the queue) | magenta | — |
| Flag `▸` (this media plays) | green | bold |
| Scroll bar thumb `█` | blue | — |
| Scroll bar track `╎` | default fg | dim |
| Cover art blocks | default fg | dim |
| Cover panel title and author | default fg | bold |
| Small cover frames in the rail | blue | dim |
| Detail line (line 35) | default fg | — |
| Description (line 36) | default fg | dim |
| Player state `▸` and the time now | green | bold |
| Seek bar, part that is done | green | — |
| Seek bar head `●` | bright green | bold |
| Seek bar, part that is left | default fg | dim |
| Speed, volume, sleep timer | cyan | — |
| Player key row | default fg | dim |
| Message box border | green | — |
| Message text, good news | green | — |
| Message text, an error | red | bold |
| Message text, a warning | yellow | — |
| Download bar, part that is done | bright blue | — |
| Download bar, part that is left | default fg | dim |
| Command line `:` prompt | yellow | bold |
| Command line text and cursor `▏` | default fg | bold |
| Command line examples | default fg | dim |
| Mouse map line | default fg | dim |
| Mouse marks `¹ ² ³ ⁴ ⁵` | magenta | — |
| Footer keys (the letters) | cyan | — |
| Footer words | default fg | dim |

Rule for the theme: nothing uses a background except the chips and the selected row,
so a light terminal and a dark terminal both stay readable.

## (e) Mouse map

| Region | Click | Double click | Wheel | Drag |
| --- | --- | --- | --- | --- |
| Context block value (line 1, library name) | Open the library list | — | Change the library | — |
| Key hints (lines 1-3) | Do that action | — | — | — |
| Filter chip `×` | Remove that filter | — | — | — |
| Filter chip body | Change that filter | — | — | — |
| `[ + add a filter ]` | Open the filter list | — | — | — |
| `SORT ADDED ▼` on line 4 | Open the sort list | — | Step through the sort modes | — |
| A column name (line 6) | Sort by it; if it already sorts, reverse it | Sort by it and go to the top | — | Move the column edge to make it wide or narrow |
| A row (lines 8-33) | Select it | Play it, or open the series | Scroll the table | Select many rows |
| The `▾` or `▸` mark in a row | Open or shut that series | — | — | — |
| The progress cell of a row | Select the row | Open the chapter list | — | — |
| Right click a row | Open the menu (queue, collection, bookmark, download, mark finished) | — | — | — |
| Scroll bar (column 115) | Jump to that place | — | Scroll | Move the list |
| `g gallery` in the panel title | Change to the gallery mode | — | — | — |
| Large cover | Play the selected media | Open the chapter list | Move to the next or the previous cover | — |
| A small cover in the rail | Select that media | Play it | Move through the covers | — |
| Description (line 36) | Give it the focus | — | Scroll the description | — |
| Seek bar (line 37) | Seek to that place | — | Move ±10 s | Seek while you hold the button |
| `1.0x`, `vol 80%`, `sleep off` | Open that control | — | Change the value | — |
| A player key word (line 38) | Do that action | — | — | — |
| Message box | Close the message | — | — | — |
| Download bar | Open the downloads view | — | — | — |
| Command line | Give it the focus | — | Step through the history | Select the text |
| Footer key word | Do that action | — | — | — |

## (f) What this design gives up

- **The Home view is no longer the first thing you see.** The rows of Recently Added
  and Discover become filters and sort modes of one table (`:sort added`), so the
  groups with names are gone. `Tab` still opens Home.
- **The description gets one line, not four.** Today the description takes the space
  below the list. Here it is one line and you must scroll it with `J/K` or open the
  media to read more.
- **The cover is smaller than a real image panel.** Today the cover panel is 62
  columns wide and can hold a true sixel image. Here the rail is 43 columns, so a real
  image is smaller. The gallery mode gets the width back, but only in that mode.
- **The table needs a wide terminal.** Below about 120 columns the narrator, the series
  and the flags columns must drop out, and below 90 columns the design falls back to
  the list of today. It is not a design for a small window.
- **More state to hold.** The sort mode, the filters, the open series and the view mode
  all must be saved and shown. That is more code and more that can go wrong than a flat
  list.
- **Two ways to do one thing.** Every action has a key, a mouse target and a command
  word. That is good for a power user and more to document for a new one.
