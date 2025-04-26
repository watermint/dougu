use super::*;
use std::any::Any;
use core::default::{DefaultPathComponents, DefaultNamespace};
use core::Path;
use essential::EssentialPath;
use resolver::{PathResolver, PathResolverRepository};
use local::{ServerInfo, StandardServerInfo, PathCredentials};

// Mock path type for testing
#[derive(Debug, Clone)]
struct MockServicePath {
    account: String,
    path: String,
}

impl MockServicePath {
    fn new(account: &str, path: &str) -> Self {
        MockServicePath {
            account: account.to_string(),
            path: path.to_string(),
        }
    }
}

impl Path for MockServicePath {
    type ComponentsType = DefaultPathComponents;
    type NamespaceType = DefaultNamespace;
    
    fn new() -> Self {
        MockServicePath {
            account: String::new(),
            path: String::new(),
        }
    }
    
    fn namespace(&self) -> &Self::NamespaceType {
        // This is a dummy implementation since we don't use these methods
        unimplemented!("Not needed for test")
    }
    
    fn namespace_mut(&mut self) -> &mut Self::NamespaceType {
        unimplemented!("Not needed for test")
    }
    
    fn components(&self) -> &Self::ComponentsType {
        unimplemented!("Not needed for test")
    }
    
    fn components_mut(&mut self) -> &mut Self::ComponentsType {
        unimplemented!("Not needed for test")
    }
    
    fn parse(path_str: &str) -> crate::core::error::Result<Self> {
        unimplemented!("Not needed for test")
    }
    
    fn to_string(&self) -> String {
        format!("{}:{}", self.account, self.path)
    }
    
    fn join(&self, _relative: &str) -> crate::core::error::Result<Self> {
        unimplemented!("Not needed for test")
    }
    
    fn parent(&self) -> Option<Self> {
        unimplemented!("Not needed for test")
    }
    
    fn file_name(&self) -> Option<String> {
        unimplemented!("Not needed for test")
    }
    
    fn normalize(&mut self) {
        // No-op for test
    }
    
    fn is_absolute(&self) -> bool {
        self.path.starts_with('/')
    }
    
    fn to_local_path(&self) -> Option<Box<dyn local::LocalPath>> {
        None
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Mock resolver for testing
struct MockServiceResolver {
    service_id: String,
    accounts: Vec<String>,
}

impl MockServiceResolver {
    fn new(service_id: &str, accounts: Vec<String>) -> Self {
        MockServiceResolver {
            service_id: service_id.to_string(),
            accounts,
        }
    }
}

impl PathResolver for MockServiceResolver {
    fn id(&self) -> &str {
        &self.service_id
    }
    
    fn can_resolve(&self, namespace: &str) -> bool {
        self.accounts.contains(&namespace.to_string())
    }
    
    fn resolve(&self, path: &EssentialPath) -> crate::core::error::Result<Box<dyn Path>> {
        let namespace = path.namespace().as_str();
        
        if !self.can_resolve(namespace) {
            return Err(crate::core::error::Error::InvalidArgument(
                format!("Account {} not supported by service {}", namespace, self.service_id)
            ));
        }
        
        let components_str = path.components().join();
        let path_str = if path.is_absolute() {
            format!("/{}", components_str)
        } else {
            components_str
        };
        
        let service_path = MockServicePath::new(namespace, &path_str);
        
        // Box the service path as a Box<dyn Path>
        Ok(Box::new(service_path))
    }
    
    fn to_essential_path(&self, path: &dyn Path) -> crate::core::error::Result<EssentialPath> {
        // Try to downcast to MockServicePath
        if let Some(service_path) = path.as_any().downcast_ref::<MockServicePath>() {
            let path_str = if service_path.path.starts_with('/') {
                format!("{}:{}", service_path.account, service_path.path)
            } else {
                format!("{}:{}", service_path.account, service_path.path)
            };
            
            EssentialPath::parse(&path_str)
        } else {
            Err(crate::core::error::Error::InvalidArgument(
                "Path is not a MockServicePath".to_string()
            ))
        }
    }
}

#[test]
fn test_essential_path_parsing() {
    // Test absolute path with namespace
    let path1 = EssentialPath::parse("account1:/sales/report").unwrap();
    assert_eq!(path1.namespace().as_str(), "account1");
    assert_eq!(path1.components().get(0), Some("sales"));
    assert_eq!(path1.components().get(1), Some("report"));
    assert!(path1.is_absolute());
    
    // Test relative path with namespace
    let path2 = EssentialPath::parse("account2:sales/forecast").unwrap();
    assert_eq!(path2.namespace().as_str(), "account2");
    assert_eq!(path2.components().get(0), Some("sales"));
    assert_eq!(path2.components().get(1), Some("forecast"));
    assert!(!path2.is_absolute());
    
    // Test absolute path without namespace
    let path3 = EssentialPath::parse("/sales/invoice").unwrap();
    assert_eq!(path3.namespace().as_str(), "");
    assert_eq!(path3.components().get(0), Some("sales"));
    assert_eq!(path3.components().get(1), Some("invoice"));
    assert!(path3.is_absolute());
    
    // Test to_string
    assert_eq!(path1.to_string(), "account1:/sales/report");
    assert_eq!(path2.to_string(), "account2:sales/forecast");
    assert_eq!(path3.to_string(), "/sales/invoice");
}

#[test]
fn test_resolver_repository() {
    // Create mock resolvers
    let dropbox_resolver = MockServiceResolver::new(
        "dropbox", 
        vec!["account1".to_string(), "dropbox-user".to_string()]
    );
    
    let onedrive_resolver = MockServiceResolver::new(
        "onedrive",
        vec!["account2".to_string(), "onedrive-user".to_string()]
    );
    
    // Create repository and register resolvers
    let mut repo = PathResolverRepository::new();
    repo.register_resolver(Box::new(dropbox_resolver));
    repo.register_resolver(Box::new(onedrive_resolver));
    
    // Test resolution
    let path1 = EssentialPath::parse("account1:/sales/report").unwrap();
    let resolved1 = repo.resolve(&path1).unwrap();
    assert_eq!(resolved1.to_string(), "account1:/sales/report");
    
    let path2 = EssentialPath::parse("account2:/sales/forecast").unwrap();
    let resolved2 = repo.resolve(&path2).unwrap();
    assert_eq!(resolved2.to_string(), "account2:/sales/forecast");
    
    // Test unresolvable path
    let path3 = EssentialPath::parse("unknown:/sales/document").unwrap();
    assert!(repo.resolve(&path3).is_err());
}

// Add a new test for ServerInfo
#[test]
fn test_server_info() {
    // Create a standard server info with all fields
    let credentials = PathCredentials {
        username: "user".to_string(),
        password: Some("pass".to_string()),
    };
    
    let mut server_info = StandardServerInfo::new(
        "fileserver",
        Some("documents"),
        Some(credentials)
    );
    
    // Test basic properties
    assert_eq!(server_info.server(), "fileserver");
    assert_eq!(server_info.share(), Some("documents"));
    assert!(server_info.credentials().is_some());
    if let Some(creds) = server_info.credentials() {
        assert_eq!(creds.username, "user");
        assert_eq!(creds.password, Some("pass".to_string()));
    }
    
    // Test properties
    server_info.add_property("protocol", "SMB");
    server_info.add_property("port", "445");
    
    // Due to the implementation limitation, we can't test properties directly
    // In a real implementation, we would use a different approach for properties
} 