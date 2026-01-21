use std::io::{Read, Write, Seek, SeekFrom};
use std::fs;
use std::path::Path;
use crate::error::{ZipResult, ZipError};
use crate::zip::ZipArchive;
use crate::types::ZipFile;
use crate::strm::os::FileStream;
use crate::crypt::pkcrypt::PkCrypt;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use crc32fast::Hasher;

pub struct ZipReader<S: Read + Write + Seek> {
    pub archive: ZipArchive<S>,
    pub current_entry: Option<usize>,
    pub password: Option<String>,
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
            password: None,
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
            password: None,
        })
    }

    pub fn list(&self) -> ZipResult<()> {
        println!("      Packed     Unpacked Ratio Method   Attribs Date     Time  CRC-32     Name");
        println!("      ------     -------- ----- ------   ------- ----     ----  ------     ----");
        for entry in &self.archive.entries {
            let ratio = if entry.uncompressed_size > 0 {
                (entry.compressed_size * 100) / entry.uncompressed_size
            } else {
                0
            };
            let method = match entry.compression_method {
                0 => "Stored",
                8 => "Deflate",
                12 => "BZip2",
                14 => "LZMA",
                93 => "Zstd",
                95 => "XZ",
                _ => "Unknown",
            };
            let crypt = if (entry.flag & crate::constants::MZ_ZIP_FLAG_ENCRYPTED) != 0 { "*" } else { " " };
            let date_str = entry.modified_date.map(|d| d.format("%m-%d-%y %H:%M").to_string()).unwrap_or_default();
            println!("{:12} {:12} {:3}% {:7}{} {:8x} {} {:8x}   {}",
                entry.compressed_size, entry.uncompressed_size, ratio, method, crypt, entry.external_fa, date_str, entry.crc, entry.filename);
        }
        Ok(())
    }

    pub fn extract_all(&mut self, destination: &str) -> ZipResult<()> {
        let entries_count = self.archive.entries.len();
        for i in 0..entries_count {
            self.current_entry = Some(i);
            self.extract_current_entry(destination)?;
        }
        Ok(())
    }

    fn extract_current_entry(&mut self, destination: &str) -> ZipResult<()> {
        let entry_idx = self.current_entry.ok_or(ZipError::Param)?;
        let entry = self.archive.entries[entry_idx].clone();

        let out_path = Path::new(destination).join(&entry.filename);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if entry.filename.ends_with('/') {
            fs::create_dir_all(&out_path)?;
            return Ok(());
        }

        println!("Extracting {}", entry.filename);

        self.archive.stream.seek(SeekFrom::Start(entry.disk_offset as u64))?;
        self.archive.read_local_file_header()?; // Skip LFH

        let mut reader: Box<dyn Read> = if (entry.flag & crate::constants::MZ_ZIP_FLAG_ENCRYPTED) != 0 {
            let pass = self.password.as_ref().ok_or(ZipError::Password)?;
            let mut pkcrypt = PkCrypt::new(pass);
            let mut encrypted_data = vec![0u8; entry.compressed_size as usize];
            self.archive.stream.read_exact(&mut encrypted_data)?;

            if encrypted_data.len() < 12 {
                return Err(ZipError::Data);
            }

            let mut decrypted_data = Vec::with_capacity(encrypted_data.len() - 12);
            for i in 0..12 {
                pkcrypt.decrypt_byte(encrypted_data[i]);
            }
            for i in 12..encrypted_data.len() {
                decrypted_data.push(pkcrypt.decrypt_byte(encrypted_data[i]));
            }
            Box::new(std::io::Cursor::new(decrypted_data))
        } else {
            Box::new((&mut self.archive.stream).take(entry.compressed_size as u64))
        };

        let mut out_data = Vec::new();

        match entry.compression_method {
            0 => {
                std::io::copy(&mut reader, &mut out_data)?;
            }
            8 => {
                let mut decoder = DeflateDecoder::new(reader);
                std::io::copy(&mut decoder, &mut out_data)?;
            }
            12 => {
                let mut decoder = bzip2::read::BzDecoder::new(reader);
                std::io::copy(&mut decoder, &mut out_data)?;
            }
            14 => {
                // LZMA in Zip is complex, usually uses a specific header.
                // For now use xz2 as a proxy if simple
                return Err(ZipError::Support);
            }
            93 => {
                let mut decoder = zstd::stream::read::Decoder::new(reader)?;
                std::io::copy(&mut decoder, &mut out_data)?;
            }
            95 => {
                let mut decoder = xz2::read::XzDecoder::new(reader);
                std::io::copy(&mut decoder, &mut out_data)?;
            }
            _ => return Err(ZipError::Support),
        }

        // Verify CRC
        let mut hasher = Hasher::new();
        hasher.update(&out_data);
        if hasher.finalize() != entry.crc {
            return Err(ZipError::Crc);
        }

        fs::write(&out_path, out_data)?;

        Ok(())
    }
}

