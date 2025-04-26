# Google Drive File System

This document describes the behavior of Google Drive file system integration as implemented in the dougu library.

## File Deletion Behavior

Google Drive implements a file deletion system with the following characteristics:

- Deleted files are moved to the "Trash" (formerly called "Bin")
- Standard retention period: 30 days for most accounts
- Google Workspace accounts may have custom retention policies set by administrators
- Files can be restored from Trash during the retention period
- Files are automatically permanently deleted after the retention period expires
- Files can be manually permanently deleted from Trash before the retention period
- Permanently deleted files cannot be recovered by standard users
- Google Workspace administrators may have the ability to recover permanently deleted files within a limited time window
- Google Drive has a storage limit for Trash; when full, the oldest items may be automatically deleted
- Files owned by the user that are deleted from shared folders will be moved to the user's Trash
- Files not owned by the user but removed from shared folders will be removed from the user's view but not moved to their Trash

### API Implementation Notes

In the dougu library implementation:

- `delete()` moves a file to the Google Drive Trash
- `permanently_delete()` bypasses the Trash and removes the file entirely
- `restore()` recovers a file from the Trash to its original location
- `list_deleted()` shows files in the Google Drive Trash
- When requesting file status, files in the Trash will report their deletion time and trash status
- The implementation respects the `trashed` property in Google Drive API

For more details on the Google Drive API, refer to the [official Google Drive API documentation](https://developers.google.com/drive/api/v3/reference). 