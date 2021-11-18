use super::BodyParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Body {
    gps_week: u16,
    gps_time: u32,
    heading: u32,
    pitch: i32,
    roll: i32,
    latitude: i32,
    longitude: i32,
    altitude: i32,
    vel_e: i32,
    vel_n: i32,
    vel_u: i32,
    baseline: u16,
    nsv1: u8,
    nsv2: u8,
    status: Status,
}

#[derive(Debug, PartialEq)]
pub struct Status(SystemStatus, RtkStatus);

#[derive(Debug, PartialEq)]
pub enum SystemStatus {
    初始化 = 0x0,
    粗对准 = 0x1,
    精对准 = 0x2,
    GPS定位 = 0x3,
    GPS定向 = 0x4,
    RTK = 0x5,
    DMI组合 = 0x6,
    DMI标定 = 0x7,
    纯惯性 = 0x8,
    零速校正 = 0x9,
    VG模式 = 0xA,
    差分定向 = 0xB,
    动态对准 = 0xC,
    动态出错 = 0xF,
}

#[derive(Debug, PartialEq)]
pub enum RtkStatus {
    Gps1Bd = 0,
    双模 = 2,
    RTK固定解 = 4,
    RTK浮点解 = 5,
}

impl FromStr for Body {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split(',');
        Ok(Self {
            gps_week: parse_field!(s => "GPFPD:GPSWeek"),
            gps_time: parse_field!(s => "GPFPD:GPSTime"; 3, 9),
            heading: parse_field!(s => "GPFPD:Heading"; 3, 6),
            pitch: parse_field!(s => "GPFPD:Pitch"; 3, 6),
            roll: parse_field!(s => "GPFPD:Roll"; 3, 6),
            latitude: parse_field!(s => "GPFPD:Latitude"; 7, 10),
            longitude: parse_field!(s => "GPFPD:Longitude"; 7, 11),
            altitude: parse_field!(s => "GPFPD:Altitude"; 2, 8),
            vel_e: parse_field!(s => "GPFPD:Ve"; 3, 7),
            vel_n: parse_field!(s => "GPFPD:Vn"; 3, 7),
            vel_u: parse_field!(s => "GPFPD:Vu"; 3, 7),
            baseline: parse_field!(s => "GPFPD:Baseline"; 3, 5),
            nsv1: parse_field!(s => "GPFPD:NSV1"),
            nsv2: parse_field!(s => "GPFPD:NSV2"),
            status: parse_field!(s => "GPFPD:Status"),
        })
    }
}

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RtkStatus::*;
        use SystemStatus::*;
        match s.as_bytes() {
            [a, b] => Ok(Self(
                match a {
                    b'0' => 初始化,
                    b'1' => 粗对准,
                    b'2' => 精对准,
                    b'3' => GPS定位,
                    b'4' => GPS定向,
                    b'5' => RTK,
                    b'6' => DMI组合,
                    b'7' => DMI标定,
                    b'8' => 纯惯性,
                    b'9' => 零速校正,
                    b'A' => VG模式,
                    b'B' => 差分定向,
                    b'C' => 动态对准,
                    b'F' => 动态出错,
                    _ => return Err(()),
                },
                match b {
                    b'0' => Gps1Bd,
                    b'2' => 双模,
                    b'4' => RTK固定解,
                    b'5' => RTK浮点解,
                    _ => return Err(()),
                },
            )),
            [..] => Err(()),
        }
    }
}
