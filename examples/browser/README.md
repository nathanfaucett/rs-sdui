# Browser example

The two application sides are separate:

- `rust/`: the Cargo crate, declarative A2UI application, native JSON stream, and WASM adapter.
- `typescript/`: Vite, TypeScript, the official A2UI renderer, and integration tests.

The example is a complete todo list: add, edit, complete/reopen, delete, filter, and clear completed tasks. Every action returns new serializable Rust state and standard A2UI updates.

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
pnpm test       # native Rust messages accepted by the official MessageProcessor
pnpm run check  # WASM build, TypeScript, and Biome checks
pnpm run build  # production WASM and Vite build
```

From the workspace root, run the native JSON stream with:

```sh
cargo run -p a2ui-browser-example
```
