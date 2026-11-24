import { AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import { type Connection, createAgentMessageStream, type Transport } from "./transport.js";

export class WebSocketTransport implements Transport {
  private readonly url: string | URL;

  constructor(url: string | URL) {
    this.url = url;
  }

  connect(): Connection {
    const socket = new WebSocket(this.url);
    const stream = createAgentMessageStream();
    const pending: string[] = [];
    let closed = false;

    socket.onopen = () => {
      for (const message of pending) socket.send(message);
      pending.length = 0;
    };
    socket.onmessage = (event: MessageEvent<unknown>) => {
      if (typeof event.data !== "string") {
        console.error("A2UI WebSocket expected a text message", event.data);
        return;
      }

      try {
        const parsed = AgentToRendererMessageSchema.safeParse(JSON.parse(event.data));
        if (parsed.success) {
          stream.publish(parsed.data);
        } else {
          console.error("A2UI WebSocket message error", parsed.error);
        }
      } catch (error) {
        console.error("A2UI WebSocket JSON error", error);
      }
    };
    socket.onerror = (event) => console.error("A2UI WebSocket error", event);
    socket.onclose = () => {
      closed = true;
      pending.length = 0;
      stream.close();
    };

    return {
      messages: stream.messages,
      send: (message) => {
        if (closed) return;
        const json = JSON.stringify(message);
        if (socket.readyState === WebSocket.OPEN) {
          socket.send(json);
        } else if (socket.readyState === WebSocket.CONNECTING) {
          pending.push(json);
        }
      },
      close: () => {
        if (closed) return;
        closed = true;
        pending.length = 0;
        socket.close();
        stream.close();
      },
    };
  }
}
