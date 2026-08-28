## Grilling Round 1: Agent Orchestration Model Decision

Based on all the previous decisions we've made:

**Architecture Decisions So Far:**
- Desktop framework: Tauri + Bun stack
- Provider abstraction: Thin wrapper around OpenAI-compatible API
- Component interface: discover/start/stop lifecycle, registry-based discovery, both direct calls and tool-calling paths
- Permission/model: Harm/data loss principle, command-level (terminal) and domain-level (browser) scoping
- Components: Terminal (Bun PTY + xterm.js) and Browser (Tauri webview)

**Agent Orchestration Requirements from Destination:**
- Master agent routes prompts
- Master agent spawns recursive tree of sub-agents
- Working over plug-and-play components
- Terminal-like UI with side tabs

**Key Questions for Orchestration Model:**
1. What triggers a sub-agent spawn?
2. What is the spawn/result protocol?
3. How is context shared back to the parent?
4. What terminates a sub-agent?
5. How does the recursive tree structure work?
6. How does this integrate with our component and permission systems?

Let me consider the factors and work through specific decisions for the orchestration model.