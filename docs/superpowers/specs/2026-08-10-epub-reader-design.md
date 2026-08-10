# Design: read an EPUB book in the application

Date: 2026-08-10
Status: Approved, not yet built
Item: T-10

## 1. Purpose

Audiobookshelf holds ebooks beside audiobooks. The application plays the audio
and it does not open the ebook. T-10 adds a reader: the user opens the ebook of
an item, reads it in the terminal, and the position goes to the server.

This document answers four questions before any code:

1. Which crate reads the EPUB, and why. Section 3.
2. How the text comes into the terminal. Section 5.
3. How the position is held and how it goes to the server. Section 6.
4. What happens when the file is hostile. Section 7.

Every number in this document comes from a measurement on 2026-08-10. The full
survey holds the commands. See section 10.

## 2. What the server gives

Measured against the sandbox of `docs/TEST-SERVER.md` on 2026-08-10, with an
EPUB of Project Gutenberg in the library.

| Request | Answer |
|---|---|
| `GET /api/items/:id` | `media.ebookFormat` is `"epub"`. `media.ebookFile.ino` and `media.ebookFile.metadata.size` name the file |
| `GET /api/items/:id/ebook` | `200`, `application/epub+zip`, 136761 bytes |
| The same with `Range: bytes=0-99` | `206 Partial Content`, `Accept-Ranges: bytes`, `Content-Range: bytes 0-99/136761` |
| `PATCH /api/me/progress/:id` with `{"ebookLocation":"toutui:3:120","ebookProgress":0.42}` | `200`. A later `GET` gives both values back, and `currentTime` and `progress` of the audio do not change |

**Two facts decide the design.**

1. The whole ebook comes from one address, and that address takes a `Range`.
2. The position of the ebook is a different field from the position of the
   audio. Therefore the reader never damages the position of the audiobook.

An item with no ebook has `ebookFormat` `null`. The user interface must offer
the reader for an item that has an ebook only.

## 3. The crates

```toml
# Reads EPUB 2 and EPUB 3. `default-features = false` removes the writer, and
# the reader does not need it. `threadsafe` keeps `Epub` as `Send + Sync`, thus
# a task can hold the book. Pure Rust: the build needs no C toolchain.
rbook = { version = "0.7.10", default-features = false, features = ["threadsafe"] }
# Turns XHTML into lines with a style. It uses html5ever, the parser of Servo,
# therefore a page with an error cannot stop the parse. The feature `xml` adds
# xml5ever for a strict XHTML document.
html2text = { version = "0.17.1", features = ["xml"] }
```

A measurement on 2026-08-10 examined `rbook`, `epub`, `epub-parser`, and
`iepub`, and `html2text`, `scraper`, `lol_html`, and `ammonia`.

**No new C.** `cargo tree -i cc` and `cargo tree -i openssl-sys` find nothing
for `rbook` and for `html2text`. Two crates are not acceptable and the survey
refuses them: `epub-parser` 0.3.4 brings `bzip2-sys`, `lzma-sys`, and
`zstd-sys`, and `iepub` 1.3.7 brings `zstd-sys`. `--no-default-features` does
not remove them.

**Why `rbook` and not `epub` 2.1.5.** One measurement decides it. A zip archive
of 2 megabytes that opens to 2 gigabytes stops the whole process with both
crates: the allocation fails, Rust calls `abort`, and `catch_unwind` cannot
help. `rbook` has `ManifestEntry::copy_bytes`, which writes into a writer of
the caller. With a writer that refuses more than 8 megabytes, the same file
gives an error and the program uses 5 megabytes of memory, and not 4102
megabytes. The crate `epub` has no such function: every read gives an owned
`Vec<u8>` or `String`, and nothing can stop it.

`rbook` also opens faster and with less memory: 0.4 milliseconds and 780
kilobytes for Moby Dick, against 1.3 milliseconds and 1908 kilobytes. It gives
a tree of the table of contents, and it gives a typed error with the name of
the resource. Its licence is Apache-2.0, and the licence of `epub` is GPL-3.0.

**What `epub` does better.** It has more downloads, and it holds the position
in the spine itself. The application holds one number, therefore that gain is
small.

**The risk of `rbook`.** Its MSRV is 1.88.0, and its API uses `impl Trait` in a
trait. Therefore the application holds the `Epub` and a number, and it never
holds a borrowed entry in a structure.

**The risk of `html2text`.** The time grows with the square of the depth of the
tags. 10000 nested `<div>` need 194 milliseconds in a release build and 1.85
seconds in a debug build. 100000 need more than 60 seconds. Section 7 gives the
answer.

## 4. The new view

