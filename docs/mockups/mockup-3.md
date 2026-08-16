# Mockup 3 — the player is the screen

File: `mockup-3.txt` (45 lines, each 160 columns or less).
View shown: the Library, with a book that plays and the queue open.

## (a) The lineage

Media-player-first. The book you listen to is always on the screen. Four programs
gave the ideas:

- **musikcube** — the transport is a permanent band at the foot of the screen, with
  its own frame and its own title. It never moves and never hides. I took the band,
  the frame around it, and the rule that the band keeps its height when a list grows.
- **ncspot** — a narrow sidebar of places on the left that does not move, and a queue
  that is a real list you can see and move, not a hidden stack. I took the sidebar and
  the visible, drag-to-move queue.
- **cmus** — one letter for one action and a two-line key strip at the foot. I took the
  key strip and the small marks in the first column of a row (`▶`, `✓`, a per-cent).
- **spotify-tui** — the cover beside the transport, and a small progress bar in each
  row of the list. I took both, and made the row bar 8 cells wide so 500 rows still read.

What is new here: an audiobook needs **two** bars, not one. A chapter bar and a book
bar, one above the other, plus a row of chapter marks under the book bar.

## (b) A walk through every region

| Line(s) | Region | What it holds |
|---|---|---|
| 1 | Top bar | The program, the user, the server, the library, the filter now on, the sort now on, the mouse state, the count of items. |
| 2 | Body frame | Three titles: `GO TO`, `Library …`, `Covers …`. The `┬` marks show where the columns start. |
| 3–28, col 1–22 | Sidebar (`GO TO`) | 11 places (Home … Settings) with a count badge for the queue and the downloads. Then `SHOW` (List / Covers / Both) and `FILTER` (All, Not started, In progress, Finished, On disk, Has a bookmark). A `●` marks what is on now. |
| 3, col 24–98 | Sort bar | Six sort modes as tabs. The one in `[ ]` is on; `▼` shows the direction. |
| 5, col 24–98 | Column heads | ST, TITLE, AUTHOR, LENGTH, PROGRESS. Click a head to sort by it. |
| 6–24 | The list | One book to a row. Column 1 is the state: `▶` plays now, `✓` finished, a per-cent, or empty for not started. A series is a row with `▸` (closed) or `▾` (open); an open series shows its books with `├`/`└` and a number. Small marks in the title (`📥` on disk, `🔖` has a bookmark). |
| 25–28 | The detail strip | The facts of the selected book, two lines of the description, and the keys for this view. |
| 3–28, col 100–160 | Cover gallery | A grid, 3 covers across and 5 down = 15 covers. Each cover is a box with art, the title inside the box, and the progress in the bottom edge. The book that plays has a thick frame. |
| 29 | Player rule | Closes the body and names the three parts below: NOW PLAYING, the book and the chapter, QUEUE. |
| 30–40, col 1–22 | Big cover | The cover of the book that plays, 18 columns wide, with the title and the reader under it. |
| 30–40, col 24–113 | Transport | Line 1: the title, the speed, the volume, the sleep timer. Line 2: the people and the year. Line 4: the chapter, with the keys to change it. Line 5: the **CH** bar, the position in the chapter, and the time left in the chapter. Line 7: the **BOOK** bar, the position in the book, the per-cent. Line 8: the chapter marks (`╫`), each one under its place on the BOOK bar. Line 9: the time left at the speed now set, and the clock time the book ends. Line 11: the buttons. |
| 30–40, col 115–160 | Queue | 7 items, each with a drag handle `≡`, the chapter for the item that plays, and the time left. Below: how to move an item and how to add one. |
| 41 | Player frame foot | |
| 42 | Downloads bar | Shows only while a download runs. |
| 43 | Message strip | One line, six seconds, with a `✕` to close it now. |
| 44–45 | Key strip | Two lines: the media keys, then the view keys and the mouse actions. |

## (c) How it answers the four problems

