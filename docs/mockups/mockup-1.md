# Mockup 1 — Toutui, the panel design

The file `mockup-1.txt` shows the Library view while a media plays. It is 45 lines and
160 columns. It is the busiest screen: a list, a filter, a sort, a big cover, a cover
gallery, a player, a message and a download, all at the same time.

---

## (a) The lineage and what I took

The lineage is the panel design of `lazygit` and `gitui`. Four programs gave ideas:

| Program | What I took |
|---|---|
| **lazygit** | The numbered panels. Each panel title starts with a digit, and that digit is also the key that moves the focus to it. The focused panel has a heavy border, the other panels have a light border. One key hint bar at the bottom that changes with the focused panel. |
| **gitui** | The two-line key hint bar: one line for the focused panel, one line for the keys that work everywhere. Also the small `[ ]` buttons in a panel title. |
| **ncmpcpp** | The player row at the bottom with the two times at the two ends of a long seek bar, and the chapter progress below the book progress. This keeps the identity of an audio player. |
| **ranger** | The idea that the space on the right is not empty space but a preview of the item under the cursor. Here the preview is the cover and the book facts, not a file. |

---

## (b) A walk of every region

| Line(s) | Region | What it shows |
|---|---|---|
| 1 | **Status bar** | The user, the server, the open library, how many rows the filter lets through (`500 of 2056 shown`), the sort mode, the filter mode, and the version. No border, so it reads as a header. |
| 2-16 | **Panel 1 — Views** | The list of all the views with the key of each one. This replaces the hidden `?` list for the most common jumps. |
| 17-24 | **Panel 2 — Sort** | The six sort modes. The `▸` marks the mode in use. The right column shows the direction (`new→old`) and the number key. `[↑↓ turn the order]` in the title turns the direction. |
| 25-33 | **Panel 3 — Filter** | Check boxes for the state filters, a genre picker, and the text box. The text box holds the live search text, so `/` is now a filter and not a separate mode. |
| 2-33 (middle) | **Panel 4 — Library** | The focused panel, so the border is heavy. A header row, then the rows: state mark, title, author, time, percent done. A series row starts with `▸` when it is closed and `▾` when it is open; the books of the open series are indented below it. `➤` is the cursor, `▶` marks the row that plays now, `✓` marks a finished book, `↓` marks a book on this disk. The last row is a small key to the marks. A scrollbar sits in the last column inside the border. |
| 2-21 (right) | **Panel 5 — Cover** | One large cover, 24 by 12 cells, of the item under the cursor. On the right of it: the series, the author, the narrator, the time and the time left, the start date, the genre, the file size, the ebook state, and a progress bar. Below: the description, which `J`/`K`/`H` move. |
| 22-33 (right) | **Panel 6 — Gallery** | A grid of ten small covers, five by two, of the rows around the cursor. Each cell shows the cover art, the percent done, and a short title. `[+ bigger]` and `[- smaller]` change the cell size, so the grid can hold four large covers or thirty small ones. |
| 34-39 | **Panel 7 — Player** | Row 1: the title, the author, the chapter, the speed, the volume, the sleep timer. Row 2: the seek bar with the two times. Row 3: the book progress and the chapter progress side by side. Row 4: the buttons for every player key. |
| 40-42 | **Message box and Downloads** | The message box holds the six second message and a `[x close]` button. The downloads panel shows each active download with its bar, its percent, its speed and a `[x stop]` button. |
| 43 | **Mouse legend** | What each mouse mark means. |
| 44 | **Panel keys** | The keys of the focused panel. This line changes when the focus changes. |
| 45 | **Global keys** | The keys that work in every view. |

---

## (c) How it answers the four problems

### 1. Mouse support everywhere

The mockup marks every mouse target with a glyph, and line 43 gives the key to the marks:

- **`◉`** in a panel title: click the title to move the focus to that panel.
- **`⇕`** in a panel title: the wheel moves this panel when the pointer is over it.
- **`↔`** in a panel title: a bar in this panel takes a drag.
- **`[ ]`** around a word: click it to do the thing it says. Every player key, every
  filter check box, the genre picker, the gallery size, the message close, and the
  download stop are `[ ]` buttons.
