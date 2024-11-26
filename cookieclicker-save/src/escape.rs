use crate::error::Error;
use base64::prelude::{Engine, BASE64_STANDARD};

#[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
pub(crate) fn decode(value: &str) -> Result<String, Error> {
    let value = urlencoding::decode(value)?;
    let value = value.trim_end_matches("!END!");
    let value = BASE64_STANDARD.decode(value)?;
    Ok(String::from_utf8(value)?)
}

#[tracing::instrument(ret(level = tracing::Level::DEBUG))]
pub(crate) fn encode(value: &str) -> String {
    let mut value = BASE64_STANDARD.encode(value);
    value.push_str("!END!");
    urlencoding::encode(&value).into_owned()
}

#[cfg(test)]
mod tests {
    use rand::distributions::DistString;

    #[test]
    fn test() {
        let mut rng = rand::thread_rng();
        let value = rand::distributions::Standard.sample_string(&mut rng, 4096);
        assert_eq!(super::decode(&super::encode(&value)).unwrap(), value,);
    }
}
