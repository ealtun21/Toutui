#!/usr/bin/env python3
"""A book whose manifest names more files than the limit of the reader.

See T-284. `Book::open` of `src/logic/reader/book.rs` counts the manifest of
the book, and a count above `MAX_ENTRIES` (4096) gives
`ReaderError::TooManyEntries`:

    let entries = epub.manifest().len();
    if entries > MAX_ENTRIES {
        return Err(ReaderError::TooManyEntries(entries));
    }

**That is the one road of the real program to that value.** The second road of
the same value stands in `src/logic/reader/pdf.rs`, and it does not reach the
user: the child of T-274 gives the parent an exit code, and the code of that
fault takes the arm `_` of `the_fault_of_the_code`.

**The data of this fault is a book, and it needs no proxy at all.**

    python3 docs/harness/a_book_of_too_many_files.py /the/path/of/the.epub 4200

The number of the command line is the number of the files of the manifest, and
4200 of them give an archive of about 700 kilobytes: far under the
`MAX_BOOK_BYTES` of 256 megabytes, therefore the book meets the limit of the
count of the files and no other limit. Each file of the manifest stands in the
archive too, and the spine names the first three of them, because a manifest
that names a file that the archive does not hold is a different fault (T-282).

**The book goes in the cache of the ebooks of the account**, because a book of
the cache costs no request of the server:

    $XDG_DATA_HOME/toutui/downloads/<the account>/<the id of the item>.epub

Keep the good file of that name for the road back, and give the place of that
media back at the end.
"""

import sys
import zipfile

out = sys.argv[1]
count = int(sys.argv[2]) if len(sys.argv) > 2 else 4200

# The third argument gives a book whose archive holds the files of the spine
# alone. `Book::open` counts the manifest of the OPF and it opens no file of it,
# therefore that book meets the same limit in an archive of some kilobytes: it
# is the book of `tests/data/hostile/13-a-book-of-too-many-files.epub`, and the
# book of the measurement holds every file.
every_file = len(sys.argv) < 4 or sys.argv[3] != "the-spine-alone"


def page(title, body):
    return ('<?xml version="1.0" encoding="utf-8"?>\n'
            '<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>'
            '<body>%s</body></html>' % (title, body))


plain = "<h1>%s</h1><p>This is a chapter of plain text. It reads at once.</p>"

# The manifest names `count` files. The navigation document stands beside them,
# therefore the count of the manifest is `count` + 1.
items = "\n".join(
    '  <item id="c%d" href="c%d.xhtml" media-type="application/xhtml+xml"/>' % (n, n)
    for n in range(count)
)

# The spine names the first three files alone. A book that the reader opens
# reads the chapter 0, and a spine of every file gives the same fault with a
# larger archive.
spine = "\n".join('  <itemref idref="c%d"/>' % n for n in range(3))

opf = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
 <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="id">the-book-of-too-many-files</dc:identifier>
  <dc:title>The Book Of Too Many Files</dc:title><dc:language>en</dc:language>
 </metadata>
 <manifest>
%s
  <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
 </manifest>
 <spine>
%s
 </spine>
</package>""" % (items, spine)

nav = page("Contents", '<nav xmlns:epub="http://www.idpf.org/2007/ops" epub:type="toc"><ol>'
           '<li><a href="c0.xhtml">One</a></li><li><a href="c1.xhtml">Two</a></li>'
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

for n in range(count if every_file else 3):
    z.writestr("OEBPS/c%d.xhtml" % n, page("Chapter %d" % n, plain % ("CHAPTER %d" % n)))

z.close()

import os
print(out, "the manifest holds %d files, and the archive holds %d bytes"
      % (count + 1, os.path.getsize(out)))
