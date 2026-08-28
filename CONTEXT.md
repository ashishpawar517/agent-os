# agent-os

A local-first desktop application that receives a prompt, routes it through a master agent which spawns sub-agents to carry the task out, over pluggable components, backed by a configurable set of LLM providers.

## Language

**Master agent**:
The top-level agent that receives a prompt and coordinates the task by spawning sub-agents.

**Sub-agent**:
An agent spawned by the master (or by another sub-agent) to carry out part of a task. May itself spawn further sub-agents, forming a recursive tree.

**Prompt**:
The input a user gives to start a task.
_Avoid_: query, request, message

**Component**:
A pluggable capability an agent invokes to act on the local machine or the web — e.g. a terminal, a browser. The unit of the plug-and-play system.
_Avoid_: plugin, module, tool, extension

**Provider**:
An LLM backend the app can call (OpenRouter, NVIDIA NIM, …) together with its configuration (endpoint, model id, credentials). A single provider may expose many models.
_Avoid_: backend, engine, model

**Session**:
One running task: its prompt, its full agent tree, and its transcript. Each session surfaces in the UI as a tab.
_Avoid_: tab, chat, conversation, job