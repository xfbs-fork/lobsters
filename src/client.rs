mod http_client;

use std::fs::{self, DirBuilder, File};
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cookie_store::CookieStore;
use directories::ProjectDirs;
use futures::{Future, IntoFuture, Stream};
use kuchiki::traits::TendrilSink;
use reqwest::header::LOCATION;
use reqwest::r#async::{ClientBuilder, Response};
use reqwest::RedirectPolicy;
use url::Url;

use crate::error::Error;
use crate::models::{NewComment, Story, StoryId};

use http_client::HttpClient;

pub struct Client {
    http: HttpClient,
}

pub struct Page(u32);

fn config_path() -> Result<PathBuf, Error> {
    ProjectDirs::from("rs", "lobste", env!("CARGO_PKG_NAME"))
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
        .ok_or_else(|| Error::HomeNotFound)
}

fn cookie_store_path() -> Result<PathBuf, Error> {
    let mut cookie_store_path = config_path()?;
    cookie_store_path.push("cookies.json");
    Ok(cookie_store_path)
}

impl Client {
    /// Create a new client
    ///
    /// Will attempt to load the cookie store if it exists.
    pub fn new(base_url: Url) -> Result<Self, Error> {
        let cookie_store_path = cookie_store_path()?;

        let cookies = if cookie_store_path.exists() {
            let cookie_file = BufReader::new(File::open(cookie_store_path)?);
            CookieStore::load_json(cookie_file).map_err(|_err| Error::CookieStore)?
        } else {
            CookieStore::default()
        };

        let client = ClientBuilder::new()
            .redirect(RedirectPolicy::none())
            .use_rustls_tls()
            .build()?;
        let http = HttpClient::new(base_url, client, Arc::new(Mutex::new(cookies)));

        Ok(Client { http })
    }

    /// Attempt to authenticate with the server
    pub fn login(
        &self,
        username_or_email: String,
        password: String,
    ) -> impl Future<Item = (), Error = Error> {
        let get_token = self.http.get("login").and_then(Self::extract_csrf_token);

        let client = self.http.clone();
        let login = move |token| {
            let params = [("email", username_or_email), ("password", password)];

            client
                .post("login", params, token)
                .and_then(|res| {
                    eprintln!("{:?}", res.status());
                    res.into_body().concat2().map_err(Error::from)
                })
                .and_then(|body| {
                    let b = std::str::from_utf8(&body).unwrap();
                    eprintln!("login body = {}", b);

                    // TODO: Determine success

                    futures::future::ok(())
                })
        };

        get_token.and_then(login)
    }

    /// Save the cookie store so that a client can be created without needing to log in first
    pub fn save_cookies(&self) -> Result<(), Error> {
        let cookie_store_path = cookie_store_path()?;
        let cookie_store_tmp_path = cookie_store_path.with_extension("tmp");

        // Ensure the directory the cookie file is stored in exists
        let config_dir = cookie_store_path.parent().ok_or_else(|| {
            Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "unable to find parent dir of cookie file",
            ))
        })?;

        if !config_dir.exists() {
            DirBuilder::new().recursive(true).create(config_dir)?;
        }

        {
            // Write out the file entirely
            let mut tmp_file = File::create(&cookie_store_tmp_path)?;
            self.http.save_cookies(&mut tmp_file)?;
        }

        // Move into place atomically
        fs::rename(cookie_store_tmp_path, cookie_store_path).map_err(Error::from)
    }

    /// Retrieve the front page stories, newest first
    pub fn index(&self, page: Option<Page>) -> impl Future<Item = Vec<Story>, Error = Error> {
        let path = page
            .map(|Page(page): Page| format!("page/{}", page))
            .unwrap_or_else(|| "".to_string());

        self.http
            .get_json(&path)
            .and_then(|mut res| res.json::<Vec<Story>>().map_err(Error::from))
    }

    /// Retrieve the comments for a story
    pub fn story(&self, story_id: &StoryId) -> impl Future<Item = Story, Error = Error> {
        let path = format!("s/{}", story_id.0);

        self.http
            .get_json(&path)
            .and_then(|mut res| res.json::<Story>().map_err(Error::from))
    }

    pub fn post_comment(
        &self,
        comment: NewComment,
    ) -> impl Future<Item = Option<String>, Error = Error> {
        // Need to fetch a page to get a CSRF token, /about seems like one of the cheapest
        // pages to fetch
        let get_token = self.http.get("about").and_then(Self::extract_csrf_token);

        let client = self.http.clone();
        let comment = move |token| {
            client
                .post("comments", comment, token)
                .and_then(|res| {
                    let location = res
                        .headers()
                        .get(LOCATION)
                        .and_then(|header| header.to_str().ok())
                        .map(|s| s.to_string());
                    res.into_body()
                        .concat2()
                        .map_err(Error::from)
                        .map(|body| (location, body))
                })
                .and_then(|(location, body)| {
                    let b = std::str::from_utf8(&body).unwrap();
                    eprintln!("body = {}", b);

                    // TODO: Determine success

                    futures::future::ok(location)
                })
        };

        get_token.and_then(comment)
    }

    fn extract_csrf_token(res: Response) -> impl Future<Item = String, Error = Error> {
        res.into_body()
            .concat2()
            .map_err(Error::from)
            .and_then(|body| {
                std::str::from_utf8(&body)
                    .map_err(|_err| Error::InvalidStr)
                    .and_then(Self::extract_csrf_token_from_html)
            })
            .into_future()
    }

    fn extract_csrf_token_from_html(body: &str) -> Result<String, Error> {
        let html = kuchiki::parse_html().one(body);
        html.select_first("meta[name='csrf-token']")
            .ok()
            .and_then(|input| {
                let attrs = input.attributes.borrow();
                attrs.get("content").map(|content| content.to_string())
            })
            .ok_or_else(|| Error::MissingHtmlElement)
    }
}

impl Page {
    pub fn new(page: u32) -> Option<Page> {
        if page > 1 {
            Some(Page(page))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_csrf_token_from_html_success() {
        let html = r#"<html><head><meta name="csrf-token" content="token" /></head></html>"#;
        assert_eq!(
            Client::extract_csrf_token_from_html(html).unwrap(),
            "token".to_string()
        );
    }

    #[test]
    fn extract_csrf_token_from_html_faile() {
        let html = r#"<html><head><title>No token</title></head></html>"#;
        match Client::extract_csrf_token_from_html(html) {
            Err(Error::MissingHtmlElement) => (),
            other => panic!("Expected Error::MissingHtmlElement got {:?}", other),
        }
    }
}
