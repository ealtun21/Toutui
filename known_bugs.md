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

`bug_id: 9bacac` — CORRECTED on 2026-08-10, commit c82c9d8
**Sync:** The user plays the book X, and then the user plays the book Y
quickly. The progress of X then holds the position of Y.
One mechanism explains this report and the two reports below. The state of the
engine is one value for the whole application, and the key that starts a media
gives its work to a new task. Therefore two playbacks can run at the same time.
The loop that follows the playback of X read that state always, thus it read
the position of Y and it reported that position for the session of X.
Every playback has its own identity now. The engine writes the identity of the
playback that it plays, and a loop reads the state only while that identity is
its own. A loop that loses the engine closes its own session and reports the
last position that it measured itself.
A measurement on 2026-08-10 ran `follow_playback` in a real process against a
real server. The engine reported the book X at 100 seconds, and then the book Y
at 4 seconds. The loop of X sent `{"currentTime":"4","timeListened":"0"}` to
the session of X, and the loop did not stop.
`tests/playback_ownership.rs` holds the rule.

`bug_id: 86384e` — CORRECTED on 2026-08-10, commit c82c9d8
**Sync:** The same condition sets the progress of X to 0 seconds. The mechanism
of `9bacac` gives this result: a book Y that starts holds a position that is
almost 0, and the loop of X reported the position of Y. The report says
"rarely", because the value depends on the moment of the read operation.
The correction is the same. The loop of X now reports the last position that
the engine gave for X.

`bug_id: dd9a649` — CORRECTED on 2026-08-10, commit c82c9d8
**Listening session:** The session of X does not always close.
The mechanism of `9bacac` also explains this report. The loop of X reads the
status of the engine to see whether the playback stopped. The engine played Y,
therefore the status was `Playing` and the loop of X never closed its session.
A second condition gave the same result: the loop closed no session when the
engine did not start the playback at all.
A measurement against Audiobookshelf 2.36.0 on 2026-08-10 shows both states of
the program. With the old behaviour the loop of X did not stop, and
`GET /api/sessions/open` held the session `ca2079ec` of X. With the correction
the server holds no open session of X, and the progress of X holds the position
of X. A loop whose playback the engine does not start in 30 seconds also closes
its session now.
`tests/sync_against_the_sandbox.rs` holds that test. Continuous integration
does not run it, because it needs a server.

`bug_id: f4a8c2` — CORRECTED on 2026-08-10, commit 21aac71
**A colour of the configuration file stops the program:** Every place that read
a colour took the three components with an index. A list that is too short then
stops the program. `load_config` also gives an error for a file that a person
cannot parse, and the old code then read an empty list. The fork found this
fault on 2026-08-10, and a measurement stopped a thread with "index out of
bounds: the len is 0 but the index is 0". `rgb_parts` in `src/config.rs` gives
the three components now, and all eleven places use it.

`bug_id: 3b6e91` — CORRECTED on 2026-08-10, commit e4b51c9
**A playback that does not start stops every later playback:**
`wait_prev_session_finished` waits while `is_loop_break` is not `1`, and it
gives that value `0` before a playback begins. The old code gave the value `1`
in the two loops that follow a playback only. Five places came back without a
loop: a server that gives an error, an item that the server does not give, an
item with no audio file, and two conditions of the offline mode. The next
playback then waited for ever, and the screen held the message "Syncing your
last listening session. Please wait...". The fork found this fault while it
examined `9bacac`. A measurement on 2026-08-10 ran `play` against a server that
answered 500, and the value stayed `0`. `play` always gives the value `1` now.
`tests/playback_wait_flag.rs` holds the rule.

`bug_id: 5c8d72` — CORRECTED on 2026-08-10, commit c342f50
**The application does not play an Opus file:** Audiobookshelf accepts Opus, and
`rodio::Decoder` has no Opus in its registry of codecs. The engine now reads the
packets with symphonia and decodes them with `opuscule`. A measurement on
2026-08-10 compared the samples with libopus over 50 files, and the largest
difference of one sample is 0.00002 of a full scale of 1.0. See T-17.

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





**NOT YET EXAMINED**

No report stays in this section. Every report of the original project has an
examination now.

**FIXED**  
`bug_id: fc695f` — CORRECTED on 2026-08-10
**Listening session:** The key `Q` did not always stop the program, therefore the user stopped it by force and the session stayed open. `sync_session_from_database` stopped the program in one branch only. The branch with no session asked `has_played_before`, and no line of the program gave that value `1` again after a playback began. The branch of the error stopped nothing. The program stops in every branch now, because the sync is the best that the program can do and it must not decide whether the program stops. A measurement on 2026-08-10 ran a real process in the condition of the fault: with the old test the function came back and the program stayed, and with the correction the program stopped.  
`bug_id: 6ac5d8` — CORRECTED on 2026-08-10
**Data loss if app crash or disgracefully quit:** Two parts answer this report. `Q` now always stops the program, therefore the user has no reason to stop it by force; see `fc695f`. A program that still stops without a correct exit leaves its row in `listening_session`, and the next start sends that position one time; see T-4 in `docs/TAKEOVER-BACKLOG.md`.  
`bug_id: bf10cd` — CORRECTED on 2026-08-10
**Launch a new media:** The application plays the audio itself and starts no other program. Therefore no user closes VLC by hand to close a session. The key `l` closes the session before it opens the new one.  
`bug_id: 06e548` — CORRECTED on 2026-08-10
**Terminal broken:** A panic gave the terminal back to the shell in the raw mode and on the alternate screen. The program installed no hook of the panic at all. `install_panic_hook` in `src/utils/exit_app.rs` gives the terminal back first and then writes the message, therefore the user can read that message and give it to a report. A measurement on 2026-08-10 ran a real process that took the terminal and then panicked: with no hook the bytes held no `ESC [ ? 1049 l`, and with the hook they hold it.  
`bug_id: 40f48d` — CORRECTED on 2026-08-10
**Cursor:** The same hook writes `ESC [ ? 25 h` and the cursor comes back. The same measurement shows the byte sequence with the hook and no sequence without it.  
`bug_id: 4b3045` — CORRECTED on 2026-08-10
**Authentification Bug:** One mechanism explains this report: the program read the database before the login wrote the user. The old code started the login with `tokio::spawn` and did not wait for it. `auth_input.rs` waits for the thread of the login now, and `auth_process` writes the user before it gives its answer. A test against a real server of Audiobookshelf 2.36.0 read the database with no wait after a first login: the user, the encrypted token, and the selected library were all present. `tests/login_against_the_sandbox.rs` holds that test.  
`bug_id: e0b61c` — CORRECTED on 2026-08-10
**VLC:** The application does not start VLC now, therefore VLC cannot continue to run after the application quits.  
`bug_id: 3f729c` — CORRECTED on 2026-08-09
**Loading time:** This fault did not occur in a test with a library of 2056 items on Audiobookshelf 2.36.0. The first screen came after 0.4 seconds, and the API gave all 2056 items in 0.48 seconds.  
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
