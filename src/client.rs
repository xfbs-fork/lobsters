use std::fs::{self, DirBuilder, File};
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cookie_store::CookieStore;
use directories::ProjectDirs;
use futures::{Future, IntoFuture, Stream};
use reqwest::header::{HeaderMap, ACCEPT, COOKIE, SET_COOKIE};
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder, Response};
use serde::Serialize;
use url::Url;

use crate::error::Error;
use crate::models::{Story, StoryId};

struct HttpClient {
    base_url: Url,
    reqwest: ReqwestClient,
    cookies: Arc<Mutex<CookieStore>>,
}

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

        let client = ClientBuilder::new().use_rustls_tls().build()?;
        let http = HttpClient {
            base_url,
            reqwest: client,
            cookies: Arc::new(Mutex::new(cookies)),
        };

        Ok(Client { http })
    }

    // FIXME: Return the unauthenticated client back?
    /// Attempt to authenticate with the server
    pub fn login(
        &self,
        username_or_email: String,
        password: String,
    ) -> impl Future<Item = (), Error = Error> {
        let params = [("email", username_or_email), ("password", password)];

        self.http
            .post_unauthenticated("login", params)
            .and_then(|res| res.into_body().concat2().map_err(Error::from))
            .and_then(|body| {
                let b = std::str::from_utf8(&body).unwrap();
                eprintln!("body = {}", b);

                // TODO: Determine success

                futures::future::ok(())
            })
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
            self.http
                .cookies
                .lock()
                .unwrap()
                .save_json(&mut tmp_file)
                .map_err(|_err| Error::CookieStore)?;
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
}

impl HttpClient {
    fn post_unauthenticated<B>(
        &self,
        path: &str,
        body: B,
    ) -> impl Future<Item = Response, Error = Error>
    where
        B: Serialize,
    {
        let request_url = self.base_url.join(path);
        let client = self.reqwest.clone();

        // This reports a recursion limit error that isn't fixed by increasing the limit ¯\_(ツ)_/¯
        // futures::future::ok(self.reqwest.clone())
        //     .and_then(|client| request_url.map(|url| (url, client)).map_err(Error::from))
        //     .and_then(move |(url, client)| {
        //         eprintln!("POST {}", url.as_str());
        //         client.post(url.as_str())
        //             .form(&body)
        //             .send()
        //             .map_err(Error::from)
        //     })

        let cookies: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                client
                    .post(url.as_str())
                    .form(&body)
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| {
                res.headers().get_all(SET_COOKIE).iter().for_each(|cookie| {
                    cookie
                        .to_str()
                        .ok()
                        .and_then(|cookie| cookies.lock().unwrap().parse(cookie, res.url()).ok());
                });

                res
            })
    }

    fn get_json(&self, path: &str) -> impl Future<Item = Response, Error = Error> {
        let request_url = self.base_url.join(path);
        let client = self.reqwest.clone();

        let cookie_get: Arc<Mutex<_>> = self.cookies.clone();
        let cookie_set: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                // Add cookies to request
                let store = cookie_get.lock().unwrap();
                let cookies = store.matches(&url);
                let cookie_headers =
                    cookies
                        .iter()
                        .fold(HeaderMap::new(), |mut headers, cookie| {
                            // NOTE(unwrap): Assumed to be safe since it was valid when put into the store
                            headers.append(COOKIE, cookie.encoded().to_string().parse().unwrap());
                            headers
                        });
                dbg!(&cookie_headers);

                client
                    .get(url.as_str())
                    .header(ACCEPT, "application/json")
                    .headers(cookie_headers)
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| {
                res.headers().get_all(SET_COOKIE).iter().for_each(|cookie| {
                    cookie.to_str().ok().and_then(|cookie| {
                        cookie_set.lock().unwrap().parse(cookie, res.url()).ok()
                    });
                });

                res
            })
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
