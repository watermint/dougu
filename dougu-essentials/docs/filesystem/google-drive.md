# Google Drive File System

This document describes how the Google Drive API is implemented within the `dougu-essentials` file system abstraction.

## Entry Representation

Google Drive represents files and folders differently from traditional file systems:

- Uses ID-based addressing rather than paths
- File resources contain metadata and references
- Special handling for Google Docs format files
- Has a concept of file "ownership" that affects visibility and access

## File Operations

### Reading and Writing

- **Reader**: Uses the `get` API with `alt=media` parameter
- **Writer**: Uses the `create` or `update` APIs with media content
- **Random Access**: Implemented using Range headers
- **Special Formats**: Google Docs formats require export API calls for conversion

### Metadata

- Files have mimeType and md5Checksum for non-Google formats
- Google Docs formats don't have traditional file sizes or checksums
- Rich metadata including permissions, capabilities, and sharing status

## Trash Management

Google Drive manages deleted files via a trash system:

- Files moved to trash remain for 30 days before automatic deletion
- Trashed files can be viewed by setting the `trashed=true` query parameter
- Files shared with others remain visible to them until permanently deleted
- Files owned by others remain in your Drive even if the owner deletes them

### API Reference

- **Delete/Trash**: Sets the `trashed` property to `true`
- **Permanent Delete**: Uses the `delete` endpoint with special permissions
- **Restore**: Sets the `trashed` property to `false`
- **Empty Trash**: Uses the `emptyTrash` endpoint

## Capabilities Implementation

### TrashManagement

- Files are kept in trash for 30 days before automatic deletion
- Files are moved to trash by setting the `trashed` property to `true`
- Trash is accessible through regular file listing with appropriate filters
- Automatic expiration after retention period

### PermanentDeletion

- Requires special permissions for permanent deletion
- Uses `delete` endpoint with specific parameters
- May require first moving to trash for normal users
- Requires additional permissions for shared files

### FileRestoration

- Sets the `trashed` property to false
- May restore to original location or parent folder
- Preserves file metadata and permissions
- Works for items that haven't exceeded retention period

### EmptyTrash

- Provides dedicated `emptyTrash` endpoint
- Permanently removes all trashed items
- Requires owner or organizer permissions
- Cannot be undone

### ListTrash

- Query with `trashed=true` parameter
- Can combine with other search parameters
- Available through standard file listing API
- Can filter by modification date, owner, etc.

### TrashMetadata

- Provides `trashed_time` property on File resources
- Fixed 30-day retention period
- Includes original parent folder information
- Maintains sharing permissions during trash period

## Sharing Capabilities

### Shared Links

- Implemented through permission creation with type=anyone
- Supports various permission levels (reader, commenter, editor)
- Can be restricted by domain
- Options for preventing downloading, printing, copying

### Shared Files and Folders

- Permissions model based on Google Workspace
- Individual and group-based permissions
- Domain restrictions and sharing controls
- Supports transfer of ownership

## Content Handling

### Google Docs Formats

- Native Google formats (Docs, Sheets, Slides, etc.) handled differently
- No direct binary representation - must be exported/imported
- Export supports multiple formats (PDF, DOCX, XLSX, etc.)

### Content Hashing

- MD5 checksums for non-Google formats
- No checksums for Google Docs formats
- Limited integrity validation compared to other providers

## API References

- [Files - Get](https://developers.google.com/drive/api/v3/reference/files/get)
- [Files - Create](https://developers.google.com/drive/api/v3/reference/files/create)
- [Files - Update](https://developers.google.com/drive/api/v3/reference/files/update)
- [Files - Delete](https://developers.google.com/drive/api/v3/reference/files/delete)
- [Files - EmptyTrash](https://developers.google.com/drive/api/v3/reference/files/emptyTrash)
- [Files - Export](https://developers.google.com/drive/api/v3/reference/files/export)
- [Permissions - Create](https://developers.google.com/drive/api/v3/reference/permissions/create)
- [Manage Downloads](https://developers.google.com/drive/api/v3/manage-downloads)
- [Manage Uploads](https://developers.google.com/drive/api/v3/manage-uploads)
- [Search Files](https://developers.google.com/drive/api/guides/search-files) 