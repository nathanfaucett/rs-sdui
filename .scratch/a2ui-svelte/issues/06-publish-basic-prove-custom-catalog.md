# 06: Publish Basic and prove a custom catalog

**What to build:** Complete `@sdui/a2ui-svelte-basic`, generate its full catalog and compiled styles, verify all official functions, and demonstrate an independent application-owned catalog in a mixed surface.

**Blocked by:** 03: Implement display and layout components; 04: Implement actions, tabs, and modal; 05: Implement input and selection components

**Status:** ready-for-agent

- [ ] All 18 component definitions and all 14 Basic function declarations are present.
- [ ] Basic function execution comes from official SDK implementations, including locale-aware behavior and `openUrl` protections.
- [ ] Zod 4 generation emits `catalog.json` that deep-equals the official fixture and official `basicCatalog.toJson()`.
- [ ] Generation is deterministic, checked into Git, and included in the published package.
- [ ] View-extended runtime catalogs preserve the generated document through `toJson()`.
- [ ] The package contains Svelte sources, declarations, component subpath exports, catalog JSON, and compiled Tailwind utilities.
- [ ] Consumers do not need to scan `node_modules` for component classes.
- [ ] A custom catalog with a new Zod 4 schema, function, and Svelte view generates its own JSON and renders beside Basic.
- [ ] Duplicate component names across catalogs resolve by implementation identity, not by a global name.
- [ ] `Catalog.fromJson` round-trip and Rust `sdui-codegen` acceptance tests pass.
- [ ] `pnpm pack` inspection confirms that tests, fixtures, Zod 4, and Node-only generator code do not enter browser entry points.
