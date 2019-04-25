use std::borrow::Cow;
use std::io::Write;

use chrono::prelude::*;
use chrono_humanize::HumanTime;
use termion::raw::RawTerminal;

use crate::{
    app::State,
    error::Error,
    text::Fancy,
    theme::{Colour, Theme},
    util,
};
use lobsters::url::Url;

type Line = Vec<Fancy>;
type Lines = Vec<Line>;

pub fn render_stories(state: &mut State, theme: &Theme, height: usize) -> Result<Lines, Error> {
    let mut lines = Vec::new();

    // Calculate the max number of digits so scores can be padded
    let digits = state.max_score_digits().unwrap_or(1);

    for (i, story) in state.stories().iter().enumerate() {
        // TODO: Map empty strings to None when parsing response
        let url = match story.url.as_str() {
            "" => None,
            url => Some(url.parse::<Url>().map_err(lobsters::Error::from)?),
        };
        let score = Fancy::new(format!("{:1$}", story.score, digits)).fg(theme.score);
        let title = Fancy::new(format!(" {}", story.title))
            .fg(theme.title)
            .bold();
        let tags = story
            .tags
            .iter()
            .filter_map(|tag| state.get_tag(&tag))
            .map(|tag| Fancy::new(format!(" {}", tag.tag)).fg(theme.tag_colour(tag)));
        let domain = Fancy::new(
            url.and_then(|url| url.domain().map(|d| format!(" {}", d)))
                .unwrap_or_else(|| "".to_string()),
        )
        .fg(theme.domain)
        .italic();

        let created_at = story.created_at.parse::<DateTime<FixedOffset>>()?;
        let meta = format!(
            "{:pad$} via {submitter} {when} | {n} comments",
            " ",
            pad = digits,
            submitter = story.submitter_user.username,
            when = HumanTime::from(created_at),
            n = story.comment_count
        );

        let mut line1 = Line::new();
        line1.push(score);
        line1.push(title);
        line1.extend(tags);
        line1.push(domain);

        // Meta line
        let mut line2 = vec![Fancy::new(meta).fg(theme.byline)];

        // Pretty sure this is breaking some software architecture rules
        if i == state.current_story_index() {
            line1 = highlight_line(line1, theme.cursor);
            line2 = highlight_line(line2, theme.cursor);
        }

        lines.push(line1);
        lines.push(line2);
    }

    Ok(limit_lines(state, lines, height))
}

fn highlight_line(line: Line, colour: Colour) -> Line {
    line.into_iter().map(|span| span.bg(colour)).collect()
}

trait Encompass<T> {
    fn encompass(&self, other: &std::ops::Range<T>) -> Option<std::cmp::Ordering>
    where
        T: PartialOrd<T>;
}

impl<T> Encompass<T> for std::ops::Range<T> {
    fn encompass(&self, other: &std::ops::Range<T>) -> Option<std::cmp::Ordering>
    where
        T: PartialOrd<T>,
    {
        if other.start < self.start {
            Some(std::cmp::Ordering::Less)
        } else if other.end > self.end {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

fn limit_lines(state: &mut State, lines: Lines, height: usize) -> Lines {
    // Work out the range of lines to render, ensuring the current story is visible
    let story_range = state.story_range();
    let ordering = state.visible_range(height).encompass(&story_range);
    let row_offset = state.row_offset_get_mut();

    match ordering {
        Some(std::cmp::Ordering::Less) => *row_offset = story_range.start,
        Some(std::cmp::Ordering::Equal) => (),
        Some(std::cmp::Ordering::Greater) => *row_offset = story_range.end - height,
        None => (),
    }

    lines
        .into_iter()
        .skip(*row_offset)
        .take(height as usize)
        .collect()
}

/// Render the lines with offset (x, y)
pub fn render_lines<W: Write>(
    lines: &[Line],
    screen: &mut RawTerminal<W>,
    col_offset: usize,
) -> Result<(), Error> {
    let (width, _height) = util::as_usize(termion::terminal_size()?);
    let empty_line = vec![0x20; width];

    write!(screen, "{}", termion::cursor::Goto(1, 1))?;

    // Limit the lines to the height and width of the terminal
    let scoped_lines = lines.iter().map(|line| {
        let cols_remaining = col_offset;

        line.iter().filter_map(move |span| {
            if cols_remaining > 0 {
                let span = span.truncate_front(cols_remaining);
                if span.is_empty() {
                    None
                } else {
                    Some(Cow::Owned(span))
                }
            } else {
                Some(Cow::Borrowed(span))
            }
        })
    });

    // Render the lines
    for (row, line) in scoped_lines.enumerate() {
        let mut col: usize = 0;

        if row != 0 {
            write!(screen, "\r\n")?;
        }

        let mut last_span = None;
        for span in line {
            let span_cols = span.cols();

            if col + span_cols < width {
                write!(screen, "{}", span)?;
                col += span_cols;
                last_span = Some(span);
            } else {
                let truncate_cols = 1 + width - col;
                let truncated = span.truncate(truncate_cols);
                write!(screen, "{}", truncated)?;
                col += truncate_cols;
                last_span = Some(Cow::Owned(truncated));
                break;
            }
        }

        // Erase the rest of the line
        // This is done in favor of ClearAll to reduce flicker
        if col < width {
            if let Some(bg) = last_span.and_then(|span| span.get_bg()) {
                // NOTE(unwrap): Safe because empty_line is all spaces
                let blank = String::from_utf8(empty_line[0..width - col].to_vec()).unwrap();
                let blank_with_bg = Fancy::new(blank).bg(bg);
                write!(screen, "{}", blank_with_bg)?;
            } else {
                screen.write_all(&empty_line[0..width - col])?;
            }
        }
    }

    screen.flush().map_err(Error::from)
}
