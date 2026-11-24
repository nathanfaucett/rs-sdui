import { spawn } from "node:child_process";
import { createServer } from "node:net";

const mode = process.argv[2];
if (mode !== "dev" && mode !== "preview") {
  throw new Error("usage: node scripts/serve.ts <dev|preview>");
}

const reservation = createServer();
await new Promise<void>((resolve, reject) => {
  reservation.once("error", reject);
  reservation.listen(0, "127.0.0.1", resolve);
});
const address = reservation.address();
if (!address || typeof address === "string") throw new Error("failed to reserve a WebSocket port");
const websocketAddress = `127.0.0.1:${address.port}`;
await new Promise<void>((resolve, reject) => {
  reservation.close((error) => (error ? reject(error) : resolve()));
});

const env = { ...process.env, A2UI_WS_ADDR: websocketAddress };
const children = [
  spawn("cargo", ["run", "--quiet", "-p", "a2ui-browser-example", "--bin", "server"], {
    env,
    stdio: "inherit",
  }),
  spawn("portless", ["a2ui-browser", "vite", mode], { env, stdio: "inherit" }),
];

let stopping = false;
function stop(code: number): void {
  if (stopping) return;
  stopping = true;
  process.exitCode = code;
  for (const child of children) {
    if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
  }
}

for (const child of children) {
  child.once("error", (error) => {
    console.error("failed to start an example process", error);
    stop(1);
  });
  child.once("exit", (code, signal) => {
    if (!stopping) stop(code ?? (signal ? 0 : 1));
  });
}
process.on("SIGINT", () => stop(0));
process.on("SIGTERM", () => stop(0));
