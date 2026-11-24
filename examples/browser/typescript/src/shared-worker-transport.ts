import { AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import { type Connection, createAgentMessageStream, type Transport } from "./transport";

export class SharedWorkerTransport implements Transport {
  connect(): Connection {
    const worker = new SharedWorker(new URL("./application-worker.ts", import.meta.url), {
      type: "module",
      name: "a2ui-todo-application",
    });
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
