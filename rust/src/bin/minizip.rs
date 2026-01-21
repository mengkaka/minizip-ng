use minizip_rs::zip_rw::ZipWriter;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: minizip-rs <archive.zip> <files...>");
        return;
    }

    let archive_path = &args[1];
    let mut writer = ZipWriter::create_file(archive_path).expect("Failed to create archive");

    for file_path in &args[2..] {
        let data = fs::read(file_path).expect("Failed to read file");
        writer.add_buffer(&data, file_path).expect("Failed to add file to archive");
        println!("Adding {}", file_path);
    }
}
