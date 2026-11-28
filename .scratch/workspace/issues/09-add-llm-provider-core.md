# 09: Add the LLM provider core

**What to build:** One host-owned asynchronous provider interface accepts authoring prompts and context, returns unapplied draft content, and supports cancellation. A deterministic fake provider exercises the interface before real WebLLM, Ollama, or browser API adapters are added.

**Blocked by:** 08: Persist the application library

**Status:** ready-for-agent

- [ ] The provider interface hides provider-specific request and response shapes from application generation and interface editing.
- [ ] Requests contain the user prompt, explicit authoring context, and cancellation; responses remain authoring drafts until validated, previewed, and explicitly applied.
- [ ] Provider adapters for WebLLM, Ollama, a direct browser LLM API, and the fake can be registered without changing either authoring workflow.
- [ ] Non-secret provider selection and model configuration can be stored in the application library; provider clients, streams, and credentials remain host-owned.
- [ ] Provider credentials stay in host-session memory and never enter root Rust state, A2UI messages, durable storage, generated source, application workers, or logs.
- [ ] Cancellation, timeouts, malformed responses, unavailable models, and provider failures produce contained host-owned errors without changing installed applications or selected interfaces.
- [ ] The fake provider returns deterministic valid application and interface drafts and can emit malformed content, cancellation, and failure cases.
- [ ] Provider configuration and authoring status use management metadata only; no custom A2UI envelope or child message field is added.
- [ ] Automated tests use only the fake provider and cover request context, cancellation, malformed output, failures, secret isolation, and adapter registration.
- [ ] No real provider network request, model download, application generation workflow, or interface editing workflow is required in this ticket.
