# Browser example

The two application sides are separate:

- `rust/`: the Cargo crate, declarative A2UI application, native JSON stream, and WASM adapter.
- `typescript/`: Vite, TypeScript, the official A2UI renderer, and integration tests.

## Run the browser

```sh
cd typescript
npm install
npm run dev
```

`npm run dev` builds `rust/` into `typescript/src/pkg/` before starting Vite.

## Validate

```sh
cd typescript
npm test       # native Rust messages accepted by the official MessageProcessor
npm run check  # WASM build and TypeScript check
npm run build  # production WASM and Vite build
```

From the workspace root, run the native JSON stream with:

```sh
cargo run -p a2ui-browser-example
```
