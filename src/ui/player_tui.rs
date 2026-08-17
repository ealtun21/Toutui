use crate::ui::the_band_of_the_player::{
    a_bar_of_a_part, the_bar_of_the_seek, the_cells_that_played, the_parts_of_the_band,
    the_parts_of_the_seek, the_percent_of_a_part, ThePartsOfTheBand,
};
use crate::ui::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Widget},
};

/// The values that the band of the player says. See T-322.
///
/// **The band reads no engine and no disk**: the render of the frame makes this
/// value of the state of the playback, and every field of it is a text or a
/// number that a test can write.
pub struct TheWordsOfTheBand {
    /// The name of the media, of `crate::logic::media_name`.
    pub title: String,
    /// The author of the media.
    pub author: String,
    /// The chapter of the place of the user, of
    /// `the_band_of_the_player::the_words_of_the_chapter`.
    pub chapter: String,
    /// A playback that runs, against a playback that a pause holds.
    pub it_plays: bool,
    /// The place of the user in the media, in seconds.
    pub position: u32,
    /// The length of the media, in seconds. A media of no length gives 0
    /// (T-180).
    pub length: u32,
    /// The place of the user in the chapter, and the length of that chapter, in
    /// seconds. A media of no chapter gives `None`.
    pub the_chapter: Option<(u32, u32)>,
    /// The speed of the playback, of `player_info`.
    pub speed: String,
    /// The volume of the playback, when it is not the volume of the file
    /// (T-80).
    pub volume: String,
    /// A short message of the engine. An example is "Reconnected".
    pub notice: Option<String>,
    /// The time that the timer for sleep has left, if a timer runs. See T-24.
    pub sleep: Option<String>,
    /// The user asked for the row of the keys of the player with the key `B`.
    ///
    /// **The render reads no disk** (T-204): the `App` holds this value, and the
    /// key `B` writes it and the disk together.
    pub the_buttons_stand: bool,
}

/// The keys of the player, on the row of the buttons of the band.
const THE_KEYS_OF_THE_PLAYER: &str =
    "Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/− | t: sleep | \
     Y: quit";

/// The mark of a playback that runs, and of a playback that a pause holds.
const THE_MARK_OF_A_PLAYBACK: [&str; 2] = ["⏸", "▶"];

/// The name of the bar of the book, on the row of the two bars.
const THE_BOOK: &str = "Book";

/// The name of the bar of the chapter, on the row of the two bars.
const THE_CHAPTER: &str = "Chapter";

/// The columns of a name of a bar, of a percent, and of the spaces of them, on
/// the row of the two bars.
///
/// `Chapter ` takes eight columns, ` 100%` takes five, and the two spaces after
/// them keep the two halves of the row apart: the first form of this row said
/// `15%Chapter` with no space at all.
const THE_WORDS_OF_A_BAR: u16 = 15;

