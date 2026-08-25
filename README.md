# agent-os

Computing through Agents, not manual.

## What this repository now provides

- A **master agent** that accepts a user prompt and orchestrates work.
- Ability to spin up **any number of sub-agents** (user-controlled, no hard cap in code).
- A **graphical UI** that shows live task progress and final results.
- **Plug-and-play tools** (toggleable at runtime), including:
  - Browser window actions
  - Generic tool-call execution simulation

## Run locally

From the repository root (`/home/runner/work/agent-os/agent-os`), run:

```bash
python3 -m http.server 8000
```

Then open `http://localhost:8000` in your browser.
