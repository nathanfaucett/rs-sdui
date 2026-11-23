import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { MessageProcessor } from "@a2ui/web_core";
import { basicCatalog } from "@a2ui/web_core/catalogs/basic/v1";

const workspace = fileURLToPath(new URL("../../../", import.meta.url));
const output = execFileSync(
  "cargo",
  ["run", "--quiet", "-p", "a2ui-browser-example"],
  {
    cwd: workspace,
    encoding: "utf8",
  },
);
const messages = JSON.parse(output);

let created;
const processor = new MessageProcessor([basicCatalog]);
processor.onSurfaceCreated((surface) => {
  if (surface.id === "main") created = surface;
});
processor.processMessages(messages);

assert.ok(
  created,
  "the official processor must create the Rust-authored surface",
);
assert.equal(created.id, "main");
assert.equal(created.dataModel.get("/name"), "A2UI");
assert.equal(created.dataModel.get("/submissions"), 0);
assert.ok(messages.some((message) => message.updateDataModel));
console.log(
  "Official A2UI MessageProcessor accepted the complete Rust state loop.",
);
