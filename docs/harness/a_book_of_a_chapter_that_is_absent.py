#!/usr/bin/env python3
"""A book whose spine names a chapter that the manifest does not hold.

See T-282. The spine of an EPUB names each chapter with an `idref`, and the
manifest of the same file names the file of each `id`. A spine that names an
`idref` of no manifest item is therefore a chapter of the book that has no
file at all: `chapter_bytes` of `src/logic/reader/book.rs` gives
`ReaderError::ChapterAbsent` for it, and that value is the one road to that
arm.

**The data of this fault is a book, and it needs no proxy at all.**

The book holds three chapters, and the second one is the chapter of no file.
The first and the third read at once, therefore the keys `n` and `p` of the
view of the reader hold a control of the same run.

    python3 docs/harness/a_book_of_a_chapter_that_is_absent.py /the/path/of/the.epub

**The book goes in the cache of the ebooks of the account**, because a book of
the cache costs no request of the server:

    $XDG_DATA_HOME/toutui/downloads/<the account>/<the id of the item>.epub

Keep the good file of that name for the road back, and give the place of that
media back at the end: **the reader keeps the chapter of a book of a name that
it read already**, therefore a second run opens at the chapter of the run
before it.
"""

import sys
import zipfile

out = sys.argv[1]

def page(title, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>'
            '<body>%s</body></html>' % (title, body))

plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"

# The spine names `c2`, and the manifest holds no item of that id: the second
# chapter of this book therefore has no file at all.
opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-of-a-chapter-that-is-absent</dc:identifier>
  <dc:title>The Book Of A Chapter That Is Absent</dc:title><dc:language>en</dc:language>
 </metadata>
 <manifest>
  <item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/>
  <item id="c3" href="c3.xhtml" media-type="application/xhtml+xml"/>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
 </manifest>
 <spine><itemref idref="c1"/><itemref idref="c2"/><itemref idref="c3"/></spine>
</package>"""

nav = page("Contents", '<nav xmlns:epub="http://www.idpf.org/2007/ops" epub:type="toc"><ol>'
           '<li><a href="c1.xhtml">One</a></li><li><a href="c3.xhtml">Three</a></li>'
           '</ol></nav>')

container = """<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
 <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"""

z = zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED)
z.writestr("mimetype", "application/epub+zip", zipfile.ZIP_STORED)
z.writestr("META-INF/container.xml", container)
z.writestr("OEBPS/content.opf", opf)
z.writestr("OEBPS/nav.xhtml", nav)
z.writestr("OEBPS/c1.xhtml", page("One", plain % "CHAPTER ONE. The first chapter"))
z.writestr("OEBPS/c3.xhtml", page("Three", plain % "CHAPTER THREE. The last chapter"))
z.close()
print(out, "the spine holds 3 chapters, and the manifest holds 2 of them")
