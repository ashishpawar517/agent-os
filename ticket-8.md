# Ticket 8: UI/Session Model Implementation

## Description
Implement the terminal-like UI with side tabs for session management, including tab-based navigation, session-to-tab mapping, component rendering within tabs, and new task creation via floating action button.

## Implementation Steps
1. Create basic UI structure with sidebar, tab bar, tab content area, status bar, and floating action button
2. Implement tab switching logic (show/hide tab panels)
3. Implement session-to-tab mapping:
   - Agent sessions: display master/sub-agent output with metadata
   - Terminal sessions: render terminal component output/input
   - Browser sessions: render browser component iframe/content
4. Implement tab close functionality with cleanup
5. Implement new session creation (+ button in sidebar and floating action button)
6. Implement status bar showing version, branch, connection status, resource usage, session timer
7. Implement component rendering within tabs:
   - Agent tabs: show agent output, metrics, suggested next steps
   - Terminal tabs: integrate xterm.js terminal emulator
   - Browser tabs: integrate Tauri webview content
8. Implement tab persistence (state maintained when switching tabs)
9. Add visual indicators for active tabs, tab types, and session status
10. Implement proper cleanup when tabs are closed

## Acceptance Criteria
- UI renders with sidebar, tab bar, content area, status bar, and floating action button
- Tab switching works correctly (hiding/showing appropriate panels)
- Session-to-tab mapping functions properly:
  - Agent tabs display agent output/logs
  - Terminal tabs show functional terminal emulator
  - Browser tabs show navigable web content
- New session creation works via both sidebar button and floating action button
- Tab close functionality properly cleans up resources
- Status bar displays correct information (version, branch, etc.)
- Component rendering within tabs works correctly:
  - Agent output formatting with metadata
  - Terminal integration with PTY backend
  - Browser integration with webview
- Tab persistence maintains state when switching between tabs
- Visual feedback indicates active tabs and tab types
- Proper cleanup occurs when tabs are closed (no resource leaks)

## Blocking Edges
- Blocked by: Ticket 1 (Project Setup)
- Blocked by: Ticket 2 (Provider Abstraction) - for agent functionality in tabs
- Blocked by: Ticket 3 (Component Interface) - for component access in tabs
- Blocked by: Ticket 4 (Terminal Component) - to render in terminal tabs
- Blocked by: Ticket 5 (Browser Component) - to render in browser tabs
- Blocked by: Ticket 6 (Permission System) - may need to display permission prompts in tabs
- Blocked by: Ticket 7 (Agent Orchestration Model) - to display agent sessions in tabs

## Blocks
- This ticket represents the final core UI implementation; subsequent tickets would be for specific agent types or advanced features

## Related Spec Sections
- Implementation Decisions: 
  - "UI/Session Model: Terminal-like interface with side tabs for navigation; each tab represents a session context (agent work, terminal usage, browsing); floating action button (+) creates new task sessions; tab content persists when switching; status bar shows version, branch, connection, resource usage, and session timer"
- User Story 2: "As a developer, I want Agent OS to provide a terminal-like interface with side tabs, so that I can easily switch between different sessions and component views."
- User Story 11: "As a developer, I want to be able to start new tasks easily through a floating action button, so that I can quickly initiate work without navigating menus."
- User Story 12: "As a developer, I want each tab in the UI to represent a distinct session context, so that I can isolate different lines of work and maintain clean state separation."
- User Story 13: "As a developer, I want agent output to be clearly displayed within tabs with metadata and suggested next steps, so that I can understand what the agent accomplished and what to do next."
- Further Notes: "UI/Session model prototype demonstrates the terminal-like interface with side tabs approach"

## Notes
This ticket implements the UI/Session model prototype demonstrated in `prototype-ui-session.html`. The actual implementation will use the Tauri + Bun stack with appropriate frontend framework (could be vanilla TS/HTML/CSS or a lightweight framework) to realize the interface shown in the prototype.