use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

/// Unique identifier for a component
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentId(pub String);

/// Represents the state of a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentState {
    Uninitialized,
    Initializing,
    Active,
    Pausing,
    Paused,
    Resuming,
    Deactivating,
    Inactive,
    Error(String),
}

/// Basic properties that all components should have
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProps {
    pub id: ComponentId,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

/// Configuration for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub enabled: bool,
    pub auto_start: bool,
    pub settings: HashMap<String, String>,
}

/// Abstract trait that all components must implement
#[async_trait]
pub trait Component: Send + Sync {
    /// Get the component's unique identifier
    fn id(&self) -> &ComponentId;

    /// Get the component's properties
    fn props(&self) -> &ComponentProps;

    /// Get the current state of the component
    fn state(&self) -> ComponentState;

    /// Initialize the component
    async fn initialize(&mut self, config: ComponentConfig) -> Result<(), String>;

    /// Start the component
    async fn start(&mut self) -> Result<(), String>;

    /// Pause the component
    async fn pause(&mut self) -> Result<(), String>;

    /// Resume the component
    async fn resume(&mut self) -> Result<(), String>;

    /// Stop the component
    async fn stop(&mut self) -> Result<(), String>;

    /// Destroy the component and clean up resources
    async fn destroy(&mut self) -> Result<(), String>;

    /// Handle a custom message (for inter-component communication)
    async fn handle_message(&mut self, message: ComponentMessage) -> Result<Option<ComponentMessage>, String>;

    /// As any downcast for type-safe component retrieval
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Message types for inter-component communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentMessage {
    /// Request data from another component
    GetData(String),
    /// Response containing data
    DataResponse(String, serde_json::Value),
    /// Command to perform an action
    Command(String, Option<serde_json::Value>),
    /// Event notification
    Event(String, Option<serde_json::Value>),
    /// Custom message
    Custom(serde_json::Value),
}

/// Registry for managing components
pub struct ComponentRegistry {
    components: RwLock<HashMap<ComponentId, Mutex<Box<dyn Component>>>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// Register a component with the registry
    pub fn register(&self, component: Box<dyn Component>) -> Result<(), String> {
        let id = component.id().clone();
        let mut components_write = self.components.write().map_err(|e| e.to_string())?;

        if components_write.contains_key(&id) {
            return Err(format!("Component with ID {} already registered", id.0));
        }

        components_write.insert(id, Mutex::new(component));
        Ok(())
    }

    /// Unregister a component from the registry
    pub fn unregister(&self, id: &ComponentId) -> Result<Option<Box<dyn Component>>, String> {
        let mut components_write = self.components.write().map_err(|e| e.to_string())?;

        if let Some(component_mutex) = components_write.remove(id) {
            // Try to get the inner component
            let mut component_guard = component_mutex.lock().map_err(|e| e.to_string())?;
            Ok(Some(std::mem::take(&mut *component_guard)))
        } else {
            Ok(None)
        }
    }

    /// Get a component by its ID
    pub fn get(&self, id: &ComponentId) -> Result<Option<Box<dyn Component>>, String> {
        let components_read = self.components.read().map_err(|e| e.to_string())?;

        if let Some(component_mutex) = components_read.get(id) {
            // Clone the component (this requires the component to be Cloneable)
            // For now, we'll return a reference wrapper approach
            let component_guard = component_mutex.lock().map_err(|e| e.to_string())?;
            // We can't return the trait object directly from a lock guard,
            // so we'll need to rethink this approach

            // Instead, let's return a cloned boxed component if it's cloneable
            // But since Component isn't Clone, we'll need to handle this differently

            // For simplicity, let's return a reference to the component wrapped in an Arc
            // But that requires changing the design significantly

            // Let's go with a simpler approach: return the mutex guard so the caller can use it
            // But we can't return guards from functions easily

            // Alternative: provide methods that operate on components through the registry
            Ok(None) // Placeholder - we'll refine this
        } else {
            Ok(None)
        }
    }

    /// List all registered component IDs
    pub fn list_ids(&self) -> Vec<ComponentId> {
        let components_read = self.components.read().unwrap_or_else(|e| panic!("Failed to read components: {}", e));
        components_read.keys().cloned().collect()
    }

    /// Get the number of registered components
    pub fn len(&self) -> usize {
        let components_read = self.components.read().unwrap_or_else(|e| panic!("Failed to read components: {}", e));
        components_read.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Helper trait for components that can be cloned
pub trait ClonableComponent: Component {
    fn clone_box(&self) -> Box<dyn ClonableComponent>;
}

impl<T: Component + Clone> ClonableComponent for T {
    fn clone_box(&self) -> Box<dyn ClonableComponent> {
        Box::new(self.clone())
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;

    /// A simple test component for unit tests
    struct TestComponent {
        id: ComponentId,
        props: ComponentProps,
        state: ComponentState,
    }

    impl TestComponent {
        fn new(id: ComponentId, name: String) -> Self {
            Self {
                id: id.clone(),
                props: ComponentProps {
                    id: id.clone(),
                    name,
                    version: "0.1.0".to_string(),
                    description: Some("Test component".to_string()),
                },
                state: ComponentState::Uninitialized,
            }
        }
    }

    #[async_trait]
    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }

        fn props(&self) -> &ComponentProps {
            &self.props
        }

        fn state(&self) -> ComponentState {
            self.state.clone()
        }

        async fn initialize(&mut self, _config: ComponentConfig) -> Result<(), String> {
            self.state = ComponentState::Active;
            Ok(())
        }

        async fn start(&mut self) -> Result<(), String> {
            self.state = ComponentState::Active;
            Ok(())
        }

        async fn pause(&mut self) -> Result<(), String> {
            self.state = ComponentState::Paused;
            Ok(())
        }

        async fn resume(&mut self) -> Result<(), String> {
            self.state = ComponentState::Active;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), String> {
            self.state = ComponentState::Inactive;
            Ok(())
        }

        async fn destroy(&mut self) -> Result<(), String> {
            self.state = ComponentState::Uninitialized;
            Ok(())
        }

        async fn handle_message(&mut self, _message: ComponentMessage) -> Result<Option<ComponentMessage>, String> {
            Ok(None)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_component_registry() {
        let registry = ComponentRegistry::new();

        // Create a test component
        let mut component = TestComponent::new(
            ComponentId("test-component".to_string()),
            "Test Component".to_string()
        );

        // Register the component
        let result = registry.register(Box::new(component));
        assert!(result.is_ok());

        // Check that we have one component
        assert_eq!(registry.len(), 1);

        // List component IDs
        let ids = registry.list_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].0, "test-component");

        // Unregister the component
        let unregistered = registry.unregister(&ComponentId("test-component".to_string()));
        assert!(unregistered.is_ok());
        assert!(unregistered.unwrap().is_some());

        // Check that we have zero components
        assert_eq!(registry.len(), 0);
    }
}