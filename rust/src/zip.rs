use std::io::{Read, Write, Seek, SeekFrom};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::error::{ZipResult, ZipError};
use crate::types::ZipFile;
use crate::constants::*;
use crate::utils::{time_t_to_dos_date, dos_date_to_time_t};

pub struct ZipArchive<S: Read + Write + Seek> {
    pub stream: S,
    pub entries: Vec<ZipFile>,
    pub comment: String,
}

impl<S: Read + Write + Seek> ZipArchive<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            entries: Vec::new(),
            comment: String::new(),
        }
    }

    pub fn read_local_file_header(&mut self) -> ZipResult<ZipFile> {
        let sig = self.stream.read_u32::<LittleEndian>()?;
        if sig != MZ_ZIP_SIG_LOCAL_FILE_HEADER {
            return Err(ZipError::Format);
        }

        let mut file = ZipFile::default();
        file.version_needed = self.stream.read_u16::<LittleEndian>()?;
        file.flag = self.stream.read_u16::<LittleEndian>()?;
        file.compression_method = self.stream.read_u16::<LittleEndian>()?;
        let last_mod_time = self.stream.read_u16::<LittleEndian>()?;
        let last_mod_date = self.stream.read_u16::<LittleEndian>()?;
        file.modified_date = Some(dos_date_to_time_t(last_mod_date, last_mod_time));

        file.crc = self.stream.read_u32::<LittleEndian>()?;
        file.compressed_size = self.stream.read_u32::<LittleEndian>()? as i64;
        file.uncompressed_size = self.stream.read_u32::<LittleEndian>()? as i64;
        let filename_len = self.stream.read_u16::<LittleEndian>()?;
        let extra_len = self.stream.read_u16::<LittleEndian>()?;

        let mut filename_buf = vec![0u8; filename_len as usize];
        self.stream.read_exact(&mut filename_buf)?;
        file.filename = String::from_utf8_lossy(&filename_buf).to_string();

        let mut extra_buf = vec![0u8; extra_len as usize];
        self.stream.read_exact(&mut extra_buf)?;
        file.extrafield = extra_buf;

        Ok(file)
    }

    pub fn write_local_file_header(&mut self, file: &ZipFile) -> ZipResult<()> {
        self.stream.write_u32::<LittleEndian>(MZ_ZIP_SIG_LOCAL_FILE_HEADER)?;
        self.stream.write_u16::<LittleEndian>(file.version_needed)?;
        self.stream.write_u16::<LittleEndian>(file.flag)?;
        self.stream.write_u16::<LittleEndian>(file.compression_method)?;

        let (dos_date, dos_time) = if let Some(dt) = file.modified_date {
            time_t_to_dos_date(dt)
        } else {
            (0, 0)
        };
        self.stream.write_u16::<LittleEndian>(dos_time)?;
        self.stream.write_u16::<LittleEndian>(dos_date)?;

        self.stream.write_u32::<LittleEndian>(file.crc)?;

        if file.zip64 {
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;
        } else {
            self.stream.write_u32::<LittleEndian>(file.compressed_size as u32)?;
            self.stream.write_u32::<LittleEndian>(file.uncompressed_size as u32)?;
        }

        self.stream.write_u16::<LittleEndian>(file.filename.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(file.extrafield.len() as u16)?;
        self.stream.write_all(file.filename.as_bytes())?;
        self.stream.write_all(&file.extrafield)?;

        Ok(())
    }

    pub fn read_central_directory(&mut self) -> ZipResult<()> {
        let eocd_pos = self.find_eocd()?;
        self.stream.seek(SeekFrom::Start(eocd_pos))?;

        let sig = self.stream.read_u32::<LittleEndian>()?;
        if sig != MZ_ZIP_SIG_END_OF_CENTRAL_DIRECTORY {
            return Err(ZipError::Format);
        }

        self.stream.seek(SeekFrom::Current(4))?; // Skip disk numbers
        let _entries_on_this_disk = self.stream.read_u16::<LittleEndian>()?;
        let total_entries = self.stream.read_u16::<LittleEndian>()?;
        let _cd_size = self.stream.read_u32::<LittleEndian>()?;
        let cd_offset = self.stream.read_u32::<LittleEndian>()?;
        let comment_len = self.stream.read_u16::<LittleEndian>()?;

        let mut comment_buf = vec![0u8; comment_len as usize];
        self.stream.read_exact(&mut comment_buf)?;
        self.comment = String::from_utf8_lossy(&comment_buf).to_string();

        self.stream.seek(SeekFrom::Start(cd_offset as u64))?;
        for _ in 0..total_entries {
            let entry = self.read_central_directory_header()?;
            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn write_central_directory(&mut self) -> ZipResult<()> {
        let cd_offset = self.stream.seek(SeekFrom::Current(0))?;
        let entries = self.entries.clone();
        for entry in &entries {
            self.write_central_directory_header(entry)?;
        }
        let cd_end = self.stream.seek(SeekFrom::Current(0))?;
        let cd_size = cd_end - cd_offset;

        self.write_end_of_central_directory(self.entries.len() as u16, cd_size as u32, cd_offset as u32)?;
        Ok(())
    }

    fn read_central_directory_header(&mut self) -> ZipResult<ZipFile> {
        let sig = self.stream.read_u32::<LittleEndian>()?;
        if sig != MZ_ZIP_SIG_CENTRAL_FILE_HEADER {
            return Err(ZipError::Format);
        }

        let mut file = ZipFile::default();
        file.version_madeby = self.stream.read_u16::<LittleEndian>()?;
        file.version_needed = self.stream.read_u16::<LittleEndian>()?;
        file.flag = self.stream.read_u16::<LittleEndian>()?;
        file.compression_method = self.stream.read_u16::<LittleEndian>()?;
        let last_mod_time = self.stream.read_u16::<LittleEndian>()?;
        let last_mod_date = self.stream.read_u16::<LittleEndian>()?;
        file.modified_date = Some(dos_date_to_time_t(last_mod_date, last_mod_time));

        file.crc = self.stream.read_u32::<LittleEndian>()?;
        file.compressed_size = self.stream.read_u32::<LittleEndian>()? as i64;
        file.uncompressed_size = self.stream.read_u32::<LittleEndian>()? as i64;
        let filename_len = self.stream.read_u16::<LittleEndian>()?;
        let extra_len = self.stream.read_u16::<LittleEndian>()?;
        let comment_len = self.stream.read_u16::<LittleEndian>()?;
        file.disk_number = self.stream.read_u16::<LittleEndian>()? as u32;
        file.internal_fa = self.stream.read_u16::<LittleEndian>()?;
        file.external_fa = self.stream.read_u32::<LittleEndian>()?;
        file.disk_offset = self.stream.read_u32::<LittleEndian>()? as i64;

        let mut filename_buf = vec![0u8; filename_len as usize];
        self.stream.read_exact(&mut filename_buf)?;
        file.filename = String::from_utf8_lossy(&filename_buf).to_string();

        let mut extra_buf = vec![0u8; extra_len as usize];
        self.stream.read_exact(&mut extra_buf)?;
        file.extrafield = extra_buf;

        let mut comment_buf = vec![0u8; comment_len as usize];
        self.stream.read_exact(&mut comment_buf)?;
        file.comment = String::from_utf8_lossy(&comment_buf).to_string();

        Ok(file)
    }

    fn write_central_directory_header(&mut self, file: &ZipFile) -> ZipResult<()> {
        self.stream.write_u32::<LittleEndian>(MZ_ZIP_SIG_CENTRAL_FILE_HEADER)?;
        self.stream.write_u16::<LittleEndian>(file.version_madeby)?;
        self.stream.write_u16::<LittleEndian>(file.version_needed)?;
        self.stream.write_u16::<LittleEndian>(file.flag)?;
        self.stream.write_u16::<LittleEndian>(file.compression_method)?;

        let (dos_date, dos_time) = if let Some(dt) = file.modified_date {
            time_t_to_dos_date(dt)
        } else {
            (0, 0)
        };
        self.stream.write_u16::<LittleEndian>(dos_time)?;
        self.stream.write_u16::<LittleEndian>(dos_date)?;

        self.stream.write_u32::<LittleEndian>(file.crc)?;
        self.stream.write_u32::<LittleEndian>(file.compressed_size as u32)?;
        self.stream.write_u32::<LittleEndian>(file.uncompressed_size as u32)?;
        self.stream.write_u16::<LittleEndian>(file.filename.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(file.extrafield.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(file.comment.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(file.disk_number as u16)?;
        self.stream.write_u16::<LittleEndian>(file.internal_fa)?;
        self.stream.write_u32::<LittleEndian>(file.external_fa)?;
        self.stream.write_u32::<LittleEndian>(file.disk_offset as u32)?;

        self.stream.write_all(file.filename.as_bytes())?;
        self.stream.write_all(&file.extrafield)?;
        self.stream.write_all(file.comment.as_bytes())?;

        Ok(())
    }

    fn write_end_of_central_directory(&mut self, total_entries: u16, cd_size: u32, cd_offset: u32) -> ZipResult<()> {
        self.stream.write_u32::<LittleEndian>(MZ_ZIP_SIG_END_OF_CENTRAL_DIRECTORY)?;
        self.stream.write_u16::<LittleEndian>(0)?; // disk number
        self.stream.write_u16::<LittleEndian>(0)?; // disk with CD
        self.stream.write_u16::<LittleEndian>(total_entries)?;
        self.stream.write_u16::<LittleEndian>(total_entries)?;
        self.stream.write_u32::<LittleEndian>(cd_size)?;
        self.stream.write_u32::<LittleEndian>(cd_offset)?;
        self.stream.write_u16::<LittleEndian>(self.comment.len() as u16)?;
        self.stream.write_all(self.comment.as_bytes())?;
        Ok(())
    }

    fn find_eocd(&mut self) -> ZipResult<u64> {
        let size = self.stream.seek(SeekFrom::End(0))?;
        let mut read_size = 1024;
        if read_size > size {
            read_size = size;
        }

        let mut buf = vec![0u8; read_size as usize];
        self.stream.seek(SeekFrom::End(-(read_size as i64)))?;
        self.stream.read_exact(&mut buf)?;

        for i in (0..=(read_size - 22)).rev() {
            if &buf[i as usize..i as usize + 4] == &[0x50, 0x4b, 0x05, 0x06] {
                return Ok(size - read_size + i);
            }
        }

        Err(ZipError::Format)
    }
}
