use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: Option<String>,
    pub created_at: u64,
    pub last_accessed: u64,
    pub tabs: Vec<String>, // Tab IDs
    pub active_tab: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Represents a UI tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UITab {
    pub id: String,
    pub session_id: String,
    pub title: String,
    pub component_type: String, // e.g., "terminal", "browser", "editor"
    pub component_state: Option<serde_json::Value>,
    pub created_at: u64,
    pub last_accessed: u64,
    pub is_active: bool,
    pub settings: HashMap<String, String>,
}

/// Configuration for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub timeout_minutes: u64,
    pub max_tabs_per_session: usize,
    pub allow_tab_restore: bool,
}

/// Manager for user sessions and tabs
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    tabs: Arc<RwLock<HashMap<String, UITab>>>,
    config: SessionConfig,
    next_session_id: Arc<Mutex<u64>>,
    next_tab_id: Arc<Mutex<u64>>,
}

impl SessionManager {
    /// Create a new session manager with default configuration
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            tabs: Arc::new(RwLock::new(HashMap::new())),
            config: SessionConfig {
                timeout_minutes: 30,
                max_tabs_per_session: 10,
                allow_tab_restore: true,
            },
            next_session_id: Arc::new(Mutex::new(0)),
            next_tab_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new session manager with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            tabs: Arc::new(RwLock::new(HashMap::new())),
            config,
            next_session_id: Arc::new(Mutex::new(0)),
            next_tab_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Generate a unique session ID
    fn generate_session_id(&self) -> String {
        let mut next_id = self.next_session_id.lock().unwrap();
        let id = format!("session-{}", *next_id);
        *next_id += 1;
        id
    }

    /// Generate a unique tab ID
    fn generate_tab_id(&self) -> String {
        let mut next_id = self.next_tab_id.lock().unwrap();
        let id = format!("tab-{}", *next_id);
        *next_id += 1;
        id
    }

