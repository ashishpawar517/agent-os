// Basic frontend entry point for Agent OS
// This serves the static files and sets up the basic UI

// For now, we'll just serve the index.html file
// In development, this will be handled by the dev server
// In production, the built files will be served

import { serve } from "bun";
import { setupTerminalWebSocket } from "./terminal";

// Simple static file server for development with WebSocket support
const port = 1420;

const server = serve({
  port: port,
  fetch(req) {
    const url = new URL(req.url);

    // Serve index.html for root path
    if (url.pathname === "/" || url.pathname === "") {
      return new Response(Bun.file("./src/index.html"), {
        headers: { "Content-Type": "text/html" }
      });
    }

    // Serve other static files from src directory
    try {
      const file = Bun.file(`.${url.pathname}`);
      return new Response(file);
    } catch {
      // If file not found, return 404
      return new Response("Not Found", { status: 404 });
    }
  },
  // WebSocket upgrade handler
  upgrade(req) {
    // Check if this is a WebSocket upgrade request to our terminal endpoint
    if (req.url === "/ws/terminal") {
      // This will be handled by our setupTerminalWebSocket function
      // We need to pass the upgrade request to it
      return setupTerminalWebSocketUpgrade(req);
    }
    // For all other requests, don't upgrade
    return undefined;
  }
});

console.log(`Server running at http://localhost:${port}`);
console.log(`Terminal WebSocket available at ws://localhost:${port}/ws/terminal`);

// We need to modify setupTerminalWebSocket to work with the upgrade handler