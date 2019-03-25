use std::fmt::{self, Display};

use termion::color::{Bg, Fg};
use termion::style::{Bold, Italic, NoBold, NoItalic, NoUnderline, Underline};

use crate::theme::Colour;

/// Fancy text (styled)
///
/// Example:
///
/// ```
/// use lobsters_cli::text::Fancy;
/// use lobsters_cli::theme::Colour;
/// use termion::color::AnsiValue;
///
/// let fancy_text = Fancy::new("Hello").fg(Colour::Ansi(AnsiValue(10))).bold();
/// ```
// Not sure about these trait objects but they work for now
#[derive(Clone)]
pub struct Fancy {
    text: String,
    fg: Option<Colour>,
    bg: Option<Colour>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Fancy {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Fancy {
            text: text.into(),
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn fg(mut self, colour: Colour) -> Self {
        self.fg = Some(colour);
        self
    }

    pub fn bg(mut self, colour: Colour) -> Self {
        self.bg = Some(colour);
        self
    }

    pub fn get_bg(&self) -> Option<Colour> {
        self.bg
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// The span is empty if the text is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The number of columns the text will need
    pub fn cols(&self) -> usize {
        // str_width can return None if there are non-printable characters in string... not quite
        // sure how that should be handled right now
        wcwidth::str_width(&self.text).unwrap_or(0)
    }

    /// Truncates the text to the supplied length preserving formatting
    pub fn truncate(&self, cols: usize) -> Self {
        let mut truncated = self.clone();
        let mut new_len = 0;

        truncated.text = truncated
            .text
            .chars()
            .take_while(|c| {
                new_len += usize::from(wcwidth::char_width(*c).unwrap_or(0));
                new_len < cols
            })
            .collect();

        truncated
    }

    /// Removes `cols` of text from the start of the string preserving formatting
    pub fn truncate_front(&self, cols: usize) -> Self {
        let mut truncated = self.clone();
        let mut removed = 0;

        truncated.text = truncated
            .text
            .chars()
            .skip_while(|c| {
                removed += usize::from(wcwidth::char_width(*c).unwrap_or(0));
                removed < cols
            })
            .collect();

        truncated
    }
}

impl Display for Fancy {
    // This is not exactly efficient generation of escape sequences but will do for now.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(colour) = &self.bg {
            write!(f, "{}", Bg(*colour))?;
        }
        if let Some(colour) = &self.fg {
            write!(f, "{}", Fg(*colour))?;
        }
        if self.bold {
            write!(f, "{}", Bold)?;
        }
        if self.italic {
            write!(f, "{}", Italic)?;
        }
        if self.underline {
            write!(f, "{}", Underline)?;
        }

        write!(f, "{}", self.text)?;

        // TODO: Consider using Reset here since we never actually stack styles instead always
        // returning to the base state.
        if self.underline {
            write!(f, "{}", NoUnderline)?;
        }
        if self.italic {
            write!(f, "{}", NoItalic)?;
        }
        if self.bold {
            write!(f, "{}", NoBold)?;
        }
        if self.fg.is_some() {
            write!(f, "{}", Fg(termion::color::Reset))?;
        }
        if self.bg.is_some() {
            write!(f, "{}", Bg(termion::color::Reset))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fancy_text() {
        let fancy_text = Fancy::new("Test")
            .fg(Colour::White)
            .bg(Colour::Ansi(termion::color::AnsiValue(4)))
            .bold()
            .underline()
            .italic();

        let expected = format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            Bg(termion::color::Blue),
            Fg(termion::color::White),
            Bold,
            Italic,
            Underline,
            "Test",
            NoUnderline,
            NoItalic,
            NoBold,
            Fg(termion::color::Reset),
            Bg(termion::color::Reset),
        );

        assert_eq!(fancy_text.to_string(), expected);
    }
}