/// Draws the band of the player, and gives the cells of the bar of the seek.
///
/// **The band held no bar at all before this stage** (T-322): the three rows of
/// the player stood in the air under the frame of the panels, with no border and
/// no title, and the place of the user stood in a percent of two digits and in
/// nothing else. The head of `crate::ui::the_band_of_the_player` holds the
/// screen of that fault.
///
/// The caller gives the whole area of the band, with its border, and it takes
/// the area of the bar of the seek for the click of the user. A band that draws
/// no bar gives `Rect::default()`, which holds no cell of the screen at all.
pub fn render_the_band(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    words: &TheWordsOfTheBand,
    bg_color: &[u8],
) -> Rect {
    if area.height < 3 || area.width < 3 {
        return Rect::default();
    }

    let of_the_band =
        Style::default().bg(crate::config::Colors::the_colour_of(bg_color, Color::Reset));

    // **The band takes every cell of its area** (T-322): the views that draw no
    // layout of their own — the statistics is one of them — write their text
    // over the rows of the band, and the row of the seek of that view then said
    // `23:44:39,├` with the `2` and the `,` of the view under it. A row of the
    // band writes the columns of its own parts alone, therefore the cells
    // between them need this.
    Clear.render(area, buf);

    let block = crate::ui::frame::a_band("Player").style(of_the_band);
    let inside = block.inner(area);
    block.render(area, buf);

    let parts = the_parts_of_the_band(inside);

    render_the_words(parts, buf, words);
    let the_bar = render_the_seek(parts, buf, words);
    render_the_bars(parts, buf, words);

    if words.the_buttons_stand && parts.the_buttons.width > 0 {
        // **A row of the band that the screen cuts says that the screen cut
        // it** (T-369, and the rule of T-304 and of T-368). This row holds 99
        // columns, therefore every terminal under 102 columns loses its end:
        // the measurement of a terminal of 80 columns gave
        // `o/i: vol +/`, which is a key and no word of its work, and the keys
        // `t` and `Y` of that row stood on the screen in no form at all.
        Paragraph::new(Span::styled(
            crate::logic::message::in_one_row(THE_KEYS_OF_THE_PLAYER, parts.the_buttons.width),
            theme::a_quiet_text(),
        ))
        .alignment(Alignment::Center)
        .render(parts.the_buttons, buf);
    }

    the_bar
}

/// Makes the spans of a line that must stand in a width of columns.
///
/// **A line of spans that is wider than its area loses the columns of its end**
/// (T-369): ratatui draws no mark of that cut, and the row 1 of the band of a
/// terminal of 60 columns therefore said `Many Hours A` for an author of the
/// name `Many Hours Author`.
///
/// [`crate::logic::message::in_one_row`] holds that rule for one text of one
/// style. This row holds four texts of four styles — the mark of the playback,
/// the title, the author, and the chapter — therefore the three points belong
/// at the end of the **line** and not at the end of each text of it: the spans
/// that stand keep their style, the span that meets the last column keeps its
/// start, and the spans after it go away.
///
/// The function is pure, therefore a test needs no screen.
fn in_one_row_of_spans<'a>(spans: Vec<Span<'a>>, width: u16) -> Vec<Span<'a>> {
    let whole: usize = spans
        .iter()
        .map(|span| crate::logic::message::the_columns_of(&span.content))
        .sum();

    if whole <= usize::from(width) {
        return spans;
    }

    if width == 0 {
        return Vec::new();
    }

    // The three points take one column of the row, and they take the style of
    // the span that they cut.
    let room = usize::from(width) - 1;
    let mut kept: Vec<Span<'a>> = Vec::with_capacity(spans.len() + 1);
    let mut columns = 0usize;
    let mut of_the_end = theme::a_quiet_text();

    for span in spans {
        let of_the_span = crate::logic::message::the_columns_of(&span.content);
        of_the_end = span.style;

        if columns + of_the_span <= room {
            columns += of_the_span;
            kept.push(span);
            continue;
        }

        let start = crate::logic::message::the_start_of_a_row(
            &span.content,
            u16::try_from(room - columns).unwrap_or(u16::MAX),
        );

        kept.push(Span::styled(start.trim_end().to_string(), span.style));
        break;
    }

    kept.push(Span::styled("…", of_the_end));

    kept
}

