# 02: Build the Svelte surface renderer

**What to build:** Implement `A2uiSurface` and `A2uiNode` over official `MessageProcessor`, `NodeResolver`, and signal APIs. Consume view-extended catalogs loaded from generated JSON, convert resolved node properties into typed Svelte view properties, and manage the complete lifecycle.

**Blocked by:** 01: Create the typed catalog foundation

**Status:** ready-for-agent

- [ ] `A2uiSurface` accepts a `SurfaceModel | undefined`, creates one resolver, and disposes it on replacement or unmount.
- [ ] `A2uiNode` selects `node.impl.view`, keys by `node.id`, and handles pending, unknown, cyclic, and disposed states.
- [ ] Resolved bindings become values, writable bindings add optional setters, and read-only bindings add no setters.
- [ ] Single, list, and nested child references remain live `ComponentNode` values and render recursively.
- [ ] Actions and validation results pass through without another binder or protocol implementation.
- [ ] Svelte subscriptions use `createSubscriber` without changing the SDK global signal implementation.
- [ ] Node and surface replacement release only adapter-owned resources; resolver ownership remains correct.
- [ ] Two catalogs can use the same component name without dispatch collisions.
- [ ] View attachment preserves every loaded JSON definition through `copyLoadedDefinition` and `toJson()`.
- [ ] Tests cover inbound updates, setters, actions, templates, relative paths, reconnection, and cleanup.
