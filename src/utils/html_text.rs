//! Converts the HTML text from the server into plain text.
//!
//! The server sends the descriptions of the books and the podcasts as HTML.
//! The terminal shows plain text only. This module removes the tags and
//! decodes the entities. It uses the standard library only.

/// Changes the HTML text into plain text.
///
/// The function removes the tags. It changes the block tags into newlines.
/// It decodes the named entities and the numeric entities. It trims each
/// line. It reduces 3 newlines or more to 2 newlines.
///
/// The function accepts bad HTML. It does not panic.
pub fn to_plain_text(html: &str) -> String {
    let chars: Vec<char> = html.chars().collect();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '<' && is_tag_start(&chars, i + 1) {
            let (name, next) = read_tag(&chars, i);
            if is_break_tag(&name) {
                out.push('\n');
            }
            i = next;
        } else if c == '&' {
            match read_entity(&chars, i) {
                Some((text, next)) => {
                    out.push_str(&text);
                    i = next;
                }
                None => {
                    out.push('&');
                    i += 1;
                }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }

    normalize(&out)
}

/// Tells if a tag starts at the given position.
///
/// A `<` character is a tag start only if a letter, a slash, an exclamation
/// mark or a question mark comes after it. Thus the function keeps a lone
/// `<` character in the text.
fn is_tag_start(chars: &[char], pos: usize) -> bool {
    match chars.get(pos) {
        Some(c) => c.is_ascii_alphabetic() || *c == '/' || *c == '!' || *c == '?',
        None => false,
    }
}

/// Reads one tag and gives its name and the position after the tag.
///
/// The name is in lower case. It keeps the first slash of an end tag.
/// The function ignores a `>` character inside an attribute value.
/// If the tag has no end, the function stops at the end of the text.
fn read_tag(chars: &[char], start: usize) -> (String, usize) {
    let mut name = String::new();
    // Move after the `<` character.
    let mut i = start + 1;

    if chars.get(i) == Some(&'/') {
        name.push('/');
        i += 1;
    }

    while let Some(c) = chars.get(i) {
        if c.is_ascii_alphanumeric() || *c == '-' || *c == '!' {
            name.extend(c.to_lowercase());
            i += 1;
        } else {
            break;
        }
    }

    let mut quote: Option<char> = None;
    while let Some(c) = chars.get(i) {
        i += 1;
        match quote {
            Some(q) => {
                if *c == q {
                    quote = None;
                }
            }
            None => {
                if *c == '"' || *c == '\'' {
                    quote = Some(*c);
                } else if *c == '>' {
                    break;
                }
            }
        }
    }

    (name, i)
}

/// Tells if the tag makes a new line.
fn is_break_tag(name: &str) -> bool {
    matches!(name, "br" | "/br" | "/p" | "/div" | "/li")
}

/// Reads one entity and gives its text and the position after the entity.
///
/// The function gives `None` if the entity has no end or if the name is not
/// known. Then the caller keeps the `&` character.
fn read_entity(chars: &[char], start: usize) -> Option<(String, usize)> {
    // The longest entity that this function accepts has 32 characters.
    let max = usize::min(chars.len(), start + 1 + 32);
    let mut body = String::new();
    let mut i = start + 1;

    while i < max {
        let c = chars[i];
        if c == ';' {
            let text = decode_entity(&body)?;
            return Some((text, i + 1));
        }
        if !c.is_ascii_alphanumeric() && c != '#' {
            return None;
        }
        body.push(c);
        i += 1;
    }

    None
}

/// Gives the text of one entity body.
///
/// The body is the part between the `&` character and the `;` character.
/// The function accepts the named entities, the decimal entities and the
/// hexadecimal entities. It gives `None` for an unknown body.
fn decode_entity(body: &str) -> Option<String> {
    if body.is_empty() {
        return None;
    }

    if let Some(number) = body.strip_prefix('#') {
        let code = if let Some(hex) = number.strip_prefix(['x', 'X']) {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            number.parse::<u32>().ok()?
        };
        return char::from_u32(code).map(String::from);
    }

    let text = match body.to_ascii_lowercase().as_str() {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => " ",
        _ => return None,
    };

    Some(text.to_string())
}

/// Cleans the spaces and the newlines of the text.
///
/// The function trims each line. It reduces 3 newlines or more to 2
/// newlines. It trims the start and the end of the text.
fn normalize(text: &str) -> String {
    let joined = text
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<&str>>()
        .join("\n");

    let mut out = String::with_capacity(joined.len());
    let mut newlines = 0;

    for c in joined.chars() {
        if c == '\n' {
            newlines += 1;
            if newlines <= 2 {
                out.push(c);
            }
        } else {
            newlines = 0;
            out.push(c);
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_inline_tags() {
        assert_eq!(
            to_plain_text("<i>Life in the North</i>"),
            "Life in the North"
        );
        assert_eq!(
            to_plain_text("a <b class=\"x\">bold</b> word"),
            "a bold word"
        );
    }

    #[test]
    fn changes_paragraph_end_into_newline() {
        assert_eq!(to_plain_text("<p>one</p><p>two</p>"), "one\ntwo");
    }

    #[test]
    fn changes_break_tags_into_newline() {
        assert_eq!(to_plain_text("one<br>two"), "one\ntwo");
        assert_eq!(to_plain_text("one<br/>two"), "one\ntwo");
        assert_eq!(to_plain_text("one<br />two"), "one\ntwo");
    }

    #[test]
    fn changes_div_and_li_end_into_newline() {
        assert_eq!(to_plain_text("<div>one</div><div>two</div>"), "one\ntwo");
        assert_eq!(
            to_plain_text("<ul><li>one</li><li>two</li></ul>"),
            "one\ntwo"
        );
    }

    #[test]
    fn decodes_named_entities() {
        assert_eq!(to_plain_text("a &amp; b"), "a & b");
        assert_eq!(to_plain_text("&lt;tag&gt;"), "<tag>");
        assert_eq!(to_plain_text("&quot;quoted&quot;"), "\"quoted\"");
        assert_eq!(to_plain_text("it&#39;s"), "it's");
        assert_eq!(to_plain_text("it&apos;s"), "it's");
        assert_eq!(to_plain_text("a&nbsp;b"), "a b");
    }

    #[test]
    fn decodes_numeric_entity() {
        assert_eq!(to_plain_text("it&#8217;s"), "it\u{2019}s");
    }

    #[test]
    fn decodes_hexadecimal_entity() {
        assert_eq!(to_plain_text("it&#x2019;s"), "it\u{2019}s");
        assert_eq!(to_plain_text("it&#X2019;s"), "it\u{2019}s");
    }

    #[test]
    fn keeps_unknown_entity() {
        assert_eq!(to_plain_text("a &unknown; b"), "a &unknown; b");
        assert_eq!(to_plain_text("100 & 200"), "100 & 200");
        assert_eq!(to_plain_text("Tom & Jerry &amp; Co"), "Tom & Jerry & Co");
    }

    #[test]
    fn accepts_nested_and_unclosed_tags() {
        assert_eq!(
            to_plain_text("<div><p><b>deep <i>text</i></b></p></div>"),
            "deep text"
        );
        assert_eq!(to_plain_text("text <b>bold"), "text bold");
        assert_eq!(to_plain_text("broken <p"), "broken");
        assert_eq!(to_plain_text("<"), "<");
        assert_eq!(to_plain_text("a < b"), "a < b");
        assert_eq!(to_plain_text("a &"), "a &");
    }

    #[test]
    fn accepts_empty_string() {
        assert_eq!(to_plain_text(""), "");
        assert_eq!(to_plain_text("   "), "");
    }

    #[test]
    fn keeps_text_without_html() {
        let text = "A bounty hunting survivor. A Galactic dark elf.";
        assert_eq!(to_plain_text(text), text);
        assert_eq!(to_plain_text("line one\nline two"), "line one\nline two");
    }

    #[test]
    fn reduces_many_newlines() {
        assert_eq!(to_plain_text("one</p></p></p></p>two"), "one\n\ntwo");
    }

    #[test]
    fn converts_description_from_the_server() {
        let html = "<p>Stories and writers featured in this anthology include: </p> \
                    <p>Tao Wong - \u{201c}Debts &amp; Dances\u{201d} covers the arrival</p>";
        assert_eq!(
            to_plain_text(html),
            "Stories and writers featured in this anthology include:\n\
             Tao Wong - \u{201c}Debts & Dances\u{201d} covers the arrival"
        );
    }
}
