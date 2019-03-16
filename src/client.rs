use std::fs::{self, DirBuilder, File};
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cookie_store::CookieStore;
use directories::ProjectDirs;
use futures::{Future, IntoFuture, Stream};
use reqwest::header::SET_COOKIE;
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder, Response};
use serde::Serialize;
use url::Url;

use crate::error::Error;

pub const LOBSTERS: &str = "https://lobste.rs/";

struct HttpClient {
    base_url: Url,
    reqwest: ReqwestClient,
    cookies: Arc<Mutex<CookieStore>>,
}

pub struct UnauthenticatedClient {
    http: HttpClient,
}

pub struct AuthenticatedClient {
    http: HttpClient,
}

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

impl UnauthenticatedClient {
    /// Create a new client that can be used to log in
    pub fn new(base_url: Url) -> Result<Self, Error> {
        let client = ClientBuilder::new().use_rustls_tls().build()?;

        let http = HttpClient {
            base_url,
            reqwest: client,
            cookies: Arc::new(Mutex::new(CookieStore::default())),
        };

        Ok(UnauthenticatedClient { http })
    }

    // FIXME: Return the unauthenticated client back?
    /// Attempt to authenticate with the server
    pub fn login(
        self,
        username_or_email: String,
        password: String,
    ) -> impl Future<Item = AuthenticatedClient, Error = Error> {
        let params = [("email", username_or_email), ("password", password)];

        self.http
            .post_unauthenticated("login", params)
            .and_then(|res| res.into_body().concat2().map_err(Error::from))
            .and_then(|body| {
                let b = std::str::from_utf8(&body).unwrap();
                eprintln!("body = {}", b);

                // TODO: Determine success

                futures::future::ok(AuthenticatedClient { http: self.http })
            })
    }
}

impl AuthenticatedClient {
    /// Load the cookie store and return an AuthenticatedClient
    ///
    /// Returns an error if the cookie store does not exist or there is a problem loading it.
    pub fn new(base_url: Url) -> Result<Self, Error> {
        let cookie_store_path = cookie_store_path()?;

        if !cookie_store_path.exists() {
            return Err(Error::CookieStore);
        }

        let cookie_file = BufReader::new(File::open(cookie_store_path)?);
        let cookies = CookieStore::load_json(cookie_file).map_err(|_err| Error::CookieStore)?;

        let client = ClientBuilder::new().use_rustls_tls().build()?;
        let http = HttpClient {
            base_url,
            reqwest: client,
            cookies: Arc::new(Mutex::new(cookies)),
        };

        Ok(AuthenticatedClient { http })
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

        // This hits the recursion limit of the compiler ¯\_(ツ)_/¯
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
}
