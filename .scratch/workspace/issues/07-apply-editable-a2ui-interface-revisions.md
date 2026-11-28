# 07: Apply editable A2UI interface revisions

**What to build:** A user can preview, apply, reopen, and roll back a fixture-created A2UI interface revision for an existing Todo backend. The host projects the selected catalog and components over the backend's current data and actions. This proves the editable-interface mechanism without an LLM or durable browser storage.

**Blocked by:** 04: Share and recover runtime sessions; 05: Reorder windows without reloading applications

**Status:** ready-for-agent

- [ ] The host owns immutable fixture interface revisions; root workspace state selects revisions by stable ID through standard workspace actions.
- [ ] A revision contains a backend ID, surface ID, catalog ID, and standard A2UI components, with no current application data or custom A2UI fields.
- [ ] On connection, projection combines the backend surface ID and current `dataModel` with the selected revision's catalog and components in a fresh child processor.
- [ ] Official A2UI SDK types and validation are used. The host does not invent or rewrite surface IDs, component IDs, data paths, event names, or action context.
- [ ] Later backend `updateDataModel` messages update the projected interface, while backend `createSurface` and `updateComponents` refresh the fallback contract without restoring the fallback view; `deleteSurface` keeps its standard behavior.
- [ ] Later fallback updates revalidate exposed actions and bindings. An incompatible change blocks affected projected actions and offers rollback without sending an action to a different positional component.
- [ ] The fixture uses stable non-list Todo controls and data bindings only. It does not project Todo's positional row components or row actions.
- [ ] Every Todo action exposed by the fixture routes unchanged to the backend and preserves its stable source component ID, event name, context, and state bindings.
- [ ] Validation uses a conservative whitelist from current fallback actions, fallback component bindings, stable component IDs, current data paths, and any app-supplied schema; relative template bindings are scoped to their declared list path, and unknown contracts and unstable positional row overrides are rejected.
- [ ] Invalid catalogs, components, surface IDs, state paths, or action contracts are rejected without changing the live interface.
- [ ] Preview uses an ephemeral mount with an explicit draft override and a separate preview session ID and runtime key; it does not select the draft revision for live windows.
- [ ] Closing preview disposes its mount, connection, processor, listeners, and ephemeral configuration exactly once without changing live windows.
- [ ] Apply and rollback may create fresh processors and connections, but they preserve the runtime key, SharedWorker session, and current todos and never replay outbound actions.
- [ ] Closing and reopening a window projects the selected revision over the latest shared-session snapshot.
- [ ] After apply, two ordinary windows with the selected revision continue to synchronize Todo data through their projected actions, while a different session remains independent.
- [ ] Renderer-local drafts may reset during apply or rollback; window reorder behavior from ticket 05 remains unchanged.
- [ ] Focused TypeScript tests and production Playwriter validation cover projection, data updates, exposed actions, invalid and unstable contracts, later contract incompatibility, preview isolation and cleanup, apply, reopen, rollback, and runtime preservation.
- [ ] No LLM, custom A2UI message, backend recompilation, durable browser-reload persistence, or application server is introduced.
