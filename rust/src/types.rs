use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub struct ZipFile {
    pub version_madeby: u16,
    pub version_needed: u16,
    pub flag: u16,
    pub compression_method: u16,
    pub modified_date: Option<DateTime<Utc>>,
    pub accessed_date: Option<DateTime<Utc>>,
    pub creation_date: Option<DateTime<Utc>>,
    pub crc: u32,
    pub compressed_size: i64,
    pub uncompressed_size: i64,
    pub filename: String,
    pub extrafield: Vec<u8>,
    pub comment: String,
    pub disk_number: u32,
    pub disk_offset: i64,
    pub internal_fa: u16,
    pub external_fa: u32,
    pub zip64: bool,
    pub aes_version: u16,
    pub aes_strength: u8,
    pub pk_verify: u16,
}
