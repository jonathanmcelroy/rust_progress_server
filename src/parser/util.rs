use std::ascii::AsciiExt;

use combine::{many, many1, satisfy, choice};
use combine::char::{char, letter, alpha_num, crlf, newline, string, string_cmp};
use combine::primitives::{Parser, Stream};

// type Parser<O> = combine::Parser<Input: &[u8], Output: O>;

pub fn till_eol<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=String> {
    many(satisfy(|c| c != '\n' && c != '\r')).skip(crlf().or(newline()))
}

pub fn one_of<I: Stream<Item=char>>(chars: &str) -> impl Parser<Input=I, Output=char> {
    let choices: Vec<_> = chars.chars().map(char).collect();
    choice(choices)
}

pub fn identifier<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=String> {
    let first = letter().or(one_of("_-"));
    let rest = many(alpha_num().or(one_of("_-")));
    (first, rest).map(|(first, rest): (_, String)| {
        let mut result = String::new();
        result.push(first);
        result.push_str(&rest);
        result
    })
}

pub fn tag_no_case<I: Stream<Item=char>>(s: &'static str) -> impl Parser<Input=I> {
    string_cmp(s, |l, r| l.eq_ignore_ascii_case(&r))
}

