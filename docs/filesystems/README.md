# File System Documentation

This directory contains documentation about the behavior and characteristics of various file systems supported by the dougu library.

## Supported File Systems

- [Local File System](./local.md) - Basic operating system file system
- [Dropbox](./dropbox.md) - Dropbox cloud storage
- [Google Drive](./google_drive.md) - Google Drive cloud storage
- [OneDrive](./onedrive.md) - Microsoft OneDrive cloud storage
- [Box](./box.md) - Box cloud storage

## Features Documented

For each file system, we document:

- File deletion behavior
- Trash/recycle bin implementation
- Retention periods
- Recovery capabilities

These details are important for implementing proper file system operations that respect the characteristics of each provider. 