pub mod constants;
pub mod error;
pub mod types;
pub mod strm;
pub mod crypt;
pub mod utils;
pub mod zip;
pub mod zip_rw;

pub use constants::*;
pub use error::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::zip_rw::ZipWriter;
    use crate::strm::mem::MemoryStream;

    #[test]
    fn test_create_zip_in_memory() {
        let mem = MemoryStream::new();
        let mut writer = ZipWriter::new(mem);
        writer.add_buffer(b"hello world", "hello.txt").expect("Failed to add buffer");
        writer.close().expect("Failed to close zip");

        let data = writer.archive.stream.into_inner();
        assert!(data.len() > 0);
        // Signature of LFH is 0x04034b50 (Little Endian: 50 4b 03 04)
        assert_eq!(&data[0..4], &[0x50, 0x4b, 0x03, 0x04]);

        // Should contain EOCD at the end (approx)
        // EOCD signature is 0x06054b50 (50 4b 05 06)
        let found_eocd = data.windows(4).any(|w| w == &[0x50, 0x4b, 0x05, 0x06]);
        assert!(found_eocd);
    }
}
