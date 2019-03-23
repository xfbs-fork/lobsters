use std::collections::HashMap;
use std::fmt;
use std::io::{self, stdin, stdout, Write};
use std::str::FromStr;

use chrono::prelude::*;
use chrono_humanize::HumanTime;
use futures::future::Future;
use structopt::StructOpt;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tokio::runtime::Runtime;

use lobsters::client::Page;
use lobsters::models::{ShortTag, Story, Tag};
use lobsters::url::{self, Url};
use lobsters::Client;

use lobsters_cli::{
    text::Fancy,
    theme::{Colour, Theme, LOBSTERS_256, LOBSTERS_GREY, LOBSTERS_MONO, LOBSTERS_TRUE},
    util,
};

#[derive(Debug)]
enum Error {
    Lobsters(lobsters::Error),
    InvalidDate(chrono::ParseError),
}

#[derive(Debug)]
struct ParseThemeError(String);

#[derive(Debug, StructOpt)]
enum Command {
    /// Login with username and password to customise view
    #[structopt(name = "login")]
    Login(Login),
    /// View front page stories (this is the default)
    #[structopt(name = "stories")]
    Stories(Stories),
}

#[derive(Debug, StructOpt)]
struct Login {}

#[derive(Debug, Default, StructOpt)]
struct Stories {
    /// Page to view
    #[structopt(short = "p", long = "page")]
    page: Option<u32>,

    /// Theme to use. Options: true, 256, grey or gray, mono
    #[structopt(
        short = "t",
        long = "theme",
        default_value = "256",
        parse(try_from_str)
    )]
    theme: UiTheme,
}

#[derive(Debug, StructOpt)]
struct App {
    /// Base URL of the remote site
    #[structopt(
        short = "b",
        long = "base-url",
        raw(default_value = "lobsters::URL"),
        parse(try_from_str = "util::parse_url")
    )]
    base_url: Url,

    #[structopt(subcommand)]
    command: Option<Command>,
}

struct StoryView {
    tag_map: TagMap,
    stories: Vec<Story>,
    current_story: usize,
    row_offset: usize,
    col_offset: usize,
}

#[derive(Debug)]
enum UiTheme {
    Grey,
    Color256,
    Mono,
    TrueColor,
}

type CommandResult = Result<(), Error>;
type Line = Vec<Fancy>;
type Lines = Vec<Line>;

struct TagMap {
    tags: HashMap<String, Tag>,
}

fn main() {
    let app = App::from_args();
    let mut rt = Runtime::new().unwrap();
    let client = Client::new(app.base_url).expect("error creating client");

    let result = match app.command.unwrap_or_default() {
        Command::Login(options) => login(&mut rt, client, options),
        Command::Stories(options) => stories(&mut rt, client, options),
    };

    match &result {
        Ok(()) => (),
        Err(Error::Lobsters(lobsters::Error::Http(err))) => {
            eprintln!("HTTP error, caused by: {:?}", err)
        }
        Err(Error::Lobsters(lobsters::Error::InvalidStr)) => {
            eprintln!("UTF-8 error: Some data that was supposed to be a string was not valid UTF-8")
        }
        Err(Error::Lobsters(lobsters::Error::Io(err))) => {
            eprintln!("IO error, caused by: {:?}", err)
        }
        Err(Error::Lobsters(lobsters::Error::Url(err))) => eprintln!("Invalid URL: {:?}", err),
        Err(Error::Lobsters(lobsters::Error::HomeNotFound)) => {
            eprintln!("Error: Unable to determine home directory of user")
        }
        // This one should carry more information
        Err(Error::Lobsters(lobsters::Error::CookieStore)) => eprintln!("Cookie store error"),
        // This one should carry more information
        Err(Error::Lobsters(lobsters::Error::MissingHtmlElement)) => {
            eprintln!("Error: Tried to find a HTML element that did not exist on the page")
        }
        Err(Error::InvalidDate(err)) => eprintln!("Unable to parse date: {:?}", err),
    }

    if result.is_err() {
        std::process::exit(1);
    }
}

