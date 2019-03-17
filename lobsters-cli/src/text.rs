use std::fmt::{self, Display};

use termion::color::{Bg, Color as Colour, Fg};
use termion::style::{Bold, Italic, NoBold, NoItalic, NoUnderline, Underline};

/// Fancy text (styled)
///
/// Example:
///
/// ```
/// use lobsters_cli::text::Fancy;
///
/// let fancy_text = Fancy::new("Hello").fg(Box::new(termion::color::Red)).bold();
/// ```
// Not sure about these trait objects but they work for now
pub struct Fancy {
    text: String,
    fg: Option<Box<dyn Colour>>,
    bg: Option<Box<dyn Colour>>,
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

    pub fn fg(mut self, colour: Box<dyn Colour>) -> Self {
        self.fg = Some(colour);
        self
    }

    pub fn bg(mut self, colour: Box<dyn Colour>) -> Self {
        self.bg = Some(colour);
        self
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
}

impl Display for Fancy {
    // This is not exactly efficient generation of escape sequences but will do for now.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(colour) = &self.bg {
            write!(f, "{}", Bg(colour.as_ref()))?;
        }
        if let Some(colour) = &self.fg {
            write!(f, "{}", Fg(colour.as_ref()))?;
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
            .fg(Box::new(termion::color::White))
            .bg(Box::new(termion::color::Blue))
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
