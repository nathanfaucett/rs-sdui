import type { A2uiSurface, LitComponentApi } from "@a2ui/lit";
// Registers <a2ui-surface>; type-only imports do not.
import "@a2ui/lit";
import { MessageProcessor } from "@a2ui/web_core";
import { basicCatalog } from "@a2ui/web_core/catalogs/basic/v1";
import init, { handle_action, initial_messages } from "./pkg/browser";
import "./style.css";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app element");

await init();

const deliver = async (message: unknown) => {
  // This local call can be replaced by WebSocket, SSE, HTTP, or another transport.
  const updates = JSON.parse(await handle_action(JSON.stringify(message)));
  processor.processMessages(updates);
};

const processor = new MessageProcessor<LitComponentApi>(
  [basicCatalog],
  (action) => deliver({ version: "1.0", action }),
  {
    outboundListener: deliver,
    errorHandler: (error) => console.error("A2UI renderer error", error),
  },
);

processor.onSurfaceCreated((surface) => {
  if (surface.id !== "main") return;

  const element = document.createElement("a2ui-surface") as A2uiSurface;
  element.surface = surface;
  target.replaceChildren(element);
});

processor.onSurfaceDeleted((surfaceId) => {
  if (surfaceId === "main") target.replaceChildren();
});

processor.processMessages(JSON.parse(initial_messages()));
