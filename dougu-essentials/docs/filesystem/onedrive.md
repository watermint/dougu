# OneDrive File System

This document describes how the OneDrive API is implemented within the `dougu-essentials` file system abstraction.

## Entry Representation

OneDrive represents files and folders through the Microsoft Graph API:

- Entries are represented as DriveItem resources
- Supports both path and ID-based addressing
- Different behavior for personal and business accounts
- Integrates with SharePoint for business accounts

## File Operations

### Reading and Writing

- **Reader**: Uses the content URL from the DriveItem
- **Writer**: Uses the `content` endpoint for small files
- **Upload Sessions**: Used for larger files (>4MB)
- **Random Access**: Supports byte ranges for partial content

### Metadata

- Files have eTag and cTag for change tracking
- Rich metadata including DriveItem properties
- SharePoint-specific metadata for business accounts

## Trash Management

OneDrive implements a recycle bin system with different retention policies:

- **Personal OneDrive**: 
  - Files are kept in recycle bin for 30 days
  - Self-service recovery during this period

- **Business/SharePoint**:
  - Files remain in recycle bin for 93 days
  - Two-stage recycle bin (site recycle bin + admin recycle bin)
  - The recycle bin can reach capacity (10% of total storage)
  - Once capacity is reached, oldest items are purged first

### API Reference

- **Delete**: DELETE request to file/folder resource
- **List Deleted**: Access the special `recyclebin` folder
- **Restore**: POST to the `restore` action on the item
- **Permanent Delete**: DELETE request to already-trashed items

## Capabilities Implementation

### TrashManagement

- Personal accounts: Files kept for 30 days
- Business accounts: Files kept for 93 days
- DELETE request to file/folder resource puts it in recycle bin
- Capacity-based purging (10% of total storage)
- Two-stage deletion process for SharePoint

### PermanentDeletion

- DELETE request to already-trashed items permanently removes them
- Two-stage deletion process for Business accounts
- Second-stage deletion requires admin privileges in SharePoint
- Cannot be undone

### FileRestoration

- POST to the `restore` action on the item
- Can restore to original or new location
- Two-stage recovery for SharePoint items
- Handles name conflicts with automatic renaming

### EmptyTrash

- No direct API for emptying entire recycle bin
- Personal accounts can empty through web interface
- Business accounts have admin controls
- Must enumerate and delete items individually via API

### ListTrash

- Access the special `recyclebin` folder
- Special endpoint for listing deleted items
- Can include deleted items in regular queries with filters
- Provides pagination for large trash folders

### TrashMetadata

- Provides `deletedDateTime` property on deleted items
- Retention period varies by account type
- Includes original path information
- Records the user who performed the deletion

## Sharing Capabilities

### Shared Links

- Uses the `createLink` action
- Supports various permission types:
  - View
  - Edit
  - Embed
  - Anonymous
- Supports expiration settings and password protection (business accounts)

### Shared Folders

- Integration with Microsoft 365 sharing features
- Permissions based on Azure AD
- Group and team-based sharing
- External sharing controls

## Content Handling

### Special Features

- Files On-Demand feature for desktop sync
- Known Folder Move integration with Windows
- Personal Vault for sensitive files (additional encryption)

### Content Hashing

- Supports QuickXOR hashing algorithm
- eTag and cTag for change detection
- Limited hash verification capabilities

## API References

- [DriveItem Resource](https://learn.microsoft.com/en-us/graph/api/resources/driveitem)
- [Get DriveItem Content](https://learn.microsoft.com/en-us/graph/api/driveitem-get-content)
- [Update DriveItem Content](https://learn.microsoft.com/en-us/graph/api/driveitem-put-content)
- [Upload Large Files](https://learn.microsoft.com/en-us/graph/api/driveitem-createuploadsession)
- [Delete DriveItem](https://learn.microsoft.com/en-us/graph/api/driveitem-delete)
- [List Deleted Items](https://learn.microsoft.com/en-us/graph/api/drive-list-deleted)
- [Restore DriveItem](https://learn.microsoft.com/en-us/graph/api/driveitem-restore)
- [Create Sharing Link](https://learn.microsoft.com/en-us/graph/api/driveitem-createlink)
- [OneDrive Recycle Bin Policy](https://support.microsoft.com/en-us/office/what-happens-when-you-delete-files-in-the-cloud-57dda49f-0e55-41eb-bb6a-c61caa8cca03) 