// Terminal implementation using Bun WebSocket API
import { spawn } from "bun";

interface TerminalSession {
  id: string;
  proc: ChildProcessWithoutNullStreams;
  send: (data: string) => void;
  close: () => void;
}

const sessions = new Map<string, TerminalSession>();

// This function will be called from the upgrade handler in main.ts
export function setupTerminalWebSocketUpgrade(request) {
  const { socket, response } = Bun.upgradeWebSocket(request, {
    open(ws) {
      // Create a new terminal session for each connection
      const id = Math.random().toString(36).substring(2, 15);
      ws["sessionId"] = id;

      // Spawn a shell process (using bash, fallback to sh)
      const shell = process.env.SHELL || "/bin/bash";
      const proc = spawn(shell, {
        stdin: "pipe",
        stdout: "pipe",
        stderr: "pipe",
      });

      // Forward stdout/stderr to websocket
      const encoder = new TextEncoder();
      proc.stdout.subscribe((data) => {
        ws.send(encoder.encode(data));
      });
      proc.stderr.subscribe((data) => {
        ws.send(encoder.encode(data));
      });

      // Handle incoming WebSocket messages (stdin to process)
      ws.on("message", (data) => {
        const sessionId = ws["sessionId"] as string | undefined;
        const session = sessions.get(sessionId ?? "");
        if (!session) {
          ws.close();
          return;
        }
        if (typeof data === "string") {
          session.proc.write(data);
        } else {
          session.proc.write(Buffer.from(data));
        }
      });

      // Handle WebSocket close
      ws.on("close", () => {
        const sessionId = ws["sessionId"] as string | undefined;
        const session = sessions.get(sessionId ?? "");
        session?.close();
      });

      sessions.set(id, {
        id,
        proc,
        send: (data: string) => {
          proc.write(data);
        },
        close: () => {
          proc.kill();
          sessions.delete(id);
        },
      });

      ws.send(encoder.encode(`\r\n[Terminal session ${id} started]\r\n$ `));
    }
  });

  return response;
}

// Keep the original function for compatibility but mark it as deprecated
export function setupTerminalWebSocket(server) {
  console.warn("setupTerminalWebSocket is deprecated, use setupTerminalWebSocketUpgrade with the upgrade handler instead");
  // This is a placeholder - the actual WebSocket handling is done via the upgrade handler in main.ts
}