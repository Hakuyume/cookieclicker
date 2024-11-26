use super::{Decode, Encode};
use crate::error::Error;
use std::fmt;

impl Decode<'_> for () {
    #[tracing::instrument(err)]
    fn decode(_: &str) -> Result<Self, Error> {
        Ok(())
    }
}
impl Encode for () {
    fn encode(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Decode<'_> for bool {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Self, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::InvalidData),
        }
    }
}
impl Encode for bool {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if *self { 1 } else { 0 })
    }
}

impl<'a> Decode<'a> for &'a str {
    #[tracing::instrument(err)]
    fn decode(value: &'a str) -> Result<Self, Error> {
        Ok(value)
    }
}
impl Encode for &str {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Decode<'_> for String {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.to_owned())
    }
}
impl Encode for String {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Decode<'_> for f64 {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}
impl Encode for f64 {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString#return_value
        if self.abs() >= 1e21 || self.abs() < 1e-6 {
            let exp = format!("{self:e}");
            let (m, e) = exp.split_once('e').unwrap_or((&exp, "0"));
            if e.starts_with(['+', '-']) {
                write!(f, "{m}e{e}")
            } else {
                write!(f, "{m}e+{e}")
            }
        } else {
            write!(f, "{self}")
        }
    }
}

macro_rules! display_from_str {
    ($ty:ty) => {
        impl Decode<'_> for $ty {
            #[tracing::instrument(err)]
            fn decode(value: &str) -> Result<Self, Error> {
                Ok(value.parse()?)
            }
        }
        impl Encode for $ty {
            fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{self}")
            }
        }
    };
}
display_from_str!(u64);
display_from_str!(usize);

#[cfg(test)]
mod tests {
    use super::super::{Decode, EncodeExt};

    #[test]
    fn test_f64() {
        fn check(value: &str) {
            assert_eq!(f64::decode(value).unwrap().display().to_string(), value);
        }
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString#using_tostring
        check("3.1622776601683794e+21");
        check("1000000000000000100");
        check("17");
        check("17.2");
    }
}
