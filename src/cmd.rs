use super::BodyParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Body {
    Get(String),
    Set(String),
    Undefined(String, String),
}

impl FromStr for Body {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(',') {
            Some((head, tail)) => match head {
                "get" => Ok(Body::Get(tail.into())),
                "set" => Ok(Body::Set(tail.into())),
                unknown => Ok(Body::Undefined(unknown.into(), tail.into())),
            },
            None => Err(BodyParseError::MissingField("CMD:Type")),
        }
    }
}

impl ToString for Body {
    fn to_string(&self) -> String {
        match self {
            Self::Get(body) => fmt_line("get", body),
            Self::Set(body) => fmt_line("set", body),
            Self::Undefined(head, body) => fmt_line(head, body),
        }
    }
}

#[inline]
fn fmt_line(head: &str, body: &str) -> String {
    format!("$cmd,{},{}*ff\r\n", head, body)
}
