// Prevents a spare console from being created attached to our program on
// windows, but only if we're running in release mode.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use chrono::prelude::*;
use chrono_humanize::HumanTime;
use futures::future::{Future, IntoFuture};
use structopt::StructOpt;
use tokio::runtime::Runtime;
use easycurses::{EasyCurses, Color, colorpair, InputMode, CursorVisibility, Input};

use lobsters::client::Page;
use lobsters::models::{Story, NewComment, ShortTag, StoryId, Tag};
use lobsters::url::{self, Url};
use lobsters::Client;

trait Colour {
    fn colour(&self) -> Color;
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
        parse(try_from_str = "parse_url")
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

struct State {
    offset: usize,
    width: usize,
    height: usize,
}

struct RenderBuffer {
    lines: Vec<String>,
}

impl RenderBuffer {
    fn render(&self, ui: &mut EasyCurses, offset: usize, width: usize, height: usize) {
        ui.move_rc(0, 0);

        for line in self.lines.iter().skip(offset).take(height) {
            line.chars().take(width).for_each(|c| { ui.print_char(c); });
        }
    }
}

fn stories(rt: &mut Runtime, client: Client, options: Stories) -> CommandResult {
    let mut ui = EasyCurses::initialize_system().unwrap();
    ui.set_title_win32("Lobsters");
    let (row_count, col_count) = ui.get_row_col_count();
    // ui.set_scroll_region(0, row_count - 1);
    // ui.set_scrolling(true);
    ui.set_echo(false);
    ui.set_input_mode(InputMode::Character);
    // ui.set_cursor_visibility(CursorVisibility::Invisible);

    let page = Page::new(options.page.unwrap_or(1));
    let future_stories = client.index(page);
    let future_tags = client.tags();
    let work = future_tags.join(future_stories);

    // Fetch tags and stories in parallel
    let (tags, stories) = rt.block_on(work)?;

    let tag_map = TagMap::new(tags);

    // Dispay the render buffer
    let mut buffer = Vec::with_capacity(stories.len() * 2);
    render_stories(&stories, &tag_map, &mut buffer)?;
    let render_buffer = RenderBuffer { lines: buffer };
    let mut state = State { offset: 0, width: col_count as usize, height: row_count as usize };

    render_buffer.render(&mut ui, state.offset, state.width, state.height);
    ui.refresh();
    loop {
        match ui.get_input() {
            Some(Input::Character('q')) => break,
            Some(Input::Character('j')) => {
                // let (mut row, col) = ui.get_cursor_rc();
                // row += 1;
                // ui.move_rc(row, col);
                state.offset += 1;
                render_buffer.render(&mut ui, state.offset, state.width, state.height);
            },
            Some(Input::Character('k')) => {
                // let (mut row, col) = ui.get_cursor_rc();
                // row -= 1;
                // ui.move_rc(row, col);
                state.offset -= 1; // do a checked sub
                render_buffer.render(&mut ui, state.offset, state.width, state.height);
            }
            Some(_) => (),
            None => (),
        }
        ui.refresh();
    }

    Ok(())
}

fn render_stories(stories: &[Story], tag_map: &TagMap, buffer: &mut Vec<String>) -> CommandResult {
    // Calculate the max number of digits so scores can be padded
    let digits = stories
        .iter()
        .map(|story| story.score.abs())
        .max()
        .map(|max| f64::from(max).log10().ceil() as usize)
        .unwrap_or(1);

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

        buffer.push(format!(
            "{score} {title} {tags} {domain}\n",
            score = score,
            title = story.title,
            tags = tags,
            domain = domain
        ));
        buffer.push(format!("{}\n", meta));
    }

    Ok(())
}

fn comment(_rt: &mut Runtime, _client: Client, _options: Comment) -> CommandResult {
    Ok(())
}

fn parse_url(src: &str) -> Result<Url, url::ParseError> {
    src.parse()
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
    fn colour(&self) -> Color {
        if self.tag == "ask" {
            Color::Red
        } else if self.tag == "meta" {
            Color::White
        } else if self.is_media {
            Color::Blue
        } else {
            Color::Yellow
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

    pub fn get_coloured<'a>(&self, name: &'a ShortTag) -> Option<String> {
        self.tags
            .get(&name.0)
            .map(|tag| name.0.clone())
    }
}
