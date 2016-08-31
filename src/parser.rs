use nom::{IResult, digit};
use nom::IResult::*;

use std::str;

pub enum PreprocessedProgress {
    PreprocessorLine(String),
    Import(String),
    Replace(String),
    Code(String),
    Comment(String),
}

named!(preprocessor_line<PreprocessedProgress>,
    chain!(
        char!('&') ~
        b: take_until_and_consume!("\n"),
        || {PreprocessedProgress::PreprocessorLine(String::from_utf8_lossy(b).into_owned())}
    )
);
