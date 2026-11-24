import assert from "node:assert/strict";
import { MessageProcessor } from "@a2ui/web_core";
import { basicCatalog } from "@a2ui/web_core/catalogs/basic/v1";
import { ActionMessageSchema, AgentToRendererMessageSchema } from "@a2ui/web_core/v1_0";
import {
  BASIC_CATALOG_ID,
  handleStreamAction,
  STREAM_SURFACE_ID,
  streamMessages,
} from "./src/deterministic-stream.ts";
import { LocalStreamTransport } from "./src/transport.ts";

const processor = new MessageProcessor([basicCatalog]);
const process = (message) => {
  processor.processMessages(AgentToRendererMessageSchema.parse(message));
};
const received = [];
const connection = new LocalStreamTransport({
  messages: streamMessages,
  surfaceId: STREAM_SURFACE_ID,
  handleAction: handleStreamAction,
}).connect();

for await (const message of connection.messages) {
  const index = received.length;
  received.push(message);
  process(message);

  if (index === 2) {
    assert.ok(processor.getSurface(STREAM_SURFACE_ID)?.componentsModel.get("root"));
    assert.equal(
      processor.getSurface(STREAM_SURFACE_ID)?.componentsModel.get("heading"),
      undefined
    );
  }

  if (index === streamMessages.length - 1) {
    const surface = processor.getSurface(STREAM_SURFACE_ID);
    const action = ActionMessageSchema.parse({
      version: "1.0",
      action: {
        name: "add_passenger",
        surfaceId: STREAM_SURFACE_ID,
        sourceComponentId: "add_passenger",
        timestamp: "2026-11-23T00:00:00Z",
        context: { passengers: surface?.dataModel.get("/trip/passengers") },
      },
    });
    connection.send(action);
  }

  if (received.length === streamMessages.length + 2) connection.close();
}

assert.equal(received.length, streamMessages.length + 2);
assert.deepEqual(
  received.slice(streamMessages.length).map((message) => message.updateDataModel?.path),
  ["/trip/passengers", "/trip/total"]
);

const surface = processor.getSurface(STREAM_SURFACE_ID);
assert.ok(surface, "the official processor must create the streamed surface");
assert.equal(surface.defaultCatalog?.id, BASIC_CATALOG_ID);
assert.deepEqual(surface.componentsModel.get("root")?.properties.children, [
  "heading",
  "summary",
  "description",
  "total_row",
  "actions",
]);
assert.equal(surface.componentsModel.get("summary")?.properties.text["@call"], "formatString");
assert.equal(
  surface.componentsModel.get("total_value")?.properties.text["@call"],
  "formatCurrency"
);
assert.deepEqual(surface.dataModel.get("/trip"), {
  from: "Berlin",
  to: "San Francisco",
  passengers: 2,
  total: 910,
});
assert.equal(processor.getSurfaces().size, 1);
console.log("Official A2UI schemas and MessageProcessor accepted the deterministic async stream.");
