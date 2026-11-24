# Browser example

A complete todo application using one Rust/WASM application and two side-by-side official Lit renderers.

- `rust/`: the Cargo crate, declarative A2UI application, native JSON stream, and WASM adapter.
- `src/`: the small Vite/TypeScript application, Lit renderer adapter, and worker entry.
- `../web`: the renderer-independent `@rs-a2ui/web` transport and worker helpers.

## Shared state and transport

`src/application-worker.ts` supplies the generated WASM module to `startApplicationWorker` from `@rs-a2ui/web/worker`. The helper initializes the application, serializes actions and async effects, broadcasts ordered standard messages, and sends `Application::snapshot()` to late connections.

The single page mounts Renderer A and Renderer B side by side. Add, edit, complete, delete, filter, or clear tasks in either renderer; the other receives the same updates. Normal A2UI input edits remain renderer-local until an action sends their resolved context. The same worker also synchronizes separate tabs.

`@rs-a2ui/web` has no rendering dependency. This example owns `src/mount-a2ui.ts`, which connects its `AsyncIterable` transport to the official Lit `MessageProcessor` and `<a2ui-surface>`. Replace that adapter to use another rendering library without changing the Rust application or worker transport.

Each renderer's **Go offline**/**Go online** toggle controls its own connection. Offline renderers keep their last UI and miss broadcasts. Going online creates a fresh `mountA2ui` and message processor before the new connection receives `Application::snapshot()`. The official processor rejects `createSurface` for an existing surface ID, so do not send a snapshot into the old processor. The other renderer keeps the worker alive during this test. Durable transports can replay missed sequence numbers instead; snapshot recovery is the default. Do not blindly replay outbound actions because their side effects can already have run.

If the application owner also stops, its in-memory state is gone; recovery then requires application-owned persistence. A full browser restart stops the local worker and starts a new todo application.

## Todo behavior

The page implements add, edit, complete/reopen, delete, filter, clear completed, and async example loading. The subtitle uses `formatString`; the documentation button uses `openUrl`. Draft and edit TextFields and their Add/Save buttons use `regex("\\S", path)` checks. Rust still validates and trims every submitted title.

## Run

From the repository root:

```sh
pnpm install
pnpm run dev
```

The command builds `@rs-a2ui/web`, builds `rust/` into `src/pkg/`, and starts the named `a2ui-browser` portless route. Portless supplies a dynamic `PORT` to Vite and prints the local `.localhost` URL.

## Validate

From the repository root:

```sh
pnpm run check   # helper build, WASM, TypeScript, and Biome
pnpm run build   # helper, WASM, and production Vite build
```

Run the native JSON stream with:

```sh
cargo run -p a2ui-browser-example
```
