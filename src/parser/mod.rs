
mod preprocessor;
mod util;
mod file_position;

pub use self::preprocessor::{
    PreprocessorASTNode,
    PreprocessorAnalysisSection,
    preprocessed_progress,
};
pub use self::file_position::{FilePosition};

/*
#[derive(Debug)]
pub enum Statement {
    Generic(String),
}
*/

/*
named!(parse_statement<Statement>,
);
*/

/*
named!(parse_progress<Vec<Statement> >,
    many1!(parse_statement)
);
*/

/*
pub fn parse(contents: &str) -> Result<Vec<Statement>, String> {
    let preprocessed_contents_vec = preprocessed_progress(contents.as_bytes());
    if preprocessed_contents_vec.is_done() {
        let result = preprocessed_contents_vec.unwrap().1;
        let preprocessed_contents = result.into_iter().fold(String::new(), |mut accum, value| {
            match value {
                PreprocessorAST::Code(ref a) => accum.push_str(a),
                PreprocessorAST::Comment(ref a) => accum.push_str(a),
                _ => {}
                // PreprocessorAST::PreprocessorLine(a) => _,
                // PreprocessorAST::Import(String),
                // PreprocessorAST::Replace(String),
            }
            return accum;
        });
        return Result::Ok(vec!(Statement::Generic(preprocessed_contents)));
    } else {
        println!("Not done");
    }

    return Result::Err(String::from("Testing"));
}
*/
