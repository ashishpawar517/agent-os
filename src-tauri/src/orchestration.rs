use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

/// Unique identifier for an agent
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// Represents a message that can be sent between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Request to perform a task
    TaskRequest {
        task_id: String,
        task_type: String,
        payload: serde_json::Value,
        reply_to: Option<AgentId>,
    },
    /// Response to a task request
    TaskResponse {
        task_id: String,
        success: bool,
        result: Option<serde_json::Value>,
        error: Option<String>,
    },
    /// Event notification
    Event {
        event_type: String,
        payload: serde_json::Value,
    },
    /// Command to control agent behavior
    Command {
        command: String,
        args: serde_json::Value,
    },
    /// Heartbeat/ping message
    Ping,
    /// Response to ping
    Pong,
}

/// Defines the capabilities of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub task_types: Vec<String>, // Types of tasks this agent can handle
    pub max_concurrent_tasks: usize,
    pub supported_events: Vec<String>,
}

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: AgentId,
    pub name: String,
    pub agent_type: String,
    pub capabilities: AgentCapabilities,
    pub max_restarts: usize,
    pub restart_delay: Duration,
}

/// State of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Initialized,
    Starting,
    Running,
    Pausing,
    Paused,
    Stopping,
    Stopped,
    Error(String),
    Terminating,
}

/// Trait that all agents must implement
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's unique identifier
    fn id(&self) -> &AgentId;

    /// Get the agent's configuration
    fn config(&self) -> &AgentConfig;

    /// Get the current state of the agent
    fn state(&self) -> AgentState;

    /// Initialize the agent
    async fn initialize(&mut self) -> Result<(), String>;

    /// Start the agent
    async fn start(&mut self) -> Result<(), String>;

    /// Pause the agent
    async fn pause(&mut self) -> Result<(), String>;

    /// Resume the agent
    async fn resume(&mut self) -> Result<(), String>;

    /// Stop the agent
    async fn stop(&mut self) -> Result<(), String>;

    /// Terminate the agent (clean shutdown)
    async fn terminate(&mut self) -> Result<(), String>;

    /// Handle an incoming message
    async fn handle_message(&mut self, message: AgentMessage) -> Result<Option<AgentMessage>, String>;

    /// Get the agent's capabilities
    fn capabilities(&self) -> &AgentCapabilities;
}

/// A simple agent implementation that can be extended
pub struct SimpleAgent {
    id: AgentId,
    config: AgentConfig,
    state: AgentState,
    capabilities: AgentCapabilities,
    message_rx: Option<mpsc::Receiver<AgentMessage>>,
    message_tx: Option<mpsc::Sender<AgentMessage>>,
}

impl SimpleAgent {
    pub fn new(config: AgentConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            id: config.id.clone(),
            config,
            state: AgentState::Initialized,
            capabilities: config.capabilities.clone(),
            message_rx: Some(rx),
            message_tx: Some(tx),
        }
    }

    /// Get a clone of the message sender
    fn get_sender(&self) -> Option<mpsc::Sender<AgentMessage>> {
        self.message_tx.as_ref().cloned()
    }
}

#[async_trait]
impl Agent for SimpleAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

    fn state(&self) -> AgentState {
        self.state.clone()
    }

    async fn initialize(&mut self) -> Result<(), String> {
        self.state = AgentState::Running;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        self.state = AgentState::Running;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.state = AgentState::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.state = AgentState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.state = AgentState::Stopped;
        Ok(())
    }

    async fn terminate(&mut self) -> Result<(), String> {
        self.state = AgentState::Stopped;
        Ok(())
    }

    async fn handle_message(&mut self, message: AgentMessage) -> Result<Option<AgentMessage>, String> {
        // Simple echo agent for demonstration - in practice, this would be customized
        match message {
            AgentMessage::Ping => Ok(Some(AgentMessage::Pong)),
            AgentMessage::TaskRequest { task_id, task_type, payload, reply_to } => {
                // Process the task (simplified)
                let result = serde_json::json!({
                    "processed_by": self.id.0,
                    "task_type": task_type,
                    "payload": payload
                });

                Ok(Some(AgentMessage::TaskResponse {
                    task_id,
                    success: true,
                    result: Some(result),
                    error: None,
                }))
            }
            _ => Ok(None), // Ignore other messages by default
        }
    }

    fn capabilities(&self) -> &AgentCapabilities {
        &self.capabilities
    }
}

/// Supervisor agent that manages other agents
pub struct SupervisorAgent {
    id: AgentId,
    config: AgentConfig,
    state: AgentState,
    capabilities: AgentCapabilities,
    managed_agents: RwLock<HashMap<AgentId, Box<dyn Agent>>>,
    message_rx: Option<mpsc::Receiver<AgentMessage>>,
    message_tx: Option<mpsc::Sender<AgentMessage>>,
    restart_strategy: RestartStrategy,
}

