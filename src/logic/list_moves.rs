//! The moves inside a list that holds a title. See T-24.
//!
//! The Home view and the view of the sequence hold a line that names a group.
//! That line is a title, and the user cannot select it. These functions give
//! the next line that the user can select.
//!
//! The caller gives one value for each line: `true` for a line of the user,
//! and `false` for a title. The functions are pure.

/// Gives the first line that the user can select.
pub fn first(lines: &[bool]) -> Option<usize> {
    lines.iter().position(|one| *one)
}

/// Gives the last line that the user can select.
pub fn last(lines: &[bool]) -> Option<usize> {
    lines.iter().rposition(|one| *one)
}

/// Gives the line after this one. The move goes to the first line at the end.
pub fn next(lines: &[bool], from: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(from + 1)
        .find(|(_, one)| **one)
        .map(|(index, _)| index)
        .or_else(|| first(lines))
}

/// Gives the line before this one. The move stops at the first line.
pub fn previous(lines: &[bool], from: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .take(from)
        .rfind(|(_, one)| **one)
        .map(|(index, _)| index)
        .or_else(|| first(lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A title, two lines, a title, and one line.
    const LINES: [bool; 5] = [false, true, true, false, true];

    #[test]
    fn the_move_goes_over_a_title() {
        assert_eq!(first(&LINES), Some(1));
        assert_eq!(last(&LINES), Some(4));
        assert_eq!(next(&LINES, 1), Some(2));
        assert_eq!(next(&LINES, 2), Some(4));
        assert_eq!(previous(&LINES, 4), Some(2));
    }

    #[test]
    fn the_move_goes_back_to_the_first_line_at_the_end() {
        assert_eq!(next(&LINES, 4), Some(1));
    }

    #[test]
    fn the_move_stops_at_the_first_line() {
        assert_eq!(previous(&LINES, 1), Some(1));
        assert_eq!(previous(&LINES, 0), Some(1));
    }

    #[test]
    fn a_list_of_titles_only_gives_no_line() {
        let titles = [false, false];

        assert_eq!(first(&titles), None);
        assert_eq!(last(&titles), None);
        assert_eq!(next(&titles, 0), None);
        assert_eq!(previous(&titles, 1), None);
    }

    #[test]
    fn an_empty_list_gives_no_line_and_no_fault() {
        assert_eq!(first(&[]), None);
        assert_eq!(last(&[]), None);
        assert_eq!(next(&[], 0), None);
        assert_eq!(previous(&[], 0), None);
    }

    /// A number that stands after the last line must not stop the program.
    #[test]
    fn a_number_that_is_too_large_gives_a_line() {
        assert_eq!(next(&LINES, 900), Some(1));
        assert_eq!(previous(&LINES, 900), Some(4));
    }
}
