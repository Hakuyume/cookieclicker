use crate::format::{self, Decode, EncodeExt};

#[test]
fn test() {
    let value = format::decode_base64(include_str!("tests/00.txt")).unwrap();
    assert_eq!(
        super::Save::decode(&value).unwrap().display().to_string(),
        value,
    );
}
