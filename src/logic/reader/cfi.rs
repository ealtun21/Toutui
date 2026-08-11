//! The place of the reader in the form of the web reader (EPUBCFI). See T-10.
//!
//! Audiobookshelf keeps the place of an ebook in `ebookLocation`. The web
//! reader writes an EPUBCFI there, for example
//! `epubcfi(/6/14[id6]!/4/2/2/2[c1]/2/1:0)`. Toutui wrote `toutui:<spine>:<line>`
//! before this module, therefore a user who read on the telephone and then in
//! the terminal found the chapter and not the line.
//!
//! This module makes the whole path, and it reads the whole path. It has two
//! directions:
//!
//! - [`to_epubcfi`] takes the number of letters that stand before the place of
//!   the user, and it gives the text of an EPUBCFI.
//! - [`parse_epubcfi`] takes such a text, and [`letters_before`] gives the
//!   number of letters that stand before that place.
//!
//! ## Why the module counts letters
//!
//! The screen holds lines, and an EPUBCFI holds a path in the tree of the
//! XHTML. The two forms have no common unit, because `html2text` makes the
//! lines: it joins the spaces, it breaks the words at the width of the screen,
//! and it writes `* ` before each item of a list.
//!
//! **A letter is the common unit.** A letter of the XHTML is a letter of the
//! screen, and the sequence is the same. The module therefore counts the
//! letters, and it counts nothing else: no space, no digit, and no mark. A
//! digit stays outside because `html2text` writes the number of each item of an
//! ordered list, and the XHTML does not hold that number.
//!
//! The count is near, and it is not exact. The alternative text of a picture is
//! an attribute of the XHTML and a text of the screen, therefore this module
//! counts that attribute as a text. A book that holds a form that one part
//! shows and the other part does not show moves the two counts apart. The error
//! is small, and the user finds their paragraph.
//!
//! ## The numbers of an EPUBCFI
//!
//! EPUBCFI gives an even number to each element and an odd number to the text
//! between two elements.
//!
//! - The element number `n` of a parent, counted from 1, has the step `2n`.
//! - The text after the element `n`, and before the element `n + 1`, has the
//!   step `2n + 1`. The text before the first element has the step 1.
//! - Two texts beside each other are one text. A comment between them changes
//!   nothing.
//!
//! The part before the `!` names the file in the package document: `/6` is the
//! spine, and the step after it is the itemref. The part after the `!` starts
//! at the children of the root element `<html>`. Therefore `!/4` is `<body>`,
//! because `<head>` is the first element and `<body>` is the second.
//!
//! The number after the `:` counts the code units of UTF-16 inside the text,
//! because a web page counts them. A text of the Latin alphabet gives the same
//! number for a character and for a code unit.
//!
//! ## A known difference with `epub.js`
//!
//! The web reader of Audiobookshelf uses `epub.js`, and `epub.js` gives the
//! step of a text with a different rule: it counts the texts only, and it does
//! not count the elements between them. The two rules agree for `<p>a text</p>`,
//! and they disagree for `<p><b>A</b>: a text</p>`. This module gives the step 3
//! to that text, because the specification of EPUBCFI gives the step 3.
//! `epub.js` gives the step 1.
//!
//! **A measurement on 2026-08-11 counted this difference.** The four books of
//! the survey hold 11343 texts with a letter, and the two rules disagree for
//! 296 of them, therefore for 2.61 per cent.
//!
//! The difference costs the user little, because [`letters_before`] takes the
//! first text that follows a path that it does not find. A text with the step 1
//! that is absent therefore gives the text with the step 3 of the same
//! paragraph. **The user loses the place inside the paragraph, and never the
//! paragraph.** See `a_place_of_epub_js_gives_the_same_paragraph`.
//!
//! A session with a web page and a real web reader must measure this. This
//! session had no browser.
//!
//! Every function here is pure. A test needs no file and no server.

use quick_xml::events::Event;

/// The largest XHTML that this module reads, in bytes. 8 megabytes.
///
/// `book::chapter_xhtml` and `render::to_lines` hold the same limit.
pub const MAX_XHTML_BYTES: usize = 8 * 1024 * 1024;

