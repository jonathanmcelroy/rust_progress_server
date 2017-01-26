pub fn u8_ref_to_string(input: &[u8]) -> String{
    String::from_utf8_lossy(input).into_owned()
}
