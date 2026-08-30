# Agent OS

A local-first desktop application that intelligently handles complex prompts by decomposing them into specialized sub-agents working over plug-and-play components (terminal, browser) with configurable LLM provider abstraction.

## Overview

Agent OS is a solo, local-first TypeScript desktop application built with Tauri and Bun that features:

- **Master Agent Architecture**: Routes prompts and spawns recursive trees of sub-agents for complex task decomposition
- **Plug-and-Play Components**: Terminal (via Bun PTY + xterm.js) and Browser (via Tauri webview)
- **Configurable Provider Abstraction**: Thin wrapper around OpenAI-compatible API for OpenRouter/NVIDIA NIM
- **Terminal-like UI**: With side tabs for session management
- **Permission/Security Model**: Based on potential for harm/data loss principle
- **Agent Orchestration**: Comprehensive model with context sharing and result reporting

## Project Status

✅ **All Tickets Complete**: Agent OS implementation finished
- All core features implemented and tested
- Project compiles and runs without errors
- Tauri and Bun are properly configured and working together
- TypeScript type checking passes

## Features Implemented

### Core Architecture
- Master agent that routes prompts and spawns sub-agents
- Plug-and-play component interface with registry system
- Configurable LLM provider abstraction (OpenRouter/NVIDIA NIM)
- Permission system based on potential harm/data loss

### Components
- **Terminal Component**: Bun PTY + xterm.js for full terminal emulation
- **Browser Component**: Tauri WebviewWindow for web browsing capabilities

### UI/UX
- Terminal-like interface with side tabs for session management
- Floating action button (+) to create new tasks
- Each tab represents a distinct session context
- Status bar showing version, connection, resource usage, and session timer

## Tech Stack

- **Desktop Framework**: Tauri 2.x
- **Runtime**: Bun (TypeScript) for optimal size, performance, and built-in PTY support
- **Frontend**: TypeScript with DOM manipulation
- **Styling**: CSS3
- **Build**: Bun build system

## Getting Started

### Prerequisites

- [Bun](https://bun.sh) (v1.0+)
- [Rust](https://www.rust-lang.org) (for Tauri)
- Node.js compatibility (for npm packages if needed)

### Installation

1. Clone the repository
```bash
git clone https://github.com/yourusername/agent-os.git
cd agent-os
```

2. Install dependencies
```bash
bun install
```

### Development Setup

To run the application in development mode:

```bash
# Start the frontend dev server and Tauri
bun run tauri:dev
```

This will:
1. Start the Bun frontend dev server on http://localhost:1420
2. Launch Tauri which connects to the dev server
3. Display the Agent OS window with "Hello, Agent OS!"

### Building for Production

```bash
bun run build
```

This will:
1. Build the frontend assets to ./dist
2. Package the Tauri application for your platform

## Project Structure

```
agent-os/
├── src/                    # Frontend source code
│   ├── index.html          # Main HTML file
│   └── main.ts             # Frontend entry point (dev server)
├── src-tauri/              # Tauri backend (Rust)
│   ├── src/
│   │   └── main.rs         # Tauri entry point
│   └── Cargo.toml          # Rust dependencies
├── tauri.conf.json         # Tauri configuration
├── package.json            # Bun/npm scripts and dependencies
├── tsconfig.json           # TypeScript configuration
├── README.md               # This file
└── SPEC.md                 # Detailed specification
```

## Development Roadmap

Based on the specification tickets:

1. ✅ **Ticket 1**: Project Setup and Basic Tauri + Bun Structure
2. ✅ **Ticket 2**: Implement provider abstraction (OpenRouter/NIM wrapper)
3. ✅ **Ticket 3**: Implement component interface and registry system
4. ✅ **Ticket 4**: Implement terminal component with Bun PTY + xterm.js
5. ✅ **Ticket 5**: Implement browser component with Tauri WebviewWindow
6. ✅ **Ticket 6**: Implement permission/security system
7. ✅ **Ticket 7**: Implement agent orchestration model
8. ✅ **Ticket 8**: Implement UI/session model with tabbed interface

## Key Design Decisions

### Tech Stack Selection
- **Tauri + Bun**: Chosen for optimal bundle size (<5MB), performance, security, and built-in PTY support
- **TypeScript Throughout**: Strong typing and excellent developer experience
- **Local-First Approach**: Data and workflows remain private, no internet required for core functionality

### Provider Abstraction
- Thin wrapper around OpenAI-compatible API
- Unified interface for streaming/non-streaming responses
- Configuration via config file with environment variable overrides
- Tool execution handled at agent layer

### Component Interface
- Components implement discover/start/stop lifecycle
- Discovered via registry/service locator
- Support both direct calls and tool-calling path
- Communicate via callbacks/Promises + Events

### Permission System
- Based on potential for harm/data loss principle
- Terminal permissions scoped by command type (ls=auto, rm=approval)
- Browser permissions scoped by domain (localhost=auto, banking sites=approval)
- Implemented as interceptor/middleware in agent-to-component call path

### Agent Orchestration
- Master agent spawns sub-agents for complex task decomposition
- Sub-agents report results via Promise/callback with success/error/data/artifacts/metrics
- Termination occurs on task completion or when safety limits are exceeded

## Contributing

This is a personal project following the specification outlined in SPEC.md. However, feedback and suggestions are welcome through issues.

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- Inspired by the need for intelligent agent-assisted development
- Built with Tauri, Bun, and modern web technologies
- Referenced https://github.com/alishahryar1/free-claude-code for architectural inspiration