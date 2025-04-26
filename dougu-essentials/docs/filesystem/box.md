# Box File System

This document describes how the Box API is implemented within the `dougu-essentials` file system abstraction.

## Entry Representation

Box represents files and folders using their API resources:

- Entries are represented as File or Folder objects
- Uses ID-based addressing rather than paths
- Extensive metadata support including custom attributes
- Strong enterprise features for compliance and governance

## File Operations

### Reading and Writing

- **Reader**: Uses the `content` API endpoint
- **Writer**: Uses the `content` endpoint for small files
- **Chunked Upload**: Used for larger files
- **Random Access**: Supports byte ranges for partial content

### Metadata

- Files have SHA1 hash for verification
- Rich metadata including custom metadata templates
- Extensive file properties for enterprise governance

## Trash Management

Box provides a comprehensive trash system:

- Files and folders are kept in trash for 90 days for both personal and business accounts
- Box Enterprise accounts can set custom retention periods
- Permanently deleted items cannot be recovered
- API access for listing, restoring, and permanently deleting trashed items

### API Reference

- **Delete**: DELETE request to file/folder resource
- **List Trash**: Query with `trash=true` parameter
- **Restore**: POST to the item's `restore` action
- **Permanent Delete**: DELETE request to items in trash

## Capabilities Implementation

### TrashManagement

- Files kept for 90 days (configurable for Enterprise)
- DELETE request to file/folder resource moves it to trash
- Enterprise accounts can set custom retention policies
- Supports admin recovery for enterprise accounts

### PermanentDeletion

- DELETE request to items in trash permanently removes them
- Cannot be undone
- Requires appropriate permissions
- Can be applied to individual items or entire folders

### FileRestoration

- POST to the item's `restore` action
- Can specify a new name or parent folder
- Handles name conflicts
- Preserves metadata and file versions

### EmptyTrash

- No direct API for emptying entire trash
- Must enumerate and delete items individually
- Enterprise retention policies may override manual deletion
- Box Governance provides additional controls

### ListTrash

- Query with `trash=true` parameter
- Dedicated endpoint for retrieving trash items
- Supports filtering and sorting of trash items
- Pagination support for large trash folders

### TrashMetadata

- Provides `trashed_at` timestamp on trashed items
- Retention period based on account configuration
- Records the user who performed the deletion
- Enterprise audit trail for deletion events

## Sharing Capabilities

### Shared Links

- Uses the `shared_links` API
- Supports password protection, expiration, and permission settings
- Can restrict access by domain or authorized users
- Supports download prevention and watermarking

### Shared Folders

- Enterprise-grade collaboration features
- Role-based access controls
- Custom workflow automation
- Compliance and governance controls

## Advanced Features

### Enterprise Security

- Box KeySafe for customer-managed encryption
- Box Shield for threat detection
- Box Governance for retention policies
- Box Zones for data residency

### Content Management

- Box Skills for AI-powered content insights
- Metadata-driven workflows
- Full-text search capabilities
- Version history and retention

### Content Hashing

- Uses SHA1 for file verification
- Strong integrity validation
- Support for file locking during edits

## API References

- [File Resource](https://developer.box.com/reference/resources/file/)
- [Folder Resource](https://developer.box.com/reference/resources/folder/)
- [Download File Content](https://developer.box.com/reference/get-files-id-content)
- [Upload File Content](https://developer.box.com/reference/post-files-content)
- [Chunked Upload](https://developer.box.com/guides/uploads/chunked/)
- [Delete File](https://developer.box.com/reference/delete-files-id)
- [Delete Folder](https://developer.box.com/reference/delete-folders-id)
- [Get Trashed Items](https://developer.box.com/reference/get-folders-trash-items)
- [Restore Item](https://developer.box.com/reference/post-files-id)
- [Shared Links](https://developer.box.com/reference/put-files-id-add-shared-link)
- [Box Trash Policy](https://support.box.com/hc/en-us/articles/360044194713-Trash-for-Files-and-Folders) 