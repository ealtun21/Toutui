const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn changelog() -> String {
    let mut changelog = String::new();

    // The screen of settings shows this text. The credit is at the top,
    // because AlbanDAVID wrote the original program.
    changelog.push_str(
        "AlbanDAVID wrote Toutui and archived it. This repository continues\n\
         that work. https://github.com/AlbanDAVID/Toutui\n\
         \n\
         The entries below this line describe the original project. Some\n\
         name a script or a package that does not exist now. The README of\n\
         this repository gives the ways to install, to update, and to\n\
         remove the fork.\n\
         \n\
         ####\n",
    );

    let changelog_01 = "Changelog Toutui v0.1.0-beta (02/21/2025) \n\
         Fixed:\n\
         \n\
         First release.
         \n\
         Changed:\n\
         \n\
         First release.
         \n\
         Enjoy!\n
         ####\n"
        .to_string();
    let changelog_02 = "Changelog Toutui v0.1.1-beta (02/24/2025) \n\
         Fixed:\n\
         \n\
         - App crash (out of bounds) when API send empty values.
         - Close listening session not always working (bug_id: fixed_dd9a64)
         \n\
         Changed:\n\
         \n\
         No change.
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_03 = "Changelog Toutui v0.1.2-beta (02/24/2025) \n\
         Fixed:\n\
         \n\
         - Partially fixed, becsause not optimal: bug_id: 9bacac Sync: If you open VLC to listen X, close VLC and quickly open VLC again to listen Y: X will still be sync — according to Y (normally, only Y has to be sync in this case).

         \n\
         Changed:\n\
         \n\
         No change.
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_04 = "Changelog Toutui v0.1.3-beta (02/03/2025) \n\
         Fixed:\n\
         \n\
         - Fix bug_id: 3f729c Loading time not optimized for library with a lot of items (long start loading and refresh time)
         \n\
         Changed:\n\
         \n\
         - Script `hello_toutui` to make installation easier.
         \n\
         Contributors:\n\
         \n\
         - dougy147, dhonus
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_05 = "Changelog Toutui v0.2.0-beta (07/03/2025) \n\
CAUTION: This version is not compatible with the previous one.  
You need to remove the database in ~/.config/toutui before proceeding. 
         Fixed:\n\
         \n\
         - From known_bugs.md, fixed:

    Find a robust solution for bug_id: 9bacac
    Fix bug_id: 86384e
    Fix bug_id: 6ac5d8
    Fix bug_id: 06e548
    Fix bug_id: e0b61c
    Fix bug_id: fc695f
    Fix bug_id: 40f48d
    Fix bug_id: bf10cd

         \n\
         Changed:\n\
         \n\
         - 
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_06 = "Changelog Toutui v0.3.0-beta (24/03/2025) \n\
CAUTION: This version is not compatible with the previous one.  
To make it work properly, perform a fresh reinstall.
\n\
         Added:\n\
         - Integrated player. Keep calm and stay in your terminal! :)
         \n\
         Fixed:\n\
         \n\
         - Fixed: issue where pressing R twice was required to refresh the app.
         - Fixed: issue causing the cursor to disappear when the application is closed.
         - Fixed: issue if app is quitted for the first time and that listening session is empty.
         \n\
         Changed:\n\
         \n\
         - Faster loading time to play an item.
         - Improved synchronization accurary.
         - Removed warning during compilation time.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_07 = "Changelog Toutui v0.3.1-beta (25/03/2025) \n\
CAUTION: This version is not compatible with v0.2.0-beta and bellow.  
To make it work properly, perform a fresh reinstall.
\n\
         Fixed:\n\
         \n\
         - Fixed: incorrect merge
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_08 = "Changelog Toutui v0.3.2-beta (26/03/2025) \n\
         Added:\n\
         \n\
         - macOS compatibility.
         \n\
         Fixed:\n\
         \n\
         - Issue with VLC buffer (if a chapter is manually changed or during jump/backward).
         - Display issue on small monitors.
         \n\
         Changed:\n\
         \n\
         - hello_toutui script improved
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n"
        .to_string();
    let changelog_09 = "Changelog Toutui v0.3.3-beta (02/04/2025) \n\
         \n\
         Changed:\n\
         \n\
         - Adding a login placeholder to specify the use of http:// or https:// for the server address.
         - Display error login message without time limit.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_10 = "Changelog Toutui v0.3.4-beta (23/04/2025) \n\
         \n\
         Fix:\n\
         \n\
         Handle empty podcast episode lists gracefully. Prevent panic and show 'No episodes' message. by @denispol in https://github.com/AlbanDAVID/Toutui/pull/22\n\
         Contributors:\n\
         \n\
         - AlbanDAVID, denispol
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_11 = "Changelog Toutui v0.3.5-beta (27/04/2025) \n\
         \n\
         Added:\n\
         - Display number of total items for continue listening, library and library settings (for books and podcasts)
         - Clap crate and a function to display the version in the CLI (e.g. `toutui --version`)
         \n\
         Fixed:\n\
         \n\
         - [macos] vlc version not displayed in listening sessions (from ABS web browser)
         - Out of bounds in Library Settings
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_12 = "Changelog Toutui v0.4.0-beta (10/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Simplified installation and updates by: 
            - Downloading the binary.
            - Compiling it from source (no local clone needed).

         -  New commands available:
            - toutui --update and toutui --uninstall cmd added.

         - Notify if an update is available directly in the app.

         - [Linux only] The app can now be launched via an app launcher.
         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID, dougy147
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    // The entry of a release that came before this build names its own
    // version. Only the newest entry takes the version of the build.
    let changelog_15 = "Changelog Toutui v0.5.0 (10/08/2026) \n\
     \n\
     Added:\n\
     - The program plays a local copy when the server does not answer, and\n\
       it sends the positions when the server answers again.\n\
     - The program updates itself with `toutui --update`. The program\n\
       compares the sum of the archive before it moves the new binary.\n\
     - The releases come from this repository, and the archives have a sum\n\
       SHA-256 that the machine writes.\n\
     \n\
     Fixed:\n\
     - `--update` installed the archived original project.\n\
     - `Mark as finished` did not always operate.\n\
     \n\
     Changed:\n\
     - The script of installation has 100 lines and not 1080. It installs\n\
       no VLC and no netcat, because the player in the program needs\n\
       neither.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_16 = "Changelog Toutui v0.6.0 (10/08/2026) \n\
     \n\
     Added:\n\
     - The cover art. The cover stands beside the description and it is\n\
       always visible. A series shows the cover of each of its books. The\n\
       cover of the media that plays is the largest one. A narrow terminal\n\
       gives the whole width to the text.\n\
     - A series takes one line of the Library view. The key `l` opens its\n\
       books, in the sequence of the series.\n\
     - The key `F` sends the position to the server at once. It does not\n\
       close the listening session.\n\
     \n\
     Changed:\n\
     - The program uses ratatui 0.30 and crossterm 0.29. `tui-textarea` is\n\
       gone, and `tui-input` takes its place. The login screen and the bar\n\
       of the search work as before.\n\
     - The build still needs no C toolchain for a library of the system,\n\
       and it needs no OpenSSL.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_17 = "Changelog Toutui v0.6.1 (10/08/2026) \n\
     \n\
     This release answers a report of a user on v0.5.0 and v0.6.0.\n\
     \n\
     Fixed:\n\
     - The program drew nothing while it started. A slow server gave a\n\
       black screen, and the user could not tell a slow server from a\n\
       program that stopped. The program now draws at once, it names the\n\
       step that it waits for, and the key Q stops it.\n\
     - The start is faster: the position of each book of the list\n\
       Continue Listening goes in one group of requests, and not one\n\
       after the other.\n\
     - A machine with no sound device could not open the program at all.\n\
       Every function that needs no sound works now.\n\
     - No index of a vector can stop the screen. The render read a\n\
       vector with the number of the selected line in 39 places, and a\n\
       list of the screen can be shorter than that number.\n\
     - The login examines the address of the server before it asks for\n\
       the password, and it says what is wrong.\n\
     \n\
     Added:\n\
     - Every line of the Home view and of the Library view has a mark:\n\
       the media that plays, a media that is finished, or the part that\n\
       the user heard.\n\
     - The settings say \"Accounts and log out\", and each entry tells\n\
       what it does.\n\
     - TOUTUI_NO_COVERS turns the cover art off. Inside tmux the program\n\
       asks the terminal nothing.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_18 = "Changelog Toutui v0.6.2 (10/08/2026) \n\
     \n\
     Fixed:\n\
     - The start cannot wait for ever now. The sound device gets five\n\
       seconds, and the program then goes on with no sound. An answer of\n\
       the server that the program cannot read no longer stops the\n\
       program.\n\
     - The start is faster with a large library. The pages of the items go\n\
       to the server together. A library of 2056 items needs five pages,\n\
       and the program asked for them one after the other before.\n\
     \n\
     Added:\n\
     - The reader of EPUB reads a book, and no screen shows it yet. The\n\
       part that reads the file refuses a book that is too large, a\n\
       chapter that is too large, and every one of twelve files that\n\
       attack the program.\n\
     - macOS has a way to remove the program with no binary:\n\
       macos/uninstall.sh. It deletes nothing, and it writes the paths\n\
       and the commands.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_19 = "Changelog Toutui v0.6.3 (11/08/2026) \n\
     \n\
     Added:\n\
     - **Read an EPUB book in the terminal.** The key e on an item that\n\
       holds an ebook opens the reader. The keys: j/k a line, Space/b a\n\
       page, n/p a chapter, t the table of contents, g/G the start and\n\
       the end, s sends the place to the server, and h leaves the book.\n\
     - The program keeps the file of the book, therefore the reader also\n\
       works with no server.\n\
     - The place of the reader goes to the field of the ebook of the\n\
       server. It changes no position of the audio.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_20 = "Changelog Toutui v0.6.4 (11/08/2026) \n\
     \n\
     Added:\n\
     - The reader opens a book where you stopped. The place comes from\n\
       the server, therefore a different machine gives the same place.\n\
     - The reader sends the place by itself: when the place changed and\n\
       30 seconds went by, and when you leave the book with h.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_21 = "Changelog Toutui v0.6.5 (11/08/2026) \n\
     \n\
     Fixed:\n\
     - **A playback that does not start no longer loses your place.**\n\
       rodio gives the position 0 until the seek finishes, and a playback\n\
       that never starts gives 0 for the whole wait. The program wrote\n\
       that 0 on the disk every second, and it gave that 0 to the server\n\
       when the session closed. The book then started at the beginning.\n\
       The program now writes nothing until the engine reaches the place\n\
       where the playback starts.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n"
        .to_string();

    let changelog_22 = format!(
        "Changelog Toutui v{} (11/08/2026) \n\
     \n\
     Added:\n\
     - **The search asks the server.** The program looked in the titles\n\
       that it holds, therefore the name of an author found nothing. The\n\
       server also finds an author, a series, a narrator, a tag, and a\n\
       genre. The screen shows the titles at once, and the answer of the\n\
       server when it comes. The title of the list says where the answer\n\
       comes from.\n\
     - The reader follows the web reader of Audiobookshelf. It reads the\n\
       chapter out of an EPUBCFI, therefore you find the correct chapter\n\
       when you read in the web page and then in the terminal.\n\
     - A log out asks one time. Press l a second time to log out, and any\n\
       other key stops the question.\n\
     - docs/T-24-coverage.md compares this program with the server,\n\
       function by function.\n\
     \n\
     Fixed:\n\
     - The Home view matched its shelf on a name for a person. A server\n\
       that gives that name in a different language gave an empty Home\n\
       view, with no error. The program matches the identity now.\n\
     - The list of the accounts no longer moves past its last line.\n\
     \n\
     Contributors:\n\
     \n\
     - AlbanDAVID (the original project), ealtun21\n\
     \n\
     Enjoy and be toutui!\n\
     ####\n",
        VERSION
    );
    let changelog_13 = "Changelog Toutui v0.4.1-beta (14/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app v0.3.5 or bellow, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Archlinux users: the app is now available in the AUR (yay -S toutui)

         Changed:\n\
         - Minor changes in the installation process.

         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();
    let changelog_14 =
    "Changelog Toutui v0.4.2-beta (15/05/2025) \n\
         \n\
         Warning:\n\
         - If you're already using the app v0.3.5 or bellow, please follow the upgrade instructions here: => 
         https://github.com/AlbanDAVID/Toutui/wiki/Major-upgrade-instruction#v--035-beta-to-v040-beta

         Added:\n\
         - Verifying file integrity using SHA-256 before installation via curl script

         Changed:\n\
         - Clarification of update/uninstall instructions

         \n\
         Contributors:\n\
         \n\
         - AlbanDAVID
         \n\
         Enjoy and be toutui!\n
         ####\n".to_string();

    changelog.push_str(&changelog_22);
    changelog.push_str(&changelog_21);
    changelog.push_str(&changelog_20);
    changelog.push_str(&changelog_19);
    changelog.push_str(&changelog_18);
    changelog.push_str(&changelog_17);
    changelog.push_str(&changelog_16);
    changelog.push_str(&changelog_15);
    changelog.push_str(&changelog_14);
    changelog.push_str(&changelog_13);
    changelog.push_str(&changelog_12);
    changelog.push_str(&changelog_11);
    changelog.push_str(&changelog_10);
    changelog.push_str(&changelog_09);
    changelog.push_str(&changelog_08);
    changelog.push_str(&changelog_07);
    changelog.push_str(&changelog_06);
    changelog.push_str(&changelog_05);
    changelog.push_str(&changelog_04);
    changelog.push_str(&changelog_03);
    changelog.push_str(&changelog_02);
    changelog.push_str(&changelog_01);

    changelog
}
