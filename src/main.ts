// Basic frontend entry point for Agent OS
// This serves the static files and sets up the basic UI

// For now, we'll just serve the index.html file
// In development, this will be handled by the dev server
// In production, the built files will be served

import { serve } from "bun";

// Simple static file server for development
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
  }
});

console.log(`Server running at http://localhost:${port}`);