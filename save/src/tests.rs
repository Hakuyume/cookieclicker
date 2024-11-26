use crate::escape;
use crate::format::{Format, FormatExt};

#[test]
fn test() {
    let value = escape::decode(include_str!("tests/00.txt")).unwrap();
    assert_eq!(
        super::Save::decode(&value).unwrap().display().to_string(),
        value,
    );
}
