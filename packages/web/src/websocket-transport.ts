import { AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import { type Connection, createAgentMessageStream, TransportError } from "./transport.js";

export function webSocketConnection(socket: WebSocket): Connection {
  const stream = createAgentMessageStream();
  const pending: string[] = [];
  let closed = false;
  let opened = false;

  function removeListeners(): void {
    socket.removeEventListener("open", onOpen);
    socket.removeEventListener("message", onMessage);
    socket.removeEventListener("close", onClose);
  }

  function finish(error?: unknown): void {
    if (closed) return;
    closed = true;
    pending.length = 0;
    removeListeners();
    socket.close();
    if (error === undefined) {
      stream.close();
    } else {
      stream.fail(error);
    }
  }

  function onOpen(): void {
    opened = true;
    try {
      for (const message of pending) socket.send(message);
      pending.length = 0;
    } catch (error) {
      finish(
        new TransportError("A2UI WebSocket send failed", "websocket", "send", {
          cause: error,
        })
      );
    }
  }

  function onMessage(event: MessageEvent<unknown>): void {
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
  }

  function onClose(event: CloseEvent): void {
    const detail = event.reason ? `: ${event.reason}` : "";
    finish(
      new TransportError(
        `A2UI WebSocket closed (${event.code})${detail}`,
        "websocket",
        opened ? "receive" : "open",
        { cause: event }
      )
    );
  }

  socket.addEventListener("open", onOpen);
  socket.addEventListener("message", onMessage);
  // Close follows a WebSocket error and carries the details needed for recovery.
  socket.addEventListener("close", onClose);

  return {
    messages: stream.messages,
    send: (message) => {
      if (closed) return;
      try {
        const json = JSON.stringify(message);
        if (socket.readyState === socket.OPEN) {
          socket.send(json);
        } else if (socket.readyState === socket.CONNECTING) {
          pending.push(json);
        }
      } catch (error) {
        finish(
          new TransportError("A2UI WebSocket send failed", "websocket", "send", { cause: error })
        );
      }
    },
    close: () => finish(),
  };
}
