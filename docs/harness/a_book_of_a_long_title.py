#!/usr/bin/env python3
"""A book whose title is longer than the screen.

See T-300. The line at the top of the reader is
`line_of_the_top` of `src/ui/reader_tui.rs`, and it says
`<the title> — chapter N of M — P%`: the title of the book comes first, and
the place of the user comes after it. That line stands in a `Paragraph` of
one row with no `wrap`, therefore a title that fills the width of the
terminal takes the number of the chapter, the count of the chapters, and the
percent away.

**A long title is no book of a test.** Project Gutenberg holds
`The Life and Adventures of Robinson Crusoe, of York, Mariner: Who Lived
Eight and Twenty Years All Alone in an Uninhabited Island on the Coast of
America` (153 characters), and a book of a series of an Audiobookshelf
library carries its subtitle in the same field.

**The data of this fault is a book, and it needs no proxy at all.**

    python3 docs/harness/a_book_of_a_long_title.py /the/path/of/the.epub

The second argument gives a title of your own; with no second argument the
book takes the title of Robinson Crusoe above. The book holds three chapters
of plain text, therefore every chapter reads at once and the keys `n` and `p`
hold a control of the same run.

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

THE_TITLE_OF_GUTENBERG = (
    "The Life and Adventures of Robinson Crusoe, of York, Mariner: Who Lived "
    "Eight and Twenty Years All Alone in an Uninhabited Island on the Coast "
    "of America"
)

title = sys.argv[2] if len(sys.argv) > 2 else THE_TITLE_OF_GUTENBERG


def page(name, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>'
            '<body>%s</body></html>' % (name, body))


plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"

opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-of-a-long-title</dc:identifier>
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
</package>""" % title

nav = page("Contents", '<nav xmlns:epub="http://www.idpf.org/2007/ops" epub:type="toc"><ol>'
           '<li><a href="c1.xhtml">One</a></li><li><a href="c2.xhtml">Two</a></li>'
           '<li><a href="c3.xhtml">Three</a></li></ol></nav>')

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
print(out, "holds 3 chapters, and its title holds %d characters" % len(title))
