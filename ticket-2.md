# Ticket 2: Provider Abstraction Implementation

## Description
Implement the thin wrapper provider abstraction around the OpenAI-compatible API that supports both OpenRouter and NVIDIA NIM. This includes configuration handling, request/response handling, and streaming support.

## Implementation Steps
1. Create ProviderConfig interface for endpoint, model, apiKey, etc.
2. Implement AgentOsProvider class with initialize() method
3. Implement createCompletion() method with unified streaming/non-streaming interface
4. Handle tools, tool_choice, parallel_tool_calls parameters
5. Add proper error handling and response parsing
6. Create configuration loading from config file with environment variable overrides
7. Add health check and model listing capabilities

## Acceptance Criteria
- Provider can be initialized with configuration
- Provider successfully makes non-streaming API calls to OpenRouter/NIM endpoints
- Provider correctly handles streaming responses when requested
- Provider properly forwards tools/tool_choice/parallel_tool_calls parameters
- Configuration loads from file with environment variable override capability
- Proper error handling for network issues, invalid responses, etc.

## Blocking Edges
- Blocked by: Ticket 1 (Project Setup)

## Blocks
- Ticket 5 (Agent Orchestration Model) - needs provider for LLM calls
- Ticket 6 (Component Interface) - components may need provider access
- Ticket 8 (UI/Session Model) - agent functionality depends on provider

## Related Spec Sections
- Implementation Decisions: "Provider Abstraction: Thin wrapper around OpenAI-compatible API; configuration via config file with environment variable overrides; unified interface for streaming/non-streaming responses; tool execution handled at agent layer"
- User Story 4: "As a developer, I want to configure Agent OS to use either OpenRouter or NVIDIA NIM as my LLM provider..."