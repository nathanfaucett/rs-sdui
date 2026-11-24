import type {
  ActionMessage,
  AgentToRendererMessage,
  RendererToAgentMessage,
} from "@a2ui/web_core/v1_0";

export interface AgentMessageStream {
  messages: AsyncIterable<AgentToRendererMessage>;
  publish(message: AgentToRendererMessage): void;
  close(): void;
}

export interface Connection {
  messages: AsyncIterable<AgentToRendererMessage>;
  send(message: RendererToAgentMessage): void;
  close(): void;
}

export interface Transport {
  connect(): Connection;
}

export interface LocalStreamOptions {
  messages: readonly AgentToRendererMessage[];
  surfaceId: string;
  handleAction(message: ActionMessage): AgentToRendererMessage[];
}

export function createAgentMessageStream(): AgentMessageStream {
  let controller: ReadableStreamDefaultController<AgentToRendererMessage> | undefined;
  let closed = false;
  const stream = new ReadableStream<AgentToRendererMessage>({
    start(streamController) {
      controller = streamController;
    },
    cancel() {
      closed = true;
    },
  });
  const messages: AsyncIterable<AgentToRendererMessage> = {
    async *[Symbol.asyncIterator]() {
      const reader = stream.getReader();
      try {
        for (let next = await reader.read(); !next.done; next = await reader.read()) {
          yield next.value;
        }
      } finally {
        reader.releaseLock();
      }
    },
  };

  return {
    messages,
    publish(message) {
      if (!closed) controller?.enqueue(message);
    },
    close() {
      if (closed) return;
      closed = true;
      controller?.close();
    },
  };
}

export class LocalStreamTransport implements Transport {
  private activeStream: AbortController | undefined;
  private readonly handleAction: (message: ActionMessage) => AgentToRendererMessage[];
  private readonly initialMessages: readonly AgentToRendererMessage[];
  private stream: AgentMessageStream | undefined;
  private surfaceCreated: boolean = false;
  private readonly surfaceId: string;

  constructor(options: LocalStreamOptions) {
    this.handleAction = options.handleAction;
    this.initialMessages = options.messages;
    this.surfaceId = options.surfaceId;
  }

  connect(): Connection {
    const stream = createAgentMessageStream();
    this.stream = stream;
    let closed = false;

    const connection: Connection = {
      messages: stream.messages,
      send: (message) => {
        if (closed || !("action" in message)) return;
        for (const update of this.handleAction(message)) stream.publish(update);
      },
      close: () => {
        if (closed) return;
        closed = true;
        this.activeStream?.abort();
        this.activeStream = undefined;
        this.surfaceCreated = false;
        stream.close();
        if (this.stream === stream) this.stream = undefined;
      },
    };

    void this.replay();
    return connection;
  }

  async replay(): Promise<void> {
    const stream = this.stream;
    if (!stream) return;

    this.activeStream?.abort();
    const currentStream = new AbortController();
    this.activeStream = currentStream;

    try {
      if (this.surfaceCreated) {
        stream.publish({
          version: "1.0",
          deleteSurface: { surfaceId: this.surfaceId },
        });
        this.surfaceCreated = false;
      }

      for (const message of this.initialMessages) {
        // Yield without timing so closing or replaying can cancel between messages.
        await Promise.resolve();
        currentStream.signal.throwIfAborted();
        stream.publish(message);
        if ("createSurface" in message) this.surfaceCreated = true;
      }
    } catch (error) {
      if (!currentStream.signal.aborted) throw error;
    } finally {
      if (this.activeStream === currentStream) this.activeStream = undefined;
    }
  }
}
