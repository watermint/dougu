# File Deletion Behaviors Across Different Platforms

This document outlines how different file systems and cloud storage providers handle file deletion, particularly focusing on aspects like deletion timestamps, trash/recycle bin functionality, and permanent deletion.

## Cloud Storage Providers

### Deletion Timestamps

Different cloud providers track when files were deleted:

- **Dropbox**: Provides deletion timestamp for deleted files
- **Google Drive**: Tracks when files were moved to trash
- **OneDrive**: Records deletion date for files in the recycle bin
- **Box**: Maintains deletion timestamp for files in trash

### Trash/Recycle Bin Functionality

Most cloud providers implement a trash or recycle bin feature:

- **Dropbox**: Files are moved to the "Deleted Files" folder and kept for 30 days (or longer for Business accounts)
- **Google Drive**: Files are moved to the "Trash" and kept for 30 days before automatic permanent deletion
- **OneDrive**: Files are moved to the "Recycle Bin" and typically kept for 30 days (93 days for business accounts)
- **Box**: Files are moved to "Trash" and kept for a configurable period (default is 30 days)

### Permanent Deletion

Behavior for permanently deleting files varies by provider:

- **Dropbox**: Files can be permanently deleted from the "Deleted Files" folder manually, or they are automatically purged after the retention period
- **Google Drive**: Files can be permanently deleted from "Trash" manually or are automatically purged after 30 days
- **OneDrive**: Files can be permanently deleted from the "Recycle Bin" or are automatically purged after the retention period
- **Box**: Files can be permanently deleted from "Trash" or are automatically purged after the configured retention period

## Local File Systems

Local file systems typically don't provide built-in trash functionality at the API level:

- **Windows**: The OS provides a Recycle Bin, but this is not directly exposed through the standard file system APIs
- **macOS**: The OS provides a Trash folder, but this is not directly exposed through the standard file system APIs
- **Linux**: Various desktop environments may provide trash functionality, but this is not part of the standard file system APIs

When using standard file system APIs:
- Delete operations are generally permanent
- No deletion timestamps are maintained by the file system
- Applications must implement their own trash/recovery mechanisms if needed 