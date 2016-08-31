use nom::{IResult, digit};
use nom::IResult::*;

use std::str;
use std::vec;

#[derive(Debug)]
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

named!(preprocessor_import<PreprocessedProgress>,
    map!(
        delimited!(
            char!('{'),
            take_until!("}"),
            char!('}')
        ),
        |b| PreprocessedProgress::Import(String::from_utf8_lossy(b).into_owned())
    )
);

named!(preprocessor_replace<PreprocessedProgress>,
    map!(
        delimited!(
            char!('{'),
            digit,
            char!('}')
        ),
        |b| PreprocessedProgress::Replace(String::from_utf8_lossy(b).into_owned())
    )
);

named!(comment<PreprocessedProgress>,
    map!(
        delimited!(
            tag!("/*"),
            many0!(
                alt!(
                    comment |
                    anychar
                )
            )
            tag!("*/")
        ),
        |b| PreprocessedProgress::Comment(String::from_utf8_lossy(b).into_owned())
    )
);

named!(pub preprocessed_progress<Vec<PreprocessedProgress> >,
    many1!(
        alt!(
            preprocessor_line |
            preprocessor_replace |
            preprocessor_import |
            map!(take_until_either!("{&"), |b| PreprocessedProgress::Code(String::from_utf8_lossy(b).into_owned()))
        )
    )
);

