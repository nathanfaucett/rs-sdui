import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { MessageProcessor } from "@a2ui/web_core";
import { basicCatalog } from "@a2ui/web_core/catalogs/basic/v1";

const workspace = fileURLToPath(new URL("../../", import.meta.url));
const output = execFileSync("cargo", ["run", "--quiet", "-p", "a2ui-browser-example"], {
  cwd: workspace,
  encoding: "utf8",
});
const messages = JSON.parse(output);

let created;
const processor = new MessageProcessor([basicCatalog]);
processor.onSurfaceCreated((surface) => {
  if (surface.id === "main") created = surface;
});
processor.processMessages(messages);

assert.ok(created, "the official processor must create the Rust-authored surface");
assert.equal(created.id, "main");
assert.deepEqual(created.dataModel.get("/todos"), [
  { id: 1, title: "Learn A2UI", completed: true },
  { id: 2, title: "Ship the todo list", completed: false },
]);
assert.equal(created.dataModel.get("/draft"), "");
assert.equal(created.dataModel.get("/filter"), "active");
assert.equal(created.dataModel.get("/nextId"), 3);
assert.ok(messages.some((message) => message.updateComponents));
assert.ok(messages.some((message) => message.updateDataModel?.path === "/todos"));
assert.ok(messages.some((message) => message.updateDataModel?.path === "/filter"));
assert.ok(messages.every((message) => !message.deleteSurface));
console.log("Official A2UI MessageProcessor accepted the complete Rust todo state loop.");
