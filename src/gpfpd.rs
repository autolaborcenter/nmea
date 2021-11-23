use super::BodyParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Body {
    pub gps_week: u16,  // 自 1980-1-6 至当前的星期数（格林尼治时间）
    pub gps_time: u32,  // 自本周日 0:00:00 至当前的毫秒数（格林尼治时间）
    pub heading: u32,   // 偏航角 [0   ,360) / (10^-3)°
    pub pitch: i32,     // 俯仰角 [-90 ,90]  / (10^-3)°
    pub roll: i32,      // 横滚角 [-180,180] / (10^-3)°
    pub latitude: i32,  // 纬度 [-90 ,90]  / (10^-7)°
    pub longitude: i32, // 经度 [-180,180] / (10^-7)°
    pub altitude: i32,  // 海拔 / (10^-2)m
    pub vel_e: i32,     // 东向速度 / mm/s
    pub vel_n: i32,     // 北向速度 / mm/s
    pub vel_u: i32,     // 天向速度 / mm/s
    pub baseline: u16,  // 基线长度 / mm
    pub nsv1: u8,       // 天线 1 星数
    pub nsv2: u8,       // 天线 2 星数
    pub status: Status, // 系统状态
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
            gps_time: parse_field!(s => "GPFPD:GPSTime"; 3),
            heading: parse_field!(s => "GPFPD:Heading"; 3),
            pitch: parse_field!(s => "GPFPD:Pitch"; 3),
            roll: parse_field!(s => "GPFPD:Roll"; 3),
            latitude: parse_field!(s => "GPFPD:Latitude"; 7),
            longitude: parse_field!(s => "GPFPD:Longitude"; 7),
            altitude: parse_field!(s => "GPFPD:Altitude"; 2),
            vel_e: parse_field!(s => "GPFPD:Ve"; 3),
            vel_n: parse_field!(s => "GPFPD:Vn"; 3),
            vel_u: parse_field!(s => "GPFPD:Vu"; 3),
            baseline: parse_field!(s => "GPFPD:Baseline"; 3),
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
                match b {
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
                match a {
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
