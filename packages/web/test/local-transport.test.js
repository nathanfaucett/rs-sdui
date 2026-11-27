import assert from "node:assert/strict";
import { test } from "node:test";
import { LocalStreamTransport, TransportError } from "../dist/transport.js";

test("rejects the message stream when local action handling fails", async () => {
  const failure = new Error("action failed");
  const transport = new LocalStreamTransport({
    messages: [],
    surfaceId: "main",
    handleAction() {
      throw failure;
    },
  });
  const connection = transport.connect();
  const done = (async () => {
    for await (const _message of connection.messages) {
      // No messages are expected.
    }
  })();

  connection.send({ action: { name: "test" } });

  await assert.rejects(done, (error) => {
    assert.ok(error instanceof TransportError);
    assert.equal(error.transport, "local");
    assert.equal(error.operation, "send");
    assert.equal(error.cause, failure);
    return true;
  });
});
