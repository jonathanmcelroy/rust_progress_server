use nom;

mod preprocessor;
use self::preprocessor::{PreprocessedProgress, preprocessed_progress};

#[derive(Debug)]
pub enum Statement {
    Generic(String),
}

named!(parse_statement<Statement>,
);

named!(parse_progress<Vec<Statement> >,
    many1!(parse_statement)
);

pub fn parse(contents: &str) -> Result<Vec<Statement>, String> {
    let preprocessed_contents_vec = preprocessed_progress(contents.as_bytes());
    if preprocessed_contents_vec.is_done() {
        let result = preprocessed_contents_vec.unwrap().1;
        let preprocessed_contents = result.into_iter().fold(String::new(), |mut accum, value| {
            match value {
                PreprocessedProgress::Code(ref a) => accum.push_str(a),
                PreprocessedProgress::Comment(ref a) => accum.push_str(a),
                _ => {}
                // PreprocessedProgress::PreprocessorLine(a) => _,
                // PreprocessedProgress::Import(String),
                // PreprocessedProgress::Replace(String),
            }
            return accum;
        });
        return Result::Ok(vec!(Statement::Generic(preprocessed_contents)));
    } else {
        println!("Not done");
    }

    return Result::Err(String::from("Testing"));
}
