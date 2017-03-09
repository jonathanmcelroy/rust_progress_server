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
use combine::primitives::{Consumed, ParseResult, ParseError, StreamOnce};

#[derive(Debug)]
pub enum FromError {
    Io(io::Error),
    Ini(ini::Error),
    Hyper(hyper::Error),
    Docopt(docopt::Error),
    JSON(serde_json::Error),
    Url(url::ParseError),
}

#[derive(Debug)]
pub enum Error {
    FromError(FromError),
    FromErrorMessage(FromError, String),
    ParseError(String),
    General(String),
}

impl Error {
    pub fn new<S>(s: S) -> Self where S: Into<String>{
        Error::General(s.into())
    }

    pub fn add_message<S>(self, s: S) -> Self where S: Into<String>{
        match self {
            Error::FromError(from_error) => Error::FromErrorMessage(from_error, s.into()),
            _ => panic!("Adding a message to an error with a message"),
        }
    }
}

pub type ProgressResult<T> = Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::FromError(FromError::Io(err)) }
} 
impl From<ini::Error> for Error {
    fn from(err: ini::Error) -> Error { Error::FromError(FromError::Ini(err)) }
} 
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error { Error::FromError(FromError::Hyper(err)) }
}
impl From<docopt::Error> for Error {
    fn from(err: docopt::Error) -> Error { Error::FromError(FromError::Docopt(err)) }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error { Error::FromError(FromError::JSON(err)) }
}
impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error { Error::FromError(FromError::Url(err)) }
}

impl Display for FromError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &FromError::Io(ref err) => write!(f, "{}", err),
            &FromError::Ini(ref err) => write!(f, "{}", err),
            &FromError::Hyper(ref err) => write!(f, "{}", err),
            &FromError::Docopt(ref err) => write!(f, "{}", err),
            &FromError::JSON(ref err) => write!(f, "{}", err),
            &FromError::Url(ref err) => write!(f, "{}", err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Error::FromError(ref from_error) => write!(f, "{}", from_error),
            &Error::FromErrorMessage(ref from_error, ref s) => write!(f, "{}: {}", s, from_error),
            &Error::ParseError(ref s) => write!(f, "Parse Error: {}", s),
            &Error::General(ref s) => write!(f, "Custom Error: {}", s),
        }
    }
}

pub fn from<O, I>(parse_result: ParseResult<O, I>) -> Result<O, Error> 
    where I: StreamOnce,
          <I as StreamOnce>::Range: fmt::Debug,
          <I as StreamOnce>::Item: fmt::Debug {
    parse_result.map(|(value, consumed)| value)
        .map_err(|consumed_err| {
            let errs = consumed_err.into_inner().errors;
            Error::ParseError(format!("{:?}", errs))
        })
}

pub fn add_message<S, E>(s: S) -> impl FnOnce(E) -> Error where S: Into<String>, Error: From<E> {
    |err| {
        Error::from(err).add_message(s)
    }
}
