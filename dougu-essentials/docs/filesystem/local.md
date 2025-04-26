# Local File System

This document describes how the local file system is implemented within the `dougu-essentials` file system abstraction.

## Entry Representation

Local file system entries are represented using native OS paths:

- Entries are addressed by platform-specific file paths
- Native OS-level file and directory manipulation
- Platform differences (Unix vs Windows) are abstracted away

## File Operations

### Reading and Writing

- **Reader**: Uses standard Rust file I/O operations
- **Writer**: Uses standard Rust file I/O operations
- **Random Access**: Native random access through `Seek` trait

### Metadata

- Access to standard file metadata (creation time, modified time, etc.)
- Access to file permissions and attributes
- Extended attributes where supported by the OS

## Trash Management

Local file systems have limited built-in trash capabilities:

- On Windows, files can be moved to Recycle Bin
- On macOS, files can be moved to Trash
- On Linux, some desktop environments support trash
- Programmatic trash operations are not standardized across platforms

### Implementation Notes

- Delete operations typically permanently remove files
- Trash operations may be implemented by moving to special locations
- No standard API for listing or restoring deleted files

## Capabilities Implementation

### TrashManagement

- Not standardized across platforms
- Typically implemented by desktop environments, not file system APIs
- Windows: Recycle Bin implementation
- macOS: Trash implementation
- Linux: Varies by desktop environment (e.g., ~/.local/share/Trash)

### PermanentDeletion

- Standard delete operations typically permanently remove files
- No dedicated API for permanent deletion vs. trash
- Some platforms offer secure deletion options (e.g., Windows `cipher` command)
- Command-line tools typically bypass trash

### FileRestoration

- Not standardized across platforms
- May be implemented by desktop environments but not as API
- Often requires manual intervention or GUI operations
- Some third-party libraries implement trash restoration

### EmptyTrash

- Platform-specific implementations
- Often available through desktop environment but not standardized API
- Some command-line utilities available (e.g., `rm -rf ~/.local/share/Trash/*` on Linux)
- Windows offers COM interfaces that can programmatically empty the recycle bin

### ListTrash

- Not standardized across platforms
- Usually not accessible through standard file APIs
- Requires platform-specific code to access trash contents
- Desktop environments provide GUI interfaces

### TrashMetadata

- Not standardized across platforms
- Some platforms store limited metadata about deleted files
- Windows: `.trashinfo` files store original path and deletion date
- macOS: similar metadata storage
- Linux: `.trashinfo` files in the Trash folder

## Sharing Capabilities

### File Sharing

- Limited to OS-level file permissions
- Network sharing depends on OS capabilities
- No built-in link sharing comparable to cloud providers

## Special Features

### Symlinks and Hardlinks

- Support for symbolic links and hard links
- Support for special file types (devices, pipes, sockets)
- Platform-specific behavior is normalized where possible

### File Locking

- Support for advisory file locking
- Support for mandatory file locking where available
- Lock implementation depends on OS capabilities

## Implementation Details

- Uses standard Rust `std::fs` for file operations
- Wraps platform-specific behavior in consistent API
- Handles path normalization and validation
- Error handling follows Rust conventions

## Platform-Specific Notes

### Windows

- Path separators normalized (both '/' and '\' supported)
- Support for extended-length paths (>260 characters)
- Access to Windows-specific attributes

### Unix/Linux

- Support for Unix file permissions
- Handling of executable bit
- Support for special file types

### macOS

- Support for macOS resource forks and extended attributes
- Integration with macOS file tags
- Handling of bundle file types 