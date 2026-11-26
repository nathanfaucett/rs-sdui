# 02: Open a dynamically loaded Todo window

**What to build:** A user can start the browser-only workspace with a preconfigured Todo backend, open a window, use the Todo application, and close the window. The root interface is SDUI, and the backend worker loads the Todo JavaScript and WASM package at runtime.

**Blocked by:** 01: Extract a reusable Todo WASM backend

**Status:** ready-for-agent

- [ ] The root Rust/WASM SDUI application owns the initial backend definition and open and close window transitions using standard A2UI messages.
- [ ] The root renderer combines the Basic Catalog with the application-owned shell catalog and mounts a child surface through a `SurfaceSlot` identified by a stable window ID.
- [ ] Opening a window creates a fresh child processor and connection, registers catalogs and listeners before connection, and mounts the first Todo surface without requiring a configured surface ID.
- [ ] The generic backend worker obtains its backend identity and module URL from management metadata in its SharedWorker name and dynamically imports the Todo module.
- [ ] Worker ports, snapshots, and actions wait correctly for asynchronous initialization without duplicating the existing worker synchronization implementation.
- [ ] A user can add and complete todos in the dynamically loaded window and then close it, which disposes its connection, processor, and slot.
- [ ] Adding the preconfigured backend definition alone does not start its backend worker.
- [ ] The production build loads separately addressable Todo JavaScript and WASM assets and does not statically import the Todo module into the host.
- [ ] Development, tests, and production preview require no application server or transport proxy.
- [ ] Focused Rust and TypeScript tests and production Playwriter validation cover the complete open, use, and close path.
