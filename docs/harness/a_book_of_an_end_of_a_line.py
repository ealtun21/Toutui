#!/usr/bin/env python3
"""A book whose title and whose name of a chapter hold an end of a line.

See T-313. `docs/harness/a_book_of_a_long_title.py` gives the reader a title
that is longer than the screen; this one gives it a title of **two** lines,
and a name of a chapter of two lines beside it.

**A literal end of a line in the `dc:title` does not reach the program**: the
parser of the XML normalizes the whitespace of the content of an element, and
the title `Alpha\\nOMEGAEND` therefore reaches `Book::title` as
`Alpha OMEGAEND`. **A character reference of `&#10;` keeps that end**, because
a parser of XML gives a character reference its character and it does not
normalize it (XML 1.0, section 2.11 and section 3.3.3). That is the road of
this book, and it is the road of every maker of an EPUB that writes a title of
more than one line.

The book holds three chapters of plain text, therefore every chapter reads at
once and the keys `n` and `p` hold a control of the same run:

    python3 docs/harness/a_book_of_an_end_of_a_line.py /the/path/of/the.epub

**The data of this fault is a book, and it needs no proxy and no change of the
sandbox at all.** The book goes in the cache of the ebooks of the account,
because a book of the cache costs no request of the server:

    $XDG_DATA_HOME/toutui/downloads/<the account>/<the id of the item>.epub

Keep the good file of that name for the road back. **The reader keeps the
chapter of a book of a name that it read already**, therefore a second run of
such a measurement opens at the chapter of the run before it.

The same book stands at `tests/data/hostile/15-a-book-of-an-end-of-a-line.epub`.
"""

import sys
import zipfile

out = sys.argv[1]

# `&#10;` is the end of a line that the parser of the XML keeps.
THE_TITLE = "Alpha&#10;OMEGAEND"
THE_NAME_OF_THE_SECOND_CHAPTER = "Beta&#10;GAMMAEND"


def page(name, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>'
            '<body>%s</body></html>' % (name, body))


plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"

opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-of-an-end-of-a-line</dc:identifier>
  <dc:title>%s</dc:title><dc:language>en</dc:language>
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
</package>""" % THE_TITLE

nav = page("Contents", '<nav xmlns:epub="http://www.idpf.org/2007/ops" epub:type="toc"><ol>'
           '<li><a href="c1.xhtml">One</a></li>'
           '<li><a href="c2.xhtml">%s</a></li>'
           '<li><a href="c3.xhtml">Three</a></li></ol></nav>'
           % THE_NAME_OF_THE_SECOND_CHAPTER)

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
z.writestr("OEBPS/c2.xhtml", page("Two", plain % "CHAPTER TWO. The chapter of the middle"))
z.writestr("OEBPS/c3.xhtml", page("Three", plain % "CHAPTER THREE. The last chapter"))
z.close()
print(out, "holds a title and a name of a chapter of two lines each")
