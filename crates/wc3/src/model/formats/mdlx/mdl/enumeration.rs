//! Validation of enum discriminators supplied by associated constants.
#[doc(hidden)]
pub fn enum_names_valid(names: &[&str]) -> bool {
    names.iter().enumerate().all(|(index, name)| {
        let mut bytes = name.bytes();
        bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && !name.eq_ignore_ascii_case("nan")
            && !name.eq_ignore_ascii_case("inf")
            && !names[..index].contains(name)
    })
}
