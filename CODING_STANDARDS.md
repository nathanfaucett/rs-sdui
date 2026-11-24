# SDUI coding standards

Use this file during review. These rules preserve the workspace architecture and prevent known integration failures.

## Library and API boundaries

- Put library crates under `crates/`. Keep browser and npm packages under `packages/`.
- Every `lib.rs` contains module declarations and project-owned exports only.
- Use `pub` or private visibility. Do not use scoped visibility such as `pub(crate)`.
- Do not re-export external crates.
- Use standard A2UI v1.0 messages. Do not add custom envelopes, message fields, compatibility transforms, or parallel protocols.
- Keep transport independent of application state and rendering. Session routing belongs in transport URLs or worker names, not A2UI messages.
- Keep `packages/web` renderer-independent. Applications choose and mount their own renderer.
- Use generated catalog builders and typed state bindings. Application code must not manually construct component IDs, data paths, component lists, or surfaces.
- Declarative events return the next state. Put I/O and other side effects in effects.
- Use official A2UI SDKs and existing maintained libraries instead of implementing their behavior again.

## Browser and production builds

- Use Tailwind utilities. Do not add custom component CSS.
- Preserve Vite's statically analyzable worker construction:

  ```ts
  new SharedWorker(new URL("./application-worker.ts", import.meta.url), options);
  ```

  Do not move the worker URL into a separate variable before passing it to the constructor. Vite can then emit the worker and its WASM assets.

- Keep worker session identity in the SharedWorker name. Keep WebSocket session identity in the URL.
- Fix bundle warnings through meaningful package or module boundaries. Do not raise `chunkSizeWarningLimit` to hide them.
- Validate worker and bundle changes from a production build. TypeScript, Biome, and `vite build` alone do not prove that the worker executes.
- After worker changes, inspect `dist/assets/` for the emitted worker JavaScript and WASM files.
- After shared-runtime changes, use production Playwriter validation to check two surfaces and one synchronized action.
- Use role and label selectors for input values. Do not use `getByText()` to locate text held by an input.
- Keep tests focused on workspace behavior. Do not retest official SDK internals.

## Dependencies

- Prefer versions compatible with the existing workspace and its minimum Rust version.
- Resolve duplicate dependency branches through compatible versions. Do not suppress `multiple_crate_versions` or add blanket Clippy allowances.
- Current constraint: `jsonschema 0.63.1` uses `syn 2` and `getrandom 0.3`. Reassess this constraint when upgrading `jsonschema`; do not preserve old pins blindly.
- After `cargo update`, check the complete workspace with Clippy. A lockfile can select a coherent older branch without changing public dependency ranges.
- Keep WASM-related crates on matching releases. `wasm-bindgen`, `js-sys`, `web-sys`, and `wasm-bindgen-futures` have linked exact requirements.

## Review completion

- Keep changes limited to the requested scope and preserve unrelated work.
- Update documentation when behavior, commands, architecture, or constraints change.
- Run the relevant commands from `AGENTS.md` sequentially.
- Report validation that actually ran. Do not hide new warnings or claim unrun browser validation.
