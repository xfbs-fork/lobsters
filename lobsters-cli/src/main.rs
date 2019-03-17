use std::collections::HashMap;
use std::io::{stdin, stdout, Write};

use ansi_term::Colour::Fixed;
use ansi_term::Style;
use ansi_term::{ANSIString, ANSIStrings};
use chrono::prelude::*;
use chrono_humanize::HumanTime;
use futures::future::{Future, IntoFuture};
use structopt::StructOpt;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tokio::runtime::Runtime;

use lobsters::client::Page;
use lobsters::models::{NewComment, ShortTag, StoryId, Tag};
use lobsters::url::{self, Url};
use lobsters::Client;

use lobsters_cli::util;

trait Colour {
    fn colour(&self) -> ansi_term::Colour;
}

enum Error {
    Lobsters(lobsters::Error),
    InvalidDate(chrono::ParseError),
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "login")]
    Login(Login),
    #[structopt(name = "stories")]
    Stories(Stories),
    #[structopt(name = "comment")]
    Comment(Comment),
}

#[derive(Debug, StructOpt)]
struct Login {}

#[derive(Debug, StructOpt)]
struct Stories {
    #[structopt(short = "p", long = "page")]
    page: Option<u32>,
}

#[derive(Debug, StructOpt)]
struct Comment {}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "lobsters",
    about = "A command line client for the Lobsters website (https://lobste.rs/)"
)]
struct App {
    #[structopt(
        short = "b",
        long = "base-url",
        raw(default_value = "lobsters::URL"),
        parse(try_from_str = "util::parse_url")
    )]
    base_url: Url,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    command: Command,
}

type CommandResult = Result<(), Error>;

struct TagMap {
    tags: HashMap<String, Tag>,
}

fn main() {
    let app = App::from_args();
    let mut rt = Runtime::new().unwrap();
    let client = Client::new(app.base_url).expect("error creating client");

    let result = match app.command {
        Command::Login(options) => login(&mut rt, client, options),
        Command::Stories(options) => stories(&mut rt, client, options),
        Command::Comment(options) => comment(&mut rt, client, options),
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
    Ok(())
}

fn stories(rt: &mut Runtime, client: Client, options: Stories) -> CommandResult {
    let page = Page::new(options.page.unwrap_or(1));
    let future_stories = client.index(page);
    let future_tags = client.tags();
    let work = future_tags.join(future_stories);

    // Fetch tags and stories in parallel
    let (tags, stories) = rt.block_on(work)?;

    let tag_map = TagMap::new(tags);

    // Calculate the max number of digits so scores can be padded
    let digits = stories
        .iter()
        .map(|story| util::count_digits(story.score))
        .max()
        .unwrap_or(1);

    {
        let screen = AlternateScreen::from(stdout());
        let mut screen = screen.into_raw_mode().map_err(lobsters::Error::Io)?;
        let stdin = stdin();

        for story in stories {
            let score = format!("{:1$}", story.score, digits);
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
            let tags = std::slice::SliceConcatExt::join(
                story
                    .tags
                    .iter()
                    .filter_map(|tag| tag_map.get_coloured(tag).map(|tag| tag.to_string()))
                    .collect::<Vec<_>>()
                    .as_slice(),
                " ",
            );
            let domain = url
                .and_then(|url| url.domain().map(|d| d.to_string()))
                .unwrap_or_else(|| "".to_string());

            write!(
                screen,
                "{score} {title} {tags} {domain}\r\n",
                score = Fixed(248).paint(score),
                title = Style::new().fg(Fixed(33)).bold().paint(story.title),
                tags = tags,
                domain = Style::new().fg(Fixed(245)).italic().paint(domain)
            )
            .map_err(lobsters::Error::Io)?;
            write!(screen, "{}\r\n", Fixed(250).paint(meta)).map_err(lobsters::Error::Io)?;
        }

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
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

fn comment(_rt: &mut Runtime, _client: Client, _options: Comment) -> CommandResult {
    println!("Not implemented yet");
    Ok(())
}

// fn old() {

//     // let work = client.index(None);
//     // let stories = rt.block_on(work).expect("error fetching stories");
//     // dbg!(&stories);

//     // https://lobste.rs/s/yon0hz/rebuilding_my_personal_infrastructure
//     let story_id = StoryId("d6qvya".to_string());
//     let work = client.story(&story_id);
//     let story = rt.block_on(work).expect("error fetching story");
//     dbg!(&story);

//     let comment = NewComment {
//         story_id: story_id,
//         comment: "Hello from Rust!".to_string(),
//         hat_id: None,
//         parent_comment_short_id: None,
//     };

//     // let login = client.login(username, password);
//     let create_comment = client.post_comment(comment);
//     // let work = login.and_then(|_res| create_comment);
//     let comment_url = rt.block_on(create_comment).expect("error posting comment");

//     if let Some(comment_url) = comment_url {
//         println!("Comment posted: {}", comment_url);
//     }

//     client.save_cookies().expect("unable to save cookies");
// }
//

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

impl Colour for Tag {
    fn colour(&self) -> ansi_term::Colour {
        if self.tag == "ask" {
            Fixed(1)
        } else if self.tag == "meta" {
            Fixed(252)
        } else if self.is_media {
            Fixed(195)
        } else {
            Fixed(229)
        }
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

    pub fn get_coloured<'a>(&self, name: &'a ShortTag) -> Option<ANSIString<'a>> {
        self.tags
            .get(&name.0)
            .map(|tag| tag.colour().paint(name.0.clone()))
    }
}
