use std::collections::HashMap;
use std::fmt;
use std::io::{self, stdin, stdout, Write};
use std::str::FromStr;

use chrono::prelude::*;
use chrono_humanize::HumanTime;
use futures::future::Future;
use structopt::StructOpt;
use termion::color::Color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::scroll;
use tokio::runtime::Runtime;

use lobsters::client::Page;
use lobsters::models::{ShortTag, Story, Tag};
use lobsters::url::Url;
use lobsters::Client;

use lobsters_cli::{
    text::Fancy,
    theme::{Theme, LOBSTERS_256, LOBSTERS_GREY, LOBSTERS_MONO, LOBSTERS_TRUE},
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

#[derive(Debug)]
enum UiTheme {
    Grey,
    Color256,
    Mono,
    TrueColor,
}

type CommandResult = Result<(), Error>;
type Line = Vec<Fancy>;

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

// Yes this is completely ridiculous... need to build better colour handling
fn render_stories<
    Score: 'static,
    Title: 'static,
    Meta: 'static,
    Ask: 'static,
    Media: 'static,
    Tag: 'static,
    Domain: 'static,
    Metadata: 'static,
>(
    stories: &[Story],
    tag_map: &TagMap,
    theme: &Theme<Score, Title, Meta, Ask, Media, Tag, Domain, Metadata>,
) -> Result<Vec<Line>, Error>
where
    Score: Color + Copy,
    Title: Color + Copy,
    Meta: Color + Copy,
    Ask: Color + Copy,
    Media: Color + Copy,
    Tag: Color + Copy,
    Domain: Color + Copy,
    Metadata: Color + Copy,
{
    let mut lines = Vec::new();

    // Calculate the max number of digits so scores can be padded
    let digits = stories
        .iter()
        .map(|story| util::count_digits(story.score))
        .max()
        .unwrap_or(1);

    for story in stories {
        // TODO: Map empty strings to None when parsing response
        let url = match story.url.as_str() {
            "" => None,
            url => Some(url.parse::<Url>().map_err(lobsters::Error::from)?),
        };
        let created_at = story.created_at.parse::<DateTime<FixedOffset>>()?;
        let meta = format!(
            "{:pad$} via {submitter} {when} | {n} comments",
            " ",
            pad = digits,
            submitter = story.submitter_user.username,
            when = HumanTime::from(created_at),
            n = story.comment_count
        );
        let domain = Fancy::new(
            url.and_then(|url| url.domain().map(|d| d.to_string()))
                .unwrap_or_else(|| "".to_string()),
        )
        .fg(theme.domain())
        .italic();

        let score = Fancy::new(format!("{:1$}", story.score, digits)).fg(theme.score());
        let title = Fancy::new(format!("{}", story.title))
            .fg(theme.title())
            .bold();
        let tags = story
            .tags
            .iter()
            .filter_map(|tag| tag_map.get(tag))
            .map(|tag| Fancy::new(tag.tag.as_str()).fg(theme.tag_colour(tag)));

        let mut line = Line::new();
        line.push(score);
        line.push(title);
        line.extend(tags);
        line.push(domain);
        lines.push(line);

        // Add meta line
        lines.push(vec![Fancy::new(meta).fg(theme.metadata())]);
    }

    Ok(lines)
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

    let tag_map = TagMap::new(tags);

    {
        let screen = AlternateScreen::from(stdout());
        let mut screen = screen.into_raw_mode()?;
        let stdin = stdin();

        // FIXME: This is terrible
        let lines = match options.theme {
            UiTheme::Color256 => render_stories(&stories, &tag_map, &LOBSTERS_256)?,
            UiTheme::TrueColor => render_stories(&stories, &tag_map, &LOBSTERS_TRUE)?,
            UiTheme::Mono => render_stories(&stories, &tag_map, &LOBSTERS_MONO)?,
            UiTheme::Grey => render_stories(&stories, &tag_map, &LOBSTERS_GREY)?,
        };

        // TODO: Proper rendering
        for line in lines {
            for (i, span) in line.iter().enumerate() {
                if i != 0 {
                    write!(screen, " {}", span)
                } else {
                    write!(screen, "{}", span)
                }?;
            }

            write!(screen, "\r\n")?;
        }

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Char('j') => {
                    write!(screen, "{}", scroll::Up(1))?;
                }
                Key::Char('k') => {
                    write!(screen, "{}", scroll::Down(1))?;
                }
                // Key::Char(c) => println!("{}", c),
                // Key::Alt(c) => println!("^{}", c),
                // Key::Ctrl(c) => println!("*{}", c),
                // Key::Esc => println!("ESC"),
                // Key::Left => println!("←"),
                // Key::Right => println!("→"),
                // Key::Up => println!("↑"),
                // Key::Down => println!("↓"),
                // Key::Backspace => println!("×"),
                _ => (),
            }
            // stdout.flush().unwrap();
        }
    }

    Ok(())
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
