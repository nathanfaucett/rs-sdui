# 11: Edit application interfaces with an LLM

**What to build:** A user can describe an interface change for an installed application. The LLM maps the current application data and actions to a revised standard A2UI interface, which uses the existing validation, isolated preview, apply, persistence, and rollback path.

**Blocked by:** 08: Persist the application library; 09: Add the LLM provider core; 10: Generate JavaScript applications with an LLM

**Status:** ready-for-agent

- [ ] Authoring context includes the user prompt, current selected interface or backend fallback surface, current `dataModel`, supported A2UI version, and selected catalog schemas.
- [ ] The host-only authoring draft contains an editable A2UI interface, not executable JavaScript or a second interface-description language.
- [ ] UI-only validation preserves the surface ID and every exposed action's source component ID, event name, context, and state bindings.
- [ ] Validation uses a conservative whitelist from current fallback actions, fallback component bindings, stable component IDs, current data paths, and any app-supplied schema; relative template bindings are scoped to their declared list path, and unknown contracts and unstable positional component overrides are rejected.
- [ ] State bindings use paths known from the current data or an app-supplied schema; validation and preview do not treat the current snapshot as a complete schema for empty collections or optional fields.
- [ ] Dynamic list edits require stable A2UI child-template component IDs and relative item bindings in both the backend fallback view and draft revision.
- [ ] Invalid catalogs, components, paths, surface IDs, or action contracts are rejected without changing the live interface.
- [ ] Preview uses an ephemeral mount with an explicit draft override and a separate preview session ID and runtime key; it does not change the selected revision for live windows, and closing it disposes all preview resources and ephemeral configuration.
- [ ] Apply uses ticket 07's projection path, preserves the backend runtime key and current session data, and never replays outbound actions.
- [ ] Shared-session windows continue to synchronize data after apply, and independent sessions remain isolated.
- [ ] Later fallback component updates revalidate exposed actions and bindings; an incompatible change blocks affected projected actions and offers rollback without sending an action to a different positional component.
- [ ] Close/reopen and browser reload restore the selected persisted revision over the latest available backend snapshot.
- [ ] Rollback restores the previous interface without intentionally resetting backend state.
- [ ] A request for new behavior or authoritative state requires the generated-module revision path instead of silently treating it as a UI-only edit.
- [ ] Malformed model output, cancellation, and provider failure leave the live interface and installed application unchanged.
- [ ] Deterministic fake-provider tests cover layout and label edits, existing bindings and actions, invalid output, preview isolation, apply, shared synchronization, persistence, and rollback.
- [ ] Production Playwriter validation demonstrates fake-provider editing of Todo and a generated application with no application server or custom A2UI messages.