/// Draws the row 1 of the band: the media at the left, and the settings of the
/// playback at the right.
fn render_the_words(
    parts: ThePartsOfTheBand,
    buf: &mut ratatui::buffer::Buffer,
    words: &TheWordsOfTheBand,
) {
    if parts.the_words.width == 0 {
        return;
    }

    let mark = THE_MARK_OF_A_PLAYBACK[usize::from(words.it_plays)];

    // **The mark of a playback that runs takes the green of ANSI** (the table
    // of the colours of the section (d) of `docs/mockups/mockup-1.md`), and no
    // colour of RGB (T-317).
    //
    // **The title, the author, and the chapter come from the server** (T-312),
    // and a text of an end of a line takes a row of its own: the row of the
    // buttons then falls outside the band and no user reads it.
    // `in_one_line` gives every end of a line one space, and the four rows of
    // the band then stand.
    let mut of_the_media = vec![
        Span::styled(
            format!(" {mark} "),
            Style::default()
                .fg(theme::AN_END_THAT_IS_GOOD)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            crate::logic::message::in_one_line(&words.title).into_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}", crate::logic::message::in_one_line(&words.author)),
            theme::a_quiet_text(),
        ),
        Span::styled(
            format!("  {}", crate::logic::message::in_one_line(&words.chapter)),
            theme::a_quiet_text(),
        ),
    ];

    if let Some(notice) = &words.notice {
        of_the_media.push(Span::styled(
            format!("  {notice}"),
            Style::default().fg(theme::A_FAULT),
        ));
    }

    // The two spaces of the start keep the settings apart from the words of
    // the media: the row of a terminal of 40 columns said
    // `A Book Of Many Hours  MSpeed 1.00x` with no space at all.
    let mut of_the_settings = format!("  Speed {}x", words.speed);

    if !words.volume.is_empty() {
        // `the_volume_of_the_row` gives ` | Vol: 70%`, and the band says its
        // own words of that value.
        of_the_settings.push_str(&format!("   Volume {}", words.volume.trim()));
    }

    if let Some(sleep) = &words.sleep {
        of_the_settings.push_str(&format!("   {sleep}"));
    }

    of_the_settings.push(' ');

    let columns = crate::logic::message::the_columns_of(&of_the_settings) as u16;

    // **A row that holds no room for the settings says the media alone**: the
    // title of the media is the value of the row, and the speed, the volume,
    // and the timer stand in the view of the settings and in the message of
    // their own key.
    //
    // **The media takes the columns that the settings leave**, and it takes no
    // column more: a paragraph of the whole row would draw its own text under
    // the settings, and the row of a narrow terminal then held the two texts on
    // the same columns.
    let of_the_row = if columns < parts.the_words.width {
        parts.the_words.width - columns
    } else {
        parts.the_words.width
    };

    // **The words of the media that the row cannot hold say that the row cut
    // them** (T-369): the title, the author, and the chapter come of the
    // server, and a row that is too narrow for the three of them lost the end
    // of the last one with no mark at all.
    Paragraph::new(Line::from(in_one_row_of_spans(of_the_media, of_the_row)))
        .alignment(Alignment::Left)
        .render(
            Rect::new(parts.the_words.x, parts.the_words.y, of_the_row, 1),
            buf,
        );

    if columns < parts.the_words.width {
        let at_the_right = Rect::new(
            parts.the_words.right() - columns,
            parts.the_words.y,
            columns,
            1,
        );

        Paragraph::new(Span::styled(of_the_settings, theme::a_quiet_text()))
            .alignment(Alignment::Right)
            .render(at_the_right, buf);
    }
}

