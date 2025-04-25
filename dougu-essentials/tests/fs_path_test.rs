use dougu_essentials::fs::{LocalPathType, default_path_type};

#[test]
fn test_default_path_type() {
    // This test simply verifies that the default_path_type function
    // returns a value appropriate for the current OS
    let path_type = default_path_type();
    
    #[cfg(target_family = "unix")]
    {
        assert_eq!(path_type, LocalPathType::Posix);
    }
    
    #[cfg(target_family = "windows")]
    {
        assert_eq!(path_type, LocalPathType::Windows);
    }
} 