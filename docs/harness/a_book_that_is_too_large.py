#!/usr/bin/env python3
"""A book whose file is larger than the limit of the reader.

See T-284. `Book::open` of `src/logic/reader/book.rs` reads the size of the
file of the book before it opens the archive, and a size above
`MAX_BOOK_BYTES` (256 megabytes) gives `ReaderError::BookTooLarge`:

    if data.is_file() && data.len() > MAX_BOOK_BYTES {
        return Err(ReaderError::BookTooLarge(data.len()));
    }

**That is the one road of the real program to that value.** The second road of
the same value stands in `src/logic/reader/pdf.rs`, and it does not reach the
user: the child of T-274 gives the parent an exit code, and the code of that
fault takes the arm `_` of `the_fault_of_the_code`.

**The data of this fault is a book, and it needs no proxy at all.**

    python3 docs/harness/a_book_that_is_too_large.py /the/path/of/the.epub 257

The number of the command line is the size of the file in megabytes. The bytes
of the padding go in the archive with **no** deflate (`ZIP_STORED`), because a
text that repeats gives an archive of some kilobytes and the limit reads the
size of the file of the disk. The book holds three chapters of plain text
beside that padding, therefore a program with a larger limit reads it.

**The book goes in the cache of the ebooks of the account**, because a book of
the cache costs no request of the server:

    $XDG_DATA_HOME/toutui/downloads/<the account>/<the id of the item>.epub

Keep the good file of that name for the road back, and give the place of that
media back at the end. **This file takes the megabytes of its command line on
the disk of the machine**: take it away after the measurement.
"""

import os
import sys
import zipfile

out = sys.argv[1]
megabytes = int(sys.argv[2]) if len(sys.argv) > 2 else 257


def page(title, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>'
            '<body>%s</body></html>' % (title, body))


plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"

opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-that-is-too-large</dc:identifier>
  <dc:title>The Book That Is Too Large</dc:title><dc:language>en</dc:language>
 </metadata>
 <manifest>
  <item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/>
  <item id="c2" href="c2.xhtml" media-type="application/xhtml+xml"/>
  <item id="c3" href="c3.xhtml" media-type="application/xhtml+xml"/>
  <item id="pad" href="pad.bin" media-type="application/octet-stream"/>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
 </manifest>
 <spine>
  <itemref idref="c1"/>
  <itemref idref="c2"/>
  <itemref idref="c3"/>
 </spine>
</package>"""

nav = page("Contents", '<nav xmlns:epub="http://www.idpf.org/2007/ops" epub:type="toc"><ol>'
           '<li><a href="c1.xhtml">One</a></li><li><a href="c2.xhtml">Two</a></li>'
           '<li><a href="c3.xhtml">Three</a></li></ol></nav>')

container = """<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
 <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"""

z = zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, allowZip64=True)
z.writestr("mimetype", "application/epub+zip", zipfile.ZIP_STORED)
z.writestr("META-INF/container.xml", container)
z.writestr("OEBPS/content.opf", opf)
z.writestr("OEBPS/nav.xhtml", nav)
z.writestr("OEBPS/c1.xhtml", page("One", plain % "CHAPTER ONE"))
z.writestr("OEBPS/c2.xhtml", page("Two", plain % "CHAPTER TWO"))
z.writestr("OEBPS/c3.xhtml", page("Three", plain % "CHAPTER THREE"))

# The padding takes no deflate, therefore the file of the disk holds these
# bytes and the limit of the size meets them.
block = b"\x00" * (1024 * 1024)
with z.open(zipfile.ZipInfo("OEBPS/pad.bin"), "w") as handle:
    for _ in range(megabytes):
        handle.write(block)

z.close()

print(out, "the file holds %d bytes" % os.path.getsize(out))
