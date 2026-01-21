use chrono::{DateTime, Utc, Datelike, Timelike, TimeZone};

pub fn time_t_to_dos_date(dt: DateTime<Utc>) -> (u16, u16) {
    let year = dt.year() as u32;
    let month = dt.month();
    let day = dt.day();
    let hour = dt.hour();
    let min = dt.minute();
    let sec = dt.second();

    let dos_date = if year < 1980 {
        0
    } else {
        ((year - 1980) << 9) | (month << 5) | day
    };

    let dos_time = (hour << 11) | (min << 5) | (sec / 2);

    (dos_date as u16, dos_time as u16)
}

pub fn dos_date_to_time_t(dos_date: u16, dos_time: u16) -> DateTime<Utc> {
    let year = ((dos_date >> 9) & 0x7F) as i32 + 1980;
    let month = ((dos_date >> 5) & 0x0F) as u32;
    let day = (dos_date & 0x1F) as u32;
    let hour = ((dos_time >> 11) & 0x1F) as u32;
    let min = ((dos_time >> 5) & 0x3F) as u32;
    let sec = ((dos_time & 0x1F) * 2) as u32;

    match Utc.with_ymd_and_hms(year, month, day, hour, min, sec) {
        chrono::LocalResult::Single(dt) => dt,
        _ => Utc.with_ymd_and_hms(1980, 1, 1, 0, 0, 0).unwrap(),
    }
}
