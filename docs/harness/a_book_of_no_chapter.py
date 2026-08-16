#!/usr/bin/env python3
"""A book whose spine names no chapter at all.

See T-283. `Book::open` of `src/logic/reader/book.rs` takes the length of the
spine, and it has no limit below: a spine of no `itemref` therefore gives a
book of `chapter_count() == 0`. `Reader::open_with_the_title` then holds
`chapter: 0`, and `go_to_chapter` guards every other road with
`chapter >= self.chapter_count()`, therefore the chapter of the reader stays 0
and the render asks the book for the chapter 0 of a book of no chapter:
`chapter_bytes` gives `ReaderError::NoSuchChapter(0)`.

**That is the one road of the real program to that value.** Every other caller
of `chapter_xhtml` stands behind `go_to_chapter`, and the guard of it holds a
book of one chapter or more.

**The data of this fault is a book, and it needs no proxy at all.**

    python3 docs/harness/a_book_of_no_chapter.py /the/path/of/the.epub

The manifest of this book holds the two files of its chapters, therefore the
archive is a good archive of EPUB and `rbook` opens it: the spine alone is
empty. A book of a manifest of nothing takes the same road, and this form
keeps the fault at the spine alone.

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

# The manifest names the files of the two chapters, and the spine names none of
# them: the book therefore holds no chapter at all.
opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-of-no-chapter</dc:identifier>
  <dc:title>The Book Of No Chapter</dc:title><dc:language>en</dc:language>
 </metadata>
 <manifest>
  <item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/>
  <item id="c2" href="c2.xhtml" media-type="application/xhtml+xml"/>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
 </manifest>
 <spine></spine>
</package>"""

nav = page("Contents", '<nav xmlns:epub="http://www.idpf.org/2007/ops" epub:type="toc"><ol>'
           '<li><a href="c1.xhtml">One</a></li><li><a href="c2.xhtml">Two</a></li>'
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
z.writestr("OEBPS/c2.xhtml", page("Two", plain % "CHAPTER TWO. The last chapter"))
z.close()
print(out, "the manifest holds 2 files, and the spine names no chapter at all")
