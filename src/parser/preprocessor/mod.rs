mod analysis_suspend;

use nom;
use combine::{choice, many1, value};
use combine::combinator::{Value};
use combine::primitives::{Parser, Stream};
use self::analysis_suspend::{AnalysisSuspendHeader, CodeBlockType, analyze_suspend, analyze_resume};
use error::{ProgressResult, Error};

#[derive(Debug, Clone)]
pub enum PreprocessorASTNode {
    AnalysisSuspend(AnalysisSuspendHeader),
    AnalysisResume,
    PreprocessorLine(String),
    Import(String),
    Replace(String),
    Code(String),
    Comment(String),
}

impl PreprocessorASTNode {
    fn get_contents(&self) -> Option<&str> {
        match self {
            &PreprocessorASTNode::AnalysisSuspend(_) => None,
            &PreprocessorASTNode::AnalysisResume => None,
            &PreprocessorASTNode::PreprocessorLine(ref contents) => Some(contents),
            &PreprocessorASTNode::Import(ref contents) => Some(contents),
            &PreprocessorASTNode::Replace(ref contents) => Some(contents),
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
        let mut result = Vec::new();
        let mut section_start = None;
        let mut contents = String::new();
        for node in nodes {
            match node {
                PreprocessorASTNode::AnalysisSuspend(header) => {
                    section_start = match section_start {
                        Some(_) => return Err(Error::new("Two 'analysis-suspend's in a row")),
                        None => Some(header)
                    };
                    if contents.trim().len() > 0 {
                        result.push(PreprocessorAnalysisSection::NotInSection{contents});
                    };
                    contents = String::new();
                },
                PreprocessorASTNode::AnalysisResume => {
                    section_start = match section_start {
                        Some(start) => {
                            result.push(PreprocessorAnalysisSection::new(start, contents));
                            None
                        },
                        None => return Err(Error::new("A 'analysis-result' without an 'analysis-suspend'"))
                    };
                    contents = String::new();
                }
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

named!(preprocessor_line<&[u8], PreprocessorASTNode>,
       do_parse!(
           char!('&') >> 
           line: take_until!("\n") >>
           (PreprocessorASTNode::PreprocessorLine(String::from_utf8_lossy(line).into_owned()))
           )
      );

named!(preprocessor_import<&[u8], PreprocessorASTNode>,
       do_parse!(
           char!('{') >>
           import: take_until!("}") >>
           char!('}') >>
           (PreprocessorASTNode::Import(String::from_utf8_lossy(import).into_owned()))
           )
      );

named!(preprocessor_replace<&[u8], PreprocessorASTNode>,
       do_parse!(
           char!('{') >>
           d: call!(nom::digit) >>
           char!('}') >>
           (PreprocessorASTNode::Replace(String::from_utf8_lossy(d).into_owned()))
           )
      );

named!(code<&[u8], PreprocessorASTNode>, map!(take_until_either!("{&"), |b| PreprocessorASTNode::Code(String::from_utf8_lossy(b).into_owned())));

/*
// TODO: add this back
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

named!(pub preprocessed_progress<&[u8], Vec<PreprocessorASTNode> >,
       many1!(
           alt_complete!(
               map!(analyze_suspend, PreprocessorASTNode::AnalysisSuspend) |
               value!(PreprocessorASTNode::AnalysisResume, analyze_resume) |
               preprocessor_line |
               preprocessor_replace |
               preprocessor_import |
               // comment |
               code
               )
           )
      );

pub fn preprocessed_progress2<I: Stream>() -> impl Parser<Input=I, Output=Vec<PreprocessorASTNode>> {
    // TODO: complete
    //return many1(
        //choice(
            //analyze_suspend,
              //)
        //);
    return value(vec![]);
}
