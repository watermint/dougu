use dougu_foundation_i18n::{init, load_translations, set_locale, t, tf, vars, I18nCommandletError};
use dougu_foundation_run::CommandletError;

#[test]
fn test_i18n_commandlet_integration() {
    // Initialize the i18n system
    init("en").unwrap();
    
    // Load translations
    load_translations("en", "../dougu-foundation-run/src/resources/en.json").unwrap();
    load_translations("es", "../dougu-foundation-run/src/resources/es.json").unwrap();
    
    // Test basic translation in English (default)
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert_eq!(error_msg, "Resource not found");
    
    // Test with variables
    let layer_msg = tf("LAYER_EXECUTION", vars!("" => "TestLayer"));
    assert_eq!(layer_msg, "Executing layer: TestLayer");
    
    // Test command error with i18n
    let error = CommandletError::with_i18n("NOT_FOUND", "RESOURCE_NOT_FOUND");
    assert_eq!(error.code, "NOT_FOUND");
    assert_eq!(error.message, "Resource not found");
    
    // Switch to Spanish
    set_locale("es").unwrap();
    
    // Test in Spanish
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert_eq!(error_msg, "Recurso no encontrado");
    
    // Test with variables in Spanish
    let layer_msg = tf("LAYER_EXECUTION", vars!("" => "TestLayer"));
    assert_eq!(layer_msg, "Ejecutando capa: TestLayer");
    
    // Test command error with i18n in Spanish
    let error = CommandletError::with_i18n("NOT_FOUND", "RESOURCE_NOT_FOUND");
    assert_eq!(error.code, "NOT_FOUND");
    assert_eq!(error.message, "Recurso no encontrado");
}

#[test]
fn test_file_commandlet_translations() {
    // Initialize the i18n system
    init("en").unwrap();
    
    // Load translations
    load_translations("en", "../dougu-command-file/src/resources/en.json").unwrap();
    load_translations("es", "../dougu-command-file/src/resources/es.json").unwrap();
    
    // Test file copy success message in English
    let msg = tf("FILE_COPY_SUCCESS", vars!(
        "source" => "file.txt",
        "destination" => "backup.txt"
    ));
    assert_eq!(msg, "Successfully copied file.txt to backup.txt");
    
    // Test file list start message in English
    let msg = tf("FILE_LIST_START", vars!(
        "directory" => "/home/user"
    ));
    assert_eq!(msg, "Listing directory: /home/user");
    
    // Switch to Spanish
    set_locale("es").unwrap();
    
    // Test file copy success message in Spanish
    let msg = tf("FILE_COPY_SUCCESS", vars!(
        "source" => "file.txt",
        "destination" => "backup.txt"
    ));
    assert_eq!(msg, "Copiado con éxito file.txt a backup.txt");
    
    // Test file list start message in Spanish
    let msg = tf("FILE_LIST_START", vars!(
        "directory" => "/home/user"
    ));
    assert_eq!(msg, "Listando directorio: /home/user");
} 