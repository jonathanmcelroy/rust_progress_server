use std::convert::From;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Write, stderr};
use std::process::exit;

use docopt;
use hyper;
use ini::ini;
use serde_json;
use url;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Ini(ini::Error),
    Hyper(hyper::Error),
    Docopt(docopt::Error),
    JSON(serde_json::Error),
    Url(url::ParseError),
    General(&'static str),
}

pub type ProgressResult<T> = Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::Io(err) }
} 
impl From<ini::Error> for Error {
    fn from(err: ini::Error) -> Error { Error::Ini(err) }
} 
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error { Error::Hyper(err) }
}
impl From<docopt::Error> for Error {
    fn from(err: docopt::Error) -> Error { Error::Docopt(err) }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error { Error::JSON(err) }
}
impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error { Error::Url(err) }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => write!(f, "{}", err),
            &Error::Ini(ref err) => write!(f, "{}", err),
            &Error::Hyper(ref err) => write!(f, "{}", err),
            &Error::Docopt(ref err) => write!(f, "{}", err),
            &Error::JSON(ref err) => write!(f, "{}", err),
            &Error::Url(ref err) => write!(f, "{}", err),
            &Error::General(ref s) => write!(f, "Error: {}", s),
        }
    }
}

pub fn unwrap_or_exit<T, E: Into<Error>>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => return t,
        Err(err) => {
            let _ = writeln!(&mut stderr(), "{}", err.into());
            exit(1);
        }
    }
}
