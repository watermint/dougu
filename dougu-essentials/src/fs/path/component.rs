// Path component implementation

/// Path component representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    /// Name of the component
    pub name: String,
    /// Whether this is a root component
    pub is_root: bool,
    /// Whether this is a parent folder reference (..)
    pub is_parent: bool,
    /// Whether this is a current folder reference (.)
    pub is_current: bool,
}

impl Component {
    /// Create a new normal component
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_root: false,
            is_parent: name == "..",
            is_current: name == ".",
        }
    }

    /// Create a root component
    pub fn root() -> Self {
        Self {
            name: "".to_string(),
            is_root: true,
            is_parent: false,
            is_current: false,
        }
    }

    /// Create a parent folder component
    pub fn parent() -> Self {
        Self {
            name: "..".to_string(),
            is_root: false,
            is_parent: true,
            is_current: false,
        }
    }

    /// Create a current folder component
    pub fn current() -> Self {
        Self {
            name: ".".to_string(),
            is_root: false,
            is_parent: false,
            is_current: true,
        }
    }
}

/// Trait for path-specific validation knowledge
pub trait PathValidationTrait: Send + Sync {
    /// Get the maximum allowed path length in bytes
    fn max_path_length(&self) -> usize;
    
    /// Get the maximum allowed filename length in bytes
    fn max_filename_length(&self) -> usize;
    
    /// Check if a filename is valid according to this path type's rules
    fn is_valid_filename(&self, name: &str) -> bool;
    
    /// Check if a complete path is valid according to this path type's rules
    fn is_valid_path(&self, path: &str) -> bool;
    
    /// Get a list of characters not allowed in filenames
    fn invalid_filename_chars(&self) -> &[char];
    
    /// Get a list of characters not allowed in paths
    fn invalid_path_chars(&self) -> &[char];
    
    /// Check if a path exceeds the maximum length
    fn exceeds_max_length(&self, path: &str) -> bool {
        path.len() > self.max_path_length()
    }
    
    /// Sanitize a filename by replacing invalid characters
    fn sanitize_filename(&self, name: &str) -> String {
        let invalid_chars = self.invalid_filename_chars();
        name.chars()
            .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
            .collect()
    }
    
    /// Sanitize a path by replacing invalid characters
    fn sanitize_path(&self, path: &str) -> String {
        let invalid_chars = self.invalid_path_chars();
        path.chars()
            .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
            .collect()
    }
    
    /// Get a list of reserved filenames that cannot be used
    fn reserved_filenames(&self) -> &[&str];
    
    /// Check if a filename is reserved
    fn is_reserved_filename(&self, name: &str) -> bool {
        self.reserved_filenames().iter().any(|&reserved| {
            reserved.eq_ignore_ascii_case(name)
        })
    }
    
    /// Check if a path contains any segment that's a reserved filename
    fn contains_reserved_filename(&self, path: &str) -> bool {
        path.split(self.path_separator())
            .any(|segment| !segment.is_empty() && self.is_reserved_filename(segment))
    }
    
    /// Get the path separator character
    fn path_separator(&self) -> char;
    
    /// Get the list of alternative path separators that might be valid
    fn alternative_separators(&self) -> &[char];
    
    /// Check if a path is absolute according to this path type's rules
    fn is_absolute_path(&self, path: &str) -> bool;
    
    /// Check if path requires escaping for shell commands
    fn requires_shell_escaping(&self, path: &str) -> bool {
        path.contains(|c: char| c.is_whitespace() || "\"'()[]{}$&<>|;#?*".contains(c))
    }
}

/// POSIX path validation implementation
pub struct PosixPathValidation {
    invalid_chars: Vec<char>,
    reserved_names: Vec<&'static str>,
}

impl Default for PosixPathValidation {
    fn default() -> Self {
        Self {
            invalid_chars: vec!['/', '\0'],
            reserved_names: vec![],  // POSIX doesn't have reserved filenames like Windows
        }
    }
}

impl PathValidationTrait for PosixPathValidation {
    fn max_path_length(&self) -> usize {
        // Many POSIX systems limit to PATH_MAX (typically 4096)
        4096
    }
    
    fn max_filename_length(&self) -> usize {
        // Many POSIX systems limit to NAME_MAX (typically 255)
        255
    }
    
    fn is_valid_filename(&self, name: &str) -> bool {
        !name.is_empty() && 
        !name.contains(|c| self.invalid_filename_chars().contains(&c)) &&
        name.len() <= self.max_filename_length()
    }
    
    fn is_valid_path(&self, path: &str) -> bool {
        !path.is_empty() && 
        !path.contains(|c| self.invalid_path_chars().contains(&c)) &&
        path.len() <= self.max_path_length()
    }
    
    fn invalid_filename_chars(&self) -> &[char] {
        &self.invalid_chars
    }
    
    fn invalid_path_chars(&self) -> &[char] {
        // Only NUL is invalid in a complete path (/ is a separator)
        &['\0']
    }
    
    fn reserved_filenames(&self) -> &[&str] {
        &self.reserved_names
    }
    
