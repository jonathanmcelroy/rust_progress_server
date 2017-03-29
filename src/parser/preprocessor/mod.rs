mod analysis_suspend;

use std::fmt;
use combine::{not_followed_by, any, choice, many, many1, satisfy, try, value, sep_by1};
use combine::combinator::{Value, parser, optional};
use combine::primitives::{Parser, Stream, ParseResult};
use combine::char::{char, digit, string, spaces};
use util::{restrict_string};
use parser::util::{identifier, till_eol, tag_no_case};
use error::{ProgressResult, Error};

use self::analysis_suspend::{AnalysisSuspendHeader, analyze_suspend, analyze_resume};
pub use self::analysis_suspend::{
    CodeBlockType
};

#[derive(Clone, PartialEq)]
pub enum PreprocessorASTNode {
    AnalysisSuspend(AnalysisSuspendHeader),
    AnalysisResume,
    PreprocessorLine(String),
    Import(String),
    Replace(String),
    Code(String),
    Comment(String),
}

impl fmt::Debug for PreprocessorASTNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PreprocessorASTNode::AnalysisSuspend(ref analysis_suspend_header) => write!(f, "AnalysisSuspend({:?})", analysis_suspend_header),
            &PreprocessorASTNode::AnalysisResume => write!(f, "AnalysisResume"),
            &PreprocessorASTNode::PreprocessorLine(ref preprocessor_line) => write!(f, "PreprocessorLine({:?})", preprocessor_line),
            &PreprocessorASTNode::Import(ref import) => write!(f, "Import({:?})", import),
            &PreprocessorASTNode::Replace(ref replace) => write!(f, "Replace({:?})", replace),
            &PreprocessorASTNode::Code(ref contents) => write!(f, "Code({:?})", restrict_string(contents)),
            &PreprocessorASTNode::Comment(ref contents) => write!(f, "Comment({:?})", restrict_string(contents))
        }
    }
}

