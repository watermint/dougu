# Dropbox File System

This document describes the behavior of Dropbox file system integration as implemented in the dougu library.

## File Deletion Behavior

Dropbox implements a file deletion system with the following characteristics:

- Deleted files are moved to the "Deleted files" folder (previously called "Trash")
- Standard retention period: 30 days for Dropbox Basic, Plus, and Family accounts
- Business accounts have a 180-day retention period by default (configurable by administrators)
- Professional/Standard/Essentials/Team accounts: 180 days retention
- Advanced/Enterprise/Education/Team Plus accounts: 365 days retention
- Files are automatically permanently deleted after the retention period expires
- Files can be manually permanently deleted from Deleted files before the retention period expires
- Files can be restored from Deleted files during the retention period
- For Business accounts, Team Admins have extended recovery options
- File version history may be preserved depending on account type
- Dropbox maintains file revisions even after deletion
- Extended version history can extend the retention periods
- Shared file deletion behavior depends on sharing permissions:
  - When the owner deletes a shared file, it's moved to the owner's Deleted files folder and removed from all members' Dropbox accounts
  - When a non-owner deletes a shared file, it's only removed from their Dropbox account

### Storage Impact and Limits

- Files in Deleted files do not count against the user's storage quota
- There is no specific size limit for the Deleted files folder
- Emptying Deleted files permanently removes those files
- Dropbox Business accounts have additional controls for retention and recovery
- Dropbox maintains version history that can be accessed for recovery
- For Dropbox Plus and Professional users, Extended Version History provides up to 180 days of version recovery
- Dropbox Paper documents have their own separate deletion system and recovery options

## API Implementation Details

### Data Model
- Entries are represented as metadata objects
- File paths use UNIX-style format with '/' separator
- Supports file revisions and version history

### Status Handling
- Normal files have no special status indicator
- Deleted files can be queried with the "include_deleted" parameter in list calls
- Provides deletion timestamp via the `delete_v2` API response

### Operations

In the dougu library implementation:

- `delete()` moves a file to Dropbox Deleted files
- `permanently_delete()` uses the `permanently_delete` API to bypass Deleted files
- `restore()` uses the `restore` API to recover files from Deleted files, and can specify a new location
- `list_deleted()` requires special parameter `include_deleted=true` in list requests
- No direct API for emptying trash; must enumerate and permanently delete items
- No direct API for permanently deleting folders; must enumerate and permanently delete items
- The implementation uses Dropbox API v2
- Path-based operations are used to interact with Deleted files
- Metadata about deleted files maintains information about original paths
- Batch operations support efficient management of multiple files
- The API implementation handles namespace_id for team and shared folder operations
- The Dropbox API provides a mechanism to permanently delete files by ID

For more details on the Dropbox API, refer to the [official Dropbox API documentation](https://www.dropbox.com/developers/documentation/http/documentation)

## Reference
- [Dropbox Data Retention Policy](https://help.dropbox.com/account-settings/data-retention-policy)
- [Dropbox API - Metadata](https://www.dropbox.com/developers/documentation/http/documentation#files-get_metadata)
- [Dropbox API - list_folder](https://www.dropbox.com/developers/documentation/http/documentation#files-list_folder)
- [Dropbox API - delete_v2](https://www.dropbox.com/developers/documentation/http/documentation#files-delete_v2)
- [Dropbox API - permanently_delete](https://www.dropbox.com/developers/documentation/http/documentation#files-permanently_delete)
- [Dropbox API - restore](https://www.dropbox.com/developers/documentation/http/documentation#files-restore) 