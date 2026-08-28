#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod provider;
mod component;
mod terminal;
mod browser;
mod security;
mod orchestration;
mod session;

#[tokio::main]
async fn main() {
    // Initialize provider manager
    let provider_manager = provider::ProviderManager::new();

    // Test getting providers
    println!("Available providers: {:?}", provider_manager.list_providers());

    if let Some(openrouter_provider) = provider_manager.get_provider("openrouter") {
        println!("Found OpenRouter provider: {}", openrouter_provider.name());
    }

    if let Some(nim_provider) = provider_manager.get_provider("nim") {
        println!("Found NIM provider: {}", nim_provider.name());
    }

    // Initialize component registry
    let component_registry = component::ComponentRegistry::new();
    println!("Component registry initialized with {} components", component_registry.len());

    // Initialize security manager
    let security_manager = security::SecurityManager::new();
    println!("Security manager initialized with {} policies", security_manager.list_policies().len());

    // Initialize agent orchestrator
    let orchestrator = orchestration::AgentOrchestrator::new();
    println!("Agent orchestrator initialized");

    // Initialize session manager
    let session_manager = session::SessionManager::new();
    println!("Session manager initialized");

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let terminal_manager = terminal::TerminalManager::new(app_handle);
            let browser_manager = browser::BrowserManager::new();
            let security_manager = security::SecurityManager::new();
            let orchestrator = orchestration::AgentOrchestrator::new();
            let session_manager = session::SessionManager::new();

            app.manage(terminal_manager);
            app.manage(browser_manager);
            app.manage(security_manager);
            app.manage(orchestrator);
            app.manage(session_manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Terminal commands
            create_terminal_session,
            terminal_send_input,
            terminal_resize,
            // Browser commands
            create_browser,
            browser_navigate,
            browser_reload,
            browser_back,
            browser_forward,
            browser_close,
            // Security commands
            add_security_policy,
            get_security_policy,
            remove_security_policy,
            set_default_policy,
            get_default_policy,
            create_security_context,
            get_security_context,
            update_context_overrides,
            is_permission_allowed,
            list_security_policies,
            list_security_contexts,
            // Orchestration commands
            register_agent,
            register_supervisor,
            unregister_agent,
            get_agent,
            list_agent_ids,
            send_message_to_agent,
            broadcast_message,
            start_all_agents,
            stop_all_agents,
            // Session commands
            create_session,
            get_session,
            destroy_session,
            touch_session,
            create_tab,
            get_tab,
            set_active_tab,
            get_active_tab,
            close_tab,
            list_sessions,
            list_session_tabs,
            cleanup_expired_sessions,
            get_session_config,
            set_session_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Session commands

/// Create a new user session
#[tauri::command]
fn create_session(
    session_manager: tauri::State<'_, session::SessionManager>,
    user_id: Option<String>,
) -> Result<String, String> {
    session_manager.create_session(user_id)
}

/// Get a session by ID
#[tauri::command]
fn get_session(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
) -> Result<Option<session::UserSession>, String> {
    Ok(session_manager.get_session(&session_id))
}

/// Destroy a session and all its tabs
#[tauri::command]
fn destroy_session(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
) -> Result<(), String> {
    session_manager.destroy_session(&session_id)
}

/// Update session last accessed time
#[tauri::command]
fn touch_session(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
) -> Result<(), String> {
    session_manager.touch_session(&session_id)
}

/// Create a new tab in a session
#[tauri::command]
fn create_tab(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
    title: String,
    component_type: String,
    component_state: Option<serde_json::Value>,
    settings: Option<serde_json::Value>,
) -> Result<String, String> {
    // Convert settings from JSON value to HashMap
    let settings_map = match settings {
        Some(serde_json::Value::Object(map)) => {
            let mut hashmap = std::collections::HashMap::new();
            for (key, value) in map {
                hashmap.insert(key, value.as_str().unwrap_or("").to_string());
            }
            hashmap
        }
        _ => std::collections::HashMap::new(),
    };

    session_manager.create_tab(
        &session_id,
        title,
        component_type,
        component_state,
        settings_map,
    )
}

/// Get a tab by ID
#[tauri::command]
fn get_tab(
    session_manager: tauri::State<'_, session::SessionManager>,
    tab_id: String,
) -> Result<Option<session::UITab>, String> {
    Ok(session_manager.get_tab(&tab_id))
}

/// Set a tab as active
#[tauri::command]
fn set_active_tab(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
    tab_id: String,
) -> Result<(), String> {
    session_manager.set_active_tab(&session_id, &tab_id)
}

/// Get the active tab for a session
#[tauri::command]
fn get_active_tab(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
) -> Result<Option<session::UITab>, String> {
    Ok(session_manager.get_active_tab(&session_id))
}

/// Close a tab
#[tauri::command]
fn close_tab(
    session_manager: tauri::State<'_, session::SessionManager>,
    tab_id: String,
) -> Result<(), String> {
    session_manager.close_tab(&tab_id)
}

/// List all sessions
#[tauri::command]
fn list_sessions(
    session_manager: tauri::State<'_, session::SessionManager>,
) -> Result<Vec<String>, String> {
    Ok(session_manager.list_sessions())
}

/// List all tabs for a session
#[tauri::command]
fn list_session_tabs(
    session_manager: tauri::State<'_, session::SessionManager>,
    session_id: String,
) -> Result<Vec<String>, String> {
    Ok(session_manager.list_session_tabs(&session_id))
}

/// Clean up expired sessions
#[tauri::command]
fn cleanup_expired_sessions(
    session_manager: tauri::State<'_, session::SessionManager>,
) -> Result<u32, String> {
    Ok(session_manager.cleanup_expired_sessions() as u32)
}

/// Get session configuration
#[tauri::command]
fn get_session_config(
    session_manager: tauri::State<'_, session::SessionManager>,
) -> Result<session::SessionConfig, String> {
    Ok(session_manager.get_config().clone())
}

/// Set session configuration
#[tauri::command]
fn set_session_config(
    session_manager: tauri::State<'_, session::SessionManager>,
    timeout_minutes: u64,
    max_tabs_per_session: usize,
    allow_tab_restore: bool,
) -> Result<(), String> {
    let config = session::SessionConfig {
        timeout_minutes,
        max_tabs_per_session,
        allow_tab_restore,
    };
    session_manager.set_config(config);
    Ok(())
}

// Security commands

/// Add a new security policy
#[tauri::command]
fn add_security_policy(
    security_manager: tauri::State<'_, security::SecurityManager>,
    name: String,
    description: Option<String>,
    permissions: Vec<String>, // JSON string representation of permissions
) -> Result<(), String> {
    // In a real implementation, we'd parse the permissions from JSON
    // For now, we'll create a simple policy with basic permissions
    let policy = security::SecurityPolicy {
        name,
        description,
        permissions: vec![
            security::Permission::FileRead("./".to_string()),
            security::Permission::FileWrite("./".to_string()),
        ],
        is_default: false,
    };

    security_manager.add_policy(policy)
}

/// Get a security policy by name
#[tauri::command]
fn get_security_policy(
    security_manager: tauri::State<'_, security::SecurityManager>,
    name: String,
) -> Result<Option<security::SecurityPolicy>, String> {
    Ok(security_manager.get_policy(&name))
}

/// Remove a security policy
#[tauri::command]
fn remove_security_policy(
    security_manager: tauri::State<'_, security::SecurityManager>,
    name: String,
) -> Result<Option<security::SecurityPolicy>, String> {
    security_manager.remove_policy(&name)
}

/// Set the default policy
#[tauri::command]
fn set_default_policy(
    security_manager: tauri::State<'_, security::SecurityManager>,
    name: String,
) -> Result<(), String> {
    security_manager.set_default_policy(&name)
}

/// Get the default policy
#[tauri::command]
fn get_default_policy(
    security_manager: tauri::State<'_, security::SecurityManager>,
) -> Result<Option<security::SecurityPolicy>, String> {
    Ok(security_manager.get_default_policy())
}

/// Create a new security context
#[tauri::command]
fn create_security_context(
    security_manager: tauri::State<'_, security::SecurityManager>,
    id: String,
    policy_names: Vec<String>,
) -> Result<security::SecurityContext, String> {
    security_manager.create_context(&id, policy_names)
}

/// Get a security context by ID
#[tauri::command]
fn get_security_context(
    security_manager: tauri::State<'_, security::SecurityManager>,
    id: String,
) -> Result<Option<security::SecurityContext>, String> {
    Ok(security_manager.get_context(&id))
}

/// Update permission overrides for a context
#[tauri::command]
fn update_context_overrides(
    security_manager: tauri::State<'_, security::SecurityManager>,
    id: String,
    overrides: String, // JSON string representation of overrides
) -> Result<(), String> {
    // In a real implementation, we'd parse the overrides from JSON
    // For now, we'll just return OK
    Ok(())
}

/// Check if a permission is allowed in a given context
#[tauri::command]
fn is_permission_allowed(
    security_manager: tauri::State<'_, security::SecurityManager>,
    context_id: String,
    permission: String, // JSON string representation of permission
) -> Result<bool, String> {
    // In a real implementation, we'd parse the permission from JSON
    // For now, we'll check a simple permission
    let perm = security::Permission::FileRead("./".to_string());
    Ok(security_manager.is_allowed(&context_id, &perm))
}

/// List all policy names
#[tauri::command]
fn list_security_policies(
    security_manager: tauri::State<'_, security::SecurityManager>,
) -> Result<Vec<String>, String> {
    Ok(security_manager.list_policies())
}

/// List all context IDs
#[tauri::command]
fn list_security_contexts(
    security_manager: tauri::State<'_, security::SecurityManager>,
) -> Result<Vec<String>, String> {
    Ok(security_manager.list_contexts())
}

// Orchestration commands

/// Register a regular agent
#[tauri::command]
fn register_agent(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
    agent_type: String,
    name: String,
    max_restarts: usize,
) -> Result<String, String> {
    let agent_id = orchestrator.generate_id();

    // Create a simple agent config
    let config = orchestration::AgentConfig {
        id: agent_id.clone(),
        name,
        agent_type,
        capabilities: orchestration::AgentCapabilities {
            task_types: vec!["task".to_string()],
            max_concurrent_tasks: 1,
            supported_events: vec![],
        },
        max_restarts,
        restart_delay: std::time::Duration::from_secs(1),
    };

    let agent = Box::new(orchestration::SimpleAgent::new(config));
    orchestrator.register_agent(agent)?;

    Ok(agent_id.0)
}

/// Register a supervisor agent
#[tauri::command]
fn register_supervisor(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
    name: String,
    max_restarts: usize,
) -> Result<String, String> {
    let agent_id = orchestrator.generate_id();

    // Create a supervisor agent config
    let config = orchestration::AgentConfig {
        id: agent_id.clone(),
        name,
        agent_type: "supervisor".to_string(),
        capabilities: orchestration::AgentCapabilities {
            task_types: vec!["supervise".to_string()],
            max_concurrent_tasks: 10,
            supported_events: vec![],
        },
        max_restarts,
        restart_delay: std::time::Duration::from_secs(1),
    };

    let agent = Box::new(orchestration::SupervisorAgent::new(config));
    orchestrator.register_supervisor(agent)?;

    Ok(agent_id.0)
}

/// Unregister an agent
#[tauri::command]
fn unregister_agent(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
    agent_id: String,
) -> Result<bool, String> {
    let id = orchestration::AgentId(agent_id);
    match orchestrator.unregister_agent(&id) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Get an agent by ID
#[tauri::command]
fn get_agent(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
    agent_id: String,
) -> Result<bool, String> {
    let id = orchestration::AgentId(agent_id);
    Ok(orchestrator.get_agent(&id).is_some())
}

/// List all agent IDs
#[tauri::command]
fn list_agent_ids(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
) -> Result<Vec<String>, String> {
    let ids = orchestrator.list_agent_ids();
    Ok(ids.iter().map(|id| id.0.clone()).collect())
}

/// Send a message to an agent
#[tauri::command]
fn send_message_to_agent(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
    agent_id: String,
    message_type: String,
) -> Result<bool, String> {
    let id = orchestration::AgentId(agent_id);
    // In a real implementation, we'd create a proper message based on message_type
    let _ = orchestrator.send_message(&id, orchestration::AgentMessage::Ping);
    Ok(true)
}

/// Broadcast a message to all agents
#[tauri::command]
fn broadcast_message(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
    message_type: String,
) -> Result<bool, String> {
    // In a real implementation, we'd create a proper message based on message_type
    let _ = orchestrator.broadcast_message(orchestration::AgentMessage::Ping);
    Ok(true)
}

/// Start all registered agents
#[tauri::command]
fn start_all_agents(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
) -> Result<bool, String> {
    // In a real implementation, we'd actually start the agents
    // For now, we'll just return OK
    Ok(true)
}

/// Stop all registered agents
#[tauri::command]
fn stop_all_agents(
    orchestrator: tauri::State<'_, orchestration::AgentOrchestrator>,
) -> Result<bool, String> {
    // In a real implementation, we'd actually stop the agents
    // For now, we'll just return OK
    Ok(true)
}

// Terminal commands (same as before)

/// Create a new terminal session
#[tauri::command]
fn create_terminal_session(
    terminal_manager: tauri::State<'_, terminal::TerminalManager>,
    command: String,
    cwd: Option<String>,
) -> Result<String, String> {
    // Parse the command into a Command struct
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or("No command provided")?;
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();

    let mut cmd = terminal::Command::new(program);
    cmd.args(&args);

    terminal_manager.create_session(cmd, cwd)
}

/// Send input to a terminal session
#[tauri::command]
fn terminal_send_input(
    terminal_manager: tauri::State<'_, terminal::TerminalManager>,
    session_id: String,
    input: String,
) -> Result<(), String> {
    if let Some(session) = terminal_manager.get_session(&session_id) {
        session.send_input(input)
    } else {
        Err(format!("Session not found: {}", session_id))
    }
}

/// Resize a terminal session
#[tauri::command]
fn terminal_resize(
    terminal_manager: tauri::State<'_, terminal::TerminalManager>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    if let Some(session) = terminal_manager.get_session(&session_id) {
        session.resize(rows, cols)
    } else {
        Err(format!("Session not found: {}", session_id))
    }
}

// Browser commands

/// Create a new browser window/tab
#[tauri::command]
fn create_browser(
    browser_manager: tauri::State<'_, browser::BrowserManager>,
    app_handle: tauri::AppHandle,
    url: String,
    label: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    resizable: bool,
    fullscreen: bool,
    decorations: bool,
    always_on_top: bool,
) -> Result<String, String> {
    let config = browser::BrowserConfig {
        url,
        label,
        width,
        height,
        resizable,
        fullscreen,
        decorations,
        always_on_top,
    };

    browser_manager.create_browser(config, &app_handle)
}

/// Navigate to a URL in a browser tab
#[tauri::command]
fn browser_navigate(
    browser_manager: tauri::State<'_, browser::BrowserManager>,
    id: String,
    url: String,
) -> Result<(), String> {
    browser_manager.navigate(&id, &url)
}

/// Reload the current page in a browser tab
#[tauri::command]
fn browser_reload(
    browser_manager: tauri::State<'_, browser::BrowserManager>,
    id: String,
) -> Result<(), String> {
    browser_manager.reload(&id)
}

/// Go back in history in a browser tab
#[tauri::command]
fn browser_back(
    browser_manager: tauri::State<'_, browser::BrowserManager>,
    id: String,
) -> Result<(), String> {
    browser_manager.back(&id)
}

/// Go forward in history in a browser tab
#[tauri::command]
fn browser_forward(
    browser_manager: tauri::State<'_, browser::BrowserManager>,
    id: String,
) -> Result<(), String> {
    browser_manager.forward(&id)
}

/// Close a browser tab/window
#[tauri::command]
fn browser_close(
    browser_manager: tauri::State<'_, browser::BrowserManager>,
    id: String,
) -> Result<(), String> {
    browser_manager.close(&id)
}

// Terminal commands

/// Create a new terminal session
#[tauri::command]
fn create_terminal_session(
    terminal_manager: tauri::State<'_, terminal::TerminalManager>,
    command: String,
    cwd: Option<String>,
) -> Result<String, String> {
    // Parse the command into a Command struct
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or("No command provided")?;
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();

    let mut cmd = Command::new(program);
    cmd.args(&args);

    terminal_manager.create_session(cmd, cwd)
}

/// Send input to a terminal session
#[tauri::command]
fn terminal_send_input(
    terminal_manager: tauri::State<'_, terminal::TerminalManager>,
    session_id: String,
    input: String,
) -> Result<(), String> {
    if let Some(session) = terminal_manager.get_session(&session_id) {
        session.send_input(input)
    } else {
        Err(format!("Session not found: {}", session_id))
    }
}

/// Resize a terminal session
#[tauri::command]
fn terminal_resize(
    terminal_manager: tauri::State<'_, terminal::TerminalManager>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    if let Some(session) = terminal_manager.get_session(&session_id) {
        session.resize(rows, cols)
    } else {
        Err(format!("Session not found: {}", session_id))
    }
}