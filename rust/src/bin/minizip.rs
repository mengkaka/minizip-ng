use clap::{Parser, Subcommand};
use minizip_rs::zip_rw::{ZipReader, ZipWriter};
use std::path::PathBuf;
use std::fs;
use std::io::{Read, Seek, SeekFrom};

#[derive(Parser)]
#[command(name = "minizip")]
#[command(about = "A Rust reimplementation of minizip", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List files in the zip archive
    List {
        /// The zip file to list
        zip_file: PathBuf,
    },
    /// Extract files from the zip archive
    Extract {
        /// The zip file to extract
        zip_file: PathBuf,

        /// Destination directory
        #[arg(short, long, default_value = ".")]
        destination: PathBuf,

        /// Password for encrypted files
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add files to a zip archive
    Add {
        /// The zip file to create/append
        zip_file: PathBuf,

        /// Files to add
        files: Vec<PathBuf>,

        /// Compression level (0-9)
        #[arg(short, long, default_value_t = 6)]
        level: i16,

        /// Password for encryption
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Remove a file from the zip archive
    Remove {
        /// The zip file to modify
        zip_file: PathBuf,

        /// File to remove
        filename: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { zip_file } => {
            let reader = ZipReader::open_file(zip_file.to_str().unwrap())
                .expect("Failed to open zip file");
            reader.list().expect("Failed to list files");
        }
        Commands::Extract { zip_file, destination, password } => {
            let mut reader = ZipReader::open_file(zip_file.to_str().unwrap())
                .expect("Failed to open zip file");
            reader.password = password;
            reader.extract_all(destination.to_str().unwrap())
                .expect("Failed to extract files");
        }
        Commands::Add { zip_file, files, level, password } => {
            let mut writer = ZipWriter::create_file(zip_file.to_str().unwrap())
                .expect("Failed to create zip file");

            if level == 0 {
                writer.compress_method = 0;
            } else {
                writer.compress_method = 8;
                writer.compress_level = level;
            }
            writer.password = password;

            for file in files {
                if !file.exists() {
                    println!("File not found: {:?}", file);
                    continue;
                }
                let filename = file.file_name().unwrap().to_str().unwrap();
                writer.add_file(file.to_str().unwrap(), filename)
                    .expect("Failed to add file");
                println!("Adding {:?}", file);
            }
            writer.close().expect("Failed to close zip file");
        }
        Commands::Remove { zip_file, filename } => {
            let mut reader = ZipReader::open_file(zip_file.to_str().unwrap())
                .expect("Failed to open zip file");

            let tmp_zip = zip_file.with_extension("tmp.zip");
            let mut writer = ZipWriter::create_file(tmp_zip.to_str().unwrap())
                .expect("Failed to create temporary zip file");

            let mut found = false;
            let entries = reader.archive.entries.clone();
            for entry in entries {
                if entry.filename == filename {
                    println!("Removing {}", filename);
                    found = true;
                    continue;
                }

                println!("Copying {}", entry.filename);
                // Seek to data
                reader.archive.stream.seek(SeekFrom::Start(entry.disk_offset as u64)).unwrap();
                reader.archive.read_local_file_header().unwrap();
                let mut data = vec![0u8; entry.compressed_size as usize];
                reader.archive.stream.read_exact(&mut data).unwrap();

                writer.add_raw_entry(entry, &data).expect("Failed to copy entry");
            }

            writer.close().expect("Failed to close temporary zip file");

            if found {
                fs::rename(&tmp_zip, &zip_file).expect("Failed to replace original zip file");
            } else {
                println!("File {} not found in archive", filename);
                fs::remove_file(&tmp_zip).ok();
            }
        }
    }
}
