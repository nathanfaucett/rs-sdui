import assert from "node:assert/strict";
import { test } from "node:test";
import { SharedWorkerTransport } from "../dist/shared-worker-transport.js";
import { TransportError } from "../dist/transport.js";

class MockMessagePort {
  closed = false;
  listeners = new Map();

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) ?? new Set();
    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type, listener) {
    this.listeners.get(type)?.delete(listener);
  }

  start() {}

  close() {
    this.closed = true;
  }

  postMessage() {}
}

class MockSharedWorker {
  port = new MockMessagePort();
  listeners = new Map();

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) ?? new Set();
    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type, listener) {
    this.listeners.get(type)?.delete(listener);
  }

  emit(type, event) {
    for (const listener of [...(this.listeners.get(type) ?? [])]) listener(event);
  }
}

test("rejects the message stream and closes the port after a worker failure", async () => {
  const worker = new MockSharedWorker();
  const connection = new SharedWorkerTransport(() => worker).connect();
  const done = (async () => {
    for await (const _message of connection.messages) {
      // No messages are expected.
    }
  })();
  const errorEvent = { message: "worker failed", filename: "worker.js", lineno: 10 };

  worker.emit("error", errorEvent);

  await assert.rejects(done, (error) => {
    assert.ok(error instanceof TransportError);
    assert.equal(error.transport, "shared-worker");
    assert.equal(error.operation, "runtime");
    assert.equal(error.cause, errorEvent);
    return true;
  });
  assert.equal(worker.port.closed, true);
  assert.equal(worker.listeners.get("error").size, 0);
});