/// Draws the row 2 of the band: the bar of the seek, with the two times at its
/// two ends. It gives the cells of that bar, for the click of the user.
fn render_the_seek(
    parts: ThePartsOfTheBand,
    buf: &mut ratatui::buffer::Buffer,
    words: &TheWordsOfTheBand,
) -> Rect {
    if parts.the_seek.width == 0 {
        return Rect::default();
    }

    let of_the_place = crate::player::integrated::player_info::format_time(words.position);
    let of_the_length = crate::player::integrated::player_info::the_length_of_the_row(words.length);

    let Some(seek) = the_parts_of_the_seek(
        parts.the_seek,
        crate::logic::message::the_columns_of(&of_the_place),
        crate::logic::message::the_columns_of(&of_the_length),
    ) else {
        // **A row that is too narrow for a bar says the two times alone**: a
        // bar of three cells says nothing of a place, and a click of it would
        // move a book of eight hours by more than two hours.
        Paragraph::new(Span::styled(
            format!(" {of_the_place} / {of_the_length}"),
            Style::default().fg(Color::Yellow),
        ))
        .render(parts.the_seek, buf);

        return Rect::default();
    };

    // **The times take the yellow of ANSI** (the table of the colours of the
    // section (d) of `docs/mockups/mockup-1.md`).
    let of_a_time = Style::default().fg(Color::Yellow);

    Paragraph::new(Span::styled(of_the_place, of_a_time)).render(seek.the_time_of_the_place, buf);
    Paragraph::new(Span::styled(of_the_length, of_a_time)).render(seek.the_length, buf);

    let bar = the_bar_of_the_seek(seek.the_bar.width, words.position, words.length);

    // **The part of the bar that played takes the one accent of the program**
    // (`crate::ui::theme::THE_ACCENT`), and the part that stays takes the
    // foreground of the terminal with the modifier `DIM`.
    let cells: Vec<char> = bar.chars().collect();
    let of_the_place = the_cells_that_played(seek.the_bar.width, words.position, words.length);

    let bar = Line::from(vec![
        Span::styled(
            cells.iter().take(of_the_place).collect::<String>(),
            Style::default().fg(theme::THE_ACCENT),
        ),
        Span::styled(
            cells.iter().skip(of_the_place).collect::<String>(),
            theme::a_quiet_text(),
        ),
    ]);

    Paragraph::new(Span::styled("├", theme::a_quiet_text()))
        .render(Rect::new(seek.the_bar.x - 1, seek.the_bar.y, 1, 1), buf);
    Paragraph::new(bar).render(seek.the_bar, buf);
    Paragraph::new(Span::styled("┤", theme::a_quiet_text()))
        .render(Rect::new(seek.the_bar.right(), seek.the_bar.y, 1, 1), buf);

    seek.the_bar
}

/// Draws the row 3 of the band: the bar of the book, and the bar of the
/// chapter.
///
/// **A media of no chapter gives the whole row to the bar of the book**, and a
/// media of no length gives no bar at all (T-180).
fn render_the_bars(
    parts: ThePartsOfTheBand,
    buf: &mut ratatui::buffer::Buffer,
    words: &TheWordsOfTheBand,
) {
    if parts.the_bars.width < 2 || words.length == 0 {
        return;
    }

    // The row of the bars keeps one space at its left, as the row of the words
    // and the row of the seek do.
    let row = Rect::new(
        parts.the_bars.x + 1,
        parts.the_bars.y,
        parts.the_bars.width - 1,
        1,
    );

    // **A row that holds no room for two bars holds the bar of the book
    // alone**, and a row that holds no room for one holds none: a bar of three
    // cells says nothing of a place at all (`THE_SMALLEST_BAR`).
    let (the_chapter, of_the_bar) = the_two_bars(row.width, words.the_chapter.is_some());

    let Some(of_the_bar) = of_the_bar else {
        return;
    };

    let mut the_bars = vec![the_words_of_a_bar(
        THE_BOOK,
        of_the_bar,
        words.position,
        words.length,
    )];

    if let Some((done, whole)) = words.the_chapter.filter(|_| the_chapter) {
        the_bars.push(the_words_of_a_bar(THE_CHAPTER, of_the_bar, done, whole));
    }

    Paragraph::new(Line::from(the_bars.concat())).render(row, buf);
}

/// The number of bars of the row 3, and the cells of each of them.
///
/// It gives `(false, None)` for a row that holds no bar at all.
fn the_two_bars(width: u16, a_chapter_stands: bool) -> (bool, Option<u16>) {
    let smallest = THE_WORDS_OF_A_BAR + crate::ui::the_band_of_the_player::THE_SMALLEST_BAR;

    if a_chapter_stands && width / 2 >= smallest {
        return (true, Some(width / 2 - THE_WORDS_OF_A_BAR));
    }

    if width >= smallest {
        return (false, Some(width - THE_WORDS_OF_A_BAR));
    }

    (false, None)
}

