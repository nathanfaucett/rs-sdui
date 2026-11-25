import assert from "node:assert/strict";
import { test } from "node:test";
import { connectMessageProcessor } from "../dist/connect.js";

test("connects a configured processor and validates outbound messages", async () => {
  const originalConsoleError = console.error;
  const errors = [];
  const order = [];
  const received = [];
  const sent = [];
  console.error = (...args) => errors.push(args);

  try {
    const incoming = {
      version: "1.0",
      deleteSurface: { surfaceId: "main" },
    };
    const action = {
      version: "1.0",
      action: {
        name: "add_todo",
        surfaceId: "main",
        sourceComponentId: "add-button",
        timestamp: "2026-11-25T00:00:00Z",
        context: { title: "Test the connector" },
      },
    };
    const transport = {
      connect() {
        order.push("connect");
        return {
          messages: (async function* () {
            yield incoming;
          })(),
          send(message) {
            sent.push(message);
          },
          close() {},
        };
      },
    };
    const processor = {
      processMessages(message) {
        received.push(message);
      },
    };

    let outboundListener;
    const connected = connectMessageProcessor(transport, (listener) => {
      order.push("processor");
      outboundListener = listener;
      return processor;
    });

    outboundListener(action);
    outboundListener({ version: "1.0", invalid: {} });
    await connected.done;

    assert.deepEqual(order, ["processor", "connect"]);
    assert.equal(connected.processor, processor);
    assert.deepEqual(received, [incoming]);
    assert.deepEqual(sent, [action]);
    assert.equal(errors.length, 1);
    assert.equal(errors[0][0], "A2UI outbound message error");
  } finally {
    console.error = originalConsoleError;
  }
});
