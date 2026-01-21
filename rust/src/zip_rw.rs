use std::io::{Read, Write, Seek, SeekFrom};
use crate::error::{ZipResult, ZipError};
use crate::zip::ZipArchive;
use crate::types::ZipFile;
use crate::strm::os::FileStream;

pub struct ZipReader<S: Read + Write + Seek> {
    pub archive: ZipArchive<S>,
    pub current_entry: Option<usize>,
}

impl ZipReader<FileStream> {
    pub fn open_file(path: &str) -> ZipResult<Self> {
        let mut fs = FileStream::new();
        fs.open(path, crate::constants::MZ_OPEN_MODE_READ)?;
        let mut archive = ZipArchive::new(fs);
        archive.read_central_directory()?;
        Ok(Self {
            archive,
            current_entry: None,
        })
    }
}

impl<S: Read + Write + Seek + 'static> ZipReader<S> {
    pub fn new(stream: S) -> ZipResult<Self> {
        let mut archive = ZipArchive::new(stream);
        archive.read_central_directory()?;
        Ok(Self {
            archive,
            current_entry: None,
        })
    }

    pub fn goto_first_entry(&mut self) -> ZipResult<()> {
        if self.archive.entries.is_empty() {
            return Err(ZipError::EndOfList);
        }
        self.current_entry = Some(0);
        Ok(())
    }

    pub fn goto_next_entry(&mut self) -> ZipResult<()> {
        if let Some(current) = self.current_entry {
            if current + 1 < self.archive.entries.len() {
                self.current_entry = Some(current + 1);
                return Ok(());
            }
        }
        Err(ZipError::EndOfList)
    }

    pub fn entry_open(&mut self) -> ZipResult<()> {
        let _entry_idx = self.current_entry.ok_or(ZipError::Param)?;
        Ok(())
    }
}

pub struct ZipWriter<S: Read + Write + Seek> {
    pub archive: ZipArchive<S>,
}

impl ZipWriter<FileStream> {
    pub fn create_file(path: &str) -> ZipResult<Self> {
        let mut fs = FileStream::new();
        fs.open(path, crate::constants::MZ_OPEN_MODE_CREATE | crate::constants::MZ_OPEN_MODE_WRITE)?;
        let archive = ZipArchive::new(fs);
        Ok(Self { archive })
    }
}

impl<S: Read + Write + Seek> ZipWriter<S> {
    pub fn new(stream: S) -> Self {
        Self {
            archive: ZipArchive::new(stream),
        }
    }

    pub fn add_buffer(&mut self, data: &[u8], filename: &str) -> ZipResult<()> {
        let mut file = ZipFile::default();
        file.filename = filename.to_string();
        file.uncompressed_size = data.len() as i64;
        file.compressed_size = data.len() as i64;
        file.compression_method = crate::constants::MZ_COMPRESS_METHOD_STORE;
        file.version_needed = 20;
        file.version_madeby = 45; // UNIX

        // Calculate CRC
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(data);
        file.crc = hasher.finalize();

        file.disk_offset = self.archive.stream.seek(SeekFrom::Current(0))? as i64;

        self.archive.write_local_file_header(&file)?;
        self.archive.stream.write_all(data)?;

        self.archive.entries.push(file);

        Ok(())
    }

    pub fn close(&mut self) -> ZipResult<()> {
        self.archive.write_central_directory()
    }
}
