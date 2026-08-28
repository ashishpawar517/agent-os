## Grilling Round 1: Component Interface Decision

Based on the architecture decisions so far:
- Tauri desktop framework with Bun TS runtime
- Thin wrapper provider abstraction around OpenAI-compatible API
- Plug-and-play components: terminal (xterm.js + Bun PTY) and browser (Tauri webview)

Key questions for component interface:
1. What contract does a component implement?
2. How does an agent invoke a component?
3. Does invocation ride the LLM tool-calling path or a direct call?

Let me consider the factors:

**Component Responsibilities:**
- Encapsulate specific functionality (terminal, browser, etc.)
- Manage their own lifecycle
- Provide a clean API for agents to use
- Potentially expose themselves as tools to LLMs

**Agent-Component Interaction:**
- Agents need to discover available components
- Agents need to instantiate/start components
- Agents need to invoke component functionality
- Components should be able to communicate back to agents (events, data)

**Invocation Paths:**
- Direct call: Agent directly calls component methods (synchronous/asynchronous)
- Tool-calling path: LLM decides to invoke a component as a tool, agent executes it, returns result to LLM

Let me work through specific decisions for the component interface.