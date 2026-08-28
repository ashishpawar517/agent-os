## Research Summary

For terminal emulation in a desktop application with web UI, the recommended approach is:

### Frontend: xterm.js
- Embeds a full-featured terminal in browser context
- Zero dependencies, performant, rich Unicode support
- API: `new Terminal().open(element)` with `write()` and `onData()` handlers

### Backend PTY Options
1. **Node.js + node-pty**: Uses `forkpty(3)` for cross-platform PTY
2. **Bun.spawn() with PTY support**: Built-in, no extra dependencies, cross-platform
3. **Deno**: Limited PTY support without third-party packages

### Recommended Approach: Bun.spawn() PTY
- **Advantages**: Built-in PTY support, cross-platform, feature-rich (resize, raw mode, etc.)
- **How it works**: 
  ```typescript
  const proc = Bun.spawn(["bash"], {
    terminal: {
      cols: 80, rows: 24,
      data(terminal, data) { /* forward to frontend */ },
      exit(...) { /* handle cleanup */ }
    }
  });
  proc.terminal.write("input\\n");
  proc.terminal.resize(100, 40);
  ```

### Connection Pattern for Tauri App
- Frontend (xterm.js in WebView) ↔ Backend (Bun/Rust via Tauri commands/events)
- Bidirectional: keystrokes → backend → PTY → output → frontend
- Resize events propagated to PTY

### Trade-offs
- **Capability**: Full terminal compatibility (shells, vim, etc.)
- **Packaging**: Smaller than Electron (~600KB Tauri + ~4-5MB Bun vs 40MB+ Electron)
- **Fidelity**: Real PTY behavior - shells act exactly as in native terminals
- **Streaming**: Efficient bidirectional data flow via PTY

## Sources
- xterm.js: https://github.com/xtermjs/xterm.js
- Bun PTY: https://bun.sh/docs/api/spawn#terminal-pty-support
- Tauri: https://v1.tauri.app/v1/guides/