**1. Mouse everywhere.** Every clickable thing in the mockup is a box, a tab, a name,
a row, or a bar — all of them are rectangles with a known place on the grid, which is
what `ratatui` gives you from a `Rect`. The map is in part (e). Three lines of the
mockup say it in the interface itself, so a new user finds it: `Click a name to go.`
(sidebar), `click a ╫ mark` (under the book bar), `Drag ≡ to move it.` (queue), and the
last footer line. The top bar shows `Mouse: on`, so a user who wants to select text
with the terminal can see the state and turn it off.

**2. Only the 16 ANSI colours.** See the table in part (d). No colour carries meaning
alone: the book that plays has `▶` **and** bright colour; the finished book has `✓`
**and** dim; the selected cover has a thick frame `┏━┓` **and** reverse video. So the
design still reads on a terminal with a strange theme, or with no colour at all.

**3. Filtering, sorting, and a series you can open.** The sort modes are a tab bar over
the list (line 3) and are also the column heads (line 5) — two ways in, one state. The
filters are a section of the sidebar with a `●` on the one now on, so the state is
always in view and never hidden in a menu. The top bar repeats both, which is what a
user reads first. A series row carries `▸` closed / `▾` open; `x` or a click on the
mark opens it, and the books come in under it with `├`/`└` and their own progress. The
series row keeps its own count and its own total length, so the closed row is still
useful.

**4. Covers fill the space.** Two answers at the same time, and this is why the `SHOW`
control has three states. `Both` (shown) gives a 61-column gallery of 15 covers beside
the list; `Covers` gives the whole 137 columns of the body to the grid (7 across, 5
down = 35 covers); `List` gives it all to the list. A cover cell is 19 x 4 — small, but
it still holds a readable title and a progress bar, so it never becomes a coloured
square with no meaning. The old 17 blank rows are now the gallery and the player band.

## (d) The colour plan

Only default fg, default bg, and the 16 ANSI names. Attributes: bold, dim, reverse,
underline.

| Region | Colour | Attribute |
|---|---|---|
| Top bar | default fg on blue | bold for the program name |
| Top bar: filter and sort now on | bright yellow on blue | bold |
| Frame lines, `┬ ┴ │ ─` | bright black | — |
| Frame titles (GO TO, Library, Covers, QUEUE) | cyan | bold |
| Sidebar: a place | default fg | — |
| Sidebar: the place now open, and its `●` | bright cyan | bold |
| Sidebar: a section head (SHOW, FILTER) | yellow | bold |
| Sidebar: a count badge `[7]` | bright black | — |
| Sidebar: the row under the mouse | default fg on bright black | — |
| Sort tabs: off | bright black | — |
| Sort tabs: on `[Added ▼]` | bright yellow | bold |
| Column heads | bright black | underline |
| List row: not started | default fg | — |
| List row: in progress, and its per-cent | bright white | — |
| List row: finished, and its `✓` | green | dim |
| List row: the book that plays, and its `▶` | bright green | bold |
| List row: the selected row | default fg on white | reverse |
| List: a series row and its `▸ ▾` | magenta | bold |
| List: the books inside a series, `├ └` | bright black | — |
| List: the row progress bar, the full part | bright blue | — |
| List: the row progress bar, the empty part | bright black | — |
| List: `📥` on disk | bright green | — |
| List: `🔖` has a bookmark | bright yellow | — |
| Detail strip: the facts | cyan | — |
| Detail strip: the description | default fg | dim |
| Cover art `░▒▓█▀▄` | bright black | — |
| Cover title in the box | default fg | — |
| Cover box frame | bright black | — |
| Cover of the book that plays | bright green | bold + reverse |
| Big cover frame and art | bright black | — |
| Transport: the title of the book | bright white | bold |
| Transport: the people, the year | default fg | dim |
| Transport: `▶` plays / `‖` paused | bright green / yellow | bold |
| Transport: the name of the chapter | bright cyan | — |
| CH bar, the full part | bright cyan | — |
| BOOK bar, the full part | bright blue | — |
| Both bars, the empty part | bright black | — |
| Both bars, the handle under the mouse | bright white | reverse |
| Chapter marks `╫` | bright black | — |
| The mark of the chapter now on | bright yellow | bold |
| Times, per-cents | default fg | — |
| The time left `−13:28`, `−4h 55m` | yellow | — |
| Buttons `[ ]` | default fg | — |
| Button under the mouse | default fg on bright black | bold |
| Speed, volume, sleep, when not at the default | bright magenta | — |
| Queue: item 1 | bright white | bold |
| Queue: items 2 and after | default fg | — |
| Queue: the handle `≡` | bright black | — |
| Queue: the item under the mouse while you drag | default fg on white | reverse |
| Downloads bar | bright blue on default bg | — |
| Message strip: news | bright cyan | — |
| Message strip: a fault | bright red | bold |
| Key strip: the key | bright yellow | bold |
| Key strip: the words | default fg | dim |

