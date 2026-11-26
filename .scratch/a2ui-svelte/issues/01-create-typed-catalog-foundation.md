# 01: Create the Zod 4 catalog foundation

**What to build:** Scaffold `@sdui/a2ui-svelte` with Zod 4 authoring helpers, A2UI JSON Schema emission, runtime JSON loading, view attachment, property mappings, and deterministic output. Prove the complete authoring-to-runtime path with one small catalog and a placeholder view.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Package exports separate Svelte, Zod 4 authoring, runtime-catalog, and Node-only generation entry points.
- [ ] Zod `4.6.7` is pinned for development; no project-owned authoring code imports Zod 3.
- [ ] Narrow Zod 4 wire helpers cover bindings, calls, dynamic values, children, actions, and checks without accidental `any`.
- [ ] `defineComponent` preserves exact schema types while pairing a Svelte view with each definition.
- [ ] The emitter handles external references, mixins, local definitions, component/function unions, conditional schemas, metadata, and deterministic ordering.
- [ ] Unsupported Zod constructs fail with useful paths instead of silent schema widening.
- [ ] `Catalog.fromJson()` creates SDK runtime schemas; `copyLoadedDefinition` preserves documents when views are attached.
- [ ] Runtime function adaptation produces executable official `FunctionImplementation` entries without changing emitted JSON.
- [ ] Wire and view property types pass positive, negative, and recursive no-`any` compile-time tests.
- [ ] A small custom catalog proves generation, runtime rendering, actions, setters, and JSON round trips.
- [ ] Root and runtime entry points contain no Zod 4, Node filesystem, or authoring-only imports.
- [ ] Package documentation explains the Zod 4 authoring boundary and the SDK’s internal Zod 3 runtime role.
