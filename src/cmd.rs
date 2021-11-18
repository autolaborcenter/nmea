use super::BodyParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Body {
    Get(String),
    Set(String),
    Through(String),
    Unknown(String, String),
}

impl FromStr for Body {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(',') {
            Some((head, tail)) => match head {
                "get" => Ok(Body::Get(tail.into())),
                "set" => Ok(Body::Set(tail.into())),
                "through" => Ok(Body::Through(tail.into())),
                unknown => Ok(Body::Unknown(unknown.into(), tail.into())),
            },
            None => Err(BodyParseError::MissingField("CMD:Type")),
        }
    }
}
