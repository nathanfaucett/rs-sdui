# `@sdui/web`

Renderer-independent browser transport and SharedWorker synchronization for standard A2UI v1.0 messages.

This package does not mount components, create renderer elements, or depend on a UI framework. Use React, Svelte, Lit, native elements, or another A2UI-compatible renderer. The example owns its Lit adapter in `packages/browser-example/src/mount-a2ui.ts`.

## Transport

Every connection exposes inbound messages as an `AsyncIterable` and accepts standard outbound renderer messages:

```ts
import { SharedWorkerTransport } from "@sdui/web";

const transport = new SharedWorkerTransport(
  () =>
    new SharedWorker(new URL("./application-worker.ts", import.meta.url), {
      type: "module",
      name: "my-application",
    }),
);
const connection = transport.connect();

for await (const message of connection.messages) {
  renderer.processMessages(message);
}

connection.send(rendererMessage);
```

`WebSocketTransport` connects the same contract to a standard WebSocket. Each text frame contains one standard A2UI JSON message:

```ts
import { WebSocketTransport } from "@sdui/web";

const transport = new WebSocketTransport(
  "wss://example.com/ws?session=my-session",
);
const connection = transport.connect();
```

Use the URL for application session routing. Do not add custom fields to A2UI messages.

`SseTransport` receives standard messages through server-sent events and sends renderer messages through HTTP POST:

```ts
import { SseTransport } from "@sdui/web/sse";

const transport = new SseTransport("/a2ui/events?session=my-session", {
  sendUrl: "/a2ui/actions?session=my-session",
});
const connection = transport.connect();
```

`sendUrl` defaults to the event URL. Each default SSE `message` event must contain one standard A2UI JSON message in `data`. The server must accept each POST body as one standard renderer-to-agent JSON message, return a successful HTTP status, and publish agent messages to the event stream. POST response bodies are ignored.

Outbound POSTs wait for the SSE connection to open and execute in send order. An SSE or POST failure closes the connection. Create a fresh connection and renderer before reconnecting; do not let an automatic SSE reconnect send a snapshot to an old renderer. Outbound messages are not retried because their effects may already have run.

`LocalStreamTransport` provides the contract for deterministic local streams. Add adapters for HTTP or another application-owned transport without changing message shapes.

### Tree shaking

The package is side-effect-free. Root named imports and these transport entry points support tree shaking:

- `@sdui/web/local`
- `@sdui/web/shared-worker`
- `@sdui/web/sse`
- `@sdui/web/websocket`
- `@sdui/web/worker`

Keep future AG-UI, A2A, MCP, or other SDK-dependent adapters in transport-specific entry points with optional peer dependencies. Do not re-export those adapters from the root entry point.

## SharedWorker application

`@sdui/web/worker` owns initialization, action serialization, ordered broadcasts, late-client snapshots, and port cleanup. Supply the generated WASM module:

```ts
import { startApplicationWorker } from "@sdui/web/worker";
import init, * as application from "./pkg/browser";

startApplicationWorker({ application, initialize: init });
```

The `sdui` Rust crate owns the `wasm_bindgen` lifecycle glue on WASM targets. Declare the application constructor and state type in the WASM crate:

```rust
sdui::export_application!(crate::application, crate::TodoApp);
```

The invoking Rust crate must depend directly on `wasm-bindgen`. The macro exports `initial_messages()`, `snapshot_messages()`, and async `handle_action(input)`. These functions exchange JSON strings containing only standard A2UI messages. TypeScript owns worker synchronization; Rust owns application execution and FFI.

## Reconnection

After a broken connection, create a new connection and fresh renderer state. The example worker and WebSocket server send a complete snapshot before new updates. Do not apply `createSurface` to a renderer that still has that surface. Durable transports can replay missed sequence numbers instead. Do not blindly replay outbound actions because their side effects can already have run.

If the application owner stops, snapshot recovery is unavailable until the application restores its state. Persistence remains application-owned.

## Scope

The package provides no renderer, persistence, executor, catalog negotiation, or custom A2UI envelopes. Its only official SDK dependency is `@a2ui/web_core` for protocol types and schema validation.
