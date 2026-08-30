use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Represents a permission that can be granted or denied
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// File system permissions
    FileRead(String),      // Path or pattern
    FileWrite(String),     // Path or pattern
    FileCreate(String),    // Path or pattern
    FileDelete(String),    // Path or pattern

    /// Network permissions
    NetworkAccess,         // General network access
    NetworkOutbound(String), // Specific domain or IP
    NetworkInbound(u16),   // Specific port

    /// System permissions
    ProcessSpawn,          // Ability to spawn processes
    ProcessKill,           // Ability to kill processes
    EnvRead,               // Read environment variables
    EnvWrite(String),      // Write to specific environment variable

    /// Device permissions
    CameraAccess,
    MicrophoneAccess,
    ScreenCapture,

    /// Custom permissions
    Custom(String),
}

/// Represents a security policy that defines what permissions are allowed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
    pub is_default: bool,
}

/// Represents a security context for a component or operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub id: String,
    pub policies: Vec<String>, // Names of policies applied
    pub overrides: HashMap<Permission, bool>, // Specific permission overrides (true=allow, false=deny)
}

/// Manager for security policies and permissions
pub struct SecurityManager {
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
    contexts: Arc<RwLock<HashMap<String, SecurityContext>>>,
    default_policy: Arc<RwLock<Option<String>>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        let mut manager = Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            default_policy: Arc::new(RwLock::new(None)),
        };

        // Create a default restrictive policy
        let default_policy = SecurityPolicy {
            name: "default".to_string(),
            description: Some("Default restrictive security policy".to_string()),
            permissions: vec![
                // Allow basic file operations in project directory
                Permission::FileRead("./".to_string()),
                Permission::FileWrite("./".to_string()),
                Permission::FileCreate("./".to_string()),
                // Allow network access to localhost and private IPs
                Permission::NetworkOutbound("localhost".to_string()),
                Permission::NetworkOutbound("127.0.0.1".to_string()),
                Permission::NetworkOutbound("::1".to_string()),
                // Allow environment read (but not write)
                Permission::EnvRead,
            ],
            is_default: true,
        };

        let policy_name = default_policy.name.clone();
        {
            let mut policies = manager.policies.write().unwrap();
            policies.insert(policy_name.clone(), default_policy);
        }
        {
            let mut default_policy_lock = manager.default_policy.write().unwrap();
            *default_policy_lock = Some(policy_name);
        }

        manager
    }

    /// Add a new security policy
    pub fn add_policy(&self, policy: SecurityPolicy) -> Result<(), String> {
        let mut policies = self.policies.write().map_err(|e| e.to_string())?;
        if policies.contains_key(&policy.name) {
            return Err(format!("Policy with name '{}' already exists", policy.name));
        }
        policies.insert(policy.name.clone(), policy);
        Ok(())
    }

    /// Get a security policy by name
    pub fn get_policy(&self, name: &str) -> Option<SecurityPolicy> {
        let policies = self.policies.read().ok()?;
        policies.get(name).cloned()
    }

    /// Remove a security policy
    pub fn remove_policy(&self, name: &str) -> Result<Option<SecurityPolicy>, String> {
        let mut policies = self.policies.write().map_err(|e| e.to_string())?;
        Ok(policies.remove(name))
    }

    /// Set the default policy
    pub fn set_default_policy(&self, name: &str) -> Result<(), String> {
        // Check if policy exists
        let policies = self.policies.read().map_err(|e| e.to_string())?;
        if !policies.contains_key(name) {
            return Err(format!("Policy '{}' does not exist", name));
        }

        // Update the default policy
        let mut default_policy = self.default_policy.write().map_err(|e| e.to_string())?;
        *default_policy = Some(name.to_string());

        // Mark the policy as default in the policy itself
        let mut policies = self.policies.write().map_err(|e| e.to_string())?;
        if let Some(policy) = policies.get_mut(name) {
            policy.is_default = true;
        }

        // Unmark other policies as default
        for (policy_name, policy) in policies.iter_mut() {
            if policy_name != name {
                policy.is_default = false;
            }
        }

        Ok(())
    }

    /// Get the default policy
    pub fn get_default_policy(&self) -> Option<SecurityPolicy> {
        let default_policy = self.default_policy.read().ok()?;
        let policy_name = default_policy.as_ref()?;
        self.get_policy(policy_name)
    }

    /// Create a new security context
    pub fn create_context(&self, id: &str, policy_names: Vec<String>) -> Result<SecurityContext, String> {
        // Validate that all policies exist
        let policies = self.policies.read().map_err(|e| e.to_string())?;
        for policy_name in &policy_names {
            if !policies.contains_key(policy_name) {
                return Err(format!("Policy '{}' does not exist", policy_name));
            }
        }

        let context = SecurityContext {
            id: id.to_string(),
            policies: policy_names,
            overrides: HashMap::new(),
        };

        let mut contexts = self.contexts.write().map_err(|e| e.to_string())?;
        contexts.insert(id.to_string(), context.clone());

        Ok(context)
    }

    /// Get a security context by ID
    pub fn get_context(&self, id: &str) -> Option<SecurityContext> {
        let contexts = self.contexts.read().ok()?;
        contexts.get(id).cloned()
    }

    /// Update permission overrides for a context
    pub fn update_overrides(
        &self,
        id: &str,
        overrides: HashMap<Permission, bool>,
    ) -> Result<(), String> {
        let mut contexts = self.contexts.write().map_err(|e| e.to_string())?;
        if let Some(context) = contexts.get_mut(id) {
            context.overrides.extend(overrides);
            Ok(())
        } else {
            Err(format!("Context '{}' not found", id))
        }
    }

    /// Check if a permission is allowed in a given context
    pub fn is_allowed(&self, context_id: &str, permission: &Permission) -> bool {
        // Get the context
        let context = match self.get_context(context_id) {
            Some(ctx) => ctx,
            None => return false, // Context doesn't exist, deny by default
        };

        // Check for explicit override first
        if let Some(&allowed) = context.overrides.get(permission) {
            return allowed;
        }

        // Check if any of the context's policies allow the permission
        let policies = match self.policies.read() {
            Ok(p) => p,
            Err(_) => return false, // Can't read policies, deny by default
        };

        for policy_name in &context.policies {
            if let Some(policy) = policies.get(policy_name) {
                if policy.permissions.contains(permission) {
                    return true; // Found in policy, allow
                }
            }
        }

        // Not explicitly allowed, deny by default
        false
    }

    /// List all policy names
    pub fn list_policies(&self) -> Vec<String> {
        let policies = self.policies.read().ok().unwrap_or_default();
        policies.keys().cloned().collect()
    }

    /// List all context IDs
    pub fn list_contexts(&self) -> Vec<String> {
        let contexts = self.contexts.read().ok().unwrap_or_default();
        contexts.keys().cloned().collect()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.get_default_policy().is_some());
        assert_eq!(manager.list_policies().len(), 1);
        assert_eq!(manager.list_policies()[0], "default");
    }

    #[test]
    fn test_permission_checking() {
        let manager = SecurityManager::new();

        // Create a context with the default policy
        let context = manager.create_context("test-context", vec!["default".to_string()]).unwrap();

        // Test that basic file read is allowed in default policy
        assert!(manager.is_allowed("test-context", &Permission::FileRead("./".to_string())));

        // Test that writing to root is not allowed
        assert!(!manager.is_allowed("test-context", &Permission::FileWrite("/".to_string())));

        // Test that localhost network access is allowed
        assert!(manager.is_allowed("test-context", &Permission::NetworkOutbound("localhost".to_string())));

        // Test that external network access is not allowed by default
        assert!(!manager.is_allowed("test-context", &Permission::NetworkOutbound("example.com".to_string())));
    }

    #[test]
    fn test_custom_policy() {
        let mut manager = SecurityManager::new();

        // Create a permissive policy
        let permissive_policy = SecurityPolicy {
            name: "permissive".to_string(),
            description: Some("Permissive policy for testing".to_string()),
            permissions: vec![
                Permission::FileRead("/".to_string()),
                Permission::FileWrite("/".to_string()),
                Permission::NetworkAccess,
            ],
            is_default: false,
        };

        manager.add_policy(permissive_policy).unwrap();

        // Create a context with the permissive policy
        let _ = manager.create_context("permissive-context", vec!["permissive".to_string()]).unwrap();

        // Test that root file access is now allowed
        assert!(manager.is_allowed("permissive-context", &Permission::FileRead("/".to_string())));
        assert!(manager.is_allowed("permissive-context", &Permission::FileWrite("/".to_string())));
        assert!(manager.is_allowed("permissive-context", &Permission::NetworkAccess));
    }
}