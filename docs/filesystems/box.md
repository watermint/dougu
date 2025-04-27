# Box File System

This document describes the behavior of Box file system integration as implemented in the dougu library.

## File Deletion Behavior

Box implements a file deletion system with the following characteristics:

- Deleted files are moved to the "Trash"
- Standard retention period: 30 days for most accounts
- Box keeps deleted files and folders in the trash can for 90 days for both personal and business accounts
- Enterprise accounts can have custom retention policies set by administrators
- Files can be restored from Trash during the retention period
- Files are automatically permanently deleted after the retention period expires
- Files can be manually permanently deleted from Trash before the retention period expires
- Enterprise administrators may have extended recovery capabilities for permanently deleted content
- Box's enterprise plans offer compliance features with legal holds and retention policies
- Collaborative content retains version history that may be recoverable
- Permanently deleted items cannot be recovered

### Storage Impact and Limits

- Files in Trash count against the user's storage quota
- Unlike some other services, Box does not have a specific size limit for Trash
- Emptying Trash immediately frees up storage space
- Enterprise accounts may have content retained in admin-accessible locations even after Trash is emptied
- Box counts each version of a file against storage quota (including deleted versions)

## API Implementation Details

### Data Model
- Entries are represented as File or Folder objects
- Uses ID-based addressing
- Extensive metadata support

### Status Handling
- Status is determined by the item's location
- Trashed items have a `trashed_at` timestamp
- Box maintains deletion timestamp for files in trash

### Operations

In the dougu library implementation:

- `delete()` moves a file to Box Trash
- `permanently_delete()` uses different approaches:
  - For items already in trash, uses the `delete` endpoint
  - Otherwise, uses the standard delete operation which moves to trash
- `restore()` uses the `restore` action on the item and can specify a new name or parent folder
- `list_deleted()` uses a query with `trash=true` parameter
- Empty trash is not directly supported in API; must enumerate and delete items
- The implementation uses Box Content API v2.0
- Files in Trash maintain metadata about their original location and deletion time
- The API provides specific endpoints for Trash management
- Permissions on shared files are respected when deletion operations occur
- The implementation handles version control aspects of Box's file system
- For permanent deletion, the `permanently=true` parameter can be used

For more details on the Box API, refer to the [official Box Content API documentation](https://developer.box.com/reference/)

## Reference
- [Box Support: Trash for Files and Folders](https://support.box.com/hc/en-us/articles/360044194713-Trash-for-Files-and-Folders)
- [Box API - Files](https://developer.box.com/reference/resources/file/)
- [Box API - Get Trashed Items](https://developer.box.com/reference/get-folders-trash-items)
- [Box API - Delete](https://developer.box.com/reference/delete-files-id)
- [Box API - Restore](https://developer.box.com/reference/post-files-id)
- [Box Community - Trash retention](https://support.box.com/hc/en-us/community/posts/360051103494-Trash-retention) 