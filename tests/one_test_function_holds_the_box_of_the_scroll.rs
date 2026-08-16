//! One test function of `the_scroll_of_a_panel` holds the box of the process.
//! See T-332.
//!
//! **The gate of CI is `cargo test` and it is a different run** (T-144 and
//! T-157): nextest gives each test a process of its own, therefore it hides a
//! test that shares a box of the process with another test of its binary.
//!
//! **The fault.** `src/logic/the_scroll_of_a_panel.rs` holds the static
//! `THE_LAST_SCROLL`, which `the_panel_of_the_render` writes and which
//! `the_scroll_after_one_step_down` reads. That module held **three** test
//! functions, and each of the three called `the_panel_of_the_render`: `cargo
//! test` runs them on three threads, therefore the render of one test took the
//! box that another test read.
//!
//! The measurement of 2026-08-17, of the tree of v0.8.161 with no change at
//! all, five runs of `cargo test -j 16 --lib the_scroll_of_a_panel`:
//!
//! ```text
//! test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured
//! test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured
//! test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured
//! test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured
//! test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured
//! ```
//!
//! `cargo nextest run` gave 1490 of 1490 in the same tree.
//!
//! **The correction** joins the three test functions into one, which is the
//! rule of `docs/HANDOVER.md`: "A box of the process needs one test function."
//! This file is the gate of that rule for this module, because a later round
//! that writes a second test function of the box brings the fault back with no
//! word of a reason.

use std::path::Path;

/// The names of the functions of the module that touch the box of the process.
///
/// **A test that reads the source with a window of a number of characters is a
/// test of the comments of that function** (the trap 209), therefore this
/// function reads whole functions: it takes the text after `mod tests`, it
/// splits it at each `    fn `, it **ends each part at the `    }` of that
/// function**, and it gives the name of every part whose body names the box.
///
/// **A part that runs to the function after it holds the doc comment of that
/// function** (the first form of this gate): the comment of the merged test
/// names `the_panel_of_the_render`, and every test before it then read as a
/// test of the box.
fn the_test_functions_of_the_box(source: &str) -> Vec<String> {
    let Some((_, of_the_tests)) = source.split_once("mod tests {") else {
        panic!("the module holds no test module at all");
    };

    of_the_tests
        .split("\n    fn ")
        .skip(1)
        .map(|part| {
            let name = part
                .split('(')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            let body = part.split("\n    }").next().unwrap_or_default().to_string();

            (name, body)
        })
        .filter(|(_, body)| {
            body.contains("the_panel_of_the_render") || body.contains("keep_the_last_scroll")
        })
        .map(|(name, _)| name)
        .collect()
}

/// One test function of the module holds the box of the process.
///
/// **The parts of this test stay in one function.**
#[test]
fn one_test_function_holds_the_box_of_the_scroll() {
    let path = Path::new("src/logic/the_scroll_of_a_panel.rs");
    let source = std::fs::read_to_string(path).expect("the module of the scroll stands");

    let of_the_box = the_test_functions_of_the_box(&source);

    assert_eq!(
        of_the_box.len(),
        1,
        "the box of the process needs one test function, and these hold it: {of_the_box:?}"
    );

    // The words of the rule stand in the module too, therefore a reader of that
    // file meets it before they write a second test function.
    let (_, of_the_tests) = source
        .split_once("mod tests {")
        .expect("the module holds a test module");
    assert!(
        of_the_tests.contains("The parts of this test stay in one function"),
        "the test of the box says the rule of the box"
    );
}

/// The function of this file reads whole functions, and not a window of the
/// characters of a file.
///
/// **A gate of a source must fail for the fault that it names** (the trap 147
/// and the trap 209): a text of two test functions of the box must give two
/// names, and a comment that names the box must give none.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_sweep_of_this_gate_reads_whole_functions() {
    let of_two = "\
mod tests {
    /// A comment that names the_panel_of_the_render and nothing more.
    #[test]
    fn the_first_of_them() {
        the_panel_of_the_render(0, \"a\", 80, 4);
    }

    #[test]
    fn the_second_of_them() {
        keep_the_last_scroll(3);
    }
}
";

    assert_eq!(
        the_test_functions_of_the_box(of_two),
        vec!["the_first_of_them", "the_second_of_them"]
    );

    let of_none = "\
mod tests {
    #[test]
    fn a_test_of_no_box() {
        assert_eq!(1, 1);
    }
}
";

    assert!(the_test_functions_of_the_box(of_none).is_empty());
}