    fn path_separator(&self) -> char {
        '/'
    }
    
    fn alternative_separators(&self) -> &[char] {
        &[]  // POSIX only recognizes / as a separator
    }
    
    fn is_absolute_path(&self, path: &str) -> bool {
        path.starts_with('/')
    }
}

/// Windows path validation implementation
pub struct WindowsPathValidation {
    invalid_chars: Vec<char>,
    reserved_names: Vec<&'static str>,
}

impl Default for WindowsPathValidation {
    fn default() -> Self {
        Self {
            invalid_chars: vec!['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0'],
            reserved_names: vec![
                "CON", "PRN", "AUX", "NUL", 
                "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
                "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
            ],
        }
    }
}

impl PathValidationTrait for WindowsPathValidation {
    fn max_path_length(&self) -> usize {
        // Windows traditionally has MAX_PATH of 260, but can be longer with \\?\ prefix
        260
    }
    
    fn max_filename_length(&self) -> usize {
        // Windows filename component limit
        255
    }
    
    fn is_valid_filename(&self, name: &str) -> bool {
        !name.is_empty() && 
        !name.contains(|c| self.invalid_filename_chars().contains(&c)) &&
        !self.is_reserved_filename(name) &&
        name.len() <= self.max_filename_length() &&
        !name.ends_with('.') &&  // Windows doesn't allow filenames ending with dot
        !name.ends_with(' ')     // Windows doesn't allow filenames ending with space
    }
    
    fn is_valid_path(&self, path: &str) -> bool {
        let extended_path = path.starts_with("\\\\?\\");
        
        !path.is_empty() && 
        !path.contains(|c| self.invalid_path_chars().contains(&c)) &&
        !self.contains_reserved_filename(path) &&
        (extended_path || path.len() <= self.max_path_length())
    }
    
    fn invalid_filename_chars(&self) -> &[char] {
        &self.invalid_chars
    }
    
    fn invalid_path_chars(&self) -> &[char] {
        // For complete paths, some characters are valid as part of the path syntax
        &['<', '>', '"', '|', '?', '*', '\0']
    }
    
    fn reserved_filenames(&self) -> &[&str] {
        &self.reserved_names
    }
    
    fn path_separator(&self) -> char {
        '\\'
    }
    
    fn alternative_separators(&self) -> &[char] {
        &['/']  // Windows also accepts / as a separator in most contexts
    }
    
    fn is_absolute_path(&self, path: &str) -> bool {
        path.starts_with('\\') || 
        (path.len() >= 2 && path.chars().nth(1) == Some(':'))  // Drive letter format C:\
    }
}

/// URL path validation implementation
pub struct UrlPathValidation {
    invalid_chars: Vec<char>,
}

impl Default for UrlPathValidation {
    fn default() -> Self {
        Self {
            invalid_chars: vec!['\0', '\r', '\n', '\t', '<', '>', '"', '\'', '`', '\\', ' ', '{', '}', '|'],
        }
    }
}

impl PathValidationTrait for UrlPathValidation {
    fn max_path_length(&self) -> usize {
        // URLs can be very long, but browsers typically limit to around 2000 chars
        2048
    }
    
    fn max_filename_length(&self) -> usize {
        // No strict limit, but keep it reasonable
        255
    }
    
    fn is_valid_filename(&self, name: &str) -> bool {
        !name.is_empty() && 
        !name.contains(|c| self.invalid_filename_chars().contains(&c))
    }
    
    fn is_valid_path(&self, path: &str) -> bool {
        !path.is_empty() && 
        !path.contains(|c| self.invalid_path_chars().contains(&c)) &&
        path.len() <= self.max_path_length()
    }
    
    fn invalid_filename_chars(&self) -> &[char] {
        &self.invalid_chars
    }
    
    fn invalid_path_chars(&self) -> &[char] {
        &['\0', '\r', '\n', '\t']  // Control characters
    }
    
    fn reserved_filenames(&self) -> &[&str] {
        &[]  // URLs don't have reserved filenames
    }
    
    fn path_separator(&self) -> char {
        '/'
    }
    
    fn alternative_separators(&self) -> &[char] {
        &[]  // URLs only use / as separator
    }
    
    fn is_absolute_path(&self, path: &str) -> bool {
        path.starts_with('/')
    }
}

impl PosixPath {
    /// Get path validation knowledge
    pub fn validation(&self) -> &dyn PathValidationTrait {
        &PosixPathValidation::default()
    }
    
    /// Validate this path
    pub fn validate(&self) -> Result<()> {
        let validator = PosixPathValidation::default();
        let path_str = self.to_string();
        
        if !validator.is_valid_path(&path_str) {
            return Err(Error::from_string(format!("Invalid POSIX path: {}", path_str)));
        }
        
        if let Some(filename) = self.file_name() {
            if !validator.is_valid_filename(&filename) {
                return Err(Error::from_string(format!("Invalid POSIX filename: {}", filename)));
            }
        }
        
        Ok(())
    }
} 