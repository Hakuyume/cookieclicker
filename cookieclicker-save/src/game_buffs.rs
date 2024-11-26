use crate::error::Error;
use crate::format;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameBuff {
    pub effect_id: usize,
    pub maximum_time: u64,
    pub time_remaining: u64,
    pub argument1: Option<f64>,
    pub argument2: Option<usize>,
    pub argument3: Option<String>,
}

pub(crate) struct Custom;

impl format::Format<'_, Vec<GameBuff>> for Custom {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Vec<GameBuff>, Error> {
        if value.is_empty() {
            Ok(Vec::new())
        } else {
            value
                .trim_end_matches(';')
                .split(';')
                .map(|v| {
                    let mut split = v.split(',');
                    Ok(GameBuff {
                        effect_id: {
                            let _span = tracing::info_span!(stringify!(effect_id)).entered();
                            format::Standard::decode(split.next().ok_or(Error::InsufficientData)?)?
                        },
                        maximum_time: {
                            let _span = tracing::info_span!(stringify!(maximum_time)).entered();
                            format::Standard::decode(split.next().ok_or(Error::InsufficientData)?)?
                        },
                        time_remaining: {
                            let _span = tracing::info_span!(stringify!(time_remaining)).entered();
                            format::Standard::decode(split.next().ok_or(Error::InsufficientData)?)?
                        },
                        argument1: {
                            let _span = tracing::info_span!(stringify!(argument1)).entered();
                            if let Some(v) = split.next() {
                                format::NoneAsEmpty::<format::Standard>::decode(v)?
                            } else {
                                None
                            }
                        },
                        argument2: {
                            let _span = tracing::info_span!(stringify!(argument2)).entered();
                            if let Some(v) = split.next() {
                                format::NoneAsEmpty::<format::Standard>::decode(v)?
                            } else {
                                None
                            }
                        },
                        argument3: {
                            let _span = tracing::info_span!(stringify!(argument3)).entered();
                            if let Some(v) = split.next() {
                                format::NoneAsEmpty::<format::Standard>::decode(v)?
                            } else {
                                None
                            }
                        },
                    })
                })
                .collect()
        }
    }

    fn encode(value: &Vec<GameBuff>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in value {
            format::Standard::encode(&v.effect_id, f)?;
            write!(f, ",")?;
            format::Standard::encode(&v.maximum_time, f)?;
            write!(f, ",")?;
            format::Standard::encode(&v.time_remaining, f)?;
            if v.argument1.is_some() || v.argument2.is_some() || v.argument3.is_some() {
                write!(f, ",")?;
                format::NoneAsEmpty::<format::Standard>::encode(&v.argument1, f)?;
            }
            if v.argument2.is_some() || v.argument3.is_some() {
                write!(f, ",")?;
                format::NoneAsEmpty::<format::Standard>::encode(&v.argument2, f)?;
            }
            if v.argument3.is_some() {
                write!(f, ",")?;
                format::NoneAsEmpty::<format::Standard>::encode(&v.argument3, f)?;
            }
            write!(f, ";")?;
        }
        Ok(())
    }
}
