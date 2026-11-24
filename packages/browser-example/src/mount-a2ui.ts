import type { A2uiSurface, LitComponentApi } from "@a2ui/lit";
// Registers <a2ui-surface>; type-only imports do not.
import "@a2ui/lit";
import { MessageProcessor } from "@a2ui/web_core";
import { basicCatalog } from "@a2ui/web_core/catalogs/basic/v1";
import { RendererToAgentMessageSchema } from "@a2ui/web_core/v1_0";
import type { Connection, Transport } from "@sdui/web";

/**
 * Mounts one transport session.
 *
 * After a connection loss, create a fresh mount before accepting a full snapshot.
 * The official processor rejects createSurface for an existing surface ID.
 */
export function mountA2ui(
  target: HTMLElement,
  surfaceId: string,
  transport: Transport
): {
  processor: MessageProcessor<LitComponentApi>;
  connection: Connection;
  done: Promise<void>;
} {
  const processor = new MessageProcessor<LitComponentApi>([basicCatalog], undefined, {
    unifiedOutbound: true,
    outboundListener: (message) => {
      const parsed = RendererToAgentMessageSchema.safeParse(message);
      if (parsed.success) {
        connection.send(parsed.data);
      } else {
        console.error("A2UI outbound message error", parsed.error);
      }
    },
    errorHandler: (error) => console.error("A2UI renderer error", error),
  });

  processor.onSurfaceCreated((surface) => {
    if (surface.id !== surfaceId) return;

    const element = document.createElement("a2ui-surface") as A2uiSurface;
    element.surface = surface;
    target.replaceChildren(element);
  });

  processor.onSurfaceDeleted((deletedSurfaceId) => {
    if (deletedSurfaceId === surfaceId) target.replaceChildren();
  });

  const connection = transport.connect();
  const consume = async () => {
    try {
      for await (const message of connection.messages) processor.processMessages(message);
    } catch (error) {
      console.error("A2UI transport stream error", error);
    }
  };
  const done = consume();
  return { processor, connection, done };
}
