# Ticket 4: Terminal Component Implementation

## Description
Implement the terminal component using Bun's built-in PTY support paired with xterm.js frontend for full terminal emulation capabilities including resize, raw mode, and proper input/output handling.

## Implementation Steps
1. TerminalComponent implementing ComponentInstance interface
2. Use Bun.spawn() with { terminal: { ... } } option for PTY support
3. Set up bidirectional communication between PTY and frontend via Tauri command/event system
4. Integrate xterm.js for terminal rendering in frontend
5. Implement resize handling (cols/rows changes forwarded to PTY)
6. Implement write() method for sending input to PTY
7. Handle data events from PTY to send output to frontend
8. Implement proper start()/stop() lifecycle (spawn/cleanup PTY process)
9. Add error handling and exit code reporting

## Acceptance Criteria
- Terminal component properly implements ComponentInstance interface
- Bun.spawn() with PTY option successfully creates pseudoterminal
- xterm.js renders terminal output correctly in frontend
- Bidirectional communication works: input → PTY → output → frontend
- Resize events are properly handled and forwarded to PTY
- Component can start/stop cleanly without resource leaks
- Basic shell commands (ls, pwd, echo, etc.) work correctly
- Supports features like colored output, cursor control, basic editing

## Blocking Edges
- Blocked by: Ticket 1 (Project Setup)
- Blocked by: Ticket 3 (Component Interface)

## Blocks
- Ticket 6 (Agent Orchestration Model) - agents need to be able to use terminal
- Ticket 7 (Permission System) - terminal permissions will be tested here
- Ticket 8 (UI/Session Model) - terminal will be rendered in tabs

## Related Spec Sections
- Implementation Decisions: 
  - "Terminal Component: Implemented using Bun.spawn() with built-in PTY support ({ terminal: { ... } }) paired with xterm.js frontend for rendering"
  - "Component Interface: Components implement discover/start/stop lifecycle..."
- User Story 9: "As a developer, I want the terminal component to provide real PTY-based terminal emulation..."
- User Story 5: "As a developer, I want sub-agents to be able to use components like terminal and browser through a well-defined interface..."