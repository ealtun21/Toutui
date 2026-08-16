#!/usr/bin/env python3
"""A book whose one chapter holds the render for longer than its limit of time.

See T-280. `MAX_CHAPTER_BYTES` of the reader is 8 megabytes, therefore a
chapter that is simply large meets the guard of the size and never reaches the
render. The time of `html2text` grows with the square of the depth of the tags:
10000 nested `<div>` need 895 milliseconds in a debug build, and 40000 of them
need more than the five seconds of `TIME_FOR_ONE_CHAPTER`, in 440214 bytes.

The book holds three chapters. The first and the third read at once, therefore
the keys `n` and `p` of the view of the reader hold a control of the same run.

    python3 docs/harness/a_book_of_a_deep_chapter.py /the/path/of/the.epub 40000

**The book goes in the cache of the ebooks of the account**, because a book of
the cache costs no request of the server:

    $XDG_DATA_HOME/toutui/downloads/<the account>/<the id of the item>.epub

Keep the good file of that name for the road back.
"""

import sys
import zipfile

out, depth = sys.argv[1], int(sys.argv[2])

def page(title, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>'
            '<body>%s</body></html>' % (title, body))

plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"
deep = "<div>" * depth + "<p>The words at the bottom of the nest.</p>" + "</div>" * depth
hostile = page("The deep chapter", "<h1>The deep chapter</h1>" + deep)

opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-deep-book</dc:identifier>
  <dc:title>The Deep Book</dc:title><dc:language>en</dc:language>
 </metadata>
 <manifest>
  <item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/>
  <item id="c2" href="c2.xhtml" media-type="application/xhtml+xml"/>
  <item id="c3" href="c3.xhtml" media-type="application/xhtml+xml"/>
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
 </manifest>
 <spine><itemref idref="c1"/><itemref idref="c2"/><itemref idref="c3"/></spine>
</package>"""

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
z.writestr("OEBPS/c2.xhtml", hostile)
z.writestr("OEBPS/c3.xhtml", page("Three", plain % "CHAPTER THREE. The last chapter"))
z.close()
print(out, "depth", depth, "chapter bytes", len(hostile))
