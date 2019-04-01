use std::fs::File;
use std::sync::{Arc, Mutex};

use cookie_store::CookieStore;
use futures::{Future, IntoFuture};
use log::info;
use reqwest::header::{HeaderMap, ACCEPT, COOKIE, SET_COOKIE};
use reqwest::r#async::{Client as ReqwestClient, Response};
use serde::Serialize;
use url::Url;

use crate::error::Error;

#[derive(Clone)]
pub(super) struct HttpClient {
    base_url: Url,
    reqwest: ReqwestClient,
    cookies: Arc<Mutex<CookieStore>>,
}

impl HttpClient {
    pub(super) fn new(
        base_url: Url,
        reqwest: ReqwestClient,
        cookies: Arc<Mutex<CookieStore>>,
    ) -> Self {
        HttpClient {
            base_url,
            reqwest,
            cookies,
        }
    }

    pub(super) fn post<B>(
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

        let cookie_set: Arc<Mutex<_>> = self.cookies.clone();
        let cookie_get: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                info!("POST {}", url.as_str());

                client
                    .post(url.as_str())
                    .header("X-CSRF-Token", csrf_token)
                    .headers(Self::cookie_headers(cookie_get, &url))
                    .form(&body)
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| Self::store_cookies(res, cookie_set))
    }

    pub(super) fn get(&self, path: &str) -> impl Future<Item = Response, Error = Error> {
        let request_url = self.base_url.join(path);
        let client = self.reqwest.clone();

        let cookie_get: Arc<Mutex<_>> = self.cookies.clone();
        let cookie_set: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                info!("GET {}", url.as_str());

                client
                    .get(url.as_str())
                    .headers(Self::cookie_headers(cookie_get, &url))
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| Self::store_cookies(res, cookie_set))
    }

    pub(super) fn get_json(&self, path: &str) -> impl Future<Item = Response, Error = Error> {
        let request_url = self.base_url.join(path);
        let client = self.reqwest.clone();

        let cookie_get: Arc<Mutex<_>> = self.cookies.clone();
        let cookie_set: Arc<Mutex<_>> = self.cookies.clone();
        request_url
            .map_err(Error::from)
            .into_future()
            .and_then(move |url| {
                client
                    .get(url.as_str())
                    .header(ACCEPT, "application/json")
                    .headers(Self::cookie_headers(cookie_get, &url))
                    .send()
                    .map_err(Error::from)
            })
            .map(move |res| Self::store_cookies(res, cookie_set))
    }

    pub(super) fn save_cookies(&self, file: &mut File) -> Result<(), Error> {
        self.cookies
            .lock()
            .unwrap()
            .save_json(file)
            .map_err(|_err| Error::CookieStore)
    }

    pub(super) fn base_url(&self) -> &Url {
        &self.base_url
    }

    fn cookie_headers(cookies: Arc<Mutex<CookieStore>>, url: &Url) -> HeaderMap {
        // Add cookies to request
        let store = cookies.lock().unwrap();
        let cookies = store.matches(url);

        cookies
            .iter()
            .fold(HeaderMap::new(), |mut headers, cookie| {
                // NOTE(unwrap): Assumed to be safe since it was valid when put into the store
                headers.append(COOKIE, cookie.encoded().to_string().parse().unwrap());
                headers
            })
    }

    fn store_cookies(res: Response, cookies: Arc<Mutex<CookieStore>>) -> Response {
        res.headers().get_all(SET_COOKIE).iter().for_each(|cookie| {
            cookie
                .to_str()
                .ok()
                .and_then(|cookie| cookies.lock().unwrap().parse(cookie, res.url()).ok());
        });

        res
    }
}