`AppView::Reader`. The key `e` opens it from the view `Library`, from the view
`Home`, from a series, and from a list, when the item has an ebook. The key `h`
goes back to the view that opened it.

The screen holds three parts.

```
┌ Alice's Adventures in Wonderland — Chapter 3 of 14 ────────── 21% ┐
│                                                                   │
│  CHAPTER III.                                                     │
│  A Caucus-Race and a Long Tale                                    │
│                                                                   │
│  They were indeed a queer-looking party that assembled on the     │
│  bank—the birds with draggled feathers, the animals with their    │
│  fur clinging close to them, and all dripping wet, cross, and     │
│  uncomfortable.                                                   │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
 j/k: line, Space/b: page, n/p: chapter, t: contents, g/G: start/end
 s: sync the position, h: back, Q: quit
```

- The title line names the book, the chapter, and the part of the book that the
  user read.
- The text stands in the middle. The line length follows the width of the
  terminal, with a largest length of 100 columns, because a long line is hard
  to read. The text stands in the middle of a wide terminal.
- The key `t` opens the table of contents as a list. The key `l` on an entry
  goes to that chapter.

The player panel stays visible. A user can listen to one book and read a
different book, and the two positions do not meet.

## 5. From the file to the screen

### 5.1 The bytes

The application asks for `GET /api/items/:id/ebook`, with the limit of
section 7. It writes the file in the directory of the downloads, beside the
audio of the same item, with the name `<item id>.epub`. A second visit reads
the file from the disk, therefore the reader also works with no server. This
follows the offline mode of T-25 and T-1.

`rbook::Epub::open` takes a path. Therefore the reader needs no `Cursor` and no
temporary file.

### 5.2 The chapters

The spine gives the sequence of the chapters. The reader holds the position in
the spine as one number, and it never holds a borrowed entry.

A chapter comes into the terminal in three steps.

1. `ManifestEntry::copy_bytes` writes the XHTML into a capped writer.
2. `html2text::config::rich().lines_from_read(bytes, width)` gives lines with a
   `RichAnnotation` for each part.
3. The reader changes each annotation into a `ratatui::style::Style`: `Emphasis`
   gives italic, `Strong` gives bold, `Link` gives underline, `Preformat` and
   `Code` give a different colour. The result is a `Vec<Line<'static>>`, and a
   `Paragraph` draws it with a scroll.

The first item of the spine is often a wrapper of the cover with no text. Moby
Dick gives 553 bytes and no line. The reader skips a chapter that gives no
line, and it does not show an empty page.

### 5.3 Where the work runs

**The render never runs on the thread that draws.** A chapter of Moby Dick
needs 3 milliseconds in a release build and 18 milliseconds in a debug build,
and the hostile file of section 7 needs seconds. Therefore a task reads and
renders the chapter, and it sends the lines through a channel. The draw takes
the lines that are ready, and it shows "Reading…" for a chapter that is not
ready. This is the same shape as the cover art of T-23.

The reader holds the lines of the chapter that it shows, of the chapter before,
and of the chapter after. Therefore the key `n` shows the next chapter at once.
A cache of three chapters of a novel costs less than one megabyte.

### 5.4 The width

A change of the width of the terminal needs a new render, because
`html2text` breaks the lines. The reader keeps the width of the last render. A
different width starts the task again, and the position stays: the reader holds
the position as a part of the chapter, and not as a number of lines.

## 6. The position

### 6.1 What the application holds

The reader holds two values: the number of the chapter in the spine, and the
first line that the screen shows. A `PATCH` to the server holds:

| Field | Value |
|---|---|
| `ebookLocation` | `toutui:<spine>:<line>`, for example `toutui:3:120` |
| `ebookProgress` | The part of the book that the user read, from 0 to 1 |

The server keeps `ebookLocation` as a text and it changes nothing in it. A
measurement on 2026-08-10 wrote `toutui:3:120` and read it back.

`ebookProgress` is a number that every client understands. The reader
calculates it from the size of the chapters: the sum of the bytes of the
chapters before, plus the part of this chapter, divided by the sum of all
chapters. That value is near the value of the reader of the web, and it is not
the same value. The bar of the web page and the bar of the application then
agree well enough.

The web reader of Audiobookshelf writes an EPUBCFI in `ebookLocation`. The
application does not understand an EPUBCFI, and it must not fail on one. The
rule is:

- A value that starts with `toutui:` gives the chapter and the line.
- Every other value gives no chapter. The application then uses
  `ebookProgress` and the sizes of the chapters, and it opens the chapter that
  holds that part of the book. The user loses the line, and not the chapter.

### 6.2 When the application sends

- After 30 seconds of reading, and not more often than that.
- When the user moves to a different chapter.
- When the user leaves the reader with `h` or stops the program with `Q`.
- When the user presses `s`.