/// The spans of one bar of the row 3: the name, the cells, and the percent.
fn the_words_of_a_bar(name: &str, width: u16, done: u32, whole: u32) -> Vec<Span<'static>> {
    let played = the_percent_of_a_part(done, whole);
    let cells = a_bar_of_a_part(width, done, whole);
    let of_the_place = the_cells_that_played(width, done, whole);
    let cells: Vec<char> = cells.chars().collect();

    vec![
        Span::styled(format!("{name:<8}"), theme::a_quiet_text()),
        Span::styled(
            cells.iter().take(of_the_place).collect::<String>(),
            Style::default().fg(theme::THE_ACCENT),
        ),
        Span::styled(
            cells.iter().skip(of_the_place).collect::<String>(),
            theme::a_quiet_text(),
        ),
        Span::styled(format!("{played:>4}%  "), theme::a_quiet_text()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The columns of a line of spans, of the crate that ratatui measures with.
    fn the_columns_of_the_spans(spans: &[Span]) -> usize {
        spans
            .iter()
            .map(|span| crate::logic::message::the_columns_of(&span.content))
            .sum()
    }

    /// The text of a line of spans.
    fn the_text_of_the_spans(spans: &[Span]) -> String {
        spans.iter().map(|span| span.content.as_ref()).collect()
    }

    /// The spans of the row 1 of the band of the measurement of T-369.
    fn the_spans_of_the_row() -> Vec<Span<'static>> {
        vec![
            Span::styled(
                " \u{25b6} ",
                Style::default().fg(theme::AN_END_THAT_IS_GOOD),
            ),
            Span::styled(
                "A Book Of Many Hours".to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Many Hours Author".to_string(), theme::a_quiet_text()),
            Span::styled(
                "  Chapter 2 of 3: The hours of the middle".to_string(),
                theme::a_quiet_text(),
            ),
        ]
    }

    /// A line of spans that is wider than its row keeps its start, and the three
    /// points say that the row cut it. See T-369.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_line_of_spans_that_is_too_wide_says_that_it_was_cut() {
        let whole = the_columns_of_the_spans(&the_spans_of_the_row());
        assert_eq!(whole, 83);

        // The control: a row that holds the whole line keeps every span of it,
        // with no mark of a cut at all.
        let of_the_control = in_one_row_of_spans(the_spans_of_the_row(), 83);
        assert_eq!(of_the_control.len(), 4);
        assert!(!the_text_of_the_spans(&of_the_control).contains('\u{2026}'));
        assert_eq!(
            the_text_of_the_spans(&in_one_row_of_spans(the_spans_of_the_row(), 200)),
            the_text_of_the_spans(&the_spans_of_the_row())
        );

        // **The line stands inside its row, and it says the cut**, at every
        // width of the measurement and at every width between them.
        for width in 1..=83u16 {
            let cut = in_one_row_of_spans(the_spans_of_the_row(), width);
            let text = the_text_of_the_spans(&cut);

            assert!(
                the_columns_of_the_spans(&cut) <= usize::from(width),
                "a line of {width} columns holds {} columns: {text:?}",
                the_columns_of_the_spans(&cut)
            );

            if width < 83 {
                assert!(
                    text.ends_with('\u{2026}'),
                    "a line of {width} columns says no cut: {text:?}"
                );
            }
        }

        // **The spans that stand keep their style**: the title of the media is
        // bold, and the author and the chapter are quiet.
        let cut = in_one_row_of_spans(the_spans_of_the_row(), 40);
        assert_eq!(
            the_text_of_the_spans(&cut),
            " \u{25b6} A Book Of Many Hours  Many Hours Aut\u{2026}"
        );
        assert!(cut[1].style.add_modifier.contains(Modifier::BOLD));

        // A width of no column gives no span at all.
        assert!(in_one_row_of_spans(the_spans_of_the_row(), 0).is_empty());
    }
}