fn login(_rt: &mut Runtime, _client: Client, _options: Login) -> CommandResult {
    //     // let login = client.login(username, password);
    Ok(())
}

fn stories(rt: &mut Runtime, client: Client, options: Stories) -> CommandResult {
    let page = Page::new(options.page.unwrap_or(1));
    let future_stories = client.index(page);
    let future_tags = client.tags();
    let work = future_tags.join(future_stories);

    // Fetch tags and stories in parallel
    print!("Loading...");
    stdout().flush()?;
    let (tags, stories) = rt.block_on(work)?;
    println!(" done.");
    let (_width, height) = util::as_usize(termion::terminal_size()?);
    let height = usize::from(height);

    if stories.len() < 1 {
        println!("There are no stories to show.");
        return Ok(());
    }

    let tag_map = TagMap::new(tags);
    let mut state = StoryView::new(stories, tag_map);

    {
        let screen = AlternateScreen::from(stdout());
        let mut screen = screen.into_raw_mode()?;
        write!(screen, "{}", cursor::Hide)?;
        let stdin = stdin();

        let theme = match options.theme {
            UiTheme::Color256 => &LOBSTERS_256,
            UiTheme::TrueColor => &LOBSTERS_TRUE,
            UiTheme::Mono => &LOBSTERS_MONO,
            UiTheme::Grey => &LOBSTERS_GREY,
        };

        let mut lines = render_stories(&mut state, theme, height)?;
        render_lines(&lines, &mut screen, state.col_offset)?;

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') | Key::Esc => break,
                Key::Char('j') | Key::Down => {
                    if state.current_story < (state.stories.len() - 1) {
                        state.current_story += 1;
                        lines = render_stories(&mut state, theme, height)?;
                        render_lines(&lines, &mut screen, state.col_offset)?;
                    }
                }
                Key::Char('k') | Key::Up => {
                    if let Some(index) = state.current_story.checked_sub(1) {
                        state.current_story = index;
                        lines = render_stories(&mut state, theme, height)?;
                        render_lines(&lines, &mut screen, state.col_offset)?;
                    }
                }
                Key::Char('h') => {
                    // TODO: Limit the number of rows
                    state.col_offset += 10;
                    render_lines(&lines, &mut screen, state.col_offset)?;
                }
                Key::Char('l') => {
                    if let Some(new_offset) = state.col_offset.checked_sub(10) {
                        state.col_offset = new_offset;
                        render_lines(&lines, &mut screen, state.col_offset)?;
                    }
                }
                Key::Char('c') => {
                    let _ = opener::open(state.comments_url()?.as_str());
                }
                Key::Char('\n') => {
                    let _ = opener::open(state.story_url()?.as_str());
                }
                _ => (),
            }
        }

        write!(screen, "{}", cursor::Show)?;
    }

    Ok(())
}

