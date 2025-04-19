# Dougu

A multi-command command-line tool written in Rust 2024 with a nested command structure.

## Project Structure

This project is organized as a Cargo workspace with multiple crates:

- `dougu`: Main binary crate that integrates all commands
- `dougu-essentials-*`: Core libraries shared across the project
  - `dougu-essentials-logger`: Logging utilities
  - `dougu-essentials-i18n`: Internationalization support
- `dougu-domain-*`: Domain-specific libraries
  - `dougu-domain-dropbox`: Dropbox API client
- `dougu-command-*`: Individual command implementations
  - `dougu-command-file`: File operations
  - `dougu-command-dropbox`: Dropbox operations

## Usage Examples

```
# File operations
$ dougu file copy source.txt destination.txt
$ dougu file move source.txt destination.txt
$ dougu file list /path/to/directory

# Dropbox operations
$ dougu dropbox file list
$ dougu dropbox file download /path/to/file.txt
$ dougu dropbox file upload local.txt /remote/path.txt
$ dougu dropbox folder create /new/folder
$ dougu dropbox folder delete /old/folder
```

## Building the Project

```
cargo build
```

## Running Tests

```
cargo test
```

## Technical Details

- Built with Rust 2024 edition
- Uses workspace resolver version 3
- Follows modular crate design for better maintainability

## License

Apache License 2.0

# Resources Directory

A new `resources` directory is used to store resource files such as workflow templates for CI/CD and other reusable configuration. If a resource cannot be found, related processes will abort as per project policy.

- `.github/workflows/` contains active workflow files for GitHub Actions.
- `resources/` contains workflow templates and other resource files. 