/// The largest number of texts that the module keeps for one chapter.
///
/// A chapter of Moby Dick gives some hundreds. A file with a million tags must
/// not take the memory of the machine.
pub const MAX_PLACES: usize = 100_000;

/// The elements whose text the screen never shows.
///
/// `html2text` shows the text of `<body>` only. This module must count the same
/// letters, therefore it steps over these elements and over their children.
const SILENT: [&str; 4] = ["head", "script", "style", "title"];

/// One text of a chapter, with the path of the EPUBCFI that names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPlace {
    /// The steps after the `!`, for example `[4, 2, 1]`.
    ///
    /// The last step is odd for a text. It is even for the alternative text of
    /// a picture, because that text is an attribute of an element.
    pub steps: Vec<usize>,
    /// The text of the place.
    pub text: String,
}

/// The place that an EPUBCFI names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfiPlace {
    /// The chapter of the spine. The first chapter is 0.
    pub spine: usize,
    /// The steps after the `!`. An EPUBCFI with no `!` gives no step.
    pub steps: Vec<usize>,
    /// The code units of UTF-16 before the place, inside the text.
    pub offset: usize,
}

/// The number of letters of a text.
///
/// A digit, a space, and a mark all count 0. See the head of this module.
pub fn letters(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

/// The state of the walk over the tree of one chapter.
struct Walk {
    places: Vec<TextPlace>,
    /// The number of element children that each open element has now. The last
    /// value belongs to the element that is open.
    seen: Vec<usize>,
    /// The steps from the root element to the element that is open.
    path: Vec<usize>,
    /// The text that the walk builds now, and the steps of that text.
    pending: String,
    pending_steps: Vec<usize>,
    /// The depth of the first element whose text the screen never shows.
    silent_at: Option<usize>,
}

impl Walk {
    fn new() -> Walk {
        Walk {
            places: Vec::new(),
            seen: Vec::new(),
            path: Vec::new(),
            pending: String::new(),
            pending_steps: Vec::new(),
            silent_at: None,
        }
    }

    /// Puts the text that the walk holds into the list.
    ///
    /// A text with no letter gives no place: such a text is the space between
    /// two tags, and no EPUBCFI of this module points at it.
    fn flush(&mut self) {
        if !self.pending.is_empty() && letters(&self.pending) > 0 && self.places.len() < MAX_PLACES
        {
            self.places.push(TextPlace {
                steps: std::mem::take(&mut self.pending_steps),
                text: std::mem::take(&mut self.pending),
            });
        }
        self.pending.clear();
        self.pending_steps.clear();
    }

    /// Adds a part of a text to the text that the walk holds.
    fn text(&mut self, part: &str) {
        if self.seen.is_empty() || self.silent_at.is_some() || part.is_empty() {
            return;
        }
        let step = 2 * self.seen.last().copied().unwrap_or(0) + 1;
        let mut steps = self.path.clone();
        steps.push(step);
        if self.pending.is_empty() {
            self.pending_steps = steps;
        } else if self.pending_steps != steps {
            self.flush();
            self.pending_steps = steps;
        }
        self.pending.push_str(part);
    }

    /// Counts one element child of the element that is open, and gives its
    /// step.
    fn count_the_child(&mut self) -> usize {
        match self.seen.last_mut() {
            Some(count) => {
                *count += 1;
                2 * *count
            }
            None => 0,
        }
    }

    /// The alternative text of a picture.
    ///
    /// `html2text` writes that text on the screen, therefore the count of the
    /// letters must hold it. The steps name the element itself, and the last
    /// step is therefore even.
    fn alternative_text(&mut self, step: usize, alt: &str) {
        if self.silent_at.is_some() || letters(alt) == 0 || self.places.len() >= MAX_PLACES {
            return;
        }
        let mut steps = self.path.clone();
        steps.push(step);
        self.places.push(TextPlace {
            steps,
            text: alt.to_string(),
        });
    }
}

/// The name of an element, with no namespace, in small letters.
fn name_of(tag: &quick_xml::events::BytesStart<'_>) -> String {
    let raw = tag.name();
    let local = raw.local_name();
    String::from_utf8_lossy(local.as_ref()).to_lowercase()
}

/// The value of the attribute `alt` of an element.
fn alt_of(tag: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attribute in tag.attributes().flatten() {
        if attribute.key.local_name().as_ref() == b"alt" {
            let raw = String::from_utf8_lossy(attribute.value.as_ref()).into_owned();
            return Some(match quick_xml::escape::unescape(&raw) {
                Ok(value) => value.into_owned(),
                // A value with a reference that this crate does not know keeps
                // its own form. The count of the letters is then near, and the
                // program does not fail.
                Err(_) => raw,
            });
        }
    }
    None
}

/// Reads the XHTML of one chapter and gives every text, in the sequence of the
/// document.
///
/// The function never fails. An XHTML that is damaged gives the places that
/// stand before the fault. An XHTML that is larger than [`MAX_XHTML_BYTES`]
/// gives no place.
pub fn text_places(xhtml: &str) -> Vec<TextPlace> {
    if xhtml.len() > MAX_XHTML_BYTES {
        return Vec::new();
    }

    let mut reader = quick_xml::Reader::from_str(xhtml);
    let config = reader.config_mut();
    // A book can come from any source. The reader must give the text of a file
    // that is not correct XML, and it must never stop the program.
    config.check_end_names = false;
    config.allow_unmatched_ends = true;
    config.allow_dangling_amp = true;
    config.check_comments = false;

    let mut walk = Walk::new();

    loop {
        match reader.read_event() {
            Err(_) | Ok(Event::Eof) => break,

            Ok(Event::Start(tag)) => {
                let name = name_of(&tag);
                if walk.seen.is_empty() {
                    // The root element. Its own step stands before the `!`, and
                    // the steps of this module start at its children.
                    walk.seen.push(0);
                    if SILENT.contains(&name.as_str()) {
                        walk.silent_at = Some(0);
                    }
                    continue;
                }
                walk.flush();
                let step = walk.count_the_child();
                if name == "img" {
                    if let Some(alt) = alt_of(&tag) {
                        walk.alternative_text(step, &alt);
                    }
                }
                walk.path.push(step);
                walk.seen.push(0);
                if walk.silent_at.is_none() && SILENT.contains(&name.as_str()) {
                    walk.silent_at = Some(walk.seen.len() - 1);
                }
            }

            Ok(Event::Empty(tag)) => {
                if walk.seen.is_empty() {
                    // A root element with no child. The chapter holds no text.
                    walk.seen.push(0);
                    continue;
                }
                walk.flush();
                let step = walk.count_the_child();
                if name_of(&tag) == "img" {
                    if let Some(alt) = alt_of(&tag) {
                        walk.alternative_text(step, &alt);
                    }
                }
            }

            Ok(Event::End(_)) => {
                walk.flush();
                if walk.seen.len() > 1 {
                    walk.path.pop();
                }
                walk.seen.pop();
                if let Some(depth) = walk.silent_at {
                    if walk.seen.len() <= depth {
                        walk.silent_at = None;
                    }
                }
            }

            Ok(Event::Text(text)) => {
                if let Ok(value) = text.xml_content(quick_xml::XmlVersion::Implicit1_0) {
                    walk.text(&value);
                }
            }

            Ok(Event::CData(data)) => {
                let value = String::from_utf8_lossy(data.as_ref()).into_owned();
                walk.text(&value);
            }

            Ok(Event::GeneralRef(reference)) => {
                // A named reference of HTML, for example `&nbsp;`, is one
                // character of the page. The module puts one space there: the
                // count of the letters does not change, and the count of the
                // characters stays correct.
                match reference.resolve_char_ref() {
                    Ok(Some(character)) => {
                        let mut buffer = [0u8; 4];
                        walk.text(character.encode_utf8(&mut buffer));
                    }
                    _ => walk.text(" "),
                }
            }

            _ => {}
        }

        if walk.places.len() >= MAX_PLACES {
            break;
        }
    }

    walk.flush();
    walk.places
}

/// The place in a text after a given number of letters, in code units of
/// UTF-16.
fn offset_of_letters(text: &str, need: usize) -> usize {
    let mut counted = 0usize;
    let mut offset = 0usize;
    for character in text.chars() {
        if counted == need {
            return offset;
        }
        if character.is_alphabetic() {
            counted += 1;
        }
        offset += character.len_utf16();
    }
    offset
}

/// The number of letters of a text before a place, in code units of UTF-16.
fn letters_before_offset(text: &str, offset: usize) -> usize {
    let mut units = 0usize;
    let mut counted = 0usize;
    for character in text.chars() {
        if units >= offset {
            break;
        }
        if character.is_alphabetic() {
            counted += 1;
        }
        units += character.len_utf16();
    }
    counted
}

/// Makes the text of an EPUBCFI for a place of a chapter.
///
/// `spine` is the number of the chapter, and the first chapter is 0.
/// `target` is the number of letters that stand before the place of the user.
///
/// A chapter with no text gives `None`, and the caller must then write its own
/// form of the place.
pub fn to_epubcfi(spine: usize, places: &[TextPlace], target: usize) -> Option<String> {
    let (index, offset) = place_of_letters(places, target)?;
    let place = places.get(index)?;

    let mut path = String::new();
    for step in &place.steps {
        path.push('/');
        path.push_str(&step.to_string());
    }

    // The step of the itemref of the spine. The first chapter is the first
    // itemref, and EPUBCFI counts the elements from 2.
    let itemref = spine.checked_add(1)?.checked_mul(2)?;

    // A step that is even names an element and not a text. The alternative text
    // of a picture has such a step, and an element takes no number after a `:`.
    let end = match place.steps.last() {
        Some(step) if step % 2 == 0 => String::new(),
        _ => format!(":{offset}"),
    };

    Some(format!("epubcfi(/6/{itemref}!{path}{end})"))
}

/// The text of a chapter that holds a given number of letters, and the place
/// inside that text.
fn place_of_letters(places: &[TextPlace], target: usize) -> Option<(usize, usize)> {
    let mut before = 0usize;
    for (index, place) in places.iter().enumerate() {
        let count = letters(&place.text);
        if target < before + count {
            return Some((index, offset_of_letters(&place.text, target - before)));
        }
        before += count;
    }
    // The place stands after the last letter of the chapter. The user then
    // stands at the end of the last text.
    let last = places.len().checked_sub(1)?;
    let text = &places[last].text;
    Some((last, text.chars().map(char::len_utf16).sum()))
}

/// Reads the text of an EPUBCFI.
///
/// The function never fails. A text that is not an EPUBCFI gives `None`, and so
/// does an EPUBCFI whose step of the spine names no itemref.
pub fn parse_epubcfi(text: &str) -> Option<CfiPlace> {
    let inside = text
        .trim()
        .strip_prefix("epubcfi(")
        .and_then(|rest| rest.strip_suffix(')'))?;

    // A range has the form `parent,start,end`. The place of the user is the
    // start of the range, and that is the parent with the first part after it.
    let mut parts = inside.split(',');
    let parent = parts.next()?;
    let start = parts.next().unwrap_or("");
    let whole = format!("{parent}{start}");

    // The part before the first `!` names the file in the package document.
    let mut halves = whole.split('!');
    let package = halves.next()?;
    let local = halves.next().unwrap_or("");

    let spine = spine_of(package)?;
    let (steps, offset) = steps_of(local);

    Some(CfiPlace {
        spine,
        steps,
        offset,
    })
}

/// The chapter of the spine that the part before the `!` names.
fn spine_of(package: &str) -> Option<usize> {
    // The last step of that part is the itemref of the spine. A step can carry
    // an assertion in brackets, for example `14[id6]`.
    let last = package.split('/').rfind(|step| !step.is_empty())?;
    let number: usize = last.split('[').next()?.trim().parse().ok()?;

    // An odd number names a text and not an element. Such a value names no
    // itemref.
    if number < 2 || !number.is_multiple_of(2) {
        return None;
    }
    Some(number / 2 - 1)
}

/// The steps and the place inside the text, of the part after the `!`.
fn steps_of(local: &str) -> (Vec<usize>, usize) {
    let mut steps = Vec::new();
    let mut offset = 0usize;

    for part in local.split('/') {
        if part.is_empty() {
            continue;
        }
        // A step can carry an assertion in brackets, and the last step can
        // carry a number after a `:`.
        let without_assertion = part.split('[').next().unwrap_or(part);
        let mut halves = without_assertion.split(':');
        let number = halves.next().unwrap_or("").trim();
        if let Ok(step) = number.parse::<usize>() {
            steps.push(step);
        }
        if let Some(after) = halves.next() {
            offset = after.trim().parse().unwrap_or(0);
        }
    }

    (steps, offset)
}

/// The number of letters of a chapter that stand before a place of an EPUBCFI.
///
/// A place that names no text of the chapter gives the letters before the first
/// text that follows it. A place after the last text gives every letter of the
/// chapter.
pub fn letters_before(places: &[TextPlace], place: &CfiPlace) -> usize {
    let mut before = 0usize;
    for text_place in places {
        if text_place.steps >= place.steps {
            return if text_place.steps == place.steps {
                before + letters_before_offset(&text_place.text, place.offset)
            } else {
                before
            };
        }
        before += letters(&text_place.text);
    }
    before
}

/// The number of letters of a chapter that stand before a line of the screen.
pub fn letters_before_line(letters_of_each_line: &[usize], line: usize) -> usize {
    letters_of_each_line
        .iter()
        .take(line)
        .copied()
        .fold(0usize, |sum, count| sum.saturating_add(count))
}

/// The line of the screen that holds a given number of letters.
///
/// A chapter with no line gives the line 0.
pub fn line_of_letters(letters_of_each_line: &[usize], target: usize) -> usize {
    let mut before = 0usize;
    for (line, count) in letters_of_each_line.iter().enumerate() {
        if target < before + count {
            return line;
        }
        before += count;
    }
    // The place stands after the last letter. The last line holds it.
    letters_of_each_line.len().saturating_sub(1)
}

#[cfg(test)]
mod tests_of_the_walk {
    use super::*;

    fn steps_and_text(xhtml: &str) -> Vec<(Vec<usize>, String)> {
        text_places(xhtml)
            .into_iter()
            .map(|place| (place.steps, place.text.trim().to_string()))
            .collect()
    }

    #[test]
    fn the_text_of_a_simple_page_stands_in_the_body() {
        // `<head>` is the element 1 and `<body>` is the element 2, therefore
        // the body has the step 4. The `<p>` is the first element of the body,
        // therefore it has the step 2. The text is the first text of the `<p>`,
        // therefore it has the step 1.
        let places = steps_and_text("<html><head/><body><p>Hello reader</p></body></html>");
        assert_eq!(vec![(vec![4, 2, 1], "Hello reader".to_string())], places);
    }

    #[test]
    fn a_body_that_is_the_first_element_has_the_step_2() {
        let places = steps_and_text("<html><body><p>one</p></body></html>");
        assert_eq!(vec![(vec![2, 2, 1], "one".to_string())], places);
    }

    #[test]
    fn each_element_of_the_body_takes_the_next_even_step() {
        let places = steps_and_text("<html><body><p>one</p><p>two</p><p>three</p></body></html>");
        assert_eq!(
            vec![
                (vec![2, 2, 1], "one".to_string()),
                (vec![2, 4, 1], "two".to_string()),
                (vec![2, 6, 1], "three".to_string()),
            ],
            places
        );
    }

    #[test]
    fn a_text_after_an_element_takes_the_next_odd_step() {
        // `alpha` stands before the first element, therefore it has the step 1.
        // `bravo` stands after the first element, therefore it has the step 3.
        let places = steps_and_text("<html><body>alpha<em>x</em>bravo</body></html>");
        assert_eq!(
            vec![
                (vec![2, 1], "alpha".to_string()),
                (vec![2, 2, 1], "x".to_string()),
                (vec![2, 3], "bravo".to_string()),
            ],
            places
        );
    }

    #[test]
    fn a_comment_between_two_texts_makes_one_text() {
        let places = steps_and_text("<html><body>one<!-- a note -->two</body></html>");
        assert_eq!(vec![(vec![2, 1], "onetwo".to_string())], places);
    }

    #[test]
    fn the_head_and_the_title_give_no_text() {
        // The screen never shows them, therefore the count of the letters must
        // not hold them.
        let places = steps_and_text(
            "<html><head><title>The title</title><style>p{color:red}</style></head>\
             <body><script>var x = 1;</script><p>the text</p></body></html>",
        );
        assert_eq!(vec![(vec![4, 4, 1], "the text".to_string())], places);
    }

    #[test]
    fn the_alternative_text_of_a_picture_counts_as_a_text() {
        // `html2text` writes that text on the screen. Its step is even, because
        // it belongs to the element and not to a text.
        let places = steps_and_text(
            "<html><body><p>before</p><img src=\"a.png\" alt=\"a horse\"/><p>after</p></body></html>",
        );
        assert_eq!(
            vec![
                (vec![2, 2, 1], "before".to_string()),
                (vec![2, 4], "a horse".to_string()),
                (vec![2, 6, 1], "after".to_string()),
            ],
            places
        );
    }

    #[test]
    fn a_space_between_two_tags_gives_no_place() {
        let places = steps_and_text("<html>\n  <body>\n    <p>one</p>\n  </body>\n</html>");
        assert_eq!(vec![(vec![2, 2, 1], "one".to_string())], places);
    }

    #[test]
    fn an_xhtml_that_is_damaged_gives_the_text_before_the_fault() {
        let places = steps_and_text("<html><body><p>one</p><p>two");
        assert!(places.iter().any(|(_, text)| text == "one"));
    }

    #[test]
    fn a_text_that_is_not_an_xhtml_gives_no_panic() {
        let rubbish: String = (0..=255u8)
            .map(|byte| byte as char)
            .collect::<String>()
            .repeat(100);
        let _ = text_places(&rubbish);
        let _ = text_places("");
        let _ = text_places("<<<>>>&&&");
    }

    #[test]
    fn it_refuses_an_xhtml_that_is_too_large() {
        let big = "a".repeat(MAX_XHTML_BYTES + 1);
        assert!(text_places(&big).is_empty());
    }
}

#[cfg(test)]
mod tests_of_the_text {
    use super::*;

    #[test]
    fn a_letter_counts_and_nothing_else_counts() {
        assert_eq!(5, letters("Hello"));
        assert_eq!(0, letters("12 34 -- ,.!"));
        assert_eq!(7, letters("The 3 cats"));
        // A letter of a different alphabet also counts.
        assert_eq!(5, letters("Ünïçø"));
        assert_eq!(3, letters("日本語"));
    }

    #[test]
    fn the_place_of_a_number_of_letters_steps_over_the_marks() {
        // "a, b" has the letters `a` and `b`. The place before `b` is 3.
        assert_eq!(0, offset_of_letters("a, b", 0));
        assert_eq!(1, offset_of_letters("a, b", 1));
        assert_eq!(4, offset_of_letters("a, b", 2));
    }

    #[test]
    fn the_two_directions_of_a_text_agree() {
        let text = "One, two. 3 three!";
        for need in 0..=letters(text) {
            let offset = offset_of_letters(text, need);
            assert_eq!(need, letters_before_offset(text, offset), "{need}");
        }
    }

    #[test]
    fn the_place_counts_the_code_units_of_utf16() {
        // A character outside the first plane takes two code units. A web page
        // counts the same two.
        let text = "\u{1F600}ab";
        assert_eq!(0, offset_of_letters(text, 0));
        // The letter `a` stands after the two code units of the first
        // character, therefore the place after it is 3 and not 2.
        assert_eq!(3, offset_of_letters(text, 1));
        assert_eq!(1, letters_before_offset(text, 3));
    }
}

#[cfg(test)]
mod tests_of_the_text_of_the_epubcfi {
    use super::*;

    fn place(steps: &[usize], text: &str) -> TextPlace {
        TextPlace {
            steps: steps.to_vec(),
            text: text.to_string(),
        }
    }

    #[test]
    fn it_makes_the_form_that_the_web_reader_writes() {
        let places = vec![place(&[4, 2, 1], "Hello reader")];
        assert_eq!(
            Some("epubcfi(/6/2!/4/2/1:0)".to_string()),
            to_epubcfi(0, &places, 0)
        );
        assert_eq!(
            Some("epubcfi(/6/14!/4/2/1:5)".to_string()),
            to_epubcfi(6, &places, 5)
        );
    }

    #[test]
    fn a_chapter_with_no_text_gives_nothing() {
        assert_eq!(None, to_epubcfi(3, &[], 0));
    }

    #[test]
    fn a_place_after_the_last_letter_gives_the_end_of_the_last_text() {
        let places = vec![place(&[4, 2, 1], "one"), place(&[4, 4, 1], "two")];
        assert_eq!(
            Some("epubcfi(/6/2!/4/4/1:3)".to_string()),
            to_epubcfi(0, &places, 9999)
        );
    }

    #[test]
    fn the_alternative_text_of_a_picture_gives_no_number_after_a_colon() {
        // The step is even, therefore it names an element, and an element takes
        // no place inside a text.
        let places = vec![place(&[4, 2], "a horse")];
        assert_eq!(
            Some("epubcfi(/6/2!/4/2)".to_string()),
            to_epubcfi(0, &places, 3)
        );
    }

    #[test]
    fn it_reads_a_real_value_of_the_web_reader() {
        assert_eq!(
            Some(CfiPlace {
                spine: 6,
                steps: vec![4, 2, 2, 2, 2, 1],
                offset: 0,
            }),
            parse_epubcfi("epubcfi(/6/14[id6]!/4/2/2/2[c1]/2/1:0)")
        );
    }

    #[test]
    fn it_reads_a_range_as_its_start() {
        assert_eq!(
            Some(CfiPlace {
                spine: 1,
                steps: vec![4, 2, 1],
                offset: 5,
            }),
            parse_epubcfi("epubcfi(/6/4!/4,/2/1:5,/2/1:9)")
        );
    }

    #[test]
    fn a_text_that_is_not_an_epubcfi_gives_nothing() {
        for text in [
            "",
            "toutui:3:120",
            "epubcfi(",
            "epubcfi()",
            "epubcfi(/6/14!/4",
            "a text of a user",
            "epubcfi(/6/13!/4)",
            "epubcfi(/6/0!/4)",
            "epubcfi(/6/abc!/4)",
            "epubcfi(/6/99999999999999999999999999!/4)",
        ] {
            assert_eq!(None, parse_epubcfi(text), "{text:?}");
        }
    }

    #[test]
    fn the_two_directions_agree_for_every_place_of_a_chapter() {
        let places = vec![
            place(&[4, 2, 1], "The first paragraph."),
            place(&[4, 4, 1], "The second one, with 3 words."),
            place(&[4, 4, 3], " and more"),
            place(&[4, 6, 1], "The end."),
        ];
        let total: usize = places.iter().map(|p| letters(&p.text)).sum();
        for target in 0..total {
            let text = to_epubcfi(2, &places, target).expect("a place must come");
            let read = parse_epubcfi(&text).expect("the text must be an EPUBCFI");
            assert_eq!(2, read.spine, "{target}");
            assert_eq!(target, letters_before(&places, &read), "{target} {text}");
        }
    }

    /// `epub.js` gives a different step to some texts. See the head of this
    /// module. The user must lose the place inside the paragraph, and never the
    /// paragraph.
    #[test]
    fn a_place_of_epub_js_gives_the_same_paragraph() {
        // The page is `<p><b>Title</b>: the text of the line</p>`, and it is the
        // second paragraph of the body.
        let places = vec![
            place(&[4, 2, 2, 1], "The paragraph before."),
            place(&[4, 4, 2, 1], "Title"),
            // This module gives the step 3 to the text after the `<b>`.
            place(&[4, 4, 3], ": the text of the line"),
        ];
        let before_the_paragraph = letters(&places[0].text);

        // `epub.js` names that same text `/4/4/1`, because it counts the texts
        // only. This module holds no such place.
        let of_epub_js = parse_epubcfi("epubcfi(/6/2!/4/4/1:12)").expect("it must be an EPUBCFI");
        let letters_of_it = letters_before(&places, &of_epub_js);

        // The answer stands inside the same paragraph: after the paragraph
        // before it, and before the end of the paragraph.
        assert!(letters_of_it >= before_the_paragraph, "{letters_of_it}");
        let end: usize = places.iter().map(|p| letters(&p.text)).sum();
        assert!(letters_of_it < end, "{letters_of_it}");
        // The place is the start of the text of that paragraph.
        assert_eq!(before_the_paragraph, letters_of_it);
    }

    #[test]
    fn a_place_that_names_no_text_gives_the_letters_before_the_next_text() {
        let places = vec![place(&[4, 2, 1], "onetwo"), place(&[4, 6, 1], "three")];
        // `/4/4` names an element between the two texts.
        let read = parse_epubcfi("epubcfi(/6/2!/4/4)").expect("the text must be an EPUBCFI");
        assert_eq!(6, letters_before(&places, &read));
        // A place before every text gives 0.
        let first = parse_epubcfi("epubcfi(/6/2!/4/1)").expect("the text must be an EPUBCFI");
        assert_eq!(0, letters_before(&places, &first));
        // A place after every text gives every letter.
        let last = parse_epubcfi("epubcfi(/6/2!/4/8/1:0)").expect("the text must be an EPUBCFI");
        assert_eq!(11, letters_before(&places, &last));
    }
}

#[cfg(test)]
mod tests_of_the_lines {
    use super::*;

    /// The letters of five lines of a chapter.
    const LINES: [usize; 5] = [10, 0, 20, 5, 15];

    #[test]
    fn the_letters_before_a_line_are_the_letters_of_the_lines_before_it() {
        assert_eq!(0, letters_before_line(&LINES, 0));
        assert_eq!(10, letters_before_line(&LINES, 1));
        assert_eq!(10, letters_before_line(&LINES, 2));
        assert_eq!(35, letters_before_line(&LINES, 4));
        // A line after the end gives every letter.
        assert_eq!(50, letters_before_line(&LINES, 99));
    }

    #[test]
    fn a_number_of_letters_gives_the_line_that_holds_it() {
        assert_eq!(0, line_of_letters(&LINES, 0));
        assert_eq!(0, line_of_letters(&LINES, 9));
        // The line 1 holds no letter, therefore the letter 10 stands in line 2.
        assert_eq!(2, line_of_letters(&LINES, 10));
        assert_eq!(3, line_of_letters(&LINES, 30));
        assert_eq!(4, line_of_letters(&LINES, 40));
        // A number after the last letter gives the last line.
        assert_eq!(4, line_of_letters(&LINES, 999));
    }

    #[test]
    fn a_chapter_with_no_line_gives_the_first_line() {
        assert_eq!(0, line_of_letters(&[], 0));
        assert_eq!(0, line_of_letters(&[], 500));
        assert_eq!(0, letters_before_line(&[], 3));
    }

    #[test]
    fn the_two_directions_of_the_lines_agree() {
        for (line, count) in LINES.iter().enumerate() {
            // A line with no letter cannot come back, because it holds no
            // letter of its own. Every other line comes back.
            if *count == 0 {
                continue;
            }
            let letters = letters_before_line(&LINES, line);
            assert_eq!(line, line_of_letters(&LINES, letters), "line {line}");
        }
    }
}
