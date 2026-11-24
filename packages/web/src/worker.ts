/// <reference lib="webworker" />

import type { AgentToRendererMessage, RendererToAgentMessage } from "@a2ui/web_core/v1_0";

export interface WorkerApplication {
  initial_messages(): string;
  snapshot_messages(): string;
  handle_action(input: string): Promise<string>;
}

export interface ApplicationWorkerOptions {
  application: WorkerApplication;
  initialize(): Promise<unknown>;
}

export function startApplicationWorker(options: ApplicationWorkerOptions): void {
  const workerScope = self as unknown as SharedWorkerGlobalScope;
  const { application } = options;
  const ports = new Set<MessagePort>();
  let queue: Promise<void> = Promise.resolve();

  function decodeMessages(json: string): AgentToRendererMessage[] {
    // The window transport validates these messages before the official renderer sees them.
    return JSON.parse(json) as AgentToRendererMessage[];
  }

  async function startApplication(): Promise<void> {
    await options.initialize();
    // New ports receive snapshots, so startup messages must not also be broadcast.
    decodeMessages(application.initial_messages());
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
      // The renderer adapter validates messages before this transport boundary.
      if (!event.data || typeof event.data !== "object" || !("action" in event.data)) return;
      const message = event.data as RendererToAgentMessage;

      enqueue(async () => {
        await applicationReady;
        const updates = decodeMessages(await application.handle_action(JSON.stringify(message)));
        broadcast(updates);
      });
    };
    port.onmessageerror = (event) => console.error("A2UI application message error", event);
    port.start();

    enqueue(async () => {
      await applicationReady;
      for (const message of decodeMessages(application.snapshot_messages())) {
        if (!post(port, message)) return;
      }
      // Earlier action broadcasts cannot reach this port before its snapshot.
      ports.add(port);
    });
  };
}