## (e) The mouse map

| Region | Click | Two clicks | Wheel | Drag |
|---|---|---|---|---|
| Sidebar place | Go to that view | — | Move in the sidebar | — |
| Sidebar `SHOW` item | Set List / Covers / Both | — | — | — |
| Sidebar `FILTER` item | Set that filter | — | — | — |
| Sidebar edge (col 22) | — | — | — | Drag to change the width of the sidebar |
| Sort tab | Sort by it; click it again to turn the direction | — | Step through the modes | — |
| Column head | Same as the tab | — | — | — |
| List row | Select the row | Play the book, or open the series | Move the list 3 rows | Select many rows |
| Series mark `▸ ▾` | Open or close the series | — | — | — |
| Row progress bar | Select the row | — | — | — |
| Detail strip | Select the text | — | Move in the description | — |
| Cover in the gallery | Select the book | Play the book | Move the grid one line of covers | — |
| Player: big cover | Show the cover large | — | — | — |
| CH bar | Go to that point in the chapter | — | ±10 s | Move in the chapter; the time follows the mouse and the seek is sent when you let go |
| BOOK bar | Go to that point in the book | — | ±1 min | Move in the book, the same way |
| Chapter mark `╫` | Start that chapter | — | Next / last chapter | — |
| `[|◀] [▶|]` | Last / next chapter | — | — | — |
| `[◀10s] [10s▶]` | Back 10 s / on 10 s | — | — | — |
| `[ ▶ ‖ ]` | Play or pause | — | — | — |
| `[■]` | Stop the media | — | — | — |
| `[1.25x − +]` | The `−`/`+` change the speed | Set 1.00x | ±0.05x | — |
| `[Vol − +]` | The `−`/`+` change the volume | Mute | ±5% | — |
| `[Sleep 25m]` | Open the timer | Turn the timer off | ±5 min | — |
| `[Bookmark]` | Set a bookmark here | — | — | — |
| Queue item | Select it | Play it now | Move the queue | Drag `≡` to move the item; drag it out to remove it |
| Queue `Clear it` | Empty the queue | — | — | — |
| Downloads bar | Open the downloads view | — | — | — |
| Message `✕` | Close the message now | — | — | — |
| Key strip | Open the full key list | — | — | — |

Rules: the mouse never plays a book with one click — one click only selects, so a slip
does not stop the book you hear. A drag on a bar does not send a seek until you let the
button go, so the server gets one request, not fifty.

## (f) What this design gives up

- **The list is shorter.** Today the list can use about 40 rows. Here it uses 19. The
  player band (11 rows), the gallery, and the sort bar take that space. A user who
  wants the long list must press `V` for `List` and hide the gallery, or fold the
  player band.
- **The description is cut to two lines.** Today the description can fill the screen
  and `J/K` moves in it. Here you must open the book to read it all.
- **The cover panel is smaller.** Today the cover panel is 62 columns of one large
  image. A sixel image now gets 19 x 4 cells in the gallery, or 18 x 7 in the player.
  For a large image the user clicks the cover in the player.
- **Three columns need width.** Under about 100 columns the gallery must go, and under
  about 70 the sidebar must go too. The design needs a set of fall-back layouts that
  the present flat list does not need.
- **More state to hold.** A filter, a sort mode, a sort direction, the set of open
  series, and the show mode all must be kept and saved. Today there is nearly none of
  this.
