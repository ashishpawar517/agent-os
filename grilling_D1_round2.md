## Grilling Round 2: Agent Orchestration Model Decisions

Based on user input and the architecture decisions made so far, here are the decisions for the agent orchestration model in agent-os:

### 1. Sub-Agent Spawn Trigger
**Decision**: Complex task decomposition
- Master agent spawns sub-agents when faced with a task that is too complex to handle directly
- Complexity factors: multiple steps, need for specialized expertise, potential for parallelization, or tasks requiring isolation
- Examples:
  - "Research the latest React hooks" → spawn research sub-agent
  - "Refactor this entire codebase" → spawn multiple refactoring sub-agents
  - "Build a authentication system" → spawn specialized auth sub-agent
- Simple tasks handled directly by master agent without spawning

### 2. Spawn Data Interface
**Decision**: Task description + context
When spawning a sub-agent, the master agent provides:
```typescript
interface SpawnOptions {
  taskDescription: string;          // Clear description of what to accomplish
  context: any;                     // Relevant context from parent agent
  priority?: 'low' | 'normal' | 'high'; // Execution priority
  timeoutMs?: number;               // Maximum execution time
  maxRetries?: number;              // Retry attempts on failure
  // Resource limits
  maxMemoryMb?: number;
  maxIterations?: number;
  // Component access restrictions
  allowedComponents?: string[];     // Which components this sub-agent can use
  providerOverride?: ProviderConfig; // Optional different provider/model
}
```

The context includes relevant information from the parent:
- Current workspace/state
- Relevant files or data
- Previous attempts or findings
- Constraints and requirements

### 3. Sub-Agent Result Reporting
**Decision**: Promise/callback with result data
Sub-agents return a Promise that resolves with a structured result:
```typescript
interface SubAgentResult {
  success: boolean;                 // Whether task completed successfully
  data?: any;                       // Result data (if successful)
  error?: string | Error;           // Error information (if failed)
  artifacts?: Array<{               // Any files or outputs produced
    type: string;                   // e.g., "file", "image", "log"
    path?: string;                  // File path if applicable
    content?: string | Buffer;      // Content if small enough
  }>;
  metrics?: {                       // Performance metrics
    durationMs: number;
    iterations: number;
    tokensUsed?: number;
    componentsUsed: Record<string, number>; // Usage count by component type
  };
  // For chaining: suggested next steps
  suggestsNextSteps?: Array<{
    description: string;
    reasoning: string;
  }>;
}
```

Usage pattern:
```typescript
// Master agent spawning a sub-agent
const subAgent = await spawnSubAgent({
  taskDescription: "Research best practices for React state management",
  context: {
    currentProject: projectInfo,
    techStack: ["React", "TypeScript", "Bun"],
    goal: "improve application state handling"
  },
  timeoutMs: 30000, // 30 second limit
  allowedComponents: ["browser"] // Only need research capability
});

const result = await subAgent.execute();

if (result.success) {
  // Process the research findings
  console.log(`Research complete: ${result.data.summary}`);
  // May suggest next steps based on result.suggestsNextSteps
} else {
  // Handle failure
  console.error(`Research failed: ${result.error}`);
  // May retry with different approach or escalate to human
}
```

### 4. Sub-Agent Termination Criteria
**Decision**: Combination of factors
Sub-agents terminate when:
- **Primary**: Task completion (success or failure)
- **Secondary**: Safety limits exceeded:
  - Timeout reached (configurable max execution time)
  - Memory usage exceeded (configurable limit)
  - Iteration/call limits exceeded (to prevent infinite loops)
  - Component usage limits exceeded
- **Tertiary**: Parent-directed termination (parent can cancel sub-agent)

This provides robustness against runaway sub-agents while ensuring legitimate work can complete.

### 5. Agent Spawning Mechanism
The master agent uses a controlled spawning mechanism:
```typescript
class MasterAgent {
  private activeSubAgents: Set<SubAgent> = new Set();
  
  async spawnSubAgent(options: SpawnOptions): Promise<SubAgent> {
    // Validate options
    // Apply defaults for missing values
    // Check system resources (don't spawn too many simultaneously)
    
    const subAgent = new SubAgent({
      id: generateUniqueId(),
      master: this,
      options: options
    });
    
    this.activeSubAgents.add(subAgent);
    
    // Clean up when sub-agent finishes
    subAgent.on('finish', () => {
      this.activeSubAgents.delete(subAgent);
    });
    
    return subAgent;
  }
  
  // For recursive spawning: sub-agents can also spawn their own children
  // but with reduced resource limits to prevent exponential growth
}
```

### 6. Context Sharing and Isolation
- **Input context**: Passed via spawn options (task description + relevant context)
- **Working isolation**: Sub-agents get their own workspace/context but can access shared resources
- **Output sharing**: Results returned via Promise; artifacts can be shared via file system or message passing
- **Memory isolation**: Each sub-agent maintains its own memory/conversation context
- **Resource tracking**: Master agent tracks resource usage of all active sub-agents

### 7. Recursive Tree Structure
- Master agent can spawn sub-agents
- Sub-agents can spawn their own sub-agents (for further decomposition)
- But with restrictions to prevent runaway recursion:
  - Decreasing resource limits with depth
  - Maximum depth limit (e.g., 3 levels)
  - Different expertise specialization at each level
- Tree structure allows for:
  - Parallel execution of independent subtasks
  - Specialized agents for different aspects of a problem
  - Pipeline patterns (output of one agent feeds into next)

### 8. Integration with Other Systems

**With Component System:**
- Sub-agents access components via the same registry system
- Permission system applies equally to master and sub-agents
- Components track which agent used them for audit/reporting

**With Permission System:**
- Sub-agents inherit permissions from parent but can be further restricted
- Spawn options can specify allowedComponents to limit access
- Permission requests flow through the same interceptor system

**With Provider Abstraction:**
- Sub-agents can use same provider as parent or specify override
- Allows for using different models (e.g., faster/cheaper for sub-tasks)
- Provider usage tracked per agent for cost/quota management

### Example Orchestration Flow

```mermaid
graph TD
    A[Master Agent: \"Build a todo app with auth\"] -->|Spawns| B[Research Sub-Agent: \"Find best auth libraries\"]
    A -->|Spawns| C[Planning Sub-Agent: \"Design app architecture\"]
    A -->|Spawns| D[Implementation Sub-Agent: \"Set up project structure\"]
    
    B -->|Results| A
    C -->|Results| A
    D -->|Results| A
    
    A -->|Based on research & planning| E[Implementation Sub-Agent: \"Implement auth features\"]
    A -->|Based on research & planning| F[Implementation Sub-Agent: \"Implement UI components\"]
    
    E -->|Results| A
    F -->|Results| A
    
    A -->|Spawns| G[Testing Sub-Agent: \"Write and run tests\"]
    G -->|Results| A
    
    A -->|Final synthesis| H[Result: Complete todo app with auth]
```

This orchestration model provides a clean, flexible foundation for the agent-os architecture that enables:
- Intelligent task decomposition
- Specialized sub-agent execution
- Robust error handling and termination
- Clean integration with all other system components
- Scalable from simple tasks to complex projects