use std::fmt::{self, Formatter};

use termion::color::{self, AnsiValue, Color as TermionColor, Rgb};

use lobsters::models;

#[derive(Clone, Copy)]
pub enum Colour {
    Ansi(AnsiValue),
    Rgb(Rgb),
    White,
}

pub struct Theme {
    pub score: Colour,
    pub meta_tag: Colour,
    pub ask_tag: Colour,
    pub media_tag: Colour,
    pub normal_tag: Colour,
    pub title: Colour,
    pub domain: Colour,
    pub metadata: Colour,
}

pub static LOBSTERS_MONO: Theme = Theme {
    score: Colour::White,
    ask_tag: Colour::White,
    media_tag: Colour::White,
    meta_tag: Colour::White,
    normal_tag: Colour::White,
    title: Colour::White,
    domain: Colour::White,
    metadata: Colour::White,
};

pub static LOBSTERS_GREY: Theme = Theme {
    score: Colour::Ansi(AnsiValue(248)),
    ask_tag: Colour::Ansi(AnsiValue(252)),
    media_tag: Colour::Ansi(AnsiValue(252)),
    meta_tag: Colour::Ansi(AnsiValue(252)),
    normal_tag: Colour::Ansi(AnsiValue(252)),
    title: Colour::Ansi(AnsiValue(254)),
    domain: Colour::Ansi(AnsiValue(245)),
    metadata: Colour::Ansi(AnsiValue(250)),
};

pub static LOBSTERS_256: Theme = Theme {
    score: Colour::Ansi(AnsiValue(248)),
    ask_tag: Colour::Ansi(AnsiValue(1)), // TODO: Find a better colour for this one
    media_tag: Colour::Ansi(AnsiValue(195)),
    meta_tag: Colour::Ansi(AnsiValue(252)),
    normal_tag: Colour::Ansi(AnsiValue(229)),
    title: Colour::Ansi(AnsiValue(33)),
    domain: Colour::Ansi(AnsiValue(245)),
    metadata: Colour::Ansi(AnsiValue(250)),
};

pub static LOBSTERS_TRUE: Theme = Theme {
    score: Colour::Rgb(Rgb(170, 170, 170)),
    ask_tag: Colour::Rgb(Rgb(240, 178, 184)),
    media_tag: Colour::Rgb(Rgb(178, 204, 240)),
    meta_tag: Colour::Rgb(Rgb(200, 200, 200)),
    normal_tag: Colour::Rgb(Rgb(213, 212, 88)),
    title: Colour::Rgb(Rgb(37, 98, 220)),
    domain: Colour::Rgb(Rgb(153, 153, 153)), // On the site this is actually the same as metadata
    metadata: Colour::Rgb(Rgb(136, 136, 136)),
};

impl Theme {
    pub fn tag_colour(&self, tag: &models::Tag) -> Colour {
        if tag.tag == "ask" || tag.tag == "show" {
            self.ask_tag
        } else if tag.tag == "meta" {
            self.meta_tag
        } else if tag.is_media {
            self.media_tag
        } else {
            self.normal_tag
        }
    }
}

impl TermionColor for Colour {
    fn write_fg(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Colour::Ansi(ansi) => ansi.write_fg(f),
            Colour::Rgb(rgb) => rgb.write_fg(f),
            Colour::White => color::White.write_fg(f),
        }
    }

    fn write_bg(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Colour::Ansi(ansi) => ansi.write_bg(f),
            Colour::Rgb(rgb) => rgb.write_bg(f),
            Colour::White => color::White.write_bg(f),
        }
    }
}
