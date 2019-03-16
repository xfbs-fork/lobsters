#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Url(url::ParseError),
    Login(LoginError),
    InvalidStr,
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
