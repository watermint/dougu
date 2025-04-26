# FTP and SFTP Protocol Support

File Transfer Protocol (FTP) and SSH File Transfer Protocol (SFTP) are widely used for transferring files between computers over a network. The `dougu-essentials` file system abstraction provides comprehensive support for FTP/SFTP operations through a unified interface.

## Overview

FTP is one of the oldest protocols for file transfer, while SFTP provides a secure alternative that operates over SSH. Both protocols are extensively used for:

- Uploading and downloading files
- Remote file management
- Website content deployment
- Data exchange between systems

## FTP/SFTP Capabilities

The file system abstraction supports the following FTP/SFTP-specific capabilities:

### FTP Core Features

- `BasicFTP`: Support for standard FTP commands (RFC 959)
  - USER, PASS for authentication
  - CWD, PWD for navigation
  - LIST, NLST for directory listing
  - RETR, STOR for file transfer
  - DELE, RMD, MKD for file/directory management

- `FTPS`: FTP over SSL/TLS for secure transfers
  - Explicit FTPS (starts unencrypted, upgrades with AUTH)
  - Implicit FTPS (encrypted from connection start)
  - Control and data channel protection options

- `ExtendedFTP`: Support for FTP extensions (RFC 3659)
  - MLSD, MLST for standardized listings
  - SIZE for file size information
  - MDTM for modification time
  - Additional status codes and features

### SFTP Features

- `SFTP`: Basic SSH File Transfer Protocol support
  - Secure authentication and data transfer
  - File operations over SSH
  - Integrated with SSH infrastructure

- `SFTPv3`: Support for SFTP protocol version 3
  - Basic file operations
  - Standard attributes
  - Initial version with wide support

- `SFTPv4Plus`: Support for SFTP version 4 or higher
  - Extended attributes
  - Symbolic link operations
  - Access control lists
  - Advanced file properties

- `SFTPLocking`: Support for optional SFTP file locking extensions
  - Advisory locking
  - Mandatory locking where supported
  - Lock violation detection

### Authentication and Transfer Features

- `PublicKeyAuth`: Public key authentication support
  - Key-based authentication for SFTP
  - Certificate validation
  - Agent forwarding capabilities

- `ServerToServerTransfer`: Support for FXP (File eXchange Protocol)
  - Direct server-to-server transfers
  - Bandwidth optimization
  - Third-party transfer orchestration

- `ResumeTransfer`: Support for interrupted transfer resumption
  - REST command in FTP
  - Partial transfer handling in SFTP
  - Checksumming for integrity verification

## Integration with File System Abstraction

FTP/SFTP capabilities integrate with the core file system abstraction through the `FTPSFTP` capability flag and the `FTPSFTPCapability` enum. To check for FTP/SFTP support:

```rust
if fs.capabilities().supports_ftpsftp() {
    // FTP/SFTP protocols are supported
    
    // Check for specific FTP/SFTP capabilities
    if fs.capabilities().has_ftpsftp_capability(FTPSFTPCapability::FTPS) {
        // Secure FTP is supported
    }
    
    if fs.capabilities().has_ftpsftp_capability(FTPSFTPCapability::SFTPv4Plus) {
        // Advanced SFTP features are available
    }
    
    if fs.capabilities().has_ftpsftp_capability(FTPSFTPCapability::ResumeTransfer) {
        // Interrupted transfers can be resumed
    }
}
```

## Practical Considerations

When working with FTP/SFTP implementations, consider the following:

1. **Connection Modes**: FTP supports both active and passive connection modes, which behave differently with firewalls and NAT. Prefer passive mode in most scenarios.

2. **Security Implications**: Regular FTP transmits credentials and data in plaintext. Use FTPS or SFTP for sensitive data.

3. **Transfer Types**: FTP supports both ASCII and binary transfer modes. Always use binary mode for non-text files to prevent corruption.

4. **Firewall Considerations**: FTP uses separate control and data connections, which can cause issues with firewalls. SFTP uses a single connection which is often easier to route.

5. **Performance Optimization**: Consider factors like buffer sizes, window sizes, and parallel transfers for optimal performance, especially over high-latency connections.

## Server Software Support

FTP/SFTP protocols are supported by various server implementations:

### FTP Servers
- vsftpd
- FileZilla Server
- ProFTPD
- Microsoft IIS FTP
- Pure-FTPd

### SFTP Servers
- OpenSSH SFTP subsystem
- ProFTPD with mod_sftp
- Bitvise SSH Server
- AWS Transfer for SFTP
- SFTPPlus

Each implementation may support different feature sets and extensions, particularly for the more advanced capabilities. 