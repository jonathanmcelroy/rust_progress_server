use parser::util::{identifier, till_eol, wspace};
use util::u8_ref_to_string;

#[derive(Debug)]
pub enum AnalysisSuspendHeader {
    VersionNumber,
    PreprocessorBlock,
    ProcedureSettings,
    CreateWindow,
    CodeBlock { block_type: CodeBlockType } ,
    Other { block_type: String }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum CodeBlockType {
    Custom { name: String, frame_name: String },
    FunctionForward { name: String, frame_name: String },
    Control { name: String, frame_name: String },
    Procedure { name: String, frame_name: String },
    Function { name: String, frame_name: String },
    Unknown { name: String },
}

named!(custom_code_block<&[u8], CodeBlockType>,
       do_parse!(
           tag_no_case!("_CUSTOM") >>
           wspace >>
           name: identifier >>
           wspace >>
           frame_name: identifier >>
           till_eol >>
           (CodeBlockType::Custom { name: u8_ref_to_string(name), frame_name: u8_ref_to_string(frame_name) })
           )
      );
named!(function_forward<&[u8], CodeBlockType>,
       do_parse!(
           tag_no_case!("_FUNCTION-FORWARD") >>
           wspace >>
           name: identifier >>
           wspace >>
           frame_name: identifier >>
           till_eol >>
           (CodeBlockType::FunctionForward { name: u8_ref_to_string(name), frame_name: u8_ref_to_string(frame_name) })
           )
      );
named!(control_code_block<&[u8], CodeBlockType>,
       do_parse!(
           tag_no_case!("_CONTROL") >>
           wspace >>
           name: identifier >>
           wspace >>
           frame_name: identifier >>
           till_eol >>
           (CodeBlockType::Control { name: u8_ref_to_string(name), frame_name: u8_ref_to_string(frame_name) })
           )
      );
named!(procedure<&[u8], CodeBlockType>,
       do_parse!(
           tag_no_case!("_PROCEDURE") >>
           wspace >>
           name: identifier >>
           wspace >>
           frame_name: identifier >>
           till_eol >>
           (CodeBlockType::Procedure { name: u8_ref_to_string(name), frame_name: u8_ref_to_string(frame_name) })
           )
      );
named!(function<&[u8], CodeBlockType>,
       do_parse!(
           tag_no_case!("_FUNCTION") >>
           wspace >>
           name: identifier >>
           wspace >>
           frame_name: identifier >>
           till_eol >>
           (CodeBlockType::Function { name: u8_ref_to_string(name), frame_name: u8_ref_to_string(frame_name) })
           )
      );
named!(unknown_code_block<&[u8], CodeBlockType>,
       do_parse!(
           name: identifier >>
           till_eol >>
           (CodeBlockType::Unknown { name: u8_ref_to_string(name) })
           )
      );

named!(block_type<&[u8], CodeBlockType>,
       alt!(
           custom_code_block |
           function_forward |
           control_code_block |
           procedure |
           function | 
           unknown_code_block
           )
      );

named!(analyze_suspend_start, tag_no_case!("&analyze-suspend"));
named!(analyze_suspend_version_numbers<&[u8], AnalysisSuspendHeader>,
       value!(
           AnalysisSuspendHeader::VersionNumber,
           tuple!(
               analyze_suspend_start,
               wspace,
               tag_no_case!("_VERSION-NUMBER"),
               till_eol
               )
           )
      );
named!(analyze_suspend_preprocessor_block<&[u8], AnalysisSuspendHeader>,
       value!(
           AnalysisSuspendHeader::PreprocessorBlock,
           tuple!(
               analyze_suspend_start,
               wspace,
               tag_no_case!("_UIB-PREPROCESSOR-BLOCK"),
               till_eol
               )
           )
      );
named!(analyze_suspend_procedure_settings<&[u8], AnalysisSuspendHeader>,
       value!(
           AnalysisSuspendHeader::ProcedureSettings,
           tuple!(
               analyze_suspend_start,
               wspace,
               tag_no_case!("_PROCEDURE-SETTINGS"),
               till_eol
               )
           )
      );
named!(analyze_suspend_create_window<&[u8], AnalysisSuspendHeader>,
       value!(
           AnalysisSuspendHeader::CreateWindow,
           tuple!(
               analyze_suspend_start,
               wspace,
               tag_no_case!("_CREATE-WINDOW"),
               till_eol
               )
           )
      );
named!(analyze_suspend_code_block<&[u8], AnalysisSuspendHeader>,
       do_parse!(
           analyze_suspend_start >>
           wspace >>
           tag_no_case!("_UIB-CODE-BLOCK") >>
           wspace >>
           block_type: block_type >>
           till_eol >>
           (AnalysisSuspendHeader::CodeBlock{ block_type })
           )
      );
named!(analyze_suspend_other<&[u8], AnalysisSuspendHeader>, 
       do_parse!(
           analyze_suspend_start >>
           wspace >>
           name: identifier >>
           till_eol>>
           (AnalysisSuspendHeader::Other { block_type: u8_ref_to_string(name) })
           )
      );

named!(pub analyze_suspend<&[u8], AnalysisSuspendHeader>,
       alt_complete!(
           analyze_suspend_version_numbers |
           analyze_suspend_preprocessor_block |
           analyze_suspend_procedure_settings |
           analyze_suspend_create_window |
           analyze_suspend_code_block |
           analyze_suspend_other
           )
      );
named!(pub analyze_resume,
       do_parse!(
           start: tag_no_case!("&analyze-resume") >>
           till_eol >>
           (start)
           )
      );
