import { A2uiSurface, type LitComponentApi } from "@a2ui/lit";
import { MessageProcessor } from "@a2ui/web_core";
import { basicCatalog } from "@a2ui/web_core/catalogs/basic/v1";
import init, { messages } from "./pkg/browser";
import "./style.css";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app element");

await init();

const processor = new MessageProcessor<LitComponentApi>(
  [basicCatalog],
  undefined,
  {
    outboundListener: (message: unknown) => {
      // Applications can forward this standard A2UI message over any transport.
      console.log("A2UI outbound message", message);
    },
  },
);

processor.onSurfaceCreated((surface) => {
  if (surface.id !== "main") return;

  const element = document.createElement("a2ui-surface") as A2uiSurface;
  element.surface = surface;
  target.replaceChildren(element);
});

processor.processMessages(JSON.parse(messages()));
