pub const MZ_VERSION: &str = "4.0.10";

pub const MZ_OK: i32 = 0;
pub const MZ_STREAM_ERROR: i32 = -1;
pub const MZ_DATA_ERROR: i32 = -3;
pub const MZ_MEM_ERROR: i32 = -4;
pub const MZ_BUF_ERROR: i32 = -5;
pub const MZ_VERSION_ERROR: i32 = -6;

pub const MZ_END_OF_LIST: i32 = -100;
pub const MZ_END_OF_STREAM: i32 = -101;

pub const MZ_PARAM_ERROR: i32 = -102;
pub const MZ_FORMAT_ERROR: i32 = -103;
pub const MZ_INTERNAL_ERROR: i32 = -104;
pub const MZ_CRC_ERROR: i32 = -105;
pub const MZ_CRYPT_ERROR: i32 = -106;
pub const MZ_EXIST_ERROR: i32 = -107;
pub const MZ_PASSWORD_ERROR: i32 = -108;
pub const MZ_SUPPORT_ERROR: i32 = -109;
pub const MZ_HASH_ERROR: i32 = -110;
pub const MZ_OPEN_ERROR: i32 = -111;
pub const MZ_CLOSE_ERROR: i32 = -112;
pub const MZ_SEEK_ERROR: i32 = -113;
pub const MZ_TELL_ERROR: i32 = -114;
pub const MZ_READ_ERROR: i32 = -115;
pub const MZ_WRITE_ERROR: i32 = -116;
pub const MZ_SIGN_ERROR: i32 = -117;
pub const MZ_SYMLINK_ERROR: i32 = -118;

pub const MZ_OPEN_MODE_READ: i32 = 0x01;
pub const MZ_OPEN_MODE_WRITE: i32 = 0x02;
pub const MZ_OPEN_MODE_READWRITE: i32 = MZ_OPEN_MODE_READ | MZ_OPEN_MODE_WRITE;
pub const MZ_OPEN_MODE_APPEND: i32 = 0x04;
pub const MZ_OPEN_MODE_CREATE: i32 = 0x08;
pub const MZ_OPEN_MODE_EXISTING: i32 = 0x10;
pub const MZ_OPEN_MODE_NOFOLLOW: i32 = 0x20;

pub const MZ_COMPRESS_METHOD_STORE: u16 = 0;
pub const MZ_COMPRESS_METHOD_DEFLATE: u16 = 8;
pub const MZ_COMPRESS_METHOD_BZIP2: u16 = 12;
pub const MZ_COMPRESS_METHOD_LZMA: u16 = 14;
pub const MZ_COMPRESS_METHOD_ZSTD: u16 = 93;
pub const MZ_COMPRESS_METHOD_XZ: u16 = 95;
pub const MZ_COMPRESS_METHOD_AES: u16 = 99;

pub const MZ_ZIP_FLAG_ENCRYPTED: u16 = 1 << 0;
pub const MZ_ZIP_FLAG_DATA_DESCRIPTOR: u16 = 1 << 3;
pub const MZ_ZIP_FLAG_UTF8: u16 = 1 << 11;

pub const MZ_ZIP_EXTENSION_ZIP64: u16 = 0x0001;
pub const MZ_ZIP_EXTENSION_NTFS: u16 = 0x000a;
pub const MZ_ZIP_EXTENSION_AES: u16 = 0x9901;
pub const MZ_ZIP_EXTENSION_UNIX1: u16 = 0x000d;

pub const MZ_HOST_SYSTEM_MSDOS: u8 = 0;
pub const MZ_HOST_SYSTEM_UNIX: u8 = 3;
pub const MZ_HOST_SYSTEM_WINDOWS_NTFS: u8 = 10;
pub const MZ_HOST_SYSTEM_OSX_DARWIN: u8 = 19;

pub const MZ_ZIP_SIG_LOCAL_FILE_HEADER: u32 = 0x04034b50;
pub const MZ_ZIP_SIG_CENTRAL_FILE_HEADER: u32 = 0x02014b50;
pub const MZ_ZIP_SIG_END_OF_CENTRAL_DIRECTORY: u32 = 0x06054b50;
pub const MZ_ZIP_SIG_ZIP64_END_OF_CENTRAL_DIRECTORY: u32 = 0x06064b50;
pub const MZ_ZIP_SIG_ZIP64_END_OF_CENTRAL_DIRECTORY_LOCATOR: u32 = 0x07064b50;
pub const MZ_ZIP_SIG_DATA_DESCRIPTOR: u32 = 0x08074b50;
