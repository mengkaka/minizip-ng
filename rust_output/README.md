# Minizip Rust Reimplementation

This is a professional Rust reimplementation of the `minizip-ng` library, providing a command-line tool for ZIP archive manipulation.

## Features

- **Compression Methods**: Supports `Deflate` and `Stored`.
- **Encryption**: Supports Traditional PKWARE encryption/decryption with password protection.
- **Integrity**: Full CRC-32 verification during extraction.
- **CLI Commands**:
  - `list`: View archive contents.
  - `add`: Create or append to an archive (with compression level and password support).
  - `extract`: Unzip files to a destination.
  - `remove`: Remove entries from an archive.
  - `help`: Detailed usage for each command.

## Usage Examples

### 1. List Files
```bash
./minizip list archive.zip
```

### 2. Add Files with Encryption
```bash
./minizip add archive.zip file1.txt file2.txt -p mypassword -l 9
```

### 3. Extract Files
```bash
./minizip extract archive.zip -d ./output -p mypassword
```

### 4. Remove a File
```bash
./minizip remove archive.zip file1.txt
```

## Compilation for M4 Mac (Apple Silicon)

Because this environment is Linux-based, a native macOS ARM64 binary cannot be directly generated here. However, I have provided the full source code which you can easily compile on your M4 Mac.

To use `minizip` on your M4 Mac, follow these steps:

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Compile the Tool**:
   Navigate to the `rust` folder and run:
   ```bash
   cargo build --release
   ```
3. **Run**:
   The binary will be at `rust/target/release/minizip`. You can use it directly:
   ```bash
   ./minizip --help
   ```

I have used `clap` to provide a robust CLI experience that matches the functionality of the original C implementation while adhering to Rust safety standards.
