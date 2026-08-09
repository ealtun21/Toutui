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

**MINOR**

`bug_id: 3a91e7` — CORRECTED on 2026-08-09, commit 14567c1
**Descriptions show HTML:** The description panel shows the HTML tags of the
description. An example is `<p>`, `<i>`, and `&amp;`. The application must
remove the tags and change the HTML entities to characters.

`bug_id: 255b86`
**Losing config after an update**: Ex: You change colors in config file and after an update, this configuration is lost and replaced by the config from main version.

`bug_id: 4b3045`
**Authentification Bug:** Even if you fill in valid credentials, the database sync can be buggy, and authentication may fail. Normally, it works on the second try.

`bug_id: 2eb9e3`
**Display:** At the launch, the app is not displayed and no error message appears (especially if you change user, quit and restart the app). Solution: quit the terminal and try it again.

`bug_id: 2d358c53`
**Mark as finished:** When a title reach the end, mark as finished not always work.

`bug_id: a49eza`
**cvlc error sync with ctrl vlc from a terminal:** If you use other command that `shutdown` to quit `cvlc` it may result of a sync issue.


**FIXED**  
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
**VLC:** `VLC` continue to run after the app is quit.  
`bug_id: fc695f`
**Listening session:** The session (that you can see in `yourserveraddress/audiobookshelf/config/sessions`) does not close when the app is quit.  
`bug_id: 40f48d`
**Cursor:** When you quit the app, terminal cursor disappear.  
`bug_id: fe4116`
**cvlc macOS:** `cvlc` option is not available for now in macOS.  