impl SupervisorAgent {
    pub fn new(config: AgentConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            id: config.id.clone(),
            config,
            state: AgentState::Initialized,
            capabilities: config.capabilities.clone(),
            managed_agents: RwLock::new(HashMap::new()),
            message_rx: Some(rx),
            message_tx: Some(tx),
            restart_strategy: RestartStrategy::OneForOne,
        }
    }

    /// Add an agent to be supervised
    pub fn add_agent(&self, agent: Box<dyn Agent>) -> Result<(), String> {
        let agent_id = agent.id().clone();
        let mut managed = self.managed_agents.write().map_err(|e| e.to_string())?;
        if managed.contains_key(&agent_id) {
            return Err(format!("Agent {} already managed by supervisor", agent_id.0));
        }
        managed.insert(agent_id, agent);
        Ok(())
    }

    /// Remove an agent from supervision
    pub fn remove_agent(&self, id: &AgentId) -> Result<Option<Box<dyn Agent>>, String> {
        let mut managed = self.managed_agents.write().map_err(|e| e.to_string())?;
        Ok(managed.remove(id))
    }

    /// Get a managed agent
    pub fn get_agent(&self, id: &AgentId) -> Option<Box<dyn Agent>> {
        let managed = self.managed_agents.read().ok()?;
        managed.get(id).cloned()
    }
}

#[async_trait]
impl Agent for SupervisorAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

    fn state(&self) -> AgentState {
        self.state.clone()
    }

    async fn initialize(&mut self) -> Result<(), String> {
        self.state = AgentState::Running;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        self.state = AgentState::Running;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.state = AgentState::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.state = AgentState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.state = AgentState::Stopped;
        // Stop all managed agents
        let managed = self.managed_agents.read().map_err(|e| e.to_string())?;
        for agent in managed.values() {
            let _ = agent.stop().await;
        }
        Ok(())
    }

    async fn terminate(&mut self) -> Result<(), String> {
        self.state = AgentState::Stopped;
        // Terminate all managed agents
        let managed = self.managed_agents.read().map_err(|e| e.to_string())?;
        for agent in managed.values() {
            let _ = agent.terminate().await;
        }
        Ok(())
    }

    async fn handle_message(&mut self, message: AgentMessage) -> Result<Option<AgentMessage>, String> {
        // Forward messages to appropriate managed agents or handle supervisor-specific messages
        match message {
            AgentMessage::Command { command, args } => {
                if command == "list_agents" {
                    let managed = self.managed_agents.read().map_err(|e| e.to_string())?;
                    let agent_ids: Vec<String> = managed.keys().map(|id| id.0.clone()).collect();
                    return Ok(Some(AgentMessage::TaskResponse {
                        task_id: "supervisor_list".to_string(),
                        success: true,
                        result: Some(serde_json::json!(agent_ids)),
                        error: None,
                    }));
                }
            }
            AgentMessage::TaskRequest { task_id, task_type, payload, reply_to } => {
                // Try to find an agent that can handle this task type
                let managed = self.managed_agents.read().map_err(|e| e.to_string())?;
                for agent in managed.values() {
                    if agent.capabilities().task_types.contains(&task_type) {
                        // Forward the task to the capable agent
                        if let Some(tx) = agent.get_sender() {
                            let _ = tx.send(AgentMessage::TaskRequest {
                                task_id: task_id.clone(),
                                task_type: task_type.clone(),
                                payload: payload.clone(),
                                reply_to: reply_to.clone(),
                            }).await;

                            // Wait for response (simplified - in practice would use proper callback)
                            return Ok(Some(AgentMessage::TaskResponse {
                                task_id,
                                success: true,
                                result: Some(serde_json::json!({ " forwarded_to": agent.id().0 })),
                                error: None,
                            }));
                        }
                    }
                }

                // No agent found that can handle the task
                return Ok(Some(AgentMessage::TaskResponse {
                    task_id,
                    success: false,
                    result: None,
                    error: format!("No agent capable of handling task type: {}", task_type),
                }));
            }
            _ => {}
        }

        Ok(None)
    }

    fn capabilities(&self) -> &AgentCapabilities {
        &self.capabilities
    }
}

/// Defines how a supervisor should handle agent failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartStrategy {
    /// Restart only the failed agent
    OneForOne,
    /// Restart all managed agents
    OneForAll,
    /// Restart agents with a certain delay
    RestartWithDelay(Duration),
}

/// Manages the lifecycle of agents in the system
pub struct AgentOrchestrator {
    agents: RwLock<HashMap<AgentId, Box<dyn Agent>>>,
    supervisors: RwLock<HashMap<AgentId, Box<dyn Agent>>>,
    next_id: Mutex<u64>,
    event_queue: Mutex<VecDeque<AgentMessage>>,
}

