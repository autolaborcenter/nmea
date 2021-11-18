use std::str::FromStr;

macro_rules! parse_field {
    // 解析一般数据
    ($s:ident => $info:expr) => {
        match $s.next() {
            Some(s) => match s.parse() {
                Ok(it) => it,
                Err(_) => return Err(BodyParseError::ParseFailed($info, s.into())),
            },
            None => return Err(BodyParseError::MissingField($info)),
        }
    };
    // 定点小数解析为整数
    // `n` 为小数位数
    ($s:ident => $info:expr; $n:expr, $max:expr) => {
        match $s.next() {
            Some(s) => {
                let b = s.as_bytes();
                if b.len() < $n + 2 {
                    return Err(BodyParseError::ParseFailed($info, s.into()));
                }
                let i = b.len() - $n - 1;
                if b[i] != b'.' {
                    return Err(BodyParseError::ParseFailed($info, s.into()));
                }
                let mut buf = [0u8; $max];
                buf[..i].copy_from_slice(&b[..i]);
                buf[i..][..$n].copy_from_slice(&b[i + 1..]);
                match unsafe { std::str::from_utf8_unchecked(&buf[..b.len() - 1]) }.parse() {
                    Ok(n) => n,
                    Err(_) => return Err(BodyParseError::ParseFailed($info, s.into())),
                }
            }
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
    GPGGA(gpgga::Body),
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
            "GPGGA" => NmeaLine::GPGGA(tail.parse()?),
            "GPRMC" => NmeaLine::GPRMC(tail.into()),
            "GPCHC" => NmeaLine::GPCHC(tail.into()),
            "cmd" => NmeaLine::CMD(tail.parse()?),
            unknown => NmeaLine::Unknown(unknown.into(), tail.into()),
        })
    }
}

#[test]
fn test_parse() {
    let mut parser = NmeaParser::<256>::default();

    const LINES: [&[u8]; 5] = [
        b"$GPFPD,0,6.000,0.000,37.623,11.899,0.0000000,0.0000000,0.00,0.000,0.000,0.000,0.000,0,0,00*4A",
        b"$GTIMU,0,6.000,3.3755,-0.0768,-3.0907,-0.1633,0.6105,0.7855,27.5*4C",
        b"$GPHPD,0,0.000,0.000,0.000,0.000,0.0000000,0.0000000,0.00,0.000,0.000,0.000,0.000,0,0,00*49",
        b"$GPGGA,235948.00,0000.0000,S,00000.0000,W,0,0,0.00,0.000,M,0.000,M,00,0000*53",
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
        assert_eq!(parser.next(), Some(result));
    }
    assert_eq!(parser.next(), None);
}
