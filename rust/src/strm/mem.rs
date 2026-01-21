use std::io::{Read, Write, Seek, SeekFrom, Cursor};
use crate::error::{ZipResult, ZipError};
use crate::strm::Stream;

pub struct MemoryStream {
    cursor: Cursor<Vec<u8>>,
}

impl MemoryStream {
    pub fn new() -> Self {
        Self { cursor: Cursor::new(Vec::new()) }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { cursor: Cursor::new(data) }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
}

impl Read for MemoryStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl Write for MemoryStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cursor.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.cursor.flush()
    }
}

impl Seek for MemoryStream {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl Stream for MemoryStream {
    fn is_open(&self) -> bool {
        true
    }
    fn get_prop_int64(&self, _prop: i32) -> ZipResult<i64> {
        Err(ZipError::Support)
    }
}
