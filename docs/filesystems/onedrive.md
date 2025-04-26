# OneDrive File System

This document describes the behavior of OneDrive file system integration as implemented in the dougu library.

## File Deletion Behavior

OneDrive implements a file deletion system with the following characteristics:

- Deleted files are moved to "Recycle Bin"
- Standard retention period: 30 days for personal accounts, customizable for business accounts
- Business/SharePoint: Files remain in recycle bin for 93 days by default
- Files are automatically permanently deleted after the retention period expires
- Files can be manually permanently deleted from Recycle Bin before the retention period expires
- Files can be restored from Recycle Bin during the retention period
- The recycle bin can reach capacity (10% of total storage), after which oldest items are purged
- For Microsoft 365 business accounts, administrators may have additional recovery options
- Deleted shared files are moved to the owner's Recycle Bin, not the Recycle Bin of users with whom the file was shared
- The OneDrive for Business second-stage Recycle Bin provides additional recovery options for administrators

### Storage Impact and Limits

- Files in Recycle Bin count against the user's storage quota
- There is no specific size limit for the Recycle Bin folder itself
- Emptying Recycle Bin immediately frees up storage space
- Microsoft 365 accounts have configurable storage retention policies
- Files in the Recycle Bin are still encrypted and secured with the same level of protection as active files
- Version history for files may be recoverable depending on the account type and settings

## API Implementation Details

### Data Model
- Entries are represented as DriveItem resources
- Supports both path and ID-based addressing
- Different behavior for personal and business accounts

### Status Handling
- Status is determined by location (normal folders vs. recyclebin)
- Can query specifically for deleted items
- Provides `deletedDateTime` property on deleted items

### Operations

In the dougu library implementation:

- `delete()` moves a file to OneDrive Recycle Bin
- `permanently_delete()` uses different approaches:
  - For items already in recycle bin, uses the `delete` endpoint
  - Otherwise, uses the standard delete operation which moves to recycle bin
- `restore()` uses the `restore` action on the item and can restore to original or new location
- `list_deleted()` accesses special folder path `/me/drive/root/recycle`
- Empty trash operation uses DELETE request to the `recycle` endpoint
- The implementation uses Microsoft Graph API
- Recycle Bin operations respect the permission model of OneDrive
- The API can track the original path of items in the Recycle Bin
- Permission inheritance for shared files and folders is maintained through deletion and restoration
- Conflict resolution is handled during restoration if the original location is no longer available

For more details on the OneDrive API, refer to the [official Microsoft Graph API documentation](https://docs.microsoft.com/en-us/graph/api/resources/driveitem)

## Reference
- [Microsoft Support: What happens when you delete files in the cloud](https://support.microsoft.com/en-us/office/what-happens-when-you-delete-files-in-the-cloud-57dda49f-0e55-41eb-bb6a-c61caa8cca03)
- [Microsoft Graph API - DriveItem](https://learn.microsoft.com/en-us/graph/api/resources/driveitem)
- [Microsoft Graph API - List Deleted](https://learn.microsoft.com/en-us/graph/api/drive-list-deleted)
- [Microsoft Graph API - Delete](https://learn.microsoft.com/en-us/graph/api/driveitem-delete)
- [Microsoft Graph API - Restore](https://learn.microsoft.com/en-us/graph/api/driveitem-restore)
- [Microsoft Support - Restore deleted files](https://support.microsoft.com/en-us/office/restore-deleted-items-from-the-site-collection-recycle-bin-5fa924ee-16d7-487b-9a0a-021b9062d14b) 