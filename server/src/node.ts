import { serve } from "@hono/node-server";
import { app } from "./app";

const port = Number(process.env.PORT ?? 3000);

serve({
  fetch: (request) => app.fetch(request, process.env),
  port,
});

console.log(`local-reader-server listening on http://127.0.0.1:${port}`);
