# mockup-7 — The Chapters view of the two bars and of the table

**The maintainer asked for a Chapters view of two bars on 2026-08-16.** The
view of the program before this mockup holds the number and the title of each
chapter and no time at all, and no key of it plays a chapter.

`mockup-7.txt` is a real grid of characters of 160 columns and 45 rows.

## The decisions of the maintainer of 2026-08-16

1. **Two bars stand over the table.** The bar of the book holds a mark `│` at
   each boundary of a chapter, therefore the user sees the whole book and the
   place of every chapter in it. The bar of the chapter of the cursor stands
   under it. This is the design of `mockup-3`, which the maintainer named on
   2026-08-16 as the shape of the player already.
2. **Each row of the table says the start of the chapter and its length**,
   beside the number and the title.
3. **A press of `Enter`, or a click on a row, plays that chapter.**
4. **The mark `▸` is the cursor and the mark `▶` is the chapter that plays
   now**: the two are not the same, because a user reads the list of a book
   that the program does not play.

## The two bars

```text
 Book  ████│████│░░░░│░░░░░│░░░░░│░░░░│░░░░░│░░░░│░░░░░│░░░  17%
 Ch 12 ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  14%
```

- **The bar of the book takes the whole length of the media**, and the marks
  stand at the boundaries of the chapters in that length. A book of 70 chapters
  in 52 columns holds more boundaries than columns: **two boundaries of the same
  column take one mark**, and the bar says nothing false for that reason.
- **The bar of the chapter is the place inside the chapter of the cursor**, and
  not inside the chapter that plays. A user that moves the cursor sees the shape
  of each chapter.
- **A click on the bar of the book moves the media to that place**, which is the
  rule of the bar of the seek of the panel 7 (T-322).

## The table

| The column | What it holds |
|---|---|
| `#` | The number of the chapter, from 1 |
| `Title` | The title of the chapter, cut with three points |
| `Start` | The moment of the start, in the whole book (`1:51:22`) |
| `Length` | The length of the chapter (`7m50s`) |

**A book of no chapters at all** keeps the two bars, with no mark in the bar of
the book, and the table then says `This book has no chapters.`

## The map of the mouse

| The action | What it does |
|---|---|
| A click on a row | Plays that chapter |
| A click on the bar of the book | Moves the media to that place |
| The wheel over the table | Moves the list |

## What this design gives up

- **The width of the title**: the two new columns take 17 columns of the row,
  therefore a long title of a chapter takes three points sooner than before.
- **The bar of the book is not exact under 40 columns**: a screen that narrow
  gives fewer columns than the book has chapters by a large number, and the
  marks then stand beside each other with no space. The view drops the marks
  under 40 columns and it keeps the bar.
