import { AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import { type Connection, createAgentMessageStream, type Transport } from "./transport.js";

export interface SseTransportOptions {
  eventSourceInit?: EventSourceInit;
  requestInit?: Omit<RequestInit, "body" | "method" | "signal">;
  sendUrl?: string | URL;
}

export class SseTransport implements Transport {
  private readonly options: SseTransportOptions;
  private readonly url: string | URL;

  constructor(url: string | URL, options: SseTransportOptions = {}) {
    this.url = url;
    this.options = options;
  }

  connect(): Connection {
    const source = new EventSource(this.url, this.options.eventSourceInit);
    const stream = createAgentMessageStream();
    const abort = new AbortController();
    const headers = new Headers(this.options.requestInit?.headers);
    if (!headers.has("content-type")) headers.set("content-type", "application/json");

    const pending: string[] = [];
    let closed = false;
    let opened = false;
    let queue: Promise<void> = Promise.resolve();

    function close(): void {
      if (closed) return;
      closed = true;
      opened = false;
      pending.length = 0;
      abort.abort();
      source.close();
      stream.close();
    }

    function fail(label: string, error: unknown): void {
      if (closed) return;
      console.error(label, error);
      close();
    }

    const post = async (json: string): Promise<void> => {
      if (closed) return;
      try {
        const response = await fetch(this.options.sendUrl ?? this.url, {
          ...this.options.requestInit,
          method: "POST",
          headers,
          body: json,
          signal: abort.signal,
        });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
      } catch (error) {
        fail("A2UI SSE send error", error);
      }
    };

    const enqueue = (json: string): void => {
      queue = queue.then(() => post(json));
    };

    source.onopen = () => {
      if (closed) return;
      opened = true;
      for (const message of pending) enqueue(message);
      pending.length = 0;
    };
    source.onmessage = (event: MessageEvent<unknown>) => {
      if (closed) return;
      try {
        const parsed = AgentToRendererMessageSchema.safeParse(JSON.parse(String(event.data)));
        if (parsed.success) {
          stream.publish(parsed.data);
        } else {
          console.error("A2UI SSE message error", parsed.error);
        }
      } catch (error) {
        console.error("A2UI SSE JSON error", error);
      }
    };
    // Automatic SSE reconnects can send a snapshot to an old renderer.
    source.onerror = (event) => fail("A2UI SSE connection error", event);

    return {
      messages: stream.messages,
      send: (message) => {
        if (closed) return;
        const json = JSON.stringify(message);
        if (opened) {
          enqueue(json);
        } else {
          pending.push(json);
        }
      },
      close,
    };
  }
}
