use ansi_term::Colour::Fixed;
use ansi_term::Style;
use ansi_term::{ANSIString, ANSIStrings};
use chrono::prelude::*;
use chrono_humanize::HumanTime;
use futures::future::{Future, IntoFuture};
use structopt::StructOpt;
use tokio::runtime::Runtime;

use lobsters::client::Page;
use lobsters::models::{NewComment, StoryId};
use lobsters::url::{self, Url};
use lobsters::Client;

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

fn login(rt: &mut Runtime, client: Client, options: Login) -> CommandResult {
    Ok(())
}

fn stories(rt: &mut Runtime, client: Client, options: Stories) -> CommandResult {
    let page = Page::new(options.page.unwrap_or(1));
    let work = client.index(page);
    let stories = rt.block_on(work)?;

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
                .map(|tag| tag.0.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
            " ",
        );
        let domain = url
            .and_then(|url| url.domain().map(|d| d.to_string()))
            .unwrap_or_else(|| "".to_string());

        println!(
            "{score} {title} {tags} {domain}",
            score = Fixed(248).paint(score),
            title = Fixed(33).paint(story.title),
            tags = tags,
            domain = Style::new().italic().paint(domain)
        );
        println!("{}", Fixed(245).paint(meta));
    }

    Ok(())
}

fn comment(rt: &mut Runtime, client: Client, options: Comment) -> CommandResult {
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
