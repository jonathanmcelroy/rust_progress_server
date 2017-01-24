use nom;

#[derive(Debug)]
pub enum PreprocessedProgress {
    PreprocessorLine(String),
    Import(String),
    Replace(String),
    Code(String),
    Comment(String),
}

named!(preprocessor_line<&[u8], PreprocessedProgress>,
       do_parse!(
           char!('&') >> 
           line: take_until!("\n") >>
           (PreprocessedProgress::PreprocessorLine(String::from_utf8_lossy(line).into_owned()))
           )
      );

named!(preprocessor_import<&[u8], PreprocessedProgress>,
       do_parse!(
           char!('{') >>
           import: take_until!("}") >>
           char!('}') >>
           (PreprocessedProgress::Import(String::from_utf8_lossy(import).into_owned()))
           )
      );

named!(preprocessor_replace<&[u8], PreprocessedProgress>,
       do_parse!(
           char!('{') >>
           d: digit >>
           char!('}') >>
           (PreprocessedProgress::Replace(String::from_utf8_lossy(d).into_owned()))
           )
      );

/*
// named!(comment<&[u8], PreprocessedProgress>,
// do_parse!(
// tag!("/*") >>
// contents: many0!(
// alt!(
// comment |
// )
// ) >>
// tag!("*/") >>
// (PreprocessedProgress::Comment(String::from_utf8_lossy(contents).into_owned()))
// )
// );
*/

named!(pub preprocessed_progress<&[u8], Vec<PreprocessedProgress> >,
       many1!(
           alt!(
               preprocessor_line |
               preprocessor_replace |
               preprocessor_import |
               map!(take_until_either!("{&"), |b| PreprocessedProgress::Code(String::from_utf8_lossy(b).into_owned()))
               )
           )
      );

