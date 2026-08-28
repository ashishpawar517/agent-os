## Grilling Round 1: Provider Abstraction Decision

Based on research from issue #3 (Provider landscape — tool-calling support in OpenRouter & NVIDIA NIM), I've gathered the following facts:

**OpenRouter:**
- OpenAI-compatible API at `https://openrouter.ai/api/v1/chat/completions`
- Supports `tools` parameter (array of function definitions)
- Supports `tool_choice` and `parallel_tool_calls` flags
- Authentication via `Authorization: Bearer <token>`

**NVIDIA NIM:**
- Self-hosted LLMs with OpenAI-compatible API endpoint (typically `:8000/v1/chat/completions`)
- Inherits function calling capabilities from upstream model
- Documentation confirms OpenAPI compatibility
- Same API shape as OpenRouter for tool calling

**Implications for Uniform Provider Abstraction:**
1. Accept `tools`, `tool_choice`, `parallel_tool_calls` in request body
2. Handle streaming responses (both support streaming)
3. Authenticate via `Authorization: Bearer <token>`
4. Route to appropriate base URL (OpenRouter vs NIM host:port)
5. Execute tool calls via agent components when model requests them

Let me work through the specific decisions needed for the provider abstraction.