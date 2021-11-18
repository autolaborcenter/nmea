use super::BodyParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Body {
    gps_week: u16,
    gps_time: u32,
    gyro_x: i32,
    gyro_y: i32,
    gyro_z: i32,
    acc_x: i32,
    acc_y: i32,
    acc_z: i32,
    tpr: i16,
}

impl FromStr for Body {
    type Err = BodyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split(',');
        Ok(Self {
            gps_week: parse_field!(s => "GTIMU:GPSWeek"),
            gps_time: parse_field!(s => "GTIMU:GPSTime"; 3, 9),
            gyro_x: parse_field!(s => "GTIMU:GyroX"; 4, 8),
            gyro_y: parse_field!(s => "GTIMU:GyroY"; 4, 8),
            gyro_z: parse_field!(s => "GTIMU:GyroZ"; 4, 8),
            acc_x: parse_field!(s => "GTIMU:AccX"; 4, 8),
            acc_y: parse_field!(s => "GTIMU:AccY"; 4, 8),
            acc_z: parse_field!(s => "GTIMU:AccZ"; 4, 8),
            tpr: parse_field!(s => "GTIMU:Tpr"; 1, 4),
        })
    }
}
