use std::fs::{self, DirBuilder, File};
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cookie_store::CookieStore;
use directories::ProjectDirs;
use futures::{Future, IntoFuture, Stream};
use kuchiki::traits::TendrilSink;
use reqwest::header::{HeaderMap, ACCEPT, COOKIE, LOCATION, SET_COOKIE};
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder, Response};
use reqwest::RedirectPolicy;
use serde::Serialize;
use url::Url;

use crate::error::Error;
use crate::models::{NewComment, Story, StoryId};

#[derive(Clone)]
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

        let client = ClientBuilder::new()
            .redirect(RedirectPolicy::none())
            .use_rustls_tls()
            .build()?;
        let http = HttpClient {
            base_url,
            reqwest: client,
            cookies: Arc::new(Mutex::new(cookies)),
        };

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
        // TODO: This is super ugly, tidy up
        // Would be good to split the response/futures handing from the html handling
        res.into_body()
            .concat2()
            .map_err(Error::from)
            .and_then(|body| {
                std::str::from_utf8(&body)
                    .map_err(|_err| Error::InvalidStr)
                    .and_then(|body| {
                        let html = kuchiki::parse_html().one(body);
                        html.select_first("meta[name='csrf-token']")
                            .ok()
                            .and_then(|input| {
                                let attrs = input.attributes.borrow();
                                attrs.get("content").map(|content| {
                                    dbg!(&content);
                                    content.to_string()
                                })
                            })
                            .ok_or_else(|| Error::MissingHtmlElement)
                    })
            })
            .into_future()
    }
}

impl HttpClient {
    fn post<B>(
        &self,
        path: &str,
        body: B,
        csrf_token: String,
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

        let cookie_set: Arc<Mutex<_>> = self.cookies.clone();
        let cookie_get: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                eprintln!("POST {}", url.as_str());

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
                    .post(url.as_str())
                    .header("X-CSRF-Token", csrf_token)
                    .headers(cookie_headers)
                    .form(&body)
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| {
                dbg!(res.headers());
                res.headers().get_all(SET_COOKIE).iter().for_each(|cookie| {
                    dbg!(&cookie);
                    cookie.to_str().ok().and_then(|cookie| {
                        cookie_set.lock().unwrap().parse(cookie, res.url()).ok()
                    });
                });

                res
            })
    }

    fn get(&self, path: &str) -> impl Future<Item = Response, Error = Error> {
        let request_url = self.base_url.join(path);
        let client = self.reqwest.clone();

        let cookie_get: Arc<Mutex<_>> = self.cookies.clone();
        let cookie_set: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                eprintln!("GET {}", url.as_str());

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
                    .headers(cookie_headers)
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| {
                res.headers().get_all(SET_COOKIE).iter().for_each(|cookie| {
                    eprintln!("Set-Cookie: {:?}", cookie);
                    cookie.to_str().ok().and_then(|cookie| {
                        cookie_set.lock().unwrap().parse(cookie, res.url()).ok()
                    });
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
