import { AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import { type Connection, createAgentMessageStream, type Transport } from "./transport.js";

export class SharedWorkerTransport implements Transport {
  private readonly createWorker: () => SharedWorker;

  constructor(createWorker: () => SharedWorker) {
    this.createWorker = createWorker;
  }

  connect(): Connection {
    const worker = this.createWorker();
    const port = worker.port;
    worker.addEventListener("error", (event) => {
      console.error("A2UI shared worker error", event.message, event.filename, event.lineno);
    });
    const stream = createAgentMessageStream();
    let closed = false;

    const onMessage = (event: MessageEvent<unknown>) => {
      const parsed = AgentToRendererMessageSchema.safeParse(event.data);
      if (parsed.success) {
        stream.publish(parsed.data);
      } else {
        console.error("A2UI worker message error", parsed.error);
      }
    };
    const onMessageError = (event: MessageEvent<unknown>) => {
      console.error("A2UI worker message error", event);
    };

    port.addEventListener("message", onMessage);
    port.addEventListener("messageerror", onMessageError);
    port.start();

    return {
      messages: stream.messages,
      send: (message) => {
        if (!closed) port.postMessage(message);
      },
      close: () => {
        if (closed) return;
        closed = true;
        port.removeEventListener("message", onMessage);
        port.removeEventListener("messageerror", onMessageError);
        port.close();
        stream.close();
      },
    };
  }
}
