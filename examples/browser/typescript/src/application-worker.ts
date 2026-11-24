/// <reference lib="webworker" />

import type { AgentToRendererMessage, RendererToAgentMessage } from "@a2ui/web_core/v1_0";
import init, { handle_action, initial_messages, snapshot_messages } from "./pkg/browser";

const workerScope = self as unknown as SharedWorkerGlobalScope;
const ports = new Set<MessagePort>();
let queue: Promise<void> = Promise.resolve();

function decodeMessages(json: string): AgentToRendererMessage[] {
  // The window transport validates these messages before the official renderer sees them.
  return JSON.parse(json) as AgentToRendererMessage[];
}

async function startApplication(): Promise<void> {
  await init();
  // New ports receive snapshots, so startup messages must not also be broadcast.
  decodeMessages(initial_messages());
}

const applicationReady = startApplication();
applicationReady.catch((error: unknown) => {
  console.error("A2UI application startup failed", error);
});

function enqueue(task: () => Promise<void>): void {
  queue = queue.then(task).catch((error: unknown) => {
    console.error("A2UI application worker error", error);
  });
}

function post(port: MessagePort, message: AgentToRendererMessage): boolean {
  try {
    port.postMessage(message);
    return true;
  } catch (error) {
    ports.delete(port);
    console.error("A2UI application port error", error);
    return false;
  }
}

function broadcast(messages: AgentToRendererMessage[]): void {
  for (const message of messages) {
    for (const port of ports) post(port, message);
  }
}

workerScope.onconnect = (event) => {
  const [port] = event.ports;
  if (!port) return;

  port.onmessage = (event: MessageEvent<unknown>) => {
    // mountA2ui validates renderer messages before this transport boundary.
    if (!event.data || typeof event.data !== "object" || !("action" in event.data)) return;
    const message = event.data as RendererToAgentMessage;

    enqueue(async () => {
      await applicationReady;
      const updates = decodeMessages(await handle_action(JSON.stringify(message)));
      broadcast(updates);
    });
  };
  port.onmessageerror = (event) => console.error("A2UI application message error", event);
  port.start();

  enqueue(async () => {
    await applicationReady;
    for (const message of decodeMessages(snapshot_messages())) {
      if (!post(port, message)) return;
    }
    // Earlier action broadcasts cannot reach this port before its snapshot.
    ports.add(port);
  });
};