    /// Get current timestamp in seconds since epoch
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| UNIX_EPOCH.duration_since(UNIX_EPOCH).unwrap())
            .as_secs()
    }

    /// Create a new user session
    pub fn create_session(&self, user_id: Option<String>) -> Result<String, String> {
        let session_id = self.generate_session_id();
        let now = Self::now();

        let session = UserSession {
            id: session_id.clone(),
            user_id,
            created_at: now,
            last_accessed: now,
            tabs: Vec::new(),
            active_tab: None,
            metadata: HashMap::new(),
        };

        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<UserSession> {
        let sessions = self.sessions.read().ok()?;
        sessions.get(session_id).cloned()
    }

    /// Update session last accessed time
    pub fn touch_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_accessed = Self::now();
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Destroy a session and all its tabs
    pub fn destroy_session(&self, session_id: &str) -> Result<(), String> {
        // Remove all tabs belonging to this session
        {
            let mut tabs = self.tabs.write().map_err(|e| e.to_string())?;
            tabs.retain(|_, tab| tab.session_id != session_id);
        }

        // Remove the session
        let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
        sessions.remove(session_id);

        Ok(())
    }

    /// Create a new tab in a session
    pub fn create_tab(
        &self,
        session_id: &str,
        title: String,
        component_type: String,
        component_state: Option<serde_json::Value>,
        settings: HashMap<String, String>,
    ) -> Result<String, String> {
        // Verify session exists
        if self.get_session(session_id).is_none() {
            return Err(format!("Session {} not found", session_id));
        }

        // Check if session has too many tabs
        let session = self.get_session(session_id).ok_or_else(|| format!("Session {} not found", session_id))?;
        if session.tabs.len() >= self.config.max_tabs_per_session {
            return Err(format!("Session {} has exceeded maximum tabs limit", session_id));
        }

        let tab_id = self.generate_tab_id();
        let now = Self::now();

        let tab = UITab {
            id: tab_id.clone(),
            session_id: session_id.to_string(),
            title,
            component_type,
            component_state,
            created_at: now,
            last_accessed: now,
            is_active: false,
            settings,
        };

        {
            let mut tabs = self.tabs.write().map_err(|e| e.to_string())?;
            tabs.insert(tab_id.clone(), tab);
        }

        // Add tab to session
        {
            let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
            if let Some(session) = sessions.get_mut(session_id) {
                session.tabs.push(tab_id.clone());
                session.last_accessed = Self::now();
            }
        }

        Ok(tab_id)
    }

    /// Get a tab by ID
    pub fn get_tab(&self, tab_id: &str) -> Option<UITab> {
        let tabs = self.tabs.read().ok()?;
        tabs.get(tab_id).cloned()
    }

    /// Set a tab as active
    pub fn set_active_tab(&self, session_id: &str, tab_id: &str) -> Result<(), String> {
        // Verify session exists
        if self.get_session(session_id).is_none() {
            return Err(format!("Session {} not found", session_id));
        }

        // Verify tab exists and belongs to session
        let tab = self.get_tab(tab_id).ok_or_else(|| format!("Tab {} not found", tab_id))?;
        if tab.session_id != session_id {
            return Err(format!("Tab {} does not belong to session {}", tab_id, session_id));
        }

        // Deactivate all tabs in session
        {
            let mut tabs = self.tabs.write().map_err(|e| e.to_string())?;
            for tab in tabs.values_mut() {
                if tab.session_id == session_id {
                    tab.is_active = false;
                }
            }
        }

        // Activate the specified tab
        {
            let mut tabs = self.tabs.write().map_err(|e| e.to_string())?;
            if let Some(tab) = tabs.get_mut(tab_id) {
                tab.is_active = true;
                tab.last_accessed = Self::now();
            }
        }

        // Update session's active tab
        {
            let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
            if let Some(session) = sessions.get_mut(session_id) {
                session.active_tab = Some(tab_id.to_string());
                session.last_accessed = Self::now();
            }
        }

        Ok(())
    }

    /// Get the active tab for a session
    pub fn get_active_tab(&self, session_id: &str) -> Option<UITab> {
        let session = self.get_session(session_id)?;
        let active_tab_id = session.active_tab.as_ref()?;
        self.get_tab(active_tab_id)
    }

    /// Close a tab
    pub fn close_tab(&self, tab_id: &str) -> Result<(), String> {
        let tab = self.get_tab(tab_id).ok_or_else(|| format!("Tab {} not found", tab_id))?;
        let session_id = &tab.session_id;

        // Remove tab from tabs collection
        {
            let mut tabs = self.tabs.write().map_err(|e| e.to_string())?;
            tabs.remove(tab_id);
        }

        // Remove tab from session's tab list
        {
            let mut sessions = self.sessions.write().map_err(|e| e.to_string())?;
            if let Some(session) = sessions.get_mut(session_id) {
                session.tabs.retain(|id| id != tab_id);

                // If this was the active tab, activate another tab or set to None
                if session.active_tab.as_deref() == Some(tab_id) {
                    if let Some(first_tab) = session.tabs.first() {
                        session.active_tab = Some(first_tab.clone());
                        // Activate the new active tab in the tabs collection
                        if let Some(tab) = self.get_tab(first_tab) {
                            let mut tabs = self.tabs.write().map_err(|e| e.to_string())?;
                            if let Some(tab) = tabs.get_mut(first_tab) {
                                tab.is_active = true;
                            }
                        }
                    } else {
                        session.active_tab = None;
                    }
                }

                session.last_accessed = Self::now();
            }
        }

        Ok(())
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().ok().unwrap_or_default();
        sessions.keys().cloned().collect()
    }

    /// List all tabs for a session
    pub fn list_session_tabs(&self, session_id: &str) -> Vec<String> {
        let session = self.get_session(session_id).unwrap_or_else(|| {
            // Return empty vec if session not found
            return UserSession {
                id: "".to_string(),
                user_id: None,
                created_at: 0,
                last_accessed: 0,
                tabs: Vec::new(),
                active_tab: None,
                metadata: HashMap::new(),
            };
        });
        session.tabs.clone()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> usize {
        let now = Self::now();
        let timeout_seconds = self.config.timeout_minutes * 60;
        let mut expired_count = 0;

        // Find expired sessions
        let mut expired_sessions = Vec::new();
        {
            let sessions = self.sessions.read().ok().unwrap_or_default();
            for (session_id, session) in sessions.iter() {
                if now - session.last_accessed > timeout_seconds {
                    expired_sessions.push(session_id.clone());
                }
            }
        }

        // Remove expired sessions
        for session_id in expired_sessions {
            if self.destroy_session(&session_id).is_ok() {
                expired_count += 1;
            }
        }

        expired_count
    }

    /// Get session configuration
    pub fn get_config(&self) -> &SessionConfig {
        &self.config
    }

    /// Set session configuration
    pub fn set_config(&mut self, config: SessionConfig) {
        self.config = config;
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.list_sessions().len(), 0);
    }

    #[tokio::test]
    async fn test_session_creation_and_management() {
        let manager = SessionManager::new();

        // Create a session
        let session_id = manager.create_session(Some("user123".to_string())).unwrap();
        assert!(!session_id.is_empty());

        // Get the session
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert_eq!(session.tabs.len(), 0);
        assert!(session.active_tab.is_none());

        // Touch the session
        manager.touch_session(&session_id).unwrap();

        // Create a tab
        let tab_id = manager.create_tab(
            &session_id,
            "Test Tab".to_string(),
            "terminal".to_string(),
            None,
            HashMap::new(),
        ).unwrap();
        assert!(!tab_id.is_empty());

        // Check session has the tab
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.tabs.len(), 1);
        assert_eq!(session.tabs[0], tab_id);

        // Set tab as active
        manager.set_active_tab(&session_id, &tab_id).unwrap();

        // Check active tab
        let active_tab = manager.get_active_tab(&session_id).unwrap();
        assert_eq!(active_tab.id, tab_id);
        assert!(active_tab.is_active);

        // Close the tab
        manager.close_tab(&tab_id).unwrap();

        // Check session no longer has the tab
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.tabs.len(), 0);
        assert!(session.active_tab.is_none());
    }
}