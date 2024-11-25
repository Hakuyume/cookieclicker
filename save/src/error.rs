#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Float(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("cannot parse bool")]
    Bool,
    #[error("insufficient data")]
    InsufficientData,
    #[error("timestamp out of range ")]
    TimestampOutOfRange,
}
