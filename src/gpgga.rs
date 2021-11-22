﻿use super::BodyParseError;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub struct Body {
    utc_time: u32,
    latitude: (u64, u8),
    ns: NS,
    longitude: (u64, u8),
    ew: EW,
    status: Status,
    nosv: u8,
    hdop: (u16, u8),
    altitude: (u32, u8),
    alt_unit: LenUnit,
    alt_ref: (i32, u8),
    alt_ref_unit: LenUnit,
    diff_age: Option<u8>,
    diff_station: Option<u16>,
}

#[derive(PartialEq, Debug)]
pub enum NS {
    N,
    S,
}

#[derive(PartialEq, Debug)]
pub enum EW {
    E,
    W,
}

#[derive(PartialEq, Debug)]
pub enum Status {
    初始化 = 0x0,
    单点定位 = 0x1,
    码差分 = 0x2,
    固定解 = 0x4,
    浮点解 = 0x5,
    正在估算 = 0x6,
    人工固定值 = 0x7,
    航位推算模式 = 0x8,
    WAAS差分 = 0x9,
}

#[derive(PartialEq, Debug)]
pub enum LenUnit {
    M,
}

impl FromStr for Body {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == ",,,,,0,,,,,,,," {
            Ok(Self {
                utc_time: 0,
                latitude: (0, 0),
                ns: NS::N,
                longitude: (0, 0),
                ew: EW::E,
                status: Status::初始化,
                nosv: 0,
                hdop: (0, 0),
                altitude: (0, 0),
                alt_unit: LenUnit::M,
                alt_ref: (0, 0),
                alt_ref_unit: LenUnit::M,
                diff_age: None,
                diff_station: None,
            })
        } else {
            let mut s = s.split(',');
            Ok(Self {
                utc_time: parse_field!(s => "GPGGA:UTCTime"; 2),
                latitude: parse_field!(s => "GPGGA:Latitude"; ?),
                ns: parse_field!(s => "GPGGA:N"),
                longitude: parse_field!(s => "GPGGA:Longitude"; ?),
                ew: parse_field!(s => "GPGGA:E"),
                status: parse_field!(s => "GPGGA:FS"),
                nosv: parse_field!(s => "GPGGA:NoSV"),
                hdop: parse_field!(s => "GPGGA:HDOP"; ?),
                altitude: parse_field!(s => "GPGGA:Altitude"; ?),
                alt_unit: parse_field!(s => "GPGGA:AltUnit"),
                alt_ref: parse_field!(s => "GPGGA:Altref"; ?),
                alt_ref_unit: parse_field!(s => "GPGGA:AltrefUnit"),
                diff_age: parse_field!(s =>? "GPGGA:DiffAge"),
                diff_station: parse_field!(s =>? "GPGGA:DiffStation"),
            })
        }
    }
}

impl FromStr for NS {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use NS::*;
        match s.as_bytes() {
            [c] => Ok(match c {
                b'N' => N,
                b'S' => S,
                _ => return Err(()),
            }),
            [..] => Err(()),
        }
    }
}

impl FromStr for EW {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use EW::*;
        match s.as_bytes() {
            [c] => Ok(match c {
                b'E' => E,
                b'W' => W,
                _ => return Err(()),
            }),
            [..] => Err(()),
        }
    }
}

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Status::*;
        match s.as_bytes() {
            [c] => Ok(match c {
                b'0' => 初始化,
                b'1' => 单点定位,
                b'2' => 码差分,
                b'4' => 固定解,
                b'5' => 浮点解,
                b'6' => 正在估算,
                b'7' => 人工固定值,
                b'8' => 航位推算模式,
                b'9' => WAAS差分,
                _ => return Err(()),
            }),
            [..] => Err(()),
        }
    }
}

impl FromStr for LenUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LenUnit::*;
        match s.as_bytes() {
            [c] => Ok(match c {
                b'M' => M,
                _ => return Err(()),
            }),
            [..] => Err(()),
        }
    }
}
