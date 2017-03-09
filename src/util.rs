pub fn u8_ref_to_string(input: &[u8]) -> String{
    String::from_utf8_lossy(input).into_owned()
}

pub fn restrict_string(to_restrict: &str) -> String {
    if to_restrict.len() < 20 {
        return to_restrict.to_string();
    } else {
        let mut result = String::with_capacity(23);
        result.push_str(&to_restrict[..10]);
        result.push_str("...");
        result.push_str(&to_restrict[to_restrict.len()-10..]);
        return result;
    }
}
