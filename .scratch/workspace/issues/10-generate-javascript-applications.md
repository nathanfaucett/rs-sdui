# 10: Generate JavaScript applications with an LLM

**What to build:** A user prompt can produce a draft JavaScript worker application and initial standard A2UI interface. After validation, isolated preview, and explicit consent, the application is installed through the persistent library and can open and handle actions without compilation or an application server.

**Blocked by:** 06: Contain backend loading failures; 08: Persist the application library; 09: Add the LLM provider core

**Status:** ready-for-agent

- [ ] A strict host-only authoring-draft schema contains JavaScript module source and an initial editable A2UI interface; it is not an A2UI envelope or child message.
- [ ] Authoring context includes the user prompt, `BackendModule` and `WorkerApplication` contract, supported A2UI version, and selected catalog schemas.
- [ ] The generated module exports the required asynchronous initializer and worker application methods and exchanges only standard A2UI JSON messages.
- [ ] Generated dynamic lists use standard A2UI child templates with stable template component IDs and relative item bindings in both the fallback view and initial editable interface.
- [ ] Schema and document validation reject malformed drafts before execution. Preview runtime checks conservatively validate the initial surface against actual data, fallback component bindings, and stable action contracts; relative template bindings are scoped to their declared list path, and absent undefined paths or conditional actions are not claimed valid.
- [ ] The interface uses standard A2UI components and bindings directly; no second interface-description language or custom renderer module is introduced.
- [ ] The user consents before generated source executes in an isolated preview worker and explicitly applies the draft before installation.
- [ ] Preview uses an ephemeral draft backend definition, mount, and runtime session with no installed definition or live selection change; closing it disposes preview resources and ephemeral configuration.
- [ ] Apply stores the module source and initial interface revision through ticket 08 and makes the application available to open after reload.
- [ ] The generated application uses the existing generic backend worker, action serialization, snapshots, shared sessions, failure containment, and interface projection.
- [ ] Malformed model output, preview failure, cancellation, and provider failure install nothing and leave existing applications unchanged.
- [ ] Generated workers receive no provider credentials or provider-client access.
- [ ] UI-only changes after installation use ticket 07's interface revision path and do not regenerate executable source; behavior changes require a new module revision and consent.
- [ ] Deterministic fake-provider tests cover valid generation, malformed output, runtime failure, cancellation, preview isolation, consent, installation, reload, and credential isolation.
- [ ] Production Playwriter validation demonstrates the complete fake-provider draft, preview, install, open, action, and reload path with no compiler, repository-owned server, or transport proxy.
