import { Hono } from "hono";
import type { Env } from "./config";
import { engineRoutes } from "./routes/engine";
import { healthRoutes } from "./routes/health";
import { remoteRoutes } from "./routes/remote";

export const app = new Hono<{ Bindings: Env }>();

app.route("/api", healthRoutes);
app.route("/api", remoteRoutes);
app.route("/api", engineRoutes);

app.notFound((c) => c.text("not found", 404));
