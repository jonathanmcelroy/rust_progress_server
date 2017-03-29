
mod preprocessor;
mod util;
mod file_position;

use combine::{skip_many, any, choice, many1, token, try, value, satisfy};
use combine::primitives::{Parser, Stream};

pub use self::preprocessor::{
    PreprocessorASTNode,
    PreprocessorAnalysisSection,
    CodeBlockType,
    preprocessed_progress,
};
pub use self::file_position::{FilePosition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    statements: Vec<Statement>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Generic(String),
}

pub fn ignore<I: Stream<Item=PreprocessorASTNode>>() -> impl Parser<Input=I, Output=PreprocessorASTNode> {
    satisfy(|node| 
            match node {
                PreprocessorASTNode::AnalysisSuspend(_) => true,
                PreprocessorASTNode::AnalysisResume => true,
                PreprocessorASTNode::PreprocessorLine(_) => true, // TODO: this may define code, so don't ignore it
                PreprocessorASTNode::Import(_) => true, // TODO: this definately defines code, so don't ignoe it
                PreprocessorASTNode::Replace(_) => true, // TODO: this may define code, so don't ignoe it
                PreprocessorASTNode::Code(_) => false,
                PreprocessorASTNode::Comment(_) => true,
            }
           )
}

pub fn statement<I: Stream<Item=PreprocessorASTNode>>() -> impl Parser<Input=I, Output=Statement> {
    skip_many(ignore()).with(any()).map(|node| 
                                   match node {
                                       PreprocessorASTNode::Code(code) => Statement::Generic(code),
                                       expr => unreachable!("This should not have code: {:?}", expr)
                                   }
                                  )
}

pub fn progress<I: Stream<Item=PreprocessorASTNode>>() -> impl Parser<Input=I, Output=Progress> {
    many1(statement()).map(|statements| Progress {
        statements
    })
}
