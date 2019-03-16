use cookie::Cookie;
use futures::{Future, IntoFuture, Stream};
use reqwest::header::SET_COOKIE;
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use crate::error::{Error, LoginError};

pub const LOBSTERS: &str = "https://lobste.rs/";

struct HttpClient {
    base_url: Url,
    reqwest: ReqwestClient,
}

pub struct UnauthenticatedClient {
    http: HttpClient,
}

pub struct Client {}

impl UnauthenticatedClient {
    pub fn new(base_url: Url) -> Result<Self, Error> {
        let client = ClientBuilder::new().use_rustls_tls().build()?;

        let http = HttpClient {
            base_url,
            reqwest: client,
        };

        Ok(UnauthenticatedClient { http })
    }

    // FIXME: Return the unauthenticated client back?
    pub fn login(
        self,
        username_or_email: String,
        password: String,
    ) -> impl Future<Item = Client, Error = Error> {
        let params = [("email", username_or_email), ("password", password)];

        self.http
            .post_unauthenticated("login", params)
            .and_then(|res| {
                res.headers()
                    .get(SET_COOKIE)
                    .ok_or_else(|| Error::Login(LoginError::NoCookie))
                    .and_then(|cookie| {
                        cookie
                            .to_str()
                            .map_err(|_err| Error::InvalidStr)
                            .and_then(|cookie| {
                                Cookie::parse(cookie)
                                    .map_err(|_err| Error::Login(LoginError::InvalidCookie))
                            })
                            .map(|cookie| cookie.into_owned())
                    })
                    .map(|cookie| (res, cookie))
            })
            .and_then(|(res, cookie)| {
                res.into_body()
                    .concat2()
                    .map_err(Error::from)
                    .map(|body| (body, cookie))
            })
            .and_then(|(body, cookie)| {
                let b = std::str::from_utf8(&body).unwrap();
                eprintln!("body = {}", b);
                // let user = serde_json::from_slice::<user::User>(&body); //.or_else(|| serde_json::from_slice::<ErrorBody>(&body))
                dbg!(&cookie);

                futures::future::ok(Client {})
            })
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
    }
}
