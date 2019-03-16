use std::io;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    InvalidStr,
    Io(io::Error),
    Login(LoginError),
    Url(url::ParseError),
    HomeNotFound,
    CookieStore,
    MissingHtmlElement,
}

#[derive(Debug)]
pub enum LoginError {
    NoCookie,
    InvalidCookie,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::Url(error)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
