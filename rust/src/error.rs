use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZipError {
    #[error("Stream error")]
    Stream,
    #[error("Data error")]
    Data,
    #[error("Memory error")]
    Memory,
    #[error("Buffer error")]
    Buffer,
    #[error("Version error")]
    Version,
    #[error("End of list")]
    EndOfList,
    #[error("End of stream")]
    EndOfStream,
    #[error("Parameter error")]
    Param,
    #[error("Format error")]
    Format,
    #[error("Internal error")]
    Internal,
    #[error("CRC error")]
    Crc,
    #[error("Crypt error")]
    Crypt,
    #[error("Exist error")]
    Exist,
    #[error("Password error")]
    Password,
    #[error("Support error")]
    Support,
    #[error("Hash error")]
    Hash,
    #[error("Open error")]
    Open,
    #[error("Close error")]
    Close,
    #[error("Seek error")]
    Seek,
    #[error("Tell error")]
    Tell,
    #[error("Read error")]
    Read,
    #[error("Write error")]
    Write,
    #[error("Sign error")]
    Sign,
    #[error("Symlink error")]
    Symlink,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ZipResult<T> = Result<T, ZipError>;
