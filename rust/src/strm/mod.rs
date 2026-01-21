use crate::error::{ZipResult, ZipError};
use std::io::{Read, Write, Seek};

pub trait Stream: Read + Write + Seek {
    fn is_open(&self) -> bool;
    fn get_prop_int64(&self, prop: i32) -> ZipResult<i64>;
    fn set_prop_int64(&mut self, _prop: i32, _value: i64) -> ZipResult<()> {
        Err(ZipError::Support)
    }
}

pub const MZ_STREAM_PROP_TOTAL_IN: i32 = 1;
pub const MZ_STREAM_PROP_TOTAL_IN_MAX: i32 = 2;
pub const MZ_STREAM_PROP_TOTAL_OUT: i32 = 3;
pub const MZ_STREAM_PROP_TOTAL_OUT_MAX: i32 = 4;
pub const MZ_STREAM_PROP_HEADER_SIZE: i32 = 5;
pub const MZ_STREAM_PROP_FOOTER_SIZE: i32 = 6;
pub const MZ_STREAM_PROP_DISK_SIZE: i32 = 7;
pub const MZ_STREAM_PROP_DISK_NUMBER: i32 = 8;
pub const MZ_STREAM_PROP_COMPRESS_LEVEL: i32 = 9;
pub const MZ_STREAM_PROP_COMPRESS_METHOD: i32 = 10;
pub const MZ_STREAM_PROP_COMPRESS_WINDOW: i32 = 11;
pub const MZ_STREAM_PROP_COMPRESS_THREADS: i32 = 12;

pub mod mem;
pub mod os;
