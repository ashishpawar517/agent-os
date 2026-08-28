# Agent OS Specification

## Problem Statement
Developers need a solo, local-first desktop application that can intelligently handle complex prompts by decomposing them into specialized sub-agents working over plug-and-play components (terminal, browser) with configurable LLM provider abstraction (OpenRouter, NVIDIA NIM), all within a terminal-like UI with side tabs for session management.

## Solution
Agent OS is a local-first TypeScript desktop application built with Tauri and Bun that features:
- A master agent that routes prompts and spawns recursive trees of sub-agents for complex task decomposition
- Plug-and-play components (terminal via Bun PTY + xterm.js, browser via Tauri webview)
- Configurable provider abstraction (thin wrapper around OpenAI-compatible API for OpenRouter/NVIDIA NIM)
- Terminal-like UI with side tabs for session management
- Permission/security model based on potential for harm/data loss
- Comprehensive agent orchestration model with context sharing and result reporting

## User Stories
1. As a developer, I want to give a single prompt to Agent OS and have it automatically decompose complex tasks into specialized sub-agents, so that I don't have to manually break down my work.
2. As a developer, I want Agent OS to provide a terminal-like interface with side tabs, so that I can easily switch between different sessions and component views.
3. As a developer, I want to be able to plug in different components (terminal, browser, etc.) into Agent OS, so that I can extend its functionality as needed.
4. As a developer, I want to configure Agent OS to use either OpenRouter or NVIDIA NIM as my LLM provider, so that I have flexibility in model choice and cost/performance trade-offs.
5. As a developer, I want sub-agents to be able to use components like terminal and browser through a well-defined interface, so that they can perform file operations, web research, and command execution.
6. As a developer, I want Agent OS to have a permission system that automatically allows safe operations but requests approval for potentially harmful actions, so that I can trust the agent to work autonomously while maintaining security.
7. As a developer, I want sub-agent execution to be time and resource-bounded, so that runaway agents don't consume excessive system resources.
8. As a developer, I want sub-agents to report their results back to the parent agent with data, metrics, and artifacts, so that I can track progress and build upon previous work.
9. As a developer, I want the terminal component to provide real PTY-based terminal emulation, so that I can run shells, editors, and command-line tools with full compatibility.
10. As a developer, I want the browser component to provide web browsing capabilities, so that agents can research information, interact with web APIs, and test web applications.
11. As a developer, I want to be able to start new tasks easily through a floating action button, so that I can quickly initiate work without navigating menus.
12. As a developer, I want each tab in the UI to represent a distinct session context, so that I can isolate different lines of work and maintain clean state separation.
13. As a developer, I want agent output to be clearly displayed within tabs with metadata and suggested next steps, so that I can understand what the agent accomplished and what to do next.
14. As a developer, I want the system to be local-first, so that my data and workflows remain private and don't require internet connectivity for core functionality.
15. As a developer, I want Agent OS to be built with TypeScript throughout, so that I get strong typing and excellent developer experience.

## Implementation Decisions
- **Tech Stack**: Tauri (desktop framework) + Bun (TypeScript runtime) selected for optimal size, performance, security, and built-in PTY support
- **Provider Abstraction**: Thin wrapper around OpenAI-compatible API; configuration via config file with environment variable overrides; unified interface for streaming/non-streaming responses; tool execution handled at agent layer
- **Component Interface**: Components implement discover/start/stop lifecycle; discovered via registry/service locator; support both direct calls (for agent internal use) and tool-calling path (for LLM-driven usage); communicate via callbacks/Promises + Events
- **Permission System**: Based on potential for harm/data loss principle; terminal permissions scoped by command type (e.g., ls=auto, rm=approval); browser permissions scoped by domain (e.g., localhost=auto, banking sites=approval); implemented as interceptor/middleware in agent-to-component call path
- **Agent Orchestration**: Master agent spawns sub-agents for complex task decomposition; spawn data includes task description + context; sub-agents report results via Promise/callback with success/error/data/artifacts/metrics; termination occurs on task completion or when safety limits (time, memory, iterations) are exceeded
- **Terminal Component**: Implemented using Bun.spawn() with built-in PTY support ({ terminal: { ... } }) paired with xterm.js frontend for rendering; supports resize, raw mode, and full terminal feature set
- **Browser Component**: Implemented using Tauri's WebviewWindow API; supports navigation, JavaScript execution, and event communication via Tauri's command/event system
- **UI/Session Model**: Terminal-like interface with side tabs for navigation; each tab represents a session context (agent work, terminal usage, browsing); floating action button (+) creates new task sessions; tab content persists when switching; status bar shows version, branch, connection, resource usage, and session timer
- **Communication**: Agent-to-component communication occurs through Tauri's command/event system; components register with central registry; permission checks happen via interceptor in the call path

## Testing Decisions
- **Unit Testing**: Test individual components (terminal, browser, provider) in isolation using Bun's built-in test runner
- **Integration Testing**: Test agent orchestration flows (spawning, result reporting, context sharing)
- **Permission Testing**: Verify that safe operations auto-approve and harmful operations trigger approval requests
- **UI Testing**: Test tab switching, session persistence, and component rendering within tabs
- **End-to-End Testing**: Test complete workflows like "research a topic and summarize findings" or "create a file and modify it"
- **Testing Philosophy**: Focus on external behavior and contracts rather than implementation details; mock external dependencies where appropriate; test error conditions and edge cases

## Out of Scope
- Multi-user, team, or cloud/self-hosted deployment (this effort is solo + local-first)
- Any component beyond terminal + browser/chromium in the first specification
- Advanced features like plugin marketplaces, remote agent synchronization, or complex workflow orchestration engines
- Built-in code editor or filesystem components (planned for future extensions)
- Integration with external issue trackers or project management tools beyond basic web access
- Machine learning model training or fine-tuning capabilities (focused on inference via providers)

## Further Notes
This specification captures all decisions made during the wayfinder process:
- Research confirmed OpenRouter and NVIDIA NIM both support function/tool-calling via OpenAI-compatible API
- Research confirmed Tauri provides sufficient webview embedding via system webviews
- Research confirmed Bun.spawn() with built-in PTY support is ideal for terminal emulation
- All grilling issues have been resolved with specific architectural decisions
- UI/Session model prototype demonstrates the terminal-like interface with side tabs approach

The spec names the core parts (master agent, sub-agent, provider, component, session, prompt) and their contracts well enough to start building the MVP. Implementation should begin with setting up the Tauri + Bun project structure, implementing the provider abstraction, creating the core component interfaces, and building the basic UI shell.