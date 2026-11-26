# 07: Integrate the browser example and validate production

**What to build:** Add a Svelte renderer mount to the existing browser example, consume the generated Basic catalog in Rust, add a complete gallery fixture, and finish documentation and production validation.

**Blocked by:** 06: Publish Basic and prove a custom catalog

**Status:** ready-for-agent

- [ ] The existing Todo application can mount through Svelte without changing transport, worker, session, or Rust application behavior.
- [ ] `apps/browser-example/rust/build.rs` consumes `packages/a2ui-svelte-basic/catalog.json`; Cargo does not invoke pnpm.
- [ ] `BASIC-NOTICE` retains accurate upstream commit and Apache 2.0 attribution.
- [ ] A gallery fixture exercises every Basic component, function, check, binding, and action through standard A2UI messages.
- [ ] README files document installation, UI SDK cascade-layer setup, theme variables, Zod 4 authoring, generated-catalog commands, custom catalogs, runtime JSON loading, and reconnect lifecycle.
- [ ] Root README and package tables list the new packages and ownership boundaries.
- [ ] All required Rust, Clippy, TypeScript, Svelte, schema, and component checks pass sequentially without new warnings.
- [ ] Production assets include the application worker and browser WASM, and the Svelte bundle excludes Zod 4, authoring code, and official Lit Basic views.
- [ ] Production Playwriter checks two surfaces, synchronized and independent actions, the full gallery, keyboard flows, and reconnection.
- [ ] Both npm package archives contain the required catalog, declarations, Svelte sources, and styles, and preview processes are stopped after validation.
