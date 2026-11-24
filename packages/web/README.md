# `@rs-a2ui/web`

Renderer-independent browser transport and SharedWorker synchronization for standard A2UI v1.0 messages.

This package does not mount components, create renderer elements, or depend on a UI framework. Use React, Svelte, Lit, native elements, or another A2UI-compatible renderer. The example owns its Lit adapter in `packages/browser-example/src/mount-a2ui.ts`.

## Transport

Every connection exposes inbound messages as an `AsyncIterable` and accepts standard outbound renderer messages:

```ts
import { SharedWorkerTransport } from "@rs-a2ui/web";

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

`LocalStreamTransport` provides the same contract for deterministic local streams. Replace the adapter for WebSocket, SSE, HTTP, or another application-owned transport. A2UI message shapes do not change.

## SharedWorker application

`@rs-a2ui/web/worker` owns initialization, action serialization, ordered broadcasts, late-client snapshots, and port cleanup. Supply the generated WASM module:

```ts
import { startApplicationWorker } from "@rs-a2ui/web/worker";
import init, * as application from "./pkg/browser";

startApplicationWorker({ application, initialize: init });
```

The companion `a2ui-wasm` Rust crate owns the `wasm_bindgen` lifecycle glue. Declare the application constructor and state type in the WASM crate:

```rust
a2ui_wasm::export_application!(crate::application, crate::TodoApp);
```

The invoking Rust crate must depend directly on `wasm-bindgen`. The macro exports `initial_messages()`, `snapshot_messages()`, and async `handle_action(input)`. These functions exchange JSON strings containing only standard A2UI messages. TypeScript owns worker synchronization; Rust owns application execution and FFI.

## Reconnection

After a broken connection, create a new connection and fresh renderer state. The worker sends a complete snapshot before new updates. Do not apply `createSurface` to a renderer that still has that surface. Durable transports can replay missed sequence numbers instead. Do not blindly replay outbound actions because their side effects can already have run.

If the application owner stops, snapshot recovery is unavailable until the application restores its state. Persistence remains application-owned.

## Scope

The package provides no renderer, network transport, persistence, executor, catalog negotiation, or custom A2UI envelopes. Its only official SDK dependency is `@a2ui/web_core` for protocol types and schema validation.
