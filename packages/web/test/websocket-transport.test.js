import assert from "node:assert/strict";
import { test } from "node:test";
import { TransportError } from "../dist/transport.js";
import { webSocketConnection } from "../dist/websocket-transport.js";

class MockWebSocket {
  CONNECTING = 0;
  OPEN = 1;
  CLOSED = 3;
  readyState = this.CONNECTING;
  closed = false;
  sent = [];
  listeners = new Map();

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) ?? new Set();
    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type, listener) {
    this.listeners.get(type)?.delete(listener);
  }

  emit(type, event = {}) {
    for (const listener of [...(this.listeners.get(type) ?? [])]) listener(event);
  }

  listenerCount(type) {
    return this.listeners.get(type)?.size ?? 0;
  }

  send(message) {
    this.sent.push(message);
  }

  close() {
    this.closed = true;
    this.readyState = this.CLOSED;
  }
}

test("adapts an application-owned WebSocket and closes it with the connection", async () => {
  const socket = new MockWebSocket();
  const connection = webSocketConnection(socket);
  const received = [];
  const done = (async () => {
    for await (const message of connection.messages) {
      received.push(message);
      connection.close();
    }
  })();

  connection.send({ action: { name: "first" } });
  assert.deepEqual(socket.sent, []);

  socket.emit("open");
  assert.deepEqual(JSON.parse(socket.sent[0]), { action: { name: "first" } });

  const expected = {
    version: "1.0",
    updateDataModel: { surfaceId: "main", path: "/count", value: 1 },
  };
  socket.emit("message", { data: JSON.stringify(expected) });
  await done;

  assert.deepEqual(received, [expected]);
  assert.equal(socket.closed, true);
  assert.equal(socket.listenerCount("message"), 0);
});

test("rejects the message stream with clean remote close details", async () => {
  const socket = new MockWebSocket();
  const connection = webSocketConnection(socket);
  const done = (async () => {
    for await (const _message of connection.messages) {
      // No messages are expected.
    }
  })();
  const closeEvent = { wasClean: true, code: 1000, reason: "done" };

  socket.emit("close", closeEvent);

  await assert.rejects(done, (error) => {
    assert.ok(error instanceof TransportError);
    assert.equal(error.transport, "websocket");
    assert.equal(error.operation, "open");
    assert.equal(error.message, "A2UI WebSocket closed (1000): done");
    assert.equal(error.cause, closeEvent);
    return true;
  });
  assert.equal(socket.listenerCount("close"), 0);
});

test("rejects the message stream with WebSocket close details after a failure", async () => {
  const socket = new MockWebSocket();
  const connection = webSocketConnection(socket);
  const done = (async () => {
    for await (const _message of connection.messages) {
      // No messages are expected.
    }
  })();
  const closeEvent = { wasClean: false, code: 1006, reason: "network lost" };

  socket.emit("open");
  socket.emit("error", {});
  socket.emit("close", closeEvent);

  await assert.rejects(done, (error) => {
    assert.ok(error instanceof TransportError);
    assert.equal(error.transport, "websocket");
    assert.equal(error.operation, "receive");
    assert.equal(error.message, "A2UI WebSocket closed (1006): network lost");
    assert.equal(error.cause, closeEvent);
    return true;
  });
  assert.equal(socket.closed, true);
});

test("rejects the message stream when a WebSocket send fails", async () => {
  const socket = new MockWebSocket();
  const failure = new Error("send failed");
  socket.send = () => {
    throw failure;
  };
  const connection = webSocketConnection(socket);
  const done = (async () => {
    for await (const _message of connection.messages) {
      // No messages are expected.
    }
  })();

  socket.readyState = socket.OPEN;
  socket.emit("open");
  connection.send({ action: { name: "test" } });

  await assert.rejects(done, (error) => {
    assert.ok(error instanceof TransportError);
    assert.equal(error.transport, "websocket");
    assert.equal(error.operation, "send");
    assert.equal(error.cause, failure);
    return true;
  });
  assert.equal(socket.closed, true);
});

test("supports application-owned retries with fresh WebSocket connections", async () => {
  const expected = {
    version: "1.0",
    updateDataModel: { surfaceId: "main", path: "/count", value: 1 },
  };
  const received = [];

  for (let attempt = 1; attempt <= 3; attempt++) {
    const socket = new MockWebSocket();
    const connection = webSocketConnection(socket);
    const done = (async () => {
      for await (const message of connection.messages) {
        received.push(message);
        connection.close();
      }
    })();

    if (attempt < 3) {
      const closeEvent = {
        wasClean: false,
        code: 1006,
        reason: `failure ${attempt}`,
      };
      socket.emit("close", closeEvent);
      await assert.rejects(done, (error) => {
        assert.ok(error instanceof TransportError);
        assert.equal(error.cause, closeEvent);
        return true;
      });
      continue;
    }

    socket.emit("open");
    socket.emit("message", { data: JSON.stringify(expected) });
    await done;
  }

  assert.deepEqual(received, [expected]);
});