impl PreprocessorASTNode {
    fn get_contents(&self) -> Option<&str> {
        match self {
            &PreprocessorASTNode::AnalysisSuspend(_) => None,
            &PreprocessorASTNode::AnalysisResume => None,
            &PreprocessorASTNode::PreprocessorLine(_) => None,
            &PreprocessorASTNode::Import(_) => None,
            &PreprocessorASTNode::Replace(_) => None,
            &PreprocessorASTNode::Code(ref contents) => Some(contents),
            &PreprocessorASTNode::Comment(ref contents) => Some(contents)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PreprocessorAnalysisSection {
    NotInSection {contents: String },
    VersionNumber,
    PreprocessorBlock { contents: String },
    ProcedureSettings { contents: String },
    CreateWindow { contents: String, attributes: Vec<(String, f32)> },
    CodeBlock { block_type: CodeBlockType, contents: String } ,
    Other { block_type: String, contents: String }
}

impl PreprocessorAnalysisSection {
    fn new(header: AnalysisSuspendHeader, contents: String) -> PreprocessorAnalysisSection {
        match header {
            AnalysisSuspendHeader::VersionNumber => PreprocessorAnalysisSection::VersionNumber,
            AnalysisSuspendHeader::PreprocessorBlock => PreprocessorAnalysisSection::PreprocessorBlock{contents},
            AnalysisSuspendHeader::ProcedureSettings => PreprocessorAnalysisSection::ProcedureSettings{contents},
            AnalysisSuspendHeader::CreateWindow => {
                let attributes = {
                    let contents_str: &str = &contents;
                    create_window().parse_stream(contents_str).unwrap().0
                };
                PreprocessorAnalysisSection::CreateWindow{contents, attributes}
            },
            AnalysisSuspendHeader::CodeBlock { block_type } => PreprocessorAnalysisSection::CodeBlock{block_type, contents},
            AnalysisSuspendHeader::Other { block_type } => PreprocessorAnalysisSection::Other{block_type, contents}
        }
    }

    pub fn from(nodes: Vec<PreprocessorASTNode>) -> ProgressResult<Vec<PreprocessorAnalysisSection>> {
        let mut line_number = 0;

        let mut result = Vec::new();
        let mut section_start = None;
        let mut contents = String::new();
        for node in nodes {
            println!("{}: {:?}", line_number, node);
            match node {
                PreprocessorASTNode::AnalysisSuspend(header) => {
                    section_start = match section_start {
                        Some(_) => return Err(Error::new(format!("Two 'analysis-suspend's in a row on line {}", line_number))),
                        None => Some(header)
                    };

                    // Get the number of lines in the current section
                    line_number += contents.chars().fold(0, |acc, c| if c == '\n' {acc+1} else {acc}) + 1;

                    if contents.trim().len() > 0 {
                        result.push(PreprocessorAnalysisSection::NotInSection{contents});
                    };

                    contents = String::new();
                },
                PreprocessorASTNode::AnalysisResume => {
                    section_start = match section_start {
                        Some(start) => {
                            // Get the number of lines in the current section
                            line_number += contents.chars().fold(0, |acc, c| if c == '\n' {acc+1} else {acc}) + 1;

                            result.push(PreprocessorAnalysisSection::new(start, contents));
                            None
                        },
                        None => return Err(Error::new(format!("A 'analysis-resume' without an 'analysis-suspend' on line {}", line_number)))
                    };

                    contents = String::new();
                },
                PreprocessorASTNode::PreprocessorLine(line) => {
                    contents.push_str(&line);
                    contents.push_str("\r\n");
                },
                PreprocessorASTNode::Import(import) => {
                    // TODO: this removes { and } from the import
                    contents.push_str(&import);
                },
                node => {
                    let maybe_node_contents = node.get_contents();
                    if let Some(node_contents) = maybe_node_contents {
                        contents.push_str(node_contents)
                    }
                }
            }
        }
        return Ok(result);
    }

    pub fn show(&self) -> String {
        match self {
            &PreprocessorAnalysisSection::NotInSection{ref contents} => format!("Not in section: {}", contents.len()),
            &PreprocessorAnalysisSection::VersionNumber => format!("Version number"),
            &PreprocessorAnalysisSection::PreprocessorBlock{ref contents} => format!("preprocessor block: {}", contents.len()),
            &PreprocessorAnalysisSection::ProcedureSettings{ref contents} => format!("procedure settings: {}", contents.len()),
            &PreprocessorAnalysisSection::CreateWindow{ref contents, ref attributes} => format!("create window: {}", contents.len()),
            &PreprocessorAnalysisSection::CodeBlock{ref block_type, ref contents} => format!("{:?}: {}", block_type, contents.len()),
            &PreprocessorAnalysisSection::Other{ref block_type, ref contents} => format!("{}: {}", block_type, contents.len())
        }
    }
}

fn create_window<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=Vec<(String, f32)>> {
    // TODO: properly get this
    let start = till_eol()
        .with(spaces())
        .with(tag_no_case("CREATE"))
        .with(spaces())
        .with(tag_no_case("WINDOW"))
        .with(spaces())
        .with(identifier())
        .with(spaces())
        .with(tag_no_case("ASSIGN"))
        .with(spaces());

    fn toInt(chars: Vec<char>) -> f32 {
        let mut acc = 0.0;
        for c in chars {
            acc *= 10.0;
            acc += c.to_digit(10).unwrap() as f32;
        }
        acc
    }
    fn toFrac(chars: Vec<char>) -> f32 {
        let mut acc = 0.0;
        for c in chars.iter().rev() {
            acc /= 10.0;
            acc += c.to_digit(10).unwrap() as f32;
        }
        acc / 10.0
    }
    let number = (many1(digit()), optional(char('.').with(many1(digit())))).map(|(intStr, mFrac): (String, _)| {
        let int: f32 = intStr.parse().unwrap();
        match mFrac {
            None => int,
            Some(frac) => int + toFrac(frac)
        }

    });
    let assign = (identifier(), spaces().with(char('=')).with(spaces()).with(number));

    return start.with(many1(assign)).skip((spaces(), char('.')));
}

fn preprocessor_line<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=PreprocessorASTNode> {
    char('&')
        .with(till_eol())
        .map(PreprocessorASTNode::PreprocessorLine)
}

fn preprocessor_import<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=PreprocessorASTNode> {
    char('{')
        .with(many1(satisfy(|c| c != '}')))
        .skip(char('}'))
        .map(PreprocessorASTNode::Import)
}

fn preprocessor_replace<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=PreprocessorASTNode> {
    char('{')
        .with(many1(digit()))
        .skip(char('}'))
        .map(PreprocessorASTNode::Replace)
}

fn code<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=PreprocessorASTNode> {
    many1(satisfy(|c| c != '{' && c != '&')).map(PreprocessorASTNode::Code)
}

fn comment<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=PreprocessorASTNode> {
    fn comment_<I: Stream<Item=char>>(input: I) -> ParseResult<PreprocessorASTNode, I> {
        let start = string("/*");
        let end = string("*/");
        // TODO: I need to be able to consider escaped chars
        let goodChar = char('/').skip(not_followed_by(char('*')))
            .or(char('*').skip(not_followed_by(char('/'))))
            .or(any());
        let all_but_surrounding = many(goodChar);

        let inner_comments = many::<Vec<(_, String)>, _>((comment(), all_but_surrounding.clone()));
        let mut parser = (start, all_but_surrounding, inner_comments, end).map(|(start, first_section, inner_comments, end): (_, String, Vec<(PreprocessorASTNode, String)>, _)| {
            let mut comment_string = String::new();
            comment_string.push_str(start);
            comment_string.push_str(&first_section);
            for (inner_comment, after_section) in inner_comments {
                if let PreprocessorASTNode::Comment(contents) = inner_comment {
                    comment_string.push_str(&contents);
                }
                comment_string.push_str(&after_section);
            }
            comment_string.push_str(end);
            PreprocessorASTNode::Comment(comment_string)
        });

        parser.parse_lazy(input).into()
    }

    parser(comment_).expected("comment")
}

pub fn preprocessed_progress<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=Vec<PreprocessorASTNode>> {
    let choices = try(analyze_suspend().map(PreprocessorASTNode::AnalysisSuspend))
        .or(try(value(PreprocessorASTNode::AnalysisResume).skip(analyze_resume())))
        .or(try(preprocessor_line()))
        .or(try(preprocessor_replace()))
        .or(try(preprocessor_import()))
        .or(try(comment()))
        .or(code());
    many1(choices)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_window() {
    }
}
