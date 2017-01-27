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
use nom;
use combine::primitives::{Consumed, ParseResult, ParseError, StreamOnce};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Ini(ini::Error),
    Hyper(hyper::Error),
    Docopt(docopt::Error),
    JSON(serde_json::Error),
    Url(url::ParseError),
    Nom(nom::IError),
    ParseError(String),
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
impl From<nom::IError> for Error {
    fn from(err: nom::IError) -> Error { Error::Nom(err) }
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
            &Error::Nom(ref err) => write!(f, "{:?}", err),
            &Error::ParseError(ref err) => write!(f, "{}", err),
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

pub fn from<O, I>(parse_result: ParseResult<O, I>) -> Result<O, Error> 
    where I: StreamOnce,
          <I as StreamOnce>::Range: fmt::Debug,
          <I as StreamOnce>::Item: fmt::Debug {
    parse_result.map(|(value, consumed)| value)
        .map_err(|consumed_err| {
            let errs = consumed_err.into_inner().errors;
            let ref err = errs[0];
            Error::ParseError(format!("{:?}", err))
        })
}
