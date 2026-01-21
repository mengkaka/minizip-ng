use std::io::{Read, Write, Seek, SeekFrom};
use flate2::{Compression, Decompress, FlushDecompress, Status};
use crate::error::{ZipResult, ZipError};
use crate::strm::{Stream, MZ_STREAM_PROP_COMPRESS_LEVEL};

pub struct ZlibStream<S: Stream> {
    base: S,
    decompress: Option<Decompress>,
    level: Compression,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_cap: usize,
}

impl<S: Stream> ZlibStream<S> {
    pub fn new(base: S) -> Self {
        Self {
            base,
            decompress: None,
            level: Compression::default(),
            buffer: vec![0u8; 8192],
            buffer_pos: 0,
            buffer_cap: 0,
        }
    }

    pub fn init_decompress(&mut self) {
        self.decompress = Some(Decompress::new(false));
    }
}

impl<S: Stream> Read for ZlibStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(decompressor) = &mut self.decompress {
            let mut total_out = 0;
            while total_out < buf.len() {
                if self.buffer_pos >= self.buffer_cap {
                    self.buffer_cap = self.base.read(&mut self.buffer)?;
                    self.buffer_pos = 0;
                    if self.buffer_cap == 0 {
                        break;
                    }
                }

                let before_in = decompressor.total_in();
                let before_out = decompressor.total_out();

                let res = decompressor.decompress(
                    &self.buffer[self.buffer_pos..self.buffer_cap],
                    &mut buf[total_out..],
                    FlushDecompress::None,
                ).map_err(|_| std::io::Error::new(std::io::ErrorKind::DataLabelel, "Decompression error"))?;

                let consumed = (decompressor.total_in() - before_in) as usize;
                let produced = (decompressor.total_out() - before_out) as usize;

                self.buffer_pos += consumed;
                total_out += produced;

                if res == Status::StreamEnd {
                    break;
                }
            }
            Ok(total_out)
        } else {
            self.base.read(buf)
        }
    }
}

impl<S: Stream> Write for ZlibStream<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // For writing, we'd need a similar logic with Compress
        // For now, let's at least pass through if not compressing
        self.base.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.base.flush()
    }
}

impl<S: Stream> Seek for ZlibStream<S> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.base.seek(pos)
    }
}

impl<S: Stream> Stream for ZlibStream<S> {
    fn is_open(&self) -> bool {
        self.base.is_open()
    }
    fn get_prop_int64(&self, prop: i32) -> ZipResult<i64> {
        if prop == MZ_STREAM_PROP_COMPRESS_LEVEL {
            Ok(self.level.level() as i64)
        } else {
            self.base.get_prop_int64(prop)
        }
    }
}