fn render_stories(state: &mut StoryView, theme: &Theme, height: usize) -> Result<Lines, Error> {
    let mut lines = Vec::new();

    // Calculate the max number of digits so scores can be padded
    let digits = state
        .stories
        .iter()
        .map(|story| util::count_digits(story.score))
        .max()
        .unwrap_or(1);

    for (i, story) in state.stories.iter().enumerate() {
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
            .filter_map(|tag| state.tag_map.get(tag))
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
        if i == state.current_story {
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

fn limit_lines(state: &mut StoryView, lines: Lines, height: usize) -> Lines {
    // Work out the range of lines to render, ensuring the current story is visible
    let current_story_offset = state.current_story * 2;
    let visible_range = state.row_offset..state.row_offset + height;
    let story_range = current_story_offset..current_story_offset + 2;

    match visible_range.encompass(&story_range) {
        Some(std::cmp::Ordering::Less) => state.row_offset = story_range.start,
        Some(std::cmp::Ordering::Equal) => (),
        Some(std::cmp::Ordering::Greater) => state.row_offset = story_range.end - height,
        None => (),
    }

    lines
        .into_iter()
        .skip(state.row_offset)
        .take(height as usize)
        .collect()
}

/// Render the lines with offset (x, y)
fn render_lines<W: Write>(
    lines: &[Line],
    screen: &mut RawTerminal<W>,
    col_offset: usize,
) -> Result<(), Error> {
    let (width, height) = util::as_usize(termion::terminal_size()?);
    let mut log = std::fs::File::create("render.log")?;
    write!(log, "Terminal dimensions: {:?}\n", (width, height))?;
    write!(
        log,
        "Line 1 {}\n",
        lines[0]
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    )?;

    let empty_line = vec![0x20; width];

    write!(screen, "{}", termion::cursor::Goto(1, 1))?;

    let scoped_lines = lines.iter().map(|line| {
        let cols_remaining = col_offset;

        line.iter().filter_map(move |span| {
            if cols_remaining > 0 {
                let span = span.truncate_front(cols_remaining);
                if span.is_empty() {
                    None
                } else {
                    Some(span)
                }
            } else {
                Some(span.clone()) // FIXME: clone
            }
        })
    });

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
                write!(log, "{}: {}\n", col, span)?;
                col += span_cols;
                last_span = Some(span);
            } else {
                let truncate_cols = 1 + width - col;
                let truncated = span.truncate(truncate_cols);
                write!(screen, "{}", truncated)?;
                write!(log, "{} (t): {}\n", col, truncated)?;
                col += truncate_cols;
                last_span = Some(truncated);
                break;
            }
        }

        // Erase the rest of the line
        // This is done in favor of ClearAll to reduce flicker
        if col < width {
            if let Some(bg) = last_span.and_then(|span| span.get_bg()) {
                // TODO: Make Fancy store text with a Cow so slice can be passed to it
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

impl From<lobsters::Error> for Error {
    fn from(err: lobsters::Error) -> Self {
        Error::Lobsters(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Self {
        Error::InvalidDate(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Lobsters(lobsters::Error::Io(err))
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Lobsters(lobsters::Error::Url(err))
    }
}

impl TagMap {
    pub fn new(tags: Vec<Tag>) -> Self {
        let tags = tags.into_iter().fold(HashMap::new(), |mut map, tag| {
            map.insert(tag.tag.clone(), tag);
            map
        });

        TagMap { tags }
    }

    pub fn get<'a>(&'a self, name: &ShortTag) -> Option<&'a Tag> {
        self.tags.get(&name.0)
    }
}

impl StoryView {
    pub fn new(stories: Vec<Story>, tag_map: TagMap) -> Self {
        assert!(stories.len() > 0, "no stories");

        StoryView {
            stories,
            tag_map,
            current_story: 0,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn current_story(&self) -> &Story {
        &self.stories[self.current_story]
    }

    pub fn story_url(&self) -> Result<Url, url::ParseError> {
        match self.current_story().url.as_str() {
            "" => self.comments_url(),
            url => url.parse::<Url>(),
        }
    }

    pub fn comments_url(&self) -> Result<Url, url::ParseError> {
        self.current_story().comments_url.parse::<Url>()
    }
}

impl FromStr for UiTheme {
    type Err = ParseThemeError;

    fn from_str(theme: &str) -> Result<Self, Self::Err> {
        match theme {
            "true" => Ok(UiTheme::TrueColor),
            "256" => Ok(UiTheme::Color256),
            "mono" => Ok(UiTheme::Mono),
            "grey" | "gray" => Ok(UiTheme::Grey),
            _ => Err(ParseThemeError(theme.to_string())),
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        UiTheme::Color256
    }
}

impl fmt::Display for ParseThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' is not a valid theme. Options are: true, 256, mono, grey or gray",
            self.0
        )
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Stories(Stories::default())
    }
}
