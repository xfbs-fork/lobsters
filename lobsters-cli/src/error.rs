use lobsters::url;
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct ParseThemeError(pub String);

#[derive(Debug)]
pub enum Error {
    Lobsters(lobsters::Error),
    InvalidDate(chrono::ParseError),
    NotATty,
}

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

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Lobsters(lobsters::Error::Io(err))
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Lobsters(lobsters::Error::Url(err))
    }
}

impl fmt::Display for ParseThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' is not a valid theme. Options are: true, 256, mono, grey or gray",
            self.0
        )
    }
}
