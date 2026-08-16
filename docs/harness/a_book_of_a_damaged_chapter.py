#!/usr/bin/env python3
"""A book whose second chapter holds a stream that the archive cannot read.

See T-277 and T-286. `Book::chapter_bytes` of `src/logic/reader/book.rs` gives
`ReaderError::TheArchiveGaveNoChapter(reason)` when `copy_bytes` of the crate
of the archive fails, and the text of it is the reason of that crate. **That is
the one road of the real program to that value**: a chapter that the manifest
does not hold takes `ReaderError::ChapterAbsent` (T-282), and a chapter that
stands above the limit takes `ReaderError::ChapterTooLarge` (T-281).

**The data of this fault is a book, and it needs no proxy at all.** The
measurement of T-277 took the real book of the sandbox and it flipped 64 bytes
of the stream of one entry of it with a hand. This harness writes that book:

    python3 docs/harness/a_book_of_a_damaged_chapter.py /the/path/of/the.epub

The book holds three chapters. The first and the third read at once, and the
deflate stream of the second one holds 64 bytes of the complement of
themselves: the central directory and the local header keep every number,
therefore `rbook` opens the book and the spine of it holds three chapters, and
the read of the chapter 2 alone fails. **The keys `n`, `p`, and `h` therefore
hold a control of the same run**: the chapter after it and the chapter before
it each read, and the key `h` leaves the book.

**The book goes in the cache of the ebooks of the account**, at

    $XDG_DATA_HOME/toutui/downloads/<the account>/<the id of the item>.epub

because a book of the cache costs no request of the server. Keep the good file
of that name for the road back, and give the place of that media back with
`PATCH /api/me/progress/:id` at the end: **the reader keeps the chapter of a
book of a name that it read already**, therefore a second run of such a
measurement opens at the chapter of the run before it.
"""

import sys
import zipfile

out = sys.argv[1]

# The number of the bytes of the stream that take the value of their own
# complement, and the place of the first of them after the start of the data of
# the entry. The values of the measurement of T-277.
DAMAGED_BYTES = 64
AFTER_THE_START = 200


def page(title, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title>'
            '</head><body>%s</body></html>' % (title, body))


plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"

# The chapter of the fault needs a stream that is longer than the place of the
# damage: a short chapter deflates to fewer than 200 bytes, and the flip of the
# bytes then lands in the central directory and not in the data.
long_body = "<h1>Chapter Two</h1>" + "".join(
    "<p>This is the paragraph %d of the chapter of the fault. The words of it "
    "repeat, therefore the deflate of the archive gives a stream of some "
    "thousands of bytes.</p>" % n for n in range(400))

opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-of-a-damaged-chapter</dc:identifier>
  <dc:title>The Book Of A Damaged Chapter</dc:title><dc:language>en</dc:language>
 </metadata>
 <manifest>
  <item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/>
  <item id="c2" href="c2.xhtml" media-type="application/xhtml+xml"/>
  <item id="c3" href="c3.xhtml" media-type="application/xhtml+xml"/>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
 </manifest>
 <spine>
  <itemref idref="c1"/><itemref idref="c2"/><itemref idref="c3"/>
 </spine>
</package>
"""

container = """<?xml version="1.0" encoding="utf-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
 <rootfiles><rootfile full-path="OEBPS/book.opf"
   media-type="application/oebps-package+xml"/></rootfiles>
</container>
"""

nav = page("Contents",
           '<nav epub:type="toc" xmlns:epub="http://www.idpf.org/2007/ops"><ol>'
           '<li><a href="c1.xhtml">Chapter One</a></li>'
           '<li><a href="c2.xhtml">Chapter Two</a></li>'
           '<li><a href="c3.xhtml">Chapter Three</a></li></ol></nav>')

with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as book:
    book.writestr("mimetype", "application/epub+zip", zipfile.ZIP_STORED)
    book.writestr("META-INF/container.xml", container)
    book.writestr("OEBPS/book.opf", opf)
    book.writestr("OEBPS/nav.xhtml", nav)
    book.writestr("OEBPS/c1.xhtml", page("Chapter One", plain % "Chapter One"))
    book.writestr("OEBPS/c2.xhtml", page("Chapter Two", long_body))
    book.writestr("OEBPS/c3.xhtml", page("Chapter Three", plain % "Chapter Three"))

# The place of the data of the entry of the chapter of the fault. The local
# header of the zip holds 30 bytes of numbers, and the name and the extra field
# of that header stand after them.
with zipfile.ZipFile(out) as book:
    entry = book.getinfo("OEBPS/c2.xhtml")
    header_offset = entry.header_offset
    compressed = entry.compress_size

with open(out, "r+b") as file_of_the_book:
    file_of_the_book.seek(header_offset + 26)
    head = file_of_the_book.read(4)
    name_length = head[0] | (head[1] << 8)
    extra_length = head[2] | (head[3] << 8)
    start = header_offset + 30 + name_length + extra_length

    if compressed < AFTER_THE_START + DAMAGED_BYTES:
        sys.exit("the stream of the chapter holds %d bytes, and the damage "
                 "needs %d of them" % (compressed,
                                       AFTER_THE_START + DAMAGED_BYTES))

    file_of_the_book.seek(start + AFTER_THE_START)
    good = file_of_the_book.read(DAMAGED_BYTES)
    file_of_the_book.seek(start + AFTER_THE_START)
    file_of_the_book.write(bytes(byte ^ 0xFF for byte in good))

print("%s: the stream of the chapter 2 holds %d bytes, and %d of them at %d "
      "after the start of it took the value of their own complement"
      % (out, compressed, DAMAGED_BYTES, AFTER_THE_START))
