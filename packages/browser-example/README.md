# Browser example

A complete todo application with configurable Rust application instances and official Lit renderers.

- `rust/`: the declarative application, native JSON stream, WebSocket session server, and WASM adapter.
- `src/`: the Vite/TypeScript page, Lit renderer adapter, and worker entry.
- `scripts/serve.ts`: starts the WebSocket server and Vite together.
- `../web`: renderer-independent processor connections and SharedWorker, WebSocket, and SSE transports.

## Instances and sessions

The page starts with two instances. **Add instance** creates another independent renderer and connection. Each instance selects:

- **WASM SharedWorker**: Rust executes in the browser.
- **Rust WebSocket**: Rust executes in the native example server.
- **Session ID**: defaults to `browser-example`.

Instances with the same backend and session ID share one application runtime and ordered updates. A different session ID creates independent state. Worker and WebSocket runtimes are separate even when their session IDs match. The default session therefore synchronizes the two initial worker instances immediately.

**Connect to session** applies backend or ID changes with a fresh renderer. **Remove** closes that instance's connection.

## Message flow and recovery

`src/application-worker.ts` supplies the generated WASM module to `startApplicationWorker`. The worker helper initializes one application per SharedWorker, serializes actions and async effects, broadcasts standard messages, and sends `Application::snapshot()` to late connections.

`rust/src/server.rs` provides the same behavior for native WebSocket sessions. It stores one started `Application` per session ID, serializes actions through a session lock, and broadcasts ordered standard messages. The session ID is in `/ws?session=<id>`; no custom A2UI fields or envelopes are used.

**Go offline** closes only that instance and preserves its stale UI. Changes made by other instances are missed. **Go online** creates a fresh message processor before receiving a full snapshot. Never apply a snapshot to the old processor because it rejects a duplicate `createSurface`. Do not blindly replay outbound actions because effects may already have run.

Server sessions remain in memory until the server stops. SharedWorker state lasts for the worker lifetime. Durable persistence remains application-owned.

`@sdui/web` has no rendering dependency. Its `connectMessageProcessor` helper owns outbound validation and the transport message loop. `src/mount-a2ui.ts` is the example-owned Lit adapter. Replace its surface mounting to use another renderer without changing either Rust runtime.

## Todo behavior

The application implements add, edit, complete/reopen, delete, filter, and clear completed. The subtitle uses `formatString`; the documentation button uses `openUrl`. Draft and edit TextFields use `regex("\\S", path)` checks. Rust validates and trims every submitted title.

## Run

From the repository root:

```sh
pnpm install
pnpm run dev
```

The command builds `@sdui/web` and WASM, reserves a dynamic internal WebSocket port, starts the Rust server, and starts the named `sdui-browser` portless route. Vite proxies same-origin `/ws` connections to Rust. Portless supplies the dynamic Vite `PORT` and prints the local `.localhost` URL.

Run only the native WebSocket server with:

```sh
pnpm --filter @sdui/browser-example run server
```

It listens on `127.0.0.1:8081` by default. Set `SERVER_WS_ADDR` to change the socket address.

Run the native JSON state-loop example with:

```sh
cargo run -p sdui-browser-example
```

## Validate

From the repository root:

```sh
cargo test -p sdui-browser-example
pnpm --filter @sdui/web test
pnpm run check
pnpm run build
```
