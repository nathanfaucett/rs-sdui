import { AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import {
  type Connection,
  createAgentMessageStream,
  type Transport,
  TransportError,
} from "./transport.js";

export class SharedWorkerTransport implements Transport {
  private readonly createWorker: () => SharedWorker;

  constructor(createWorker: () => SharedWorker) {
    this.createWorker = createWorker;
  }

  connect(): Connection {
    const worker = this.createWorker();
    const port = worker.port;
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
      finish(
        new TransportError("A2UI shared worker message failed", "shared-worker", "receive", {
          cause: event,
        })
      );
    };
    const onError = (event: ErrorEvent) => {
      finish(
        new TransportError("A2UI shared worker failed", "shared-worker", "runtime", {
          cause: event,
        })
      );
    };
    const finish = (error?: TransportError): void => {
      if (closed) return;
      closed = true;
      worker.removeEventListener("error", onError);
      port.removeEventListener("message", onMessage);
      port.removeEventListener("messageerror", onMessageError);
      port.close();
      if (error === undefined) {
        stream.close();
      } else {
        stream.fail(error);
      }
    };

    worker.addEventListener("error", onError);
    port.addEventListener("message", onMessage);
    port.addEventListener("messageerror", onMessageError);
    port.start();

    return {
      messages: stream.messages,
      send: (message) => {
        if (closed) return;
        try {
          port.postMessage(message);
        } catch (error) {
          finish(
            new TransportError("A2UI shared worker send failed", "shared-worker", "send", {
              cause: error,
            })
          );
        }
      },
      close: () => finish(),
    };
  }
}
