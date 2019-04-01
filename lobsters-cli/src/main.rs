use std::io::{stdin, stdout, Write};
use std::str::FromStr;

use env_logger::Env;
use futures::future::Future;
use structopt::StructOpt;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tokio::runtime::Runtime;

use lobsters::client::Page;
use lobsters::url::Url;
use lobsters::Client;

use lobsters_cli::{
    app::State,
    error::{Error, ParseThemeError},
    render::{render_lines, render_stories},
    theme::themes::*,
    util,
};

const HORIZONTAL_SCROLL_AMOUNT: usize = 10;

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

#[derive(Debug)]
enum UiTheme {
    Grey,
    Color256,
    Mono,
    TrueColor,
}

type CommandResult = Result<(), Error>;

fn main() {
    let env = Env::new().filter("LOBSTERS_LOG");
    env_logger::init_from_env(env);

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
        Err(Error::Lobsters(lobsters::Error::Authorisation)) => eprintln!("Error: Not authorised"),
        Err(Error::InvalidDate(err)) => eprintln!("Unable to parse date: {:?}", err),
        Err(Error::NotATty) => {
            eprintln!("Error: This program needs a tty (you can't pipe or redirect its output)")
        }
    }

    if result.is_err() {
        std::process::exit(1);
    }
}

fn login(rt: &mut Runtime, client: Client, _options: Login) -> CommandResult {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    stdout.write_all(b"username: ")?;
    stdout.flush()?;

    let username = stdin.read_line()?;
    if username.is_none() {
        return Ok(());
    }

    stdout.write_all(b"password: ")?;
    stdout.flush()?;

    let password = stdin.read_passwd(&mut stdout)?;
    if password.is_none() {
        return Ok(());
    }
    let _ = stdout.write_all(b"\n");

    // NOTE: unwrap is safe due to is_none checks above
    let login = client.login(username.unwrap(), password.unwrap());

    let res = rt.block_on(login);

    if res.is_ok() {
        // Ignore result since they have successfully logged in, no point showing an error now
        let _ = stdout.write_all(b"Ok\n");
    }

    res.map_err(Error::from)
}

fn stories(rt: &mut Runtime, client: Client, options: Stories) -> CommandResult {
    let page = Page::new(options.page.unwrap_or(1));
    let future_stories = client.index(page);
    let future_tags = client.tags();
    let work = future_tags.join(future_stories);

    if !termion::is_tty(&stdout()) {
        return Err(Error::NotATty);
    }

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

    let mut state = State::new(stories, tags);

    // Switch to alternate screen and enter main loop
    {
        let screen = AlternateScreen::from(stdout());
        let mut screen = screen.into_raw_mode()?;
        let stdin = stdin();

        // Hide the cursor
        write!(screen, "{}", cursor::Hide)?;

        let theme = match options.theme {
            UiTheme::Color256 => &LOBSTERS_256,
            UiTheme::TrueColor => &LOBSTERS_TRUE,
            UiTheme::Mono => &LOBSTERS_MONO,
            UiTheme::Grey => &LOBSTERS_GREY,
        };

        // Render initial UI
        let mut lines = render_stories(&mut state, theme, height)?;
        render_lines(&lines, &mut screen, state.col_offset())?;

        // Main loop
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') | Key::Esc => break,
                Key::Char('j') | Key::Up => {
                    if state.next_story() {
                        lines = render_stories(&mut state, theme, height)?;
                        render_lines(&lines, &mut screen, state.col_offset())?;
                    }
                }
                Key::Char('k') | Key::Down => {
                    if state.prev_story() {
                        lines = render_stories(&mut state, theme, height)?;
                        render_lines(&lines, &mut screen, state.col_offset())?;
                    }
                }
                Key::Char('h') => {
                    if state.scroll_left(HORIZONTAL_SCROLL_AMOUNT) {
                        render_lines(&lines, &mut screen, state.col_offset())?;
                    }
                }
                Key::Char('l') => {
                    if state.scroll_right(HORIZONTAL_SCROLL_AMOUNT) {
                        render_lines(&lines, &mut screen, state.col_offset())?;
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

        // Restore the cursor before returning
        write!(screen, "{}", cursor::Show)?;
    }

    Ok(())
}

impl Default for Command {
    fn default() -> Self {
        Command::Stories(Stories::default())
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