- A click on a list row moves the cursor to it. Two clicks play or open it.
- A click on the `▸` or `▾` of a series row opens or closes the series.
- A drag on the seek bar in the player moves the position. A drag on the scrollbar of
  panel 4 moves the list.

Every mouse action has a key that does the same thing, so the keyboard user loses nothing.

### 2. A theme from the terminal

The design uses only the default foreground, the default background and the 16 ANSI
colours. See the table in (d). The rules that keep it safe:

- The background is always the default background. No panel paints a block of colour.
- The focus is shown by the **shape** of the border (heavy `═║` against light `─│`),
  not only by a colour. A terminal with a low contrast theme still shows the focus.
- Only **one** accent colour, cyan, is used, and only for the focused panel and the
  cursor row.
- Text that must be readable is the default foreground. Only marks and numbers take a
  colour.
- Dim is used for the unfocused borders instead of a grey, because grey is a fixed
  colour but dim follows the theme.

### 3. Filtering, sorting and open series

Panel 2 and panel 3 make the modes visible at all times, so the user does not have to
remember which filter is on. The status bar repeats the two modes as words.

- Sort: added first, title, author, duration, progress, and series with book number.
  The direction turns with `↑↓` or with a click on the title button.
- Filter: not finished, finished, started, on this disk, has an ebook, a genre, and a
  free text box. More than one check box can be on at the same time.
- A series row is `▸ The Sand Sea [5 books]` when it is closed. `Enter`, `l`, `→` or a
  click on the `▸` makes it `▾` and puts the books below it, indented and numbered, as
  line 8 to line 11 show. The series row keeps the total time and the mean progress.

### 4. Covers fill the space

The old design lost about 17 rows. This design gives the whole right column, 60 columns
by 32 rows, to covers, and it always holds two things:

- **Panel 5** is one large cover, 24 by 12 cells, of the item under the cursor, with the
  facts of the book beside it. This is the "one large cover" case.
- **Panel 6** is the grid gallery. It shows the covers around the cursor, and it fills
  whatever height is left. `[+ bigger]` and `[- smaller]` change the cell size, so the
  grid always fits a whole number of cells and no cover becomes too small to read. The
  minimum cell keeps room for the title and the percent.

If the terminal draws real images (sixel or kitty), the block art in both panels is
replaced by the real cover in the same cells. If it does not, the block art stays and
the screen is still full.

---

## (d) The colour plan

Only the default foreground, the default background and the 16 ANSI names.

| Region | Colour | Attribute |
|---|---|---|
| Screen background | default bg | — |
| Body text, list rows, description | default fg | — |
| Focused panel border and title (panel 4 here) | cyan | bold |
| Unfocused panel border and title | default fg | dim |
| The panel number in a title (`1`, `2`, ...) | yellow | bold |
| The `◉ ⇕ ↔` mouse marks | magenta | — |
| `[ ]` buttons: the brackets | blue | — |
| `[ ]` buttons: the word inside | default fg | — |
| Status bar text | default fg | dim |
| Status bar: the connected dot `●` | green | — |
| Status bar: the library name, the sort and filter words | cyan | — |
| Status bar: the version | default fg | dim |
| Cursor row `➤` and its text | black on cyan | bold |
| The playing row `▶` and its text | green | bold |
| Finished mark `✓` and a `100%` | green | — |
| A percent from 1 to 99 | yellow | — |
| A `-` for no progress | default fg | dim |
| On this disk mark `↓` | blue | — |
| Series open/close mark `▸ ▾` | magenta | — |
| The books inside an open series | default fg | dim |
| List header row | default fg | bold, dim |
| Scrollbar thumb `█` | cyan | — |
| Scrollbar track `│` and the arrows | default fg | dim |
| Sort/filter: the mode in use, the `[x]` box | green | bold |
| Sort/filter: a mode not in use, a `[ ]` box | default fg | dim |
| Filter text box content | default fg | — |
| Filter text box cursor `▏` | cyan | blink |
| Cover art blocks | default fg | — |
| Cover art border `▛▜▙▟▌▐` | blue | — |
| Cover fact names (Series, Author, ...) | default fg | dim |
| Cover fact values | default fg | — |
| Player: the play mark `▶` | green | bold |
| Player: the title | default fg | bold |
| Player: the author, the chapter | default fg | dim |
| Player: the played part of the seek bar `█` | cyan | — |
| Player: the seek head `▒` | white | bold |
| Player: the part not played `░` | default fg | dim |
| Player: the times | yellow | — |
| Player: the speed, the volume, the sleep timer | default fg | — |
| Message box border and text (news) | blue | — |
| Message box border and text (a fault) | red | bold |
| Downloads bar | green | — |
| Key hint bar: the key | yellow | — |
| Key hint bar: the words | default fg | dim |
| Mouse legend line | default fg | dim |

