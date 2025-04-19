# DB Command Line Tool

A multi-function command line tool written in Rust with a nested command structure.

## Project Structure

The project is organized as a Rust workspace with the following crates:

- `db`: Main executable
- `essentials`: Shared library with common functionality
- `db-file`: File command implementation
- `db-dropbox`: Dropbox command implementation

## Command Examples

```
# File operations
$ db file copy --source file1.txt --destination file2.txt
$ db file list

# Dropbox operations
$ db dropbox file list
$ db dropbox file upload --source file.txt --destination /path/in/dropbox
$ db dropbox file download --source /path/in/dropbox --destination local_file.txt
$ db dropbox account info
$ db dropbox account usage
```

## Building

```
cargo build --release
```

The binary will be available at `target/release/db`.

## Development

To add new commands:

1. Create a new crate in the `crates/commands` directory
2. Implement the command interface
3. Add the crate as a dependency in `crates/db/Cargo.toml`
4. Update the main command enum in `crates/db/src/main.rs`


