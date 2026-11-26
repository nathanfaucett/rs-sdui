# 03: Add and remove WASM backend definitions

**What to build:** A user can add a same-origin WASM backend by display name and JavaScript module URL, then remove backend definitions. Adding configuration starts no application. Removing a definition closes every window that uses it.

**Blocked by:** 02: Open a dynamically loaded Todo window

**Status:** ready-for-agent

- [ ] Root SDUI inputs and actions support backend names and module URLs without host-only controls or custom A2UI fields.
- [ ] Backend and window IDs are stable and deterministic across pure root transitions.
- [ ] Empty names, malformed URLs, and non-same-origin module URLs are rejected with useful validation behavior.
- [ ] Adding a valid backend definition updates the workspace without loading its module or starting a worker.
- [ ] A user can add the reusable Todo package by its module URL and open it successfully.
- [ ] Removing a backend first removes all its desired windows, then removes the definition.
- [ ] Host reconciliation closes and disposes every removed window and removes all associated surface slots exactly once.
- [ ] Removing one backend does not change another backend's definitions, windows, connections, or state.
- [ ] Rust transition tests, host reconciliation tests, and production browser validation cover add, deferred loading, cascade removal, and isolation.
