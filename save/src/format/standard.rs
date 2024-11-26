use super::Format;
use crate::error::Error;
use std::fmt;

pub(crate) struct Standard;

impl Format<'_, ()> for Standard {
    #[tracing::instrument(err)]
    fn decode(_: &str) -> Result<(), Error> {
        Ok(())
    }

    fn encode(_: &(), _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Format<'_, bool> for Standard {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<bool, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::InvalidData),
        }
    }

    fn encode(value: &bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if *value { 1 } else { 0 })
    }
}

impl<'a> Format<'a, &'a str> for Standard {
    #[tracing::instrument(err)]
    fn decode(value: &'a str) -> Result<&'a str, Error> {
        Ok(value)
    }

    fn encode(value: &&'a str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{value}")
    }
}

impl Format<'_, String> for Standard {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<String, Error> {
        Ok(value.to_owned())
    }

    fn encode(value: &String, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{value}")
    }
}

impl Format<'_, f64> for Standard {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<f64, Error> {
        Ok(value.parse()?)
    }

    fn encode(value: &f64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString#return_value
        if value.abs() >= 1e21 || value.abs() < 1e-6 {
            let exp = format!("{value:e}");
            let (m, e) = exp.split_once('e').unwrap_or((&exp, "0"));
            if e.starts_with(['+', '-']) {
                write!(f, "{m}e{e}")
            } else {
                write!(f, "{m}e+{e}")
            }
        } else {
            write!(f, "{value}")
        }
    }
}

macro_rules! display_from_str {
    ($ty:ty) => {
        impl Format<'_, $ty> for Standard {
            #[tracing::instrument(err)]
            fn decode(value: &str) -> Result<$ty, Error> {
                Ok(value.parse()?)
            }

            fn encode(value: &$ty, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{value}")
            }
        }
    };
}
display_from_str!(u64);
display_from_str!(usize);

#[cfg(test)]
mod tests {
    use super::super::Format;
    use super::Standard;

    #[test]
    #[tracing_test::traced_test]
    fn test_f64() {
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString#using_tostring
        <Standard as Format<'_, f64>>::check_inverse("3.1622776601683794e+21").unwrap();
        <Standard as Format<'_, f64>>::check_inverse("1000000000000000100").unwrap();
        <Standard as Format<'_, f64>>::check_inverse("17").unwrap();
        <Standard as Format<'_, f64>>::check_inverse("17.2").unwrap();
    }
}
