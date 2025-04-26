# Local File System

This document describes the behavior of local file systems as implemented in the dougu library.

## File Deletion Behavior

Local file systems typically handle file deletion as follows:

- The operating system's file system API typically does not have a built-in trash/recycle bin concept at the API level.
- Files deleted through standard file system operations (`unlink`, `rmdir`, etc.) are immediately removed from the file system.
- Operating systems may implement their own trash/recycle bin functionality at a higher level:
  - Windows: Files deleted through the Explorer UI go to the Recycle Bin
  - macOS: Files deleted through Finder go to the Trash
  - Linux: Desktop environments like GNOME and KDE implement their own trash folders
- Files deleted via command line tools (`rm`, `del`) or direct API calls typically bypass the OS trash system and are removed permanently.
- Permanently deleted files may be recoverable using specialized data recovery software, but this is not guaranteed and depends on factors like:
  - How much time has passed since deletion
  - Whether the storage blocks have been reused
  - The type of storage media (HDDs vs. SSDs have different characteristics)

In the dougu library, the local file system implementation follows the standard operating system behavior:

- `delete()` operations directly call the operating system's file removal functions and do not use the trash/recycle bin.
- There is no built-in capability to list deleted files or recover them once deleted.
- The `permanently_delete()` operation is equivalent to the standard `delete()` operation.
- The `restore()` operation is not supported for local file systems as deleted files are not tracked.

When implementing file management with trash/recycle bin capabilities, the library needs to provide this functionality above the file system API level. 