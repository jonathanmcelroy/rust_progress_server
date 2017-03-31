use combine::{choice, many1, tokens, try, value};
use combine::char::{spaces};
use combine::primitives::{Parser, Stream};

use parser::file_position::{FilePositionM, wrap};
use parser::util::{identifier, till_eol, tag_no_case};
use util::u8_ref_to_string;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisSuspendHeader {
    VersionNumber,
    PreprocessorBlock,
    ProcedureSettings,
    CreateWindow,
    CodeBlock { block_type: CodeBlockType } ,
    Other { block_type: String }
}

type AnalysisSuspendHeaderFP = FilePositionM<AnalysisSuspendHeader>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CodeBlockType {
    Custom { name: String, frame_name: String },
    FunctionForward { name: String, frame_name: String },
    Control { name: String, frame_name: String },
    Procedure { name: String, frame_name: String },
    Function { name: String, frame_name: String },
    Unknown { name: String },
}

type CodeBlockTypeFP = FilePositionM<CodeBlockType>;

fn custom_code_block<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    let name = tag_no_case("_CUSTOM")
        .with(spaces())
        .with(identifier());
    let frame_name = spaces()
        .with(identifier());
    (name, frame_name).map(|(name, frame_name)| CodeBlockType::Custom { name, frame_name })
}


fn function_forward<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    let name = tag_no_case("_FUNCTION-FORWARD")
        .with(spaces())
        .with(identifier());
    let frame_name = spaces()
        .with(identifier());
    (name, frame_name).map(|(name, frame_name)| CodeBlockType::FunctionForward { name, frame_name })
}

fn control_code_block<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    let name = tag_no_case("_CONTROL")
        .with(spaces())
        .with(identifier());
    let frame_name = spaces()
        .with(identifier());
    (name, frame_name).map(|(name, frame_name)| CodeBlockType::Control { name, frame_name })
}

fn procedure<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    let name = tag_no_case("_PROCEDURE")
        .with(spaces())
        .with(identifier());
    let frame_name = spaces()
        .with(identifier());
    (name, frame_name).map(|(name, frame_name)| CodeBlockType::Procedure { name, frame_name })
}

fn function<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    let name = tag_no_case("_FUNCTION")
        .with(spaces())
        .with(identifier());
    let frame_name = spaces()
        .with(identifier());
    (name, frame_name).map(|(name, frame_name)| CodeBlockType::Function { name, frame_name })
}

fn unknown_code_block<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    identifier().map(|name| CodeBlockType::Unknown { name })
}

fn block_type<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=CodeBlockType> {
    // TODO: add the other choices
    try(custom_code_block())
        .or(try(function_forward()))
        .or(try(control_code_block()))
        .or(try(procedure()))
        .or(try(function()))
        .or(unknown_code_block())
}

fn analyze_suspend_start<I: Stream<Item=char>>() -> impl Parser<Input=I> {
    tag_no_case("&analyze-suspend")
}

fn analyze_suspend_version_numbers<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    value(AnalysisSuspendHeader::VersionNumber)
        .skip((analyze_suspend_start(), spaces(), tag_no_case("_VERSION-NUMBER"), till_eol()))
}

fn analyze_suspend_preprocessor_block<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    value(AnalysisSuspendHeader::PreprocessorBlock)
        .skip((analyze_suspend_start(), spaces(), tag_no_case("_UIB-PREPROCESSOR-BLOCK"), till_eol()))
}

fn analyze_suspend_procedure_settings<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    value(AnalysisSuspendHeader::ProcedureSettings)
        .skip((analyze_suspend_start(), spaces(), tag_no_case("_PROCEDURE-SETTINGS"), till_eol()))
}

fn analyze_suspend_create_window<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    value(AnalysisSuspendHeader::CreateWindow)
        .skip((analyze_suspend_start(), spaces(), tag_no_case("_CREATE-WINDOW"), till_eol()))
}

fn analyze_suspend_code_block<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    analyze_suspend_start()
        .with(spaces())
        .with(tag_no_case("_UIB-CODE-BLOCK"))
        .with(spaces())
        .with(block_type())
        .skip(till_eol())
        .map(|block_type| AnalysisSuspendHeader::CodeBlock{ block_type })
}

fn analyze_suspend_other<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    analyze_suspend_start()
        .with(spaces())
        .with(identifier())
        .skip(till_eol())
        .map(|block_type| AnalysisSuspendHeader::Other { block_type })
}

pub fn analyze_suspend<I: Stream<Item=char>>() -> impl Parser<Input=I, Output=AnalysisSuspendHeader> {
    // TODO: finish all options
    try(analyze_suspend_version_numbers())
        .or(try(analyze_suspend_preprocessor_block()))
        .or(try(analyze_suspend_procedure_settings()))
        .or(try(analyze_suspend_create_window()))
        .or(try(analyze_suspend_code_block()))
        .or(analyze_suspend_other())
}


pub fn analyze_resume<I: Stream<Item=char>>() -> impl Parser<Input=I> {
    tag_no_case("&analyze-resume").skip(till_eol())
}

#[cfg(test)]
mod tests {
    use combine::Parser;

    use error::from;

    use super::{AnalysisSuspendHeader, CodeBlockType, analyze_suspend_code_block, custom_code_block};

    #[test]
    fn test_custom_code_block() {
        let code = "_CUSTOM _DEFINITIONS fFrameWin";
        let result = from(custom_code_block().parse_stream(code));
        if result.is_err() {
            println!("Error: {:?}", result);
            assert!(false);
        }
        let parse = result.unwrap();

        let expected = CodeBlockType::Custom {
            name: "_DEFINITIONS".to_string(),
            frame_name: "fFrameWin".to_string(),
        };
        assert_eq!(expected, parse);
    }

    #[test]
    fn analyze_suspend_custom_code_block() {

        let code = "&ANALYZE-SUSPEND _UIB-CODE-BLOCK _CUSTOM _DEFINITIONS fFrameWin\r\n";
        let result = from(analyze_suspend_code_block().parse_stream(code));
        if result.is_err() {
            println!("Error: {:?}", result);
            assert!(false);
        }
        let parse = result.unwrap();

        let expected = AnalysisSuspendHeader::CodeBlock { 
            block_type: CodeBlockType::Custom {
                name: "_DEFINITIONS".to_string(),
                frame_name: "fFrameWin".to_string(),
            }
        };
        assert_eq!(expected, parse);
    }
}
