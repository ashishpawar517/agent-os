use tauri::{Manager, WebviewWindow, WebviewUrl, WebviewWindowBuilder};
use tauri::WindowBuilder;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Represents a browser tab/window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub loading: bool,
}

/// Configuration for a browser window/tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub url: String,
    pub label: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: bool,
    pub fullscreen: bool,
    pub decorations: bool,
    pub always_on_top: bool,
}

/// Manager for browser windows/tabs
pub struct BrowserManager {
    tabs: Arc<Mutex<HashMap<String, BrowserTab>>>,
    windows: Arc<Mutex<HashMap<String, WebviewWindow>>>,
    next_id: Arc<Mutex<u64>>,
}

impl BrowserManager {
    /// Create a new browser manager
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(HashMap::new())),
            windows: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new browser tab/window
    pub fn create_browser(
        &self,
        config: BrowserConfig,
        app_handle: &tauri::AppHandle,
    ) -> Result<String, String> {
        let mut next_id = self.next_id.lock().map_err(|e| e.to_string())?;
        let id = format!("browser-{}", *next_id);
        *next_id += 1;

        let label = config.label.unwrap_or_else(|| id.clone());
        let width = config.width.unwrap_or(800);
        let height = config.height.unwrap_or(600);

        // Create the webview window
        let builder = WebviewWindowBuilder::new(app_handle, &label, WebviewUrl::External(config.url.parse().map_err(|e| e.to_string())?))
            .title("Agent OS Browser")
            .inner_size(width, height)
            .resizable(config.resizable)
            .fullscreen(config.fullscreen)
            .decorations(config.decorations)
            .always_on_top(config.always_on_top);

        let window = builder.build().map_err(|e| e.to_string())?;

        // Set up event listeners for the window
        let window_id = window.id();
        let tabs_clone = self.tabs.clone();
        let windows_clone = self.windows.clone();

        // Listen for title changes
        {
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::event::WindowEvent::Focused(true) = event {
                    // Update tab as active when focused
                    // In a full implementation, we'd update the active tab state
                }
            });
        }

        // Listen for URL changes (navigation)
        {
            let tabs_clone = tabs_clone.clone();
            let id_clone = id.clone();
            window.on_navigate(move |url| {
                if let Some(url_str) = url.to_str() {
                    // Update the tab's URL when navigation occurs
                    if let Ok(mut tabs) = tabs_clone.lock() {
                        if let Some(tab) = tabs.get_mut(&id_clone) {
                            tab.url = url_str.to_string();
                        }
                    }
                }
            });
        }

        // Store the tab and window
        let tab = BrowserTab {
            id: id.clone(),
            url: config.url.clone(),
            title: "Loading...".to_string(),
            loading: true,
        };

        {
            let mut tabs = self.tabs.lock().map_err(|e| e.to_string())?;
            tabs.insert(id.clone(), tab);
        }

        {
            let mut windows = self.windows.lock().map_err(|e| e.to_string())?;
            windows.insert(id.clone(), window);
        }

        Ok(id)
    }

    /// Get a browser tab by ID
    pub fn get_tab(&self, id: &str) -> Option<BrowserTab> {
        let tabs = self.tabs.lock().ok()?;
        tabs.get(id).cloned()
    }

    /// Get a browser window by ID
    pub fn get_window(&self, id: &str) -> Option<WebviewWindow> {
        let windows = self.windows.lock().ok()?;
        windows.get(id).cloned()
    }

    /// Navigate to a URL in an existing browser tab
    pub fn navigate(&self, id: &str, url: &str) -> Result<(), String> {
        if let Some(window) = self.get_window(id) {
            // Evaluate JavaScript to change location
            let _ = window.eval(&format!("window.location.href = '{}';", url.replace("'", "\\'")));
            Ok(())
        } else {
            Err(format!("Browser window not found: {}", id))
        }
    }

    /// Reload the current page
    pub fn reload(&self, id: &str) -> Result<(), String> {
        if let Some(window) = self.get_window(id) {
            let _ = window.eval("window.location.reload();");
            Ok(())
        } else {
            Err(format!("Browser window not found: {}", id))
        }
    }

    /// Go back in history
    pub fn back(&self, id: &str) -> Result<(), String> {
        if let Some(window) = self.get_window(id) {
            let _ = window.eval("window.history.back();");
            Ok(())
        } else {
            Err(format!("Browser window not found: {}", id))
        }
    }

    /// Go forward in history
    pub fn forward(&self, id: &str) -> Result<(), String> {
        if let Some(window) = self.get_window(id) {
            let _ = window.eval("window.history.forward();");
            Ok(())
        } else {
            Err(format!("Browser window not found: {}", id))
        }
    }

    /// Close a browser tab/window
    pub fn close(&self, id: &str) -> Result<(), String> {
        let mut tabs = self.tabs.lock().map_err(|e| e.to_string())?;
        let mut windows = self.windows.lock().map_err(|e| e.to_string())?;

        tabs.remove(id);
        if let Some(window) = windows.remove(id) {
            let _ = window.close();
        }

        Ok(())
    }

    /// List all browser tab IDs
    pub fn list_tabs(&self) -> Vec<String> {
        let tabs = self.tabs.lock().ok().unwrap_or_default();
        tabs.keys().cloned().collect()
    }

    /// Get the number of browser tabs
    pub fn len(&self) -> usize {
        let tabs = self.tabs.lock().ok().unwrap_or_default();
        tabs.len()
    }

    /// Check if the manager is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use tauri::api::async_runtime::block_on;

    // Note: Proper unit tests for Tauri require a runtime context
    // These are basic structure tests

    #[test]
    fn test_browser_manager_creation() {
        let manager = BrowserManager::new();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_browser_tab_struct() {
        let tab = BrowserTab {
            id: "test".to_string(),
            url: "https://example.com".to_string(),
            title: "Test Page".to_string(),
            loading: false,
        };
        assert_eq!(tab.id, "test");
        assert_eq!(tab.url, "https://example.com");
        assert_eq!(tab.title, "Test Page");
        assert!(!tab.loading);
    }
}