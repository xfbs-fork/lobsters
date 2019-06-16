//! Errors

use std::io;

/// The main error type of the library
#[derive(Debug)]
pub enum Error {
    /// An error related to performing a HTTP request
    Http(reqwest::Error),
    /// An attempt was made to convert data into a string that was not valid UTF-8
    InvalidStr,
    /// An I/O error
    Io(io::Error),
    /// An attempt to parse a string that was not a valid URL
    Url(url::ParseError),
    /// User home directory could not be determined
    HomeNotFound,
    /// An error related to maintaining the cookie store
    CookieStore,
    /// A desired HTML element was unable to be found in the markup
    MissingHtmlElement,
    /// The authenticity token that is required to login via 2fa is missing.
    MissingAuthenticityToken,
    /// The request was not authorised or login attemp failed
    Authorisation,
    /// Needs 2fa token
    Needs2FA,
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
