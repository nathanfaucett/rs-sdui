# Browser example

The two application sides are separate:

- `rust/`: the Cargo crate, declarative A2UI application, native JSON stream, and WASM adapter.
- `typescript/`: Vite, TypeScript, the official A2UI renderer, module SharedWorker, transport adapters, and integration tests.

## Shared state and transport

The entire todo application runs once in `src/application-worker.ts`. The worker owns the Rust/WASM `Application`, serializes every action and async effect, and broadcasts ordered standard messages to all connections. A late connection receives `Application::snapshot()` before joining broadcasts, so it sees current state and components.

The single page mounts Renderer A and Renderer B side by side. Add, edit, complete, delete, filter, or clear tasks in either renderer; the other receives the same updates. Normal A2UI input edits remain renderer-local until an action sends their resolved context. The same worker also synchronizes separate tabs.

`src/transport.ts` defines the application-owned `Transport` and `Connection` interfaces plus the shared `mountA2ui` host. Every connection exposes inbound messages as an `AsyncIterable` and accepts outbound renderer messages through `send()`. `SharedWorkerTransport` connects the todo app. `LocalStreamTransport` connects the deterministic agent stream. The adapters carry only standard A2UI messages. Replace an adapter to use WebSocket, SSE, HTTP, or another transport without changing the Rust application or renderer host.

Reconnect with a fresh `mountA2ui` and message processor. The new connection must receive `Application::snapshot()` before new updates. The official processor rejects `createSurface` for an existing surface ID, so do not send a snapshot into the old processor. Each renderer's **Reconnect** button demonstrates this while the other renderer keeps the worker alive. Durable transports can replay missed sequence numbers instead; snapshot recovery is the default. Do not blindly replay outbound actions because their side effects can already have run. If the application owner also stops, its in-memory state is gone; recovery then requires application-owned persistence. A full browser restart stops the local worker and starts a new todo application.

The todo page at `/` is a complete todo list: add, edit, complete/reopen, delete, filter, and clear completed tasks. Every action returns new serializable Rust state and standard A2UI updates. The subtitle uses `formatString` to display `/filter` without replacing its Text component. A borderless button opens the A2UI documentation through `openUrl`.

Draft and edit TextFields and their Add/Save buttons use `regex("\\S", path)` checks. Empty or whitespace-only titles disable these actions immediately. Rust still validates and trims every submitted title.

The initial empty state includes **Load example tasks**. This uses `on_effect` and browser `fetch` to read `typescript/public/todos.json` asynchronously. The loader supports both Window and WorkerGlobalScope, so it works inside the shared application worker. Native tests read the same asset. The effect appends validated tasks and returns the next state through the normal A2UI update loop.

`stream.test.mjs` streams a deterministic Berlin-to-San-Francisco trip planner through `LocalStreamTransport`. Seven standard v1 envelopes arrive 300 ms apart: an empty surface, scoped data, a root with unresolved child references, then component batches. The official processor validates the stream and its scoped passenger and total action updates. This is a protocol integration test, not a second HTML page. No LLM or API key is required.

## Run the browser

```sh
cd typescript
pnpm install
pnpm run dev
```

`pnpm run dev` builds `rust/` into `typescript/src/pkg/` and starts the named `a2ui-browser` portless route. Portless supplies a dynamic `PORT` to Vite and prints the local `.localhost` URL.

## Validate

```sh
cd typescript
pnpm test       # Rust state loop and deterministic async stream accepted by official schemas/processor
pnpm run check  # WASM build, TypeScript, and Biome checks
pnpm run build  # production WASM and Vite build
```

From the workspace root, run the native JSON stream with:

```sh
cargo run -p a2ui-browser-example
```
