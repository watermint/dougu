# Dropbox File System

This document describes how the Dropbox API is implemented within the `dougu-essentials` file system abstraction.

## Entry Representation

Dropbox represents files and folders as metadata objects:

- File paths use UNIX-style format with '/' separator
- Files have content hash and revision information
- Supports block-based uploading for large files

## File Operations

### Reading and Writing

- **Reader**: Uses the `download` API endpoint with support for range requests
- **Writer**: Uses the `upload` API for small files and the `upload_session` APIs for larger files
- **Random Access**: Implemented using the `download` API with Range header

### Metadata

- Content hash uses a custom SHA-256 block-based algorithm
- Supports file revisions with full version history
- Provides rich metadata including sharing information

## Trash Management

Dropbox provides a trash system with these characteristics:

- Files are kept in trash for 30-365 days depending on account type:
  - Basic/Plus/Family: 30 days
  - Professional/Standard/Essentials/Team: 180 days
  - Advanced/Enterprise/Education/Team Plus: 365 days
- Extended version history can extend retention periods
- API access available for listing, restoring, and permanently deleting trashed items

### API Reference

- **Delete**: `delete_v2` endpoint moves files to trash
- **Permanent Delete**: `permanently_delete` endpoint bypasses trash
- **Restore**: `restore` endpoint recovers items from trash, with optional new location

## Capabilities Implementation

### TrashManagement

- Files are kept in trash for 30-365 days depending on account type
- Files can be moved to trash with the `delete_v2` endpoint
- No direct endpoint to empty entire trash

### PermanentDeletion

- Provides dedicated `permanently_delete` API for immediate deletion
- Can be applied to files not already in trash to bypass trash stage
- Can delete files regardless of sharing status or permissions

### FileRestoration

- Uses `restore` API for recovery
- Can optionally restore to new location
- Supports path conflict resolution strategies
- Preserves sharing permissions when restored

### EmptyTrash

- No direct API for emptying entire trash
- Must enumerate and delete items individually

### ListTrash

- Requires `include_deleted=true` parameter in list operations
- Can filter deleted files with additional parameters

### TrashMetadata

- Provides deleted timestamp via metadata
- Can calculate expiry based on account type retention policy
- Includes original path information

## Sharing Capabilities

### Shared Links

- Created via the `create_shared_link_with_settings` API
- Supports password protection, expiration, and access level settings
- Can control whether downloads are permitted or view-only

### Shared Folders

- Powerful collaboration features with granular permissions
- Supports folder mount/unmount functionality
- Provides member management with varying access levels

## Content Hashing

Dropbox uses a custom content hashing algorithm:

- Based on SHA-256
- Uses 4MB blocks for file chunking
- Provides strong consistency guarantees for file integrity

## API References

- [Files - Download](https://www.dropbox.com/developers/documentation/http/documentation#files-download)
- [Files - Upload](https://www.dropbox.com/developers/documentation/http/documentation#files-upload)
- [Files - Delete](https://www.dropbox.com/developers/documentation/http/documentation#files-delete_v2)
- [Files - Permanently Delete](https://www.dropbox.com/developers/documentation/http/documentation#files-permanently_delete)
- [Files - Restore](https://www.dropbox.com/developers/documentation/http/documentation#files-restore)
- [Sharing - Create Shared Link](https://www.dropbox.com/developers/documentation/http/documentation#sharing-create_shared_link_with_settings)
- [Account - Data Retention Policy](https://help.dropbox.com/account-settings/data-retention-policy) 