//! The words of the key `z` name the three panels, and they say no `1 to 3`.
//! See T-330, the part 1.
//!
//! **The fault, of the report of the maintainer of 2026-08-16.** The user hid
//! the panels 1, 2, and 3 with the key `z`, and every word of the program then
//! said `1 to 3`. **`1 to 3` reads as `1 and 3`**: the maintainer read that the
//! panel 2 stood, and they looked for it on a screen that did not hold it.
//!
//! **The measurement of the real program v0.8.159 inside tmux**, at 160 columns
//! and 45 rows, of the Home view of the library `Large` of the sandbox, after
//! the key `z`:
//!
//! ```text
//!                     The panels 1 to 3 are hidden. Press the key z for them.
//!   j/k: move  …  Q: quit  f: sequence  z: the panels 1 to 3
//!   z               Hide the panels 1 to 3, and show them again
//! ```
//!
//! **The correction.** A list of three names takes two commas and the word
//! `and`, at each of the four places: the footer of the panel 4 of the list
//! (`src/ui/keys.rs`), the line of the key `z` of the view of the key `?`
//! (`src/ui/keys.rs`), the two messages of the press of the key
//! (`src/app.rs`), and the entry of the release that named the key
//! (`src/utils/changelog.rs`).
//!
//! **This gate is a sweep of every word for the user.** The four places above
//! stand in three files, and a fifth place of a later round would say `1 to 3`
//! again with no gate to stop it: this test therefore reads the three files
//! and it holds that no word of the panels of the stack says `1 to 3` at all.

/// The three files that hold a word for the user of the panels of the stack.
const THE_FILES_OF_THE_WORDS: &[&str] = &["src/ui/keys.rs", "src/app.rs", "src/utils/changelog.rs"];

/// No word for the user of this program says `panels 1 to 3`.
///
/// **The parts of this test stay in one function.**
#[test]
fn no_word_of_the_program_says_the_panels_1_to_3() {
    for name in THE_FILES_OF_THE_WORDS {
        let of_the_file = std::fs::read_to_string(name).unwrap_or_else(|_| panic!("{name}"));

        for (number, line) in of_the_file.lines().enumerate() {
            assert!(
                !line.contains("panels 1 to 3"),
                "{name}:{} says `panels 1 to 3`, and a list of three names takes \
                 two commas and the word `and` (T-330): {line}",
                number + 1
            );
        }
    }
}

/// The footer of the panel 4 of the list, with the stack away, names the three
/// panels.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_footer_of_the_list_names_the_three_panels() {
    use toutui::ui::frame::ThePanel;
    use toutui::ui::keys::the_footer_of_a_panel;

    let of_the_view = "j/k: move  l: play or open  Q: quit";

    let with_no_stack = the_footer_of_a_panel(of_the_view, true, false, ThePanel::TheList, true);
    assert!(
        with_no_stack.ends_with("z: the panels 1, 2, and 3"),
        "the footer of the mode that hides the stack names the three panels: {with_no_stack}"
    );

    // **The footer of the mode that keeps the stack says no number at all**: the
    // key `z` there hides the panels that the digit `1` names beside it, and the
    // words `hide them` therefore hold the three of them with no list. The
    // correction of T-330 must not write a list of three names into this row.
    let with_the_stack = the_footer_of_a_panel(of_the_view, true, true, ThePanel::TheList, true);
    assert!(with_the_stack.ends_with("1/Ctrl+h: the panels  z: hide them"));
}

/// The line of the key `z` of the view of the key `?` names the three panels.
#[test]
fn the_view_of_every_key_names_the_three_panels() {
    let of_the_panels = toutui::ui::keys::GROUPS
        .iter()
        .find(|group| group.name.starts_with("The panels"))
        .expect("the group of the panels of the frame stands");

    let the_key = of_the_panels
        .keys
        .iter()
        .find(|key| key.key == "z")
        .expect("the group of the panels names the key z");

    assert_eq!(
        the_key.what,
        "Hide the panels 1, 2, and 3, and show them again"
    );
}

/// The entry of the changelog of the release that gave the key `z` names the
/// three panels.
///
/// **The changelog is a word for the user of every release**, and the user of
/// today reads the entry of v0.8.153 in the settings screen: an entry that says
/// `1 to 3` holds the fault of this item for every reader of it.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_entry_of_the_changelog_names_the_three_panels() {
    let of_the_changelog =
        std::fs::read_to_string("src/utils/changelog.rs").expect("src/utils/changelog.rs");

    // The words of the entry hold a `\` of the end of a line of Rust, therefore
    // the source of them holds no whole sentence: the two marks below are the
    // parts that stay on one line.
    assert!(of_the_changelog.contains("The key `z` hides the panels 1, 2, and 3"));
    assert!(of_the_changelog.contains("the footer says `z: the panels 1, 2, and 3`"));
}