impl AgentOrchestrator {
    /// Create a new agent orchestrator
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            supervisors: RwLock::new(HashMap::new()),
            next_id: Mutex::new(0),
            event_queue: Mutex::new(VecDeque::new()),
        }
    }

    /// Generate a unique agent ID
    fn generate_id(&self) -> AgentId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = format!("agent-{}", *next_id);
        *next_id += 1;
        AgentId(id)
    }

    /// Register a regular agent
    pub fn register_agent(&self, mut agent: Box<dyn Agent>) -> Result<AgentId, String> {
        let agent_id = agent.id().clone();
        let mut agents = self.agents.write().map_err(|e| e.to_string())?;

        if agents.contains_key(&agent_id) {
            return Err(format!("Agent with ID {} already registered", agent_id.0));
        }

        agents.insert(agent_id.clone(), agent);
        Ok(agent_id)
    }

    /// Register a supervisor agent
    pub fn register_supervisor(&self, mut agent: Box<dyn Agent>) -> Result<AgentId, String> {
        let agent_id = agent.id().clone();
        let mut supervisors = self.supervisors.write().map_err(|e| e.to_string())?;

        if supervisors.contains_key(&agent_id) {
            return Err(format!("Supervisor with ID {} already registered", agent_id.0));
        }

        supervisors.insert(agent_id.clone(), agent);
        Ok(agent_id)
    }

    /// Get an agent by ID
    pub fn get_agent(&self, id: &AgentId) -> Option<Box<dyn Agent>> {
        // Check regular agents first
        let agents = self.agents.read().ok()?;
        if let Some(agent) = agents.get(id) {
            return Some(agent.clone());
        }

        // Check supervisors
        let supervisors = self.supervisors.read().ok()?;
        if let Some(agent) = supervisors.get(id) {
            return Some(agent.clone());
        }

        None
    }

    /// Unregister an agent
    pub fn unregister_agent(&self, id: &AgentId) -> Result<Option<Box<dyn Agent>>, String> {
        // Check regular agents
        {
            let mut agents = self.agents.write().map_err(|e| e.to_string())?;
            if let Some(agent) = agents.remove(id) {
                return Ok(Some(agent));
            }
        }

        // Check supervisors
        {
            let mut supervisors = self.supervisors.write().map_err(|e| e.to_string())?;
            if let Some(agent) = supervisors.remove(id) {
                return Ok(Some(agent));
            }
        }

        Ok(None)
    }

    /// List all agent IDs
    pub fn list_agent_ids(&self) -> Vec<AgentId> {
        let agents = self.agents.read().ok().unwrap_or_default();
        let supervisors = self.supervisors.read().ok().unwrap_or_default();

        let mut ids: Vec<AgentId> = agents.keys().cloned().collect();
        ids.extend(supervisors.keys().cloned());
        ids
    }

    /// Send a message to an agent
    pub fn send_message(&self, agent_id: &AgentId, message: AgentMessage) -> Result<(), String> {
        if let Some(mut agent) = self.get_agent(agent_id) {
            // In a real implementation, we'd need to get the agent's message sender
            // For now, we'll just return OK - the actual messaging would happen through
            // the agent's internal channels
            Ok(())
        } else {
            Err(format!("Agent {} not found", agent_id.0))
        }
    }

    /// Broadcast a message to all agents
    pub fn broadcast_message(&self, message: AgentMessage) -> Result<(), String> {
        let agent_ids = self.list_agent_ids();
        for agent_id in agent_ids {
            let _ = self.send_message(&agent_id, message.clone());
        }
        Ok(())
    }

    /// Start all registered agents
    pub async fn start_all_agents(&self) -> Result<(), String> {
        // Start supervisors first
        let supervisors = self.supervisors.read().map_err(|e| e.to_string())?;
        for agent in supervisors.values() {
            agent.start().await?;
        }

        // Then start regular agents
        let agents = self.agents.read().map_err(|e| e.to_string())?;
        for agent in agents.values() {
            agent.start().await?;
        }

        Ok(())
    }

    /// Stop all registered agents
    pub async fn stop_all_agents(&self) -> Result<(), String> {
        // Stop regular agents first
        let agents = self.agents.read().map_err(|e| e.to_string())?;
        for agent in agents.values() {
            agent.stop().await?;
        }

        // Then stop supervisors
        let supervisors = self.supervisors.read().map_err(|e| e.to_string())?;
        for agent in supervisors.values() {
            agent.stop().await?;
        }

        Ok(())
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_orchestrator_creation() {
        let orchestrator = AgentOrchestrator::new();
        assert_eq!(orchestrator.list_agent_ids().len(), 0);
    }

    #[tokio::test]
    async fn test_simple_agent_creation_and_lifecycle() {
        let config = AgentConfig {
            id: AgentId("test-agent".to_string()),
            name: "Test Agent".to_string(),
            agent_type: "simple".to_string(),
            capabilities: AgentCapabilities {
                task_types: vec!["test".to_string()],
                max_concurrent_tasks: 1,
                supported_events: vec![],
            },
            max_restarts: 3,
            restart_delay: Duration::from_secs(1),
        };

        let mut agent = SimpleAgent::new(config);
        assert_eq!(agent.state(), AgentState::Initialized);

        agent.initialize().await.unwrap();
        assert_eq!(agent.state(), AgentState::Running);

        agent.pause().await.unwrap();
        assert_eq!(agent.state(), AgentState::Paused);

        agent.resume().await.unwrap();
        assert_eq!(agent.state(), AgentState::Running);

        agent.stop().await.unwrap();
        assert_eq!(agent.state(), AgentState::Stopped);
    }
}