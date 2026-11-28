# 08: Persist the application library

**What to build:** A host-owned application library stores installed backend definitions, generated JavaScript module source, interface revisions, selected revision IDs, and workspace configuration. Installed applications and their selected interfaces survive a browser reload, and uninstall removes their records and windows.

**Blocked by:** 03: Add and remove WASM backend definitions; 07: Apply editable A2UI interface revisions

**Status:** ready-for-agent

- [ ] Durable browser storage keeps application definitions, generated JavaScript source, immutable interface revisions, selected revision IDs, ID counters, and non-secret workspace configuration.
- [ ] The host loads and validates durable configuration before root initialization and reconciliation.
- [ ] An application-owned management bootstrap supplies restored configuration to the root worker before its first authoritative root snapshot; it does not replay A2UI actions or change the child `WorkerApplication` contract.
- [ ] Host reconciliation waits for the restored root snapshot, preventing default initial state from creating or removing mounts.
- [ ] Invalid durable data produces a contained recovery error and no partial reconciliation; the user can reset the damaged library.
- [ ] The backend definition ID remains the stable application identity across reloads and transient module URL changes.
- [ ] Installing a same-origin WASM backend by module URL preserves deferred loading: installation alone starts no worker.
- [ ] A fixture-generated JavaScript module satisfying `BackendModule` can be installed from stored source and opened in the generic backend worker without a compiler or application server.
- [ ] Stored generated source receives one stable transient module URL per loaded module revision so matching backend and session IDs share a runtime during that page lifetime.
- [ ] Transient blob or module URLs are not persisted as application identity or durable package location.
- [ ] Reload restores installed applications, workspace windows, ID counters, and selected interface revisions without duplicate IDs, mounts, or connections; runtime snapshots follow existing worker availability rules.
- [ ] Uninstall first closes and disposes all associated windows, then removes the definition, generated source, and interface revisions without affecting another application.
- [ ] Uninstall does not claim to terminate an already running SharedWorker or delete application-owned remote state.
- [ ] Runtime-state persistence remains application-owned and is not implied by application-library persistence.
- [ ] Provider credentials, model streams, transport resources, and DOM resources are never stored in the application library.
- [ ] Tests cover storage round trips, pre-snapshot root restoration, invalid-data recovery, generated-source loading, stable identity and counters, shared runtime keys, reload reconciliation, uninstall cleanup, and isolation between applications.
- [ ] Production Playwriter validation proves install, open, interface selection, browser reload, reopen, and uninstall from the static build with no application server.
