use nom;
use combine;

// type Parser<O> = combine::Parser<Input: &[u8], Output: O>;

named!(pub till_eol<&[u8], Vec<char>>,
       do_parse!(
           chars_end: many_till!(nom::anychar, nom::line_ending) >>
           (chars_end.0)
           )
      );

named!(pub wspace, call!(nom::space));

fn is_identifier_character(ch: u8) -> bool {
    nom::is_alphanumeric(ch) || ch == b'-' || ch == b'_'
}
named!(pub identifier, take_while1!(is_identifier_character));
