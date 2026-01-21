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
        let compressed_size_low = self.stream.read_u32::<LittleEndian>()?;
        let uncompressed_size_low = self.stream.read_u32::<LittleEndian>()?;
        let filename_len = self.stream.read_u16::<LittleEndian>()?;
        let extra_len = self.stream.read_u16::<LittleEndian>()?;

        let mut filename_buf = vec![0u8; filename_len as usize];
        self.stream.read_exact(&mut filename_buf)?;
        file.filename = String::from_utf8_lossy(&filename_buf).to_string();

        let mut extra_buf = vec![0u8; extra_len as usize];
        self.stream.read_exact(&mut extra_buf)?;
        file.extrafield = extra_buf.clone();

        file.compressed_size = compressed_size_low as i64;
        file.uncompressed_size = uncompressed_size_low as i64;

        if compressed_size_low == 0xFFFFFFFF || uncompressed_size_low == 0xFFFFFFFF {
            self.parse_zip64_extra(&extra_buf, &mut file, true)?;
        }

        Ok(file)
    }

    fn parse_zip64_extra(&self, extra: &[u8], file: &mut ZipFile, is_lfh: bool) -> ZipResult<()> {
        let mut pos = 0;
        while pos + 4 <= extra.len() {
            let tag = u16::from_le_bytes([extra[pos], extra[pos+1]]);
            let size = u16::from_le_bytes([extra[pos+2], extra[pos+3]]) as usize;
            pos += 4;
            if tag == 0x0001 {
                let mut epos = pos;
                if file.uncompressed_size == 0xFFFFFFFF && epos + 8 <= pos + size {
                    file.uncompressed_size = i64::from_le_bytes(extra[epos..epos+8].try_into().unwrap());
                    epos += 8;
                }
                if file.compressed_size == 0xFFFFFFFF && epos + 8 <= pos + size {
                    file.compressed_size = i64::from_le_bytes(extra[epos..epos+8].try_into().unwrap());
                    epos += 8;
                }
                if !is_lfh {
                    if file.disk_offset == 0xFFFFFFFF && epos + 8 <= pos + size {
                        file.disk_offset = i64::from_le_bytes(extra[epos..epos+8].try_into().unwrap());
                        // epos += 8;
                    }
                }
                file.zip64 = true;
                break;
            }
            pos += size;
        }
        Ok(())
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

        let mut extra = file.extrafield.clone();
        if file.uncompressed_size >= 0xFFFFFFFF || file.compressed_size >= 0xFFFFFFFF {
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;

            let mut z64 = Vec::new();
            z64.write_u16::<LittleEndian>(0x0001)?; // Tag
            z64.write_u16::<LittleEndian>(16)?;     // Size
            z64.write_i64::<LittleEndian>(file.uncompressed_size)?;
            z64.write_i64::<LittleEndian>(file.compressed_size)?;
            extra.extend_from_slice(&z64);
        } else {
            self.stream.write_u32::<LittleEndian>(file.compressed_size as u32)?;
            self.stream.write_u32::<LittleEndian>(file.uncompressed_size as u32)?;
        }

        self.stream.write_u16::<LittleEndian>(file.filename.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(extra.len() as u16)?;
        self.stream.write_all(file.filename.as_bytes())?;
        self.stream.write_all(&extra)?;

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
        let entries_on_this_disk = self.stream.read_u16::<LittleEndian>()?;
        let total_entries = self.stream.read_u16::<LittleEndian>()?;
        let cd_size = self.stream.read_u32::<LittleEndian>()?;
        let cd_offset = self.stream.read_u32::<LittleEndian>()?;
        let comment_len = self.stream.read_u16::<LittleEndian>()?;

        let mut n_entries = total_entries as u64;
        let mut offset = cd_offset as u64;

        if entries_on_this_disk == 0xFFFF || total_entries == 0xFFFF || cd_size == 0xFFFFFFFF || cd_offset == 0xFFFFFFFF {
            if eocd_pos >= 20 {
                self.stream.seek(SeekFrom::Start(eocd_pos - 20))?;
                let sig64loc = self.stream.read_u32::<LittleEndian>()?;
                if sig64loc == MZ_ZIP_SIG_ZIP64_END_OF_CENTRAL_DIRECTORY_LOCATOR {
                    self.stream.seek(SeekFrom::Current(4))?; // skip disk
                    let eocd64_offset = self.stream.read_u64::<LittleEndian>()?;
                    self.stream.seek(SeekFrom::Start(eocd64_offset))?;
                    let sig64 = self.stream.read_u32::<LittleEndian>()?;
                    if sig64 == MZ_ZIP_SIG_ZIP64_END_OF_CENTRAL_DIRECTORY {
                        self.stream.seek(SeekFrom::Current(28))?; // skip sizes and versions
                        n_entries = self.stream.read_u64::<LittleEndian>()?;
                        let _cd_size64 = self.stream.read_u64::<LittleEndian>()?;
                        offset = self.stream.read_u64::<LittleEndian>()?;
                    }
                }
            }
        }

        let mut comment_buf = vec![0u8; comment_len as usize];
        self.stream.seek(SeekFrom::Start(eocd_pos + 22))?;
        self.stream.read_exact(&mut comment_buf)?;
        self.comment = String::from_utf8_lossy(&comment_buf).to_string();

        self.stream.seek(SeekFrom::Start(offset))?;
        for _ in 0..n_entries {
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

        if self.entries.len() >= 0xFFFF || cd_offset >= 0xFFFFFFFF || cd_size >= 0xFFFFFFFF {
            self.write_zip64_eocd(self.entries.len() as u64, cd_size, cd_offset)?;
        }
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
        let compressed_size_low = self.stream.read_u32::<LittleEndian>()?;
        let uncompressed_size_low = self.stream.read_u32::<LittleEndian>()?;
        let filename_len = self.stream.read_u16::<LittleEndian>()?;
        let extra_len = self.stream.read_u16::<LittleEndian>()?;
        let comment_len = self.stream.read_u16::<LittleEndian>()?;
        file.disk_number = self.stream.read_u16::<LittleEndian>()? as u32;
        file.internal_fa = self.stream.read_u16::<LittleEndian>()?;
        file.external_fa = self.stream.read_u32::<LittleEndian>()?;
        let disk_offset_low = self.stream.read_u32::<LittleEndian>()?;

        let mut filename_buf = vec![0u8; filename_len as usize];
        self.stream.read_exact(&mut filename_buf)?;
        file.filename = String::from_utf8_lossy(&filename_buf).to_string();

        let mut extra_buf = vec![0u8; extra_len as usize];
        self.stream.read_exact(&mut extra_buf)?;
        file.extrafield = extra_buf.clone();

        let mut comment_buf = vec![0u8; comment_len as usize];
        self.stream.read_exact(&mut comment_buf)?;
        file.comment = String::from_utf8_lossy(&comment_buf).to_string();

        file.compressed_size = compressed_size_low as i64;
        file.uncompressed_size = uncompressed_size_low as i64;
        file.disk_offset = disk_offset_low as i64;

        if compressed_size_low == 0xFFFFFFFF || uncompressed_size_low == 0xFFFFFFFF || disk_offset_low == 0xFFFFFFFF {
            self.parse_zip64_extra(&extra_buf, &mut file, false)?;
        }

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

        let mut extra = file.extrafield.clone();
        let use_zip64 = file.compressed_size >= 0xFFFFFFFF || file.uncompressed_size >= 0xFFFFFFFF || file.disk_offset >= 0xFFFFFFFF;

        if use_zip64 {
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;
        } else {
            self.stream.write_u32::<LittleEndian>(file.compressed_size as u32)?;
            self.stream.write_u32::<LittleEndian>(file.uncompressed_size as u32)?;
        }

        self.stream.write_u16::<LittleEndian>(file.filename.len() as u16)?;

        if use_zip64 {
            let mut z64_data = Vec::new();
            z64_data.write_i64::<LittleEndian>(file.uncompressed_size)?;
            z64_data.write_i64::<LittleEndian>(file.compressed_size)?;
            z64_data.write_i64::<LittleEndian>(file.disk_offset)?;

            extra.write_u16::<LittleEndian>(0x0001)?; // Tag
            extra.write_u16::<LittleEndian>(z64_data.len() as u16)?; // Size
            extra.extend(z64_data);
        }

        self.stream.write_u16::<LittleEndian>(extra.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(file.comment.len() as u16)?;
        self.stream.write_u16::<LittleEndian>(file.disk_number as u16)?;
        self.stream.write_u16::<LittleEndian>(file.internal_fa)?;
        self.stream.write_u32::<LittleEndian>(file.external_fa)?;

        if use_zip64 {
            self.stream.write_u32::<LittleEndian>(0xFFFFFFFF)?;
        } else {
            self.stream.write_u32::<LittleEndian>(file.disk_offset as u32)?;
        }

        self.stream.write_all(file.filename.as_bytes())?;
        self.stream.write_all(&extra)?;
        self.stream.write_all(file.comment.as_bytes())?;

        Ok(())
    }

    fn write_zip64_eocd(&mut self, n_entries: u64, cd_size: u64, cd_offset: u64) -> ZipResult<()> {
        let eocd64_offset = self.stream.seek(SeekFrom::Current(0))?;
        self.stream.write_u32::<LittleEndian>(MZ_ZIP_SIG_ZIP64_END_OF_CENTRAL_DIRECTORY)?;
        self.stream.write_u64::<LittleEndian>(44)?; // Size of EOCD64 record
        self.stream.write_u16::<LittleEndian>(45)?; // Made by
        self.stream.write_u16::<LittleEndian>(45)?; // Needed
        self.stream.write_u32::<LittleEndian>(0)?; // disk
        self.stream.write_u32::<LittleEndian>(0)?; // disk CD
        self.stream.write_u64::<LittleEndian>(n_entries)?;
        self.stream.write_u64::<LittleEndian>(n_entries)?;
        self.stream.write_u64::<LittleEndian>(cd_size)?;
        self.stream.write_u64::<LittleEndian>(cd_offset)?;

        self.stream.write_u32::<LittleEndian>(MZ_ZIP_SIG_ZIP64_END_OF_CENTRAL_DIRECTORY_LOCATOR)?;
        self.stream.write_u32::<LittleEndian>(0)?; // disk
        self.stream.write_u64::<LittleEndian>(eocd64_offset)?;
        self.stream.write_u32::<LittleEndian>(1)?; // total disks
        Ok(())
    }

    fn write_end_of_central_directory(&mut self, total_entries: u16, cd_size: u32, cd_offset: u32) -> ZipResult<()> {
        self.stream.write_u32::<LittleEndian>(MZ_ZIP_SIG_END_OF_CENTRAL_DIRECTORY)?;
        self.stream.write_u16::<LittleEndian>(0)?; // disk number
        self.stream.write_u16::<LittleEndian>(0)?; // disk with CD
        let n = if total_entries >= 0xFFFF { 0xFFFF } else { total_entries };
        self.stream.write_u16::<LittleEndian>(n)?;
        self.stream.write_u16::<LittleEndian>(n)?;
        let s = if cd_size >= 0xFFFFFFFF { 0xFFFFFFFF } else { cd_size };
        self.stream.write_u32::<LittleEndian>(s)?;
        let o = if cd_offset >= 0xFFFFFFFF { 0xFFFFFFFF } else { cd_offset };
        self.stream.write_u32::<LittleEndian>(o)?;
        self.stream.write_u16::<LittleEndian>(self.comment.len() as u16)?;
        self.stream.write_all(self.comment.as_bytes())?;
        Ok(())
    }

    fn find_eocd(&mut self) -> ZipResult<u64> {
        let size = self.stream.seek(SeekFrom::End(0))?;
        let mut search_size = 65536 + 22;
        if search_size > size {
            search_size = size;
        }

        let mut buf = vec![0u8; search_size as usize];
        self.stream.seek(SeekFrom::End(-(search_size as i64)))?;
        self.stream.read_exact(&mut buf)?;

        for i in (0..=(search_size - 22)).rev() {
            if &buf[i as usize..i as usize + 4] == &[0x50, 0x4b, 0x05, 0x06] {
                return Ok(size - search_size + i);
            }
        }

        Err(ZipError::Format)
    }
}
