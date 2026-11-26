# 05: Reorder windows without reloading applications

**What to build:** A user can move Todo windows earlier and later in the workspace. Movement changes only visual order. Todo state, unsubmitted drafts, renderer state, workers, processors, and connections remain intact.

**Blocked by:** 04: Share and recover runtime sessions

**Status:** ready-for-agent

- [ ] Root SDUI actions reorder windows through stable window IDs and handle first, last, and invalid movement requests safely.
- [ ] Moving a window preserves its Todo items and unsubmitted renderer-local draft.
- [ ] Moving a window does not create a worker, processor, or connection and does not trigger a snapshot or surface recreation.
- [ ] Surface slot attachment and detachment are idempotent during DOM movement.
- [ ] Slot disconnection caused by movement does not close the associated backend connection.
- [ ] Repeated root snapshots or equivalent data updates do not duplicate mounts or restart unchanged windows.
- [ ] Host reconciliation updates only placement for an unchanged window ID.
- [ ] Rust transition tests and TypeScript lifecycle tests prove stable ordering and resource identity.
- [ ] Production Playwriter validation proves that both Rust state and renderer-local drafts survive repeated movement in shared and independent sessions.
