# mockup-6 — The Home view of the bands of covers

**The maintainer asked for a new Home view on 2026-08-16**, and this file holds
it. The Home view of the program before this mockup is one table of 55 rows,
with a row of a title over each shelf (`Continue Listening`, `Continue Series`,
`Recently Added`, `Recent Series`, `Discover`, `Listen Again`). That table says
the same four columns for a shelf of 4 media that the user plays now and for a
shelf of 500 media that the server found: **the shelf of the design is a band
of covers, and the table is not**.

`mockup-6.txt` is a real grid of characters of 160 columns and 45 rows.

## The decisions of the maintainer of 2026-08-16

1. **Every shelf is a band of covers**, with the title of the shelf over it.
   The view scrolls up and down through the bands, and left and right inside
   one band.
2. **The shelves come of the server, in the sequence that the server gives**
   (`GET /api/libraries/<id>/personalized`), and the program holds no list of
   its own and no setting of its own.
3. **The keys `j` and `k` move to the shelf above and under**, and the keys `h`
   and `l` move along the covers of one shelf. **`Enter` plays or opens** the
   media of the cursor. This is a new meaning of `h` and `l` in this view, and
   the footer of the view says it.
4. **A cell holds the picture and its border and nothing else**: no percentage,
   no title, and no bar. **The panel 5 says the facts** of the media of the
   cursor, which it does for the list of today already.
5. **The old table goes away.** One Home view, and no setting that turns
   between two shapes.

## The regions

| The region | What it holds |
|---|---|
| The title of a band | The name of the shelf, a line to the right border, and the count: `6 of 24` |
| The arrow `›` | The band holds more media at the right. A `‹` stands at the left when the band moved |
| A cell | The picture of the cover, in a border of one line. The cell of the cursor takes a border of a heavy line |
| The panel 5 | The facts of the media of the cursor, under the large picture of it |
| The panel 6 | The gallery, which the key `6` gives the focus. It is the grid of the whole shelf of the cursor |

## The shape of a narrow screen

**Under 100 columns the stack of the panels 1 to 3 goes away** (the rule of the
frame of T-320), and the band holds fewer cells. The count in the title of the
band says how many of them the user sees.

```text
┌4 Home ◉ ⇕──────────────────────────────────┐
│ Continue Listening ───────────── 3 of 24  ›│
│ ┏━━━━━━━━┓ ┌────────┐ ┌────────┐           │
│ ┃▓▒░▒▓█▒░┃ │░▒▓█▓▒░▒│ │▒▓█░▒▓█░│           │
│ ┃▒░▓█▒░▓█┃ │▓░▒█▓░▒█│ │░▓▒█░▓▒█│           │
│ ┗━━━━━━━━┛ └────────┘ └────────┘           │
│                                            │
│ Continue Series ──────────────── 3 of 11  ›│
```

**A screen of fewer than 12 rows for the view holds one band**, and the keys
`j` and `k` then move from shelf to shelf one at a time, with no other band on
the screen.

## The terminal that draws no pictures

`TOUTUI_NO_COVERS`, a terminal with no protocol of pictures, and a media whose
cover the server does not hold: **the cell keeps its border and its place**,
and it holds the title of the media, cut to the width of the cell. The band
does not become a table, because the keys must not change with the terminal.

```text
│ Continue Listening ───────────────────── 6 of 24  ›│
│ ┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐        │
│ ┃The     ┃ │Depthle…│ │Counter…│ │Reborn …│        │
│ ┃Kingkil…┃ │Hunger  │ │Soul    │ │as a De…│        │
│ ┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘        │
```

## The map of the mouse

| The action | What it does |
|---|---|
| A click on a cover | Takes that media, and it gives the focus to that band |
| Two clicks on a cover | Plays or opens that media |
| The wheel over a band | Moves that band to the left and to the right |
| A click on the title of a band | Gives the focus to that band |

## What this design gives up

- **The columns of the table**: the author, the length, and the percentage of a
  media do not stand beside its cover. The panel 5 says them for the media of
  the cursor alone, therefore a user who compares two media must move to each.
- **The long shelf reads slowly**: `Recently Added` of 55 media needs 9 presses
  of `l` to reach its end, where the table of today needed one press of `G`.
  The keys `g` and `G` take the two ends of a band for that reason.
- **The height**: a band takes 6 rows, therefore a screen of 34 rows holds 5
  bands and no more. The table of today held 32 media on the same screen.
