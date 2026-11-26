import type { ComponentApi, MessageProcessor, OutboundMessageListener } from "@a2ui/web_core";
import { RendererToAgentMessageSchema } from "@a2ui/web_core/v1_0";
import type { Connection, Transport } from "./transport.js";

/**
 * Connects one freshly configured message processor to a transport.
 *
 * The factory runs before connecting so it can register surface listeners first.
 * After a disconnect, create a new processor and connection before accepting a snapshot.
 * The returned done promise rejects when the transport stream or processor fails.
 */
export function connectMessageProcessor<T extends ComponentApi>(
  transport: Transport,
  createProcessor: (outboundListener: OutboundMessageListener) => MessageProcessor<T>
): {
  processor: MessageProcessor<T>;
  connection: Connection;
  done: Promise<void>;
} {
  let connection: Connection;
  const processor = createProcessor((message) => {
    const parsed = RendererToAgentMessageSchema.safeParse(message);
    if (parsed.success) {
      connection.send(parsed.data);
    } else {
      console.error("A2UI outbound message error", parsed.error);
    }
  });

  connection = transport.connect();
  const consume = async () => {
    for await (const message of connection.messages) processor.processMessages(message);
  };
  const done = consume();
  return { processor, connection, done };
}