pub struct ZipWriter<S: Read + Write + Seek> {
    pub archive: ZipArchive<S>,
    pub compress_method: u16,
    pub compress_level: i16,
    pub password: Option<String>,
}

impl ZipWriter<FileStream> {
    pub fn create_file(path: &str) -> ZipResult<Self> {
        let mut fs = FileStream::new();
        fs.open(path, crate::constants::MZ_OPEN_MODE_CREATE | crate::constants::MZ_OPEN_MODE_WRITE)?;
        let archive = ZipArchive::new(fs);
        Ok(Self {
            archive,
            compress_method: crate::constants::MZ_COMPRESS_METHOD_DEFLATE,
            compress_level: -1,
            password: None,
        })
    }
}

impl<S: Read + Write + Seek> ZipWriter<S> {
    pub fn new(stream: S) -> Self {
        Self {
            archive: ZipArchive::new(stream),
            compress_method: crate::constants::MZ_COMPRESS_METHOD_DEFLATE,
            compress_level: -1,
            password: None,
        }
    }

    pub fn add_file(&mut self, path: &str, filename_in_zip: &str) -> ZipResult<()> {
        let data = fs::read(path)?;
        self.add_buffer(&data, filename_in_zip)
    }

    pub fn add_buffer(&mut self, data: &[u8], filename: &str) -> ZipResult<()> {
        let mut file = ZipFile::default();
        file.filename = filename.to_string();
        file.uncompressed_size = data.len() as i64;
        file.compression_method = self.compress_method;
        file.version_needed = 20;
        file.version_madeby = 45; // UNIX
        file.modified_date = Some(chrono::Utc::now());

        let mut hasher = Hasher::new();
        hasher.update(data);
        file.crc = hasher.finalize();

        file.disk_offset = self.archive.stream.seek(SeekFrom::Current(0))? as i64;

        let mut final_data = match self.compress_method {
            0 => data.to_vec(),
            8 => {
                let level = if self.compress_level == -1 { Compression::default() } else { Compression::new(self.compress_level as u32) };
                let mut encoder = DeflateEncoder::new(Vec::new(), level);
                encoder.write_all(data)?;
                encoder.finish()?
            },
            12 => {
                let level = if self.compress_level == -1 { bzip2::Compression::default() } else { bzip2::Compression::new(self.compress_level as u32) };
                let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), level);
                encoder.write_all(data)?;
                encoder.finish()?
            },
            93 => {
                let level = if self.compress_level == -1 { 3 } else { self.compress_level as i32 };
                let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), level)?;
                encoder.write_all(data)?;
                encoder.finish()?
            },
            95 => {
                let level = if self.compress_level == -1 { 6 } else { self.compress_level as u32 };
                let mut encoder = xz2::write::XzEncoder::new(Vec::new(), level);
                encoder.write_all(data)?;
                encoder.finish()?
            },
            _ => return Err(ZipError::Support),
        };

        if let Some(pass) = &self.password {
            file.flag |= crate::constants::MZ_ZIP_FLAG_ENCRYPTED;
            let mut pkcrypt = PkCrypt::new(pass);
            let mut encrypted = Vec::with_capacity(final_data.len() + 12);

            // 12 bytes header
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let mut header = [0u8; 12];
            for i in 0..11 {
                header[i] = rng.gen();
            }
            header[11] = (file.crc >> 24) as u8; // Verifier

            for &b in &header {
                encrypted.push(pkcrypt.encrypt_byte(b));
            }
            for &b in &final_data {
                encrypted.push(pkcrypt.encrypt_byte(b));
            }
            final_data = encrypted;
        }

        file.compressed_size = final_data.len() as i64;

        self.archive.write_local_file_header(&file)?;
        self.archive.stream.write_all(&final_data)?;

        self.archive.entries.push(file);

        Ok(())
    }

    pub fn add_raw_entry(&mut self, mut file: ZipFile, data: &[u8]) -> ZipResult<()> {
        file.disk_offset = self.archive.stream.seek(SeekFrom::Current(0))? as i64;
        self.archive.write_local_file_header(&file)?;
        self.archive.stream.write_all(data)?;
        self.archive.entries.push(file);
        Ok(())
    }

    pub fn delete_file(&mut self, filename: &str) -> ZipResult<()> {
        self.archive.entries.retain(|e| e.filename != filename);
        Ok(())
    }

    pub fn close(&mut self) -> ZipResult<()> {
        self.archive.write_central_directory()
    }
}
