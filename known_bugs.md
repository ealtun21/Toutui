**MAJOR**

`bug_id: 7f2c10` — CORRECTED on 2026-08-09, commit 6961659
**Offline mode does not play a book:** The server sends a ZIP archive from
`GET /api/items/:id/download`. It sends a ZIP archive for every book, also for
a book that has one audio file. The application writes the archive to the disk
and records the path of the archive in the `downloads` table. Then it gives
that path to VLC. VLC cannot play a ZIP archive. Therefore no downloaded book
plays.
The application must open the archive, take the audio files, and record the
path of the audio file.
Measured on Audiobookshelf 2.36.0 on 2026-08-09. The archive of the test book
contains `Cover.jpg` and one `.mp3` file.

`bug_id: 8f31aa` — CORRECTED on 2026-08-10, commit a52a34d
**A media does not stay marked as finished:** The server answered 200, the
progress showed 100 percent, and `isFinished` stayed false. The item then
stayed in "Continue listening". The cause is the sequence of the keys in the
body: `{"currentTime":..,"isFinished":true,"progress":..}` does not mark the
media, and `{"progress":..,"isFinished":true,"currentTime":..}` marks it.
`serde_json` writes the keys in the sequence of the alphabet, thus the
application always made the first body. The application sends the mark in its
own request now. Measured on Audiobookshelf 2.36.0.

`bug_id: 1c7e42` — CORRECTED on 2026-08-10, commit 597ca2d
**The key `G` in an empty list stops the application:** `select_last`
calculated `len() - 1`. A library with no item, a search with no result, and a
podcast with no episode all give this condition. A debug build stopped with
"attempt to subtract with overflow".

`bug_id: 4d9b03` — CORRECTED on 2026-08-10, commit c9a68d8
**The key `X` removes one file only:** The table `downloads` holds the path of
the first file. A book with many audio files therefore kept every other file
on the disk, and the user had no way to remove them from the application.

**MINOR**

`bug_id: 3a91e7` — CORRECTED on 2026-08-09, commit 14567c1
**Descriptions show HTML:** The description panel shows the HTML tags of the
description. An example is `<p>`, `<i>`, and `&amp;`. The application must
remove the tags and change the HTML entities to characters.

`bug_id: 7c5e18` — CORRECTED on 2026-08-10, commit bc9ceb0
**The application does not start without the server:** The application
downloaded a book for the offline mode, and it still needed the server to
start and to play. The screen stayed empty. The application starts now, it
shows the media of the disk, it plays them, and it sends the position when the
server answers again.

`bug_id: 4b3045`
**Authentification Bug:** Even if you fill in valid credentials, the database sync can be buggy, and authentication may fail. Normally, it works on the second try.




**FIXED**  
`bug_id: 255b86` — CORRECTED on 2026-08-10
**Losing config after an update:** The script of the original project merged `config.example.toml` into the configuration of the user at every installation. Line 471 of `hello_toutui.sh` reads `$pseudo_escape_line`, and nothing gives that name a value. Therefore the test became `grep -E "^"`, that pattern agrees with every line, and the script added no line of the user that `config.example.toml` does not name. The merge also wrote the file again from the text of the example, thus the comments and the sequence of the example replaced those of the user. The fork writes `config.toml` only when that file is absent, and it merges nothing. `--update` moves one file: the binary. A test on 2026-08-10 changed a colour, added an option, and installed a newer release: the file of the user and the secret key did not change. `tests/update.rs` holds the guard.  
`bug_id: 2d358c53` — CORRECTED on 2026-08-10
**Mark as finished:** The engine now reports that the media came to its end, and the application sends `isFinished`. A test with a real server shows `isFinished=true` and `progress=1` after the book comes to its end.  
`bug_id: 2eb9e3` — CORRECTED on 2026-08-10
**Display:** At the launch, the app was not displayed and no error message appeared. The application does not start a separate program now, thus this fault cannot occur.  
`bug_id: a49eza` — CORRECTED on 2026-08-10
**cvlc error sync with ctrl vlc from a terminal:** The application has no remote control interface now.  
`bug_id: fe4116` — CORRECTED on 2026-08-10
**cvlc macOS:** The application does not use `cvlc` on macOS now. It plays the audio itself.  
`bug_id: 9bacac` 
**Sync**: If you open VLC to listen X, close VLC and quickly open VLC again to listen Y: X will still be sync — according to Y (normally, only Y has to be sync in this case).   
`bug_id: 86384e` 
**Sync**: Rarely and especially if you open VLC to listen X, close VLC and quickly open VLC again to listen Y: the progress of X is set to 0 seconds.  
`bug_id: 06e548` 
**Terminal broken**: The terminal is broken after the app is quit.  
`bug_id: 6ac5d8` 
**Data loss if app crash or disgracefully quit**: If app crash, the last session is not closed.  
`bug_id: bf10cd` 
**Launch a new media**: Have to close manually VLC to close and sync a session.  
`bug_id: 3f729c` 
**Loading time**: for now, not optimized for a library with a lot of items (long start loading and refresh time)  
NOTE 2026-08-09: this bug did not occur in a test with a library of 2056 items
on Audiobookshelf 2.36.0. The first screen appeared after 0.4 seconds. The API
gave all 2056 items in 0.48 seconds. Examine this bug again before you do work
on it.  
`bug_id: dd9a649`
**Listening Session:** Sometimes, the session (that you can see in `yourserveraddress/audiobookshelf/config/sessions`) does not close correctly, especially if you open VLC, quit it quickly, and start another book.  
`bug_id: e0b61c`
**VLC:** `VLC` continue to run after the app is quit. The application does not start VLC now.  
`bug_id: fc695f`
**Listening session:** The session (that you can see in `yourserveraddress/audiobookshelf/config/sessions`) does not close when the app is quit.  
`bug_id: 40f48d`
**Cursor:** When you quit the app, terminal cursor disappear.