A colour is never the only sign of a state. `✓`, `↓`, `▶` and `➤` all say the same thing
as their colour, so the design still works with no colour at all.

---

## (e) The mouse map

| Region | Click | Two clicks | Wheel | Drag |
|---|---|---|---|---|
| Panel title (any) | Move the focus to the panel | — | — | — |
| Status bar: library name | Open the library picker | — | Next / last library | — |
| Status bar: `Sort:` word | Move the focus to panel 2 | — | Next / last sort mode | — |
| Status bar: `Filter:` word | Move the focus to panel 3 | — | — | — |
| Panel 1 row (a view) | Open that view | — | Move the panel | — |
| Panel 2 row (a sort mode) | Use that sort mode | Turn the direction | Move the panel | — |
| Panel 2 `[↑↓ turn the order]` | Turn the direction | — | — | — |
| Panel 3 check box | Turn the filter on or off | — | Move the panel | — |
| Panel 3 `[All]` genre | Open the genre list | — | Next / last genre | — |
| Panel 3 text box | Put the text cursor there | Clear the text | — | Pick the text |
| Panel 4 row | Move the cursor to the row | Play or open the row | Move the list | Pick many rows |
| Panel 4 `▸` or `▾` on a series | Open or close the series | — | — | — |
| Panel 4 header word (Title, Author, Time, Done) | Sort by that column | Turn the direction | — | — |
| Panel 4 scrollbar | Jump to that place | — | Move the list | Move the list |
| Panel 5 cover art | Play the media | — | Move to the last / next item | — |
| Panel 5 fact buttons `[l] [n] [h] [e] [D] [M] [b]` | Do the thing | — | — | — |
| Panel 5 progress bar | Move the position of this book | — | — | Move the position |
| Panel 5 description text | — | — | Move the text | — |
| Panel 6 cover cell | Move the cursor to that book | Play that book | Move the grid one row | — |
| Panel 6 `[+]` `[-]` | Make the cells bigger or smaller | — | — | — |
| Panel 7 seek bar | Jump to that time | — | Back / on 10 s | Move the position |
| Panel 7 buttons | Do the thing on the button | — | — | — |
| Panel 7 `Volume 70%` | — | — | Volume down / up | — |
| Panel 7 `Speed x1.25` | — | — | Slower / faster | — |
| Message box `[x close]` | Close the message now | — | — | — |
| Downloads row | Open the downloads view | — | Move the list | — |
| Downloads `[x stop]` | Stop that download | — | — | — |
| Key hint bar: a key word | Do that key | — | — | — |

A right click anywhere opens a small menu with the same items as the panel key line.

## (f) What this design gives up

- **List width.** The list is 66 columns, not the full width. A very long title is cut.
  Today the list can be almost 100 columns wide. The cover panel of today was also 62
  columns, so the true loss is small, but the sort and filter panels do take 34 columns
  that the list had.
- **Rows on the screen.** The list shows 29 rows. Today it shows about 34, because
  today has no border on the list and no header row.
- **A quiet screen.** Today, when nothing plays and there is no message, the screen is
  mostly empty and calm. This design is always full. A user who wants a small, quiet
  screen will find it busy. The answer is a key that hides panels 1, 2 and 3 and gives
  their columns to the list, but that is a second mode and not the default.
- **Small terminals.** Below about 120 columns the three columns do not fit and the
  design must drop the left column, then the right column. Today's design bends more
  easily.
- **The description.** Today the description has the full width under the list. Here it
  has four lines in a 56 column panel, so a long description needs more scrolling.
- **One more key layer.** The number keys 1 to 6 now belong to the sort modes and the
  panel focus. Any future use of a bare digit is taken.
