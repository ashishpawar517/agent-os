## Grilling Round 2: Component Interface Decisions

Based on user input, here are the decisions for the component interface in agent-os:

### 1. Component Lifecycle Contract
**Decision**: discover/start/stop
- **discover**: System ability to find what components are available (via registry)
- **start**: Initialize the component, allocate resources, prepare for use
  - For terminal: spawn PTY process, set up frontend connection
  - For browser: create webview window, load initial page
- **stop**: Clean up resources, shut down processes, dispose of state
  - For terminal: terminate PTY process, close connections
  - For browser: destroy webview window, cleanup resources

Components implement this lifecycle interface:
```typescript
interface ComponentLifecycle {
  discover(): Promise<ComponentMetadata[]>; // What's available
  start(config?: ComponentConfig): Promise<void>; // Initialize for use
  stop(): Promise<void>; // Cleanup and shutdown
}
```

### 2. Component Discovery and Instantiation
**Decision**: Registry/service locator pattern
- Central ComponentRegistry maintains available components
- Components register themselves with the registry at startup
- Agents lookup components by type/name from the registry
- Registry handles instantiation and lifecycle management

```typescript
class ComponentRegistry {
  private components: Map<string, ComponentFactory> = new Map();
  
  register(type: string, factory: ComponentFactory): void;
  unregister(type: string): void;
  getComponent(type: string): Promise<ComponentInstance>;
  listAvailable(): string[]; // Types of components available
}

// Usage by agent:
const registry = ComponentRegistry.getInstance();
const terminal = await registry.getComponent("terminal");
await terminal.start();
const result = await terminal.execute("ls -la");
await terminal.stop();
```

### 3. Component Invocation Mechanisms
**Decision**: Both paths supported
- **Direct calls**: Agents invoke components directly for internal operations
  - Synchronous/asynchronous methods on component instances
  - Used when agent needs immediate results or direct control
  - Example: `terminal.execute("git status")` for agent's internal use
  
- **Tool-calling path**: LLMs invoke components as tools via the provider abstraction
  - Component functionality exposed as tools to LLMs
  - Agent layer executes the tool call and returns results to LLM
  - Example: LLM decides to "search web for React hooks" → invokes browser tool

This dual approach provides flexibility:
- Agent internal logic uses direct calls for efficiency
- LLM reasoning and decision-making uses tool-calling for transparency

### 4. Component-Agent Communication
**Decision**: Callbacks/Promises + Events
- **Synchronous operations**: Return Promises for immediate results
  - `component.execute(command)` → Promise<Result>
  - `component.query(state)` → Promise<State>
  
- **Asynchronous streaming/data**: Event emitters for ongoing communication
  - Component emits events for data, state changes, errors
  - Agents subscribe to events they care about
  
```typescript
interface ComponentEvents {
  on(event: string, listener: Function): void;
  off(event: string, listener: Function): void;
}

// Terminal example:
terminal.on('data', (data) => {
  // Handle incoming terminal output
});
terminal.on('exit', (code) => {
  // Handle process termination
});

// Browser example:
browser.on('navigate', (url) => {
  // Handle navigation events
});
browser.on('load', () => {
  // Handle page load completion
});
```

### Component Interface Shape
Combining these decisions, a component interface looks like:

```typescript
interface ComponentDescriptor {
  type: string; // Unique identifier (e.g., "terminal", "browser")
  name: string; // Human-readable name
  description?: string; // What the component does
  version?: string;
  // Metadata for discovery and documentation
}

interface ComponentConfig {
  // Component-specific configuration
  // Terminal: { shell: "bash", cols: 80, rows: 24 }
  // Browser: { startupPage: "about:blank", securityPolicy: "strict" }
}

interface ComponentInstance extends ComponentLifecycle {
  readonly descriptor: ComponentDescriptor;
  
  // Component-specific methods (examples)
  // Terminal-specific:
  execute?(command: string): Promise<string>;
  resize?(cols: number, rows: number): void;
  write?(data: string): void;
  
  // Browser-specific:
  navigate?(url: string): Promise<void>;
  evaluate?(script: string): Promise<any>;
  goBack?(): Promise<void>;
  goForward?(): Promise<void>;
  
  // Event handling
  on(event: string, listener: Function): void;
  off(event: string, listener: Function): void;
  once(event: string, listener: Function): void;
}

interface ComponentFactory {
  create(config?: ComponentConfig): Promise<ComponentInstance>;
  getDescriptor(): ComponentDescriptor;
}
```

