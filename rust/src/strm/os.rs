use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use crate::error::{ZipResult, ZipError};
use crate::strm::Stream;

pub struct FileStream {
    file: Option<File>,
}

impl FileStream {
    pub fn new() -> Self {
        Self { file: None }
    }

    pub fn open(&mut self, path: &str, mode: i32) -> ZipResult<()> {
        use std::fs::OpenOptions;
        let mut options = OpenOptions::new();

        if (mode & crate::constants::MZ_OPEN_MODE_READ) != 0 {
            options.read(true);
        }
        if (mode & crate::constants::MZ_OPEN_MODE_WRITE) != 0 {
            options.write(true);
        }
        if (mode & crate::constants::MZ_OPEN_MODE_CREATE) != 0 {
            options.create(true);
        }
        if (mode & crate::constants::MZ_OPEN_MODE_APPEND) != 0 {
            options.append(true);
        }

        let file = options.open(path)?;
        self.file = Some(file);
        Ok(())
    }
}

impl Read for FileStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.file {
            Some(f) => f.read(buf),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "File not open")),
        }
    }
}

impl Write for FileStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.file {
            Some(f) => f.write(buf),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "File not open")),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.file {
            Some(f) => f.flush(),
            None => Ok(()),
        }
    }
}

impl Seek for FileStream {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match &mut self.file {
            Some(f) => f.seek(pos),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "File not open")),
        }
    }
}

impl Stream for FileStream {
    fn is_open(&self) -> bool {
        self.file.is_some()
    }
    fn get_prop_int64(&self, _prop: i32) -> ZipResult<i64> {
        Err(ZipError::Support)
    }
}
