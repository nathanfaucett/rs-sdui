# 01: Extract a reusable Todo WASM backend

**What to build:** The existing Todo Rust application becomes an independently buildable, server-free WASM backend. The existing browser example consumes the same application without behavior changes. This prepares one real backend that both the browser example and workspace can use without duplication.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Todo state, actions, views, Basic Catalog generation, and WASM adaptation are shared without copying their implementations.
- [ ] Native binaries, server integration, and browser-page integration are separate from the reusable Todo backend.
- [ ] The Todo backend builds independently into web-target JavaScript and WASM assets that satisfy the existing worker application contract.
- [ ] The backend package contains no native application server or server startup requirement.
- [ ] The existing browser example consumes the shared Todo application and preserves its current native JSON, WebSocket, and WASM behavior.
- [ ] Existing Todo and browser-example tests pass after the extraction.
- [ ] Relevant Rust and browser validation confirms that the independently built backend assets execute successfully.
