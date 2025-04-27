# Google Drive File System

This document describes the behavior of Google Drive file system integration as implemented in the dougu library.

## File Deletion Behavior

Google Drive implements a file deletion system with the following characteristics:

- Deleted files are moved to "Trash" (formerly called "Bin")
- Standard retention period: 30 days
- Files are automatically permanently deleted after the retention period expires
- Files can be manually permanently deleted from Trash before the retention period expires
- Files can be restored from Trash during the retention period
- For Google Workspace accounts, administrators may have additional recovery options
- Files shared with others remain visible to them until permanently deleted
- Files owned by others remain in your Drive even if the owner deletes them
- Permanently deleted files (from trash) cannot be recovered
- When shared files are deleted, they are removed from all users who had access
- Google Drive preserves version history for Google Workspace files (Docs, Sheets, etc.)

### Storage Impact and Limits

- Files in Trash count against the user's storage quota
- There is no specific size limit for the Trash folder itself
- Emptying Trash immediately frees up storage space
- Google Drive has a specific cleanup policy for freeing space when approaching quota limits
- Google Workspace administrators can set custom retention policies that may override standard behavior
- Each version of a Google Workspace file (Docs, Sheets, etc.) also consumes storage quota

## API Implementation Details

### Data Model
- Entries are represented as File resources
- Uses ID-based addressing rather than paths
- Supports file revisions with versioning
- Special handling for Google Docs format files

### Status Handling
- Status is determined by the `trashed` property
- Can filter for trashed files in queries
- Provides `trashed_time` property on File resources

### Operations

In the dougu library implementation:

- `delete()` moves a file to Google Drive Trash
- `permanently_delete()` first moves to trash if not already trashed (sets `trashed` property to true), then uses the `delete` endpoint with proper permissions
- `restore()` sets the `trashed` property to false
- `list_deleted()` uses a query with `trashed=true` parameter
- Empty trash operation uses DELETE request to the `recycle` endpoint
- The implementation uses Google Drive API v3
- Trash operations respect the permission model of Google Drive
- The API handles file revisions separately from the main file deletion process
- Files owned by the user and shared files they have edit access to can be trashed
- For shared drives, permissions are evaluated at the time of the operation
- Permanent deletion requires special permissions and uses the `delete` endpoint
- When performing delete operations, the `supportsAllDrives=true` parameter can be used

For more details on the Google Drive API, refer to the [official Google Drive API documentation](https://developers.google.com/drive/api/v3/reference)

## Reference
- [Google Drive Help: Delete or restore files](https://support.google.com/drive/answer/2375102)
- [Google Drive API - Files](https://developers.google.com/drive/api/v3/reference/files)
- [Google Drive API - Delete](https://developers.google.com/drive/api/v3/reference/files/delete)
- [Google Drive API - Update](https://developers.google.com/drive/api/v3/reference/files/update)
- [Google Drive API - Search](https://developers.google.com/drive/api/guides/search-files)
- [Google Support - Restore from trash](https://support.google.com/drive/answer/11334581) 