A server that does not answer must not lose the position. The reader writes the
position in the table `pending_progress`, and the task of T-25 sends it when
the server answers again.

**The reader opens no listening session.** A session belongs to the audio. The
key `F` of T-32 belongs to the audio, and the key `s` of the reader belongs to
the ebook.

## 7. A file that is hostile

A user can hold a book from any source. The reader must not stop the program,
and it must not take all the memory. A measurement on 2026-08-10 gave twelve
hostile files to `rbook` and to `epub`. Nothing stopped the program with a
panic, and nothing waited for ever. One file stopped the process: the zip bomb.

| The file | What must happen |
|---|---|
| A file that is not a zip archive | The reader shows "This file is not an EPUB." |
| An archive with no `container.xml` | The same message |
| An OPF that names a file that is absent | The chapter shows "This chapter is absent." The other chapters still work |
| A name in the archive with `../../../../etc/passwd` | Nothing. Both crates hold the name inside the archive, and they touch no file of the disk. Measured |
| A billion laughs attack | Nothing. html5ever, xml5ever, and quick-xml take no entity of a DTD. Measured |
| A zip bomb: 2 megabytes that opens to 2 gigabytes | **The rule of section 7.1** |
| 10000 nested `<div>` | **The rule of section 7.2** |
| A binary file with the name of an XHTML file | The parser gives text with no meaning, and the reader shows it. No crash |

### 7.1 Three limits of size

1. **The whole book.** The application refuses a file that is larger than 256
   megabytes, and it refuses an archive that holds more than 4096 entries. The
   message names the size.
2. **One chapter.** Every read goes through `copy_bytes` into a writer that
   refuses more than **8 megabytes**. The largest chapter of the four books of
   the measurement is 160 kilobytes, therefore this limit refuses no real book.
   A chapter that is too large shows "This chapter is too large.", and the
   reader still opens the next chapter.
3. **The answer of the server.** The download uses the limit of the whole book.

The measurement of the writer with a limit: the zip bomb gives an error, and
the program uses 5 megabytes. The same program with no limit uses 4102
megabytes and then stops with `abort`.

### 7.2 A limit of time

The task that renders a chapter also holds a limit of time. A render that
does not finish in **5 seconds** stops, and the chapter shows "This chapter is
too complex." The task runs on `spawn_blocking`, therefore a slow render blocks
no other work. The user can still move to a different chapter.

### 7.3 What a fault may do

`rbook` has no `unsafe` code of its own. `html2text` brings 129 lines of
`unsafe` through `tendril`, and every user of html5ever has that. Therefore a
fault of the crates gives a panic or wrong text, and not damage of the memory.
The reader runs inside the guard of T-17: a panic gives the terminal back and
stops the one chapter.

## 8. The work, in steps

Each step is its own commit, and the gate runs before each commit.

| Step | The work | The proof |
|---|---|---|
| 1 | The two crates, and `cargo tree -i cc` in CI | The tree holds no new C |
| 2 | `src/logic/reader/book.rs`: open a file, give the chapters and the table of contents, with every limit of section 7 | Tests with the four books and the twelve hostile files, in a debug build |
| 3 | `src/logic/reader/render.rs`: XHTML to `Vec<Line>` | Tests of the style, of the width, and of the empty chapter |
| 4 | `src/logic/reader/position.rs`: `toutui:<spine>:<line>`, and the part of the book | Pure tests, and a test of an EPUBCFI of the web reader |
| 5 | `GET /api/items/:id/ebook` with a limit, into the directory of the downloads | A test against the sandbox |
| 6 | `AppView::Reader`, the keys, and the task of the render | A run in a pseudo terminal against the sandbox |
| 7 | The sync of the position | A test against the sandbox: `PATCH`, then `GET`, then the audio position did not change |

The files of the hostile measurement stay in the repository under
`tests/data/hostile/`, because a test must not need the network. Their total
size is small: the zip bomb is 2 megabytes of zeros.

## 9. What this design does not do

- No EPUBCFI. The application reads that value and it does not write it.
- No picture inside the text. T-23 shows the cover, and a picture inside a
  chapter shows its alternative text.
- No PDF, no MOBI, and no CBZ. Audiobookshelf holds those formats, and no pure
  Rust reader of them meets the rule of the dependencies today.
- No search inside the book, and no note. Those are later work.

## 10. Where the numbers come from

The survey of 2026-08-10 holds every command and every number: the trees of the
dependencies, the four books of Project Gutenberg, the twelve hostile files,
the memory of each step, and the time of each render. The measurement used
throwaway crates outside the repository.

The sandbox now holds an EPUB. `docs/TEST-SERVER.md` section 6g tells how to
add it.
