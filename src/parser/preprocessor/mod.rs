mod analysis_suspend;

use std::fmt;
use combine::{choice, many1, satisfy, try, value};
use combine::combinator::{Value};
use combine::primitives::{Parser, Stream};
use combine::char::{char, digit};
use self::analysis_suspend::{AnalysisSuspendHeader, CodeBlockType, analyze_suspend, analyze_resume};
use util::{restrict_string};
use parser::util::{till_eol};
use error::{ProgressResult, Error};

#[derive(Clone)]
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
pub enum PreprocessorAnalysisSection {
    NotInSection {contents: String },
    VersionNumber,
    PreprocessorBlock { contents: String },
    ProcedureSettings { contents: String },
    CreateWindow { contents: String },
    CodeBlock { block_type: CodeBlockType, contents: String } ,
    Other { block_type: String, contents: String }
}

impl PreprocessorAnalysisSection {
    fn new(header: AnalysisSuspendHeader, contents: String) -> PreprocessorAnalysisSection {
        match header {
            AnalysisSuspendHeader::VersionNumber => PreprocessorAnalysisSection::VersionNumber,
            AnalysisSuspendHeader::PreprocessorBlock => PreprocessorAnalysisSection::PreprocessorBlock{contents},
            AnalysisSuspendHeader::ProcedureSettings => PreprocessorAnalysisSection::ProcedureSettings{contents},
            AnalysisSuspendHeader::CreateWindow => PreprocessorAnalysisSection::CreateWindow{contents},
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
            &PreprocessorAnalysisSection::CreateWindow{ref contents} => format!("create window: {}", contents.len()),
            &PreprocessorAnalysisSection::CodeBlock{ref block_type, ref contents} => format!("{:?}: {}", block_type, contents.len()),
            &PreprocessorAnalysisSection::Other{ref block_type, ref contents} => format!("{}: {}", block_type, contents.len())
        }
    }
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

/*
// TODO: add comments back
named!(pub comment<&[u8], PreprocessorASTNode>,
do_parse!(
tag!("/*") >>
contents: many0!(
alt_complete!(
comment |
call!(nom::anychar)
)
) >>
tag!("*/") >>
(PreprocessorASTNode::Comment(String::from_utf8_lossy(contents).into_owned()))
)
);
*/

pub fn preprocessed_progress<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=Vec<PreprocessorASTNode>> {
    let choices = try(analyze_suspend().map(PreprocessorASTNode::AnalysisSuspend))
        .or(try(value(PreprocessorASTNode::AnalysisResume).skip(analyze_resume())))
        .or(try(preprocessor_line()))
        .or(try(preprocessor_replace()))
        .or(try(preprocessor_import()))
        // .or(comment())
        .or(code());
    many1(choices)
}
