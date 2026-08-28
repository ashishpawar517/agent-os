# Ticket 3: Component Interface and Registry System

## Description
Define the component interface contract and implement the registry/service locator pattern for component discovery and management. This includes the discover/start/stop lifecycle methods and registration mechanism.

## Implementation Steps
1. Define ComponentDescriptor interface for component metadata
2. Define ComponentConfig interface for component-specific configuration
3. Define ComponentLifecycle interface with discover(), start(), stop() methods
4. Define ComponentInstance interface extending ComponentLifecycle with event handling
5. Define ComponentFactory interface for creating component instances
6. Implement ComponentRegistry class with register/unregister/getComponent/listAvailable methods
7. Implement basic event handling system (on/off/once methods)
8. Create TypeScript types/utils for component communication

## Acceptance Criteria
- Components can be registered with the registry
- Components can be discovered via getComponent(type) method
- Component lifecycle methods (discover/start/stop) are properly defined
- Event handling system works for component-agent communication
- TypeScript interfaces provide proper typing for component contracts
- Registry supports both direct instantiation and factory patterns

## Blocking Edges
- Blocked by: Ticket 1 (Project Setup)

## Blocks
- Ticket 4 (Terminal Component) - needs component interface
- Ticket 5 (Browser Component) - needs component interface
- Ticket 6 (Agent Orchestration Model) - agents need to discover/use components
- Ticket 7 (Permission System) - components will be intercepted through registry

## Related Spec Sections
- Implementation Decisions: "Component Interface: Components implement discover/start/stop lifecycle; discovered via registry/service locator; support both direct calls (for agent internal use) and tool-calling path (for LLM-driven usage); communicate via callbacks/Promises + Events"
- User Story 3: "As a developer, I want to be able to plug in different components (terminal, browser, etc.) into Agent OS..."
- User Story 5: "As a developer, I want sub-agents to be able to use components like terminal and browser through a well-defined interface..."