### Implementation Examples

**Terminal Component:**
```typescript
class TerminalComponent implements ComponentInstance {
  readonly descriptor = {
    type: "terminal",
    name: "Terminal",
    description: "Provides terminal emulation via PTY and xterm.js"
  };
  
  private ptyProcess: ChildProcess | null = null;
  private isStarted = false;
  
  async start(config?: TerminalConfig): Promise<void> {
    // Spawn shell process with PTY via Bun.spawn()
    // Set up communication bridge to frontend
    this.isStarted = true;
  }
  
  async stop(): Promise<void> {
    if (this.ptyProcess) {
      await this.ptyProcess.kill();
      this.ptyProcess = null;
    }
    this.isStarted = false;
  }
  
  async execute(command: string): Promise<string> {
    if (!this.isStarted) throw new Error("Terminal not started");
    // Execute command in PTY and return output as Promise
  }
  
  // Event handling for data output, resize, etc.
}
```

**Browser Component:**
```typescript
class BrowserComponent implements ComponentInstance {
  readonly descriptor = {
    type: "browser",
    name: "Web Browser",
    description: "Provides web browsing capabilities via Tauri webview"
  };
  
  private webview: WebviewWindow | null = null;
  private isStarted = false;
  
  async start(config?: BrowserConfig): Promise<void> {
    // Create Tauri webview window
    // Load initial page or about:blank
    this.isStarted = true;
  }
  
  async stop(): Promise<void> {
    if (this.webview) {
      await this.webview.close();
      this.webview = null;
    }
    this.isStarted = false;
  }
  
  async navigate(url: string): Promise<void> {
    if (!this.isStarted) throw new Error("Browser not started");
    // Navigate webview to URL
  }
  
  async evaluate(script: string): Promise<any> {
    // Execute script in webview context and return result
  }
  
  // Event handling for navigation, page load, etc.
}
```

### Agent Usage Patterns

**Direct Call Pattern (Agent Internal Logic):**
```typescript
class CodeAgent {
  private terminal: ComponentInstance;
  
  async initialize() {
    const registry = ComponentRegistry.getInstance();
    this.terminal = await registry.getComponent("terminal");
    await this.terminal.start();
  }
  
  async runBuildCommand(): Promise<string> {
    const result = await this.terminal.execute("npm run build");
    return result;
  }
  
  async cleanup() {
    await this.terminal.stop();
  }
}
```

**LLM Tool-Calling Pattern:**
```typescript
// In provider abstraction, when LLM requests tool use:
async handleToolCall(toolCall: ToolCall): Promise<ToolResult> {
  const { name, arguments } = toolCall;
  
  // Map tool name to component + method
  if (name === "terminal_execute") {
    const registry = ComponentRegistry.getInstance();
    const terminal = await registry.getComponent("terminal");
    await terminal.start();
    try {
      const output = await terminal.execute(arguments.command);
      return {
        toolCallId: toolCall.id,
        output: String(output)
      };
    } finally {
      await terminal.stop(); // Or keep alive for reuse
    }
  }
  // ... other component tools
}
```

### Benefits of This Approach

1. **Clear Separation of Concerns**:
   - Components encapsulate specific functionality
   - Agents orchestrate component usage
   - Provider abstraction handles LLM communication

2. **Flexibility in Invocation**:
   - Direct calls for performance-critical agent logic
   - Tool-calling for LLM-driven decision making

3. **Plug-and-Play Extensibility**:
   - New components can be added by implementing the interface
   - Auto-discovery via registry
   - No changes needed to core agent logic

4. **Proper Resource Management**:
   - Explicit lifecycle prevents resource leaks
   - Components manage their own complex initialization

5. **Rich Communication**:
   - Promises for immediate results
   - Events for streaming data and state changes

This component interface provides the foundation for a truly plug-and-play system where components like terminal and browser can be seamlessly integrated into the agent-os architecture while maintaining clean separation of concerns and flexible usage patterns.