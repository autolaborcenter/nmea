use std::str::FromStr;

macro_rules! parse_field {
    // 解析一般数据
    ($s:ident => $info:expr) => {
        match $s.next() {
            Some(s) => match crate::parse_any(s) {
                Some(it) => it,
                None => return Err(BodyParseError::ParseFailed($info, s.to_string())),
            },
            None => return Err(BodyParseError::MissingField($info)),
        }
    };
    // 解析一般数据
    ($s:ident =>? $info:expr) => {
        match $s.next() {
            Some(s) => match crate::parse_option(s) {
                Some(it) => it,
                None => return Err(BodyParseError::ParseFailed($info, s.to_string())),
            },
            None => return Err(BodyParseError::MissingField($info)),
        }
    };
    // 定点小数解析为整数
    ($s:ident => $info:expr; ?) => {
        match $s.next() {
            Some(s) => match crate::parse_fixed_unknown(s) {
                Some(it) => it,
                None => return Err(BodyParseError::ParseFailed($info, s.to_string())),
            },
            None => return Err(BodyParseError::MissingField($info)),
        }
    };
    // 定点小数解析为整数
    // `n` 为小数位数
    ($s:ident => $info:expr; $n:expr) => {
        match $s.next() {
            Some(s) => match crate::parse_fixed(s, $n) {
                Some(it) => it,
                None => return Err(BodyParseError::ParseFailed($info, s.to_string())),
            },
            None => return Err(BodyParseError::MissingField($info)),
        }
    };
}

pub mod cmd;
pub mod gpfpd;
pub mod gpgga;
pub mod gphpd;
pub mod gtimu;
mod parser;

pub use parser::NmeaParser;

#[derive(Debug)]
pub enum BodyParseError {
    MissingField(&'static str),
    ParseFailed(&'static str, String),
}

/// 来自星网宇达的 NMEA 消息类型
#[derive(Debug, PartialEq)]
pub enum NmeaLine {
    GPFPD(gpfpd::Body),
    GTIMU(gtimu::Body),
    GPHPD(gphpd::Body),
    GPGGA(gpgga::Body, String),
    GPRMC(String),
    GPCHC(String),
    CMD(cmd::Body),
    Unknown(String, String),
}

impl FromStr for NmeaLine {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (head, tail) = s.split_once(',').unwrap();
        Ok(match head {
            "GPFPD" => NmeaLine::GPFPD(tail.parse()?),
            "GTIMU" => NmeaLine::GTIMU(tail.parse()?),
            "GPHPD" => NmeaLine::GPHPD(tail.parse()?),
            "GPGGA" => NmeaLine::GPGGA(tail.parse()?, tail.into()),
            "GPRMC" => NmeaLine::GPRMC(tail.into()),
            "GPCHC" => NmeaLine::GPCHC(tail.into()),
            "cmd" => NmeaLine::CMD(tail.parse()?),
            unknown => NmeaLine::Unknown(unknown.into(), tail.into()),
        })
    }
}

#[inline]
pub fn rebuild_nema(head: &str, tail: &str, cs: u8) -> String {
    format!("${},{}*{:2X}", head, tail, cs)
}

#[inline]
fn parse_any<T: FromStr>(s: &str) -> Option<T> {
    match s.parse::<T>() {
        Ok(it) => Some(it),
        Err(_) => None,
    }
}

#[inline]
fn parse_option<T: FromStr>(s: &str) -> Option<Option<T>> {
    if s.is_empty() {
        Some(None)
    } else {
        match s.parse::<T>() {
            Ok(it) => Some(Some(it)),
            Err(_) => None,
        }
    }
}

fn parse_fixed_unknown<T: FromStr>(s: &str) -> Option<(T, u8)> {
    let b = s.as_bytes();
    let n = match b.iter().rev().enumerate().find(|(_, b)| **b == b'.') {
        Some((n, _)) => n,
        None => return None,
    };
    let i = b.len() - n - 1;
    let mut buf = [0u8; 16];
    buf[..i].copy_from_slice(&b[..i]);
    buf[i..][..n].copy_from_slice(&b[i + 1..]);
    match unsafe { std::str::from_utf8_unchecked(&buf[..b.len() - 1]) }.parse() {
        Ok(x) => Some((x, n as u8)),
        Err(_) => None,
    }
}

fn parse_fixed<T: FromStr>(s: &str, n: usize) -> Option<T> {
    let b = s.as_bytes();
    if b.len() < n + 2 {
        return None;
    }
    let i = b.len() - n - 1;
    if b[i] != b'.' {
        return None;
    }
    let mut buf = [0u8; 16];
    buf[..i].copy_from_slice(&b[..i]);
    buf[i..][..n].copy_from_slice(&b[i + 1..]);
    match unsafe { std::str::from_utf8_unchecked(&buf[..b.len() - 1]) }.parse() {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

#[test]
fn test_parse() {
    let mut parser = NmeaParser::<256>::default();

    const LINES: [&[u8]; 5] = [
        b"$GPFPD,2185,108150.400,272.628,2.722,0.188,39.9926157,116.3269623,-308580.94,0.003,-0.033,-3243.491,10.191,15,18,04*63",
        b"$GTIMU,0,6.000,3.3755,-0.0768,-3.0907,-0.1633,0.6105,0.7855,27.5*4C",
        b"$GPHPD,0,0.000,0.000,0.000,0.000,0.0000000,0.0000000,0.00,0.000,0.000,0.000,0.000,0,0,00*49",
        b"$GPGGA,060220.00,3959.55874779,N,11619.61828897,E,1,17,1.6,60.1397,M,-9.2862,M,,*42",
        b"$cmd,get,product,newton-m3*ff",
    ];

    for (i, line) in LINES.iter().enumerate() {
        parser.as_buf()[..line.len()].copy_from_slice(line);
        parser.notify_received(line.len());
        let result = std::str::from_utf8(&line[1..line.len() - 3])
            .unwrap()
            .parse()
            .unwrap();
        println!("i = {}, {:?}", i, result);
        assert_eq!(parser.next().unwrap().0, result);
    }
    assert_eq!(parser.next(), None);
}

#[test]
fn test_rebuild_gpgga() {
    const LINE: &str =
        "$GPGGA,060220.00,3959.55874779,N,11619.61828897,E,1,17,1.6,60.1397,M,-9.2862,M,,*42";

    let mut parser = NmeaParser::<256>::default();
    parser.as_buf()[..LINE.len()].copy_from_slice(LINE.as_bytes());
    parser.notify_received(LINE.len());
    if let Some((NmeaLine::GPGGA(_, tail), cs)) = parser.next() {
        assert_eq!(rebuild_nema("GPGGA", tail.as_str(), cs), LINE);
    } else {
        panic!("Parse failed.");
    }
}
