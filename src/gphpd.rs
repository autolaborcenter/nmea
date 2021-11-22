use super::BodyParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Body {
    gps_week: u16,
    gps_time: u32,
    heading: u32,
    pitch: i32,
    track: i32,
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
pub enum Status {
    初始化 = 0x0,
    GPS定位 = 0x3,
    GPS定向 = 0x4,
    RTK定位 = 0x5,
    RTK定向 = 0xB,
}

impl FromStr for Body {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split(',');
        Ok(Self {
            gps_week: parse_field!(s => "GPHPD:GPSWeek"),
            gps_time: parse_field!(s => "GPHPD:GPSTime"; 3),
            heading: parse_field!(s => "GPHPD:Heading"; 3),
            pitch: parse_field!(s => "GPHPD:Pitch"; 3),
            track: parse_field!(s => "GPHPD:Track"; 3),
            latitude: parse_field!(s => "GPHPD:Latitude"; 7),
            longitude: parse_field!(s => "GPHPD:Longitude"; 7),
            altitude: parse_field!(s => "GPHPD:Altitude"; 2),
            vel_e: parse_field!(s => "GPHPD:Ve"; 3),
            vel_n: parse_field!(s => "GPHPD:Vn"; 3),
            vel_u: parse_field!(s => "GPHPD:Vu"; 3),
            baseline: parse_field!(s => "GPHPD:Baseline"; 3),
            nsv1: parse_field!(s => "GPHPD:NSV1"),
            nsv2: parse_field!(s => "GPHPD:NSV2"),
            status: parse_field!(s => "GPHPD:Status"),
        })
    }
}

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Status::*;
        match s.parse::<u8>() {
            Ok(n) => Ok(match n {
                0x0 => 初始化,
                0x3 => GPS定位,
                0x4 => GPS定向,
                0x5 => RTK定位,
                0xB => RTK定向,
                _ => return Err(()),
            }),
            Err(_) => Err(()),
        }
    }
}
