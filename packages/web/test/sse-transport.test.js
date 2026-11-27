import assert from "node:assert/strict";
import { test } from "node:test";
import { SseTransport } from "../dist/sse-transport.js";
import { TransportError } from "../dist/transport.js";

class MockEventSource {
  static instances = [];

  constructor(url, eventSourceInit) {
    this.url = String(url);
    this.eventSourceInit = eventSourceInit;
    this.closed = false;
    MockEventSource.instances.push(this);
  }

  close() {
    this.closed = true;
  }

  emitOpen() {
    this.onopen?.({});
  }

  emitMessage(data) {
    this.onmessage?.({ data });
  }

  emitError() {
    this.onerror?.({});
  }
}

async function withMocks(fetchImplementation, run) {
  const originalEventSource = globalThis.EventSource;
  const originalFetch = globalThis.fetch;
  const originalConsoleError = console.error;
  const errors = [];
  MockEventSource.instances = [];
  globalThis.EventSource = MockEventSource;
  globalThis.fetch = fetchImplementation;
  console.error = (...args) => errors.push(args);

  try {
    await run(errors);
  } finally {
    if (originalEventSource === undefined) {
      Reflect.deleteProperty(globalThis, "EventSource");
    } else {
      globalThis.EventSource = originalEventSource;
    }
    globalThis.fetch = originalFetch;
    console.error = originalConsoleError;
  }
}

async function waitFor(predicate) {
  for (let attempt = 0; attempt < 20 && !predicate(); attempt += 1) {
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  assert.ok(predicate());
}

test("receives standard A2UI messages", async () => {
  await withMocks(
    async () => {
      throw new Error("unexpected fetch");
    },
    async () => {
      const transport = new SseTransport("/a2ui?session=test", {
        eventSourceInit: { withCredentials: true },
      });
      const connection = transport.connect();
      const source = MockEventSource.instances[0];
      const expected = {
        version: "1.0",
        updateDataModel: { surfaceId: "main", path: "/count", value: 1 },
      };
      const received = [];
      const done = (async () => {
        for await (const message of connection.messages) {
          received.push(message);
          connection.close();
        }
      })();

      source.emitMessage(JSON.stringify(expected));
      await done;

      assert.equal(source.url, "/a2ui?session=test");
      assert.equal(source.eventSourceInit.withCredentials, true);
      assert.equal(source.closed, true);
      assert.deepEqual(received, [expected]);
    }
  );
});

test("waits for SSE open and preserves outbound POST order", async () => {
  let releaseFirst;
  const firstResponse = new Promise((resolve) => {
    releaseFirst = resolve;
  });
  const calls = [];

  await withMocks(
    async (url, init) => {
      calls.push({ url: String(url), init });
      if (calls.length === 1) await firstResponse;
      return { ok: true, status: 204 };
    },
    async () => {
      const transport = new SseTransport("/events", {
        sendUrl: "/actions",
        requestInit: { headers: { authorization: "test-token" } },
      });
      const connection = transport.connect();
      const source = MockEventSource.instances[0];

      connection.send({ action: { name: "first" } });
      connection.send({ action: { name: "second" } });
      assert.equal(calls.length, 0);

      source.emitOpen();
      await waitFor(() => calls.length === 1);
      assert.equal(calls.length, 1);
      releaseFirst();
      await waitFor(() => calls.length === 2);

      assert.deepEqual(
        calls.map(({ init }) => JSON.parse(init.body).action.name),
        ["first", "second"]
      );
      assert.equal(calls[0].url, "/actions");
      assert.equal(calls[0].init.method, "POST");
      assert.equal(calls[0].init.headers.get("content-type"), "application/json");
      assert.equal(calls[0].init.headers.get("authorization"), "test-token");
      connection.close();
    }
  );
});

test("rejects and closes instead of automatically reconnecting after an SSE error", async () => {
  await withMocks(
    async () => ({ ok: true, status: 204 }),
    async () => {
      const connection = new SseTransport("/events").connect();
      const source = MockEventSource.instances[0];
      const done = (async () => {
        for await (const _message of connection.messages) {
          // No messages are expected.
        }
      })();

      source.emitError();

      await assert.rejects(done, (error) => {
        assert.ok(error instanceof TransportError);
        assert.equal(error.transport, "sse");
        assert.equal(error.operation, "open");
        return true;
      });
      assert.equal(source.closed, true);
    }
  );
});

test("rejects and closes after a failed outbound POST", async () => {
  await withMocks(
    async () => ({ ok: false, status: 503 }),
    async () => {
      const connection = new SseTransport("/events").connect();
      const source = MockEventSource.instances[0];
      const done = (async () => {
        for await (const _message of connection.messages) {
          // No messages are expected.
        }
      })();

      source.emitOpen();
      connection.send({ action: { name: "test" } });

      await assert.rejects(done, (error) => {
        assert.ok(error instanceof TransportError);
        assert.equal(error.transport, "sse");
        assert.equal(error.operation, "send");
        assert.match(error.message, /503/);
        assert.equal(error.cause.status, 503);
        return true;
      });
      assert.equal(source.closed, true);
    }
  );
});
