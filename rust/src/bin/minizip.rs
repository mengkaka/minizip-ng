use minizip_rs::zip_rw::{ZipReader, ZipWriter};
use std::env;
use std::path::Path;

fn help() {
    println!("Usage: minizip [-l|-x] [-d dir] [-p password] [-0 to -9] file.zip [files...]");
    println!("  -l  List files");
    println!("  -x  Extract files");
    println!("  -d  Destination directory");
    println!("  -p  Password");
    println!("  -0  Store only");
    println!("  -1 to -9  Compression level");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        help();
        return;
    }

    let mut do_list = false;
    let mut do_extract = false;
    let mut destination = ".".to_string();
    let mut zip_file = String::new();
    let mut files = Vec::new();
    let mut compress_level: i16 = -1;
    let mut compress_method = 8; // Deflate
    let mut password: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => do_list = true,
            "-x" => do_extract = true,
            "-d" => {
                if i + 1 < args.len() {
                    destination = args[i + 1].clone();
                    i += 1;
                }
            }
            "-p" => {
                if i + 1 < args.len() {
                    password = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            arg if arg.starts_with('-') && arg.len() == 2 && arg.chars().nth(1).unwrap().is_digit(10) => {
                compress_level = arg[1..2].parse::<i16>().unwrap();
                if compress_level == 0 {
                    compress_method = 0;
                } else {
                    compress_method = 8;
                }
            }
            arg => {
                if zip_file.is_empty() {
                    zip_file = arg.to_string();
                } else {
                    files.push(arg.to_string());
                }
            }
        }
        i += 1;
    }

    if zip_file.is_empty() {
        help();
        return;
    }

    if do_list {
        let reader = ZipReader::open_file(&zip_file).expect("Failed to open zip file");
        reader.list().expect("Failed to list files");
    } else if do_extract {
        let mut reader = ZipReader::open_file(&zip_file).expect("Failed to open zip file");
        reader.password = password;
        reader.extract_all(&destination).expect("Failed to extract files");
    } else {
        // Add files
        let mut writer = ZipWriter::create_file(&zip_file).expect("Failed to create zip file");
        writer.compress_method = compress_method;
        writer.compress_level = compress_level;
        writer.password = password;

        for file in files {
            let path = Path::new(&file);
            if !path.exists() {
                println!("File not found: {}", file);
                continue;
            }
            let filename = path.file_name().unwrap().to_str().unwrap();
            writer.add_file(&file, filename).expect("Failed to add file");
            println!("Adding {}", file);
        }
        writer.close().expect("Failed to close zip file");
    }
}
