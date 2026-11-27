# SDUI coding standards

Use this file during review. These durable rules preserve the workspace architecture and prevent known integration failures. `AGENTS.md` owns validation procedures.

## Library and API boundaries

- Put library crates under `crates/`, reusable browser and npm packages under `packages/`, and applications under `apps/`.
- Keep `lib.rs` focused on module declarations and project-owned exports.
- Do not re-export external crates.
- Use standard A2UI v1.0 messages. Do not add custom envelopes, message fields, compatibility transforms, or parallel protocols.
- Keep transport independent of application state and rendering. Session routing belongs in transport URLs or worker names, not A2UI messages.
- Keep `packages/web` renderer-independent. Applications choose and mount their own renderer.
- Keep every transport tree-shakable. Use side-effect-free modules and transport-specific subpath exports. Keep SDK-dependent adapters out of the root entry point and declare their SDKs as optional peer dependencies.
- Use generated catalog builders and typed state bindings. Application code must not manually construct component IDs, data paths, component lists, or surfaces.
- Declarative events return the next state. Put I/O and other side effects in effects.
- Use official A2UI SDKs and existing maintained libraries instead of implementing their behavior again.

## Browser code

- Style browser application components with Tailwind utilities.
- Keep worker session identity in the SharedWorker name. Keep WebSocket session identity in the URL.
- Fix bundle warnings through meaningful package or module boundaries. Do not raise `chunkSizeWarningLimit` to hide them.
- Keep tests focused on workspace behavior. Do not retest official SDK internals.

## Dependencies

- Prefer versions compatible with the existing workspace and its minimum Rust version.
- Resolve duplicate dependency branches through compatible versions. Do not suppress `multiple_crate_versions` or add blanket Clippy allowances.
- Keep WASM-related crates on matching releases. `wasm-bindgen`, `js-sys`, `web-sys`, and `wasm-bindgen-futures` have linked exact requirements.

## Review completion

- Update documentation when behavior, commands, architecture, or constraints change.
- Complete and report validation through the workflow in `AGENTS.md`.
