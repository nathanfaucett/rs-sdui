# 12: Add the Ollama LLM provider

**What to build:** A user can select an Ollama model on a user-supplied local or server endpoint and use it for application generation and interface editing through the common provider interface.

**Blocked by:** 10: Generate JavaScript applications with an LLM; 11: Edit application interfaces with an LLM

**Status:** ready-for-agent

- [ ] The host adapter uses Ollama's documented HTTP API directly from the browser and requires no repository-owned proxy or application server.
- [ ] The user can configure the Ollama base URL and select or enter a model; non-secret configuration can persist through the provider core.
- [ ] Application generation and interface editing use the same authoring contexts, validation, preview, apply, and rollback paths as the fake provider.
- [ ] Requests support cancellation and bounded waiting, with contained errors for connection failure, unavailable models, timeouts, cancellation, and malformed responses.
- [ ] CORS, mixed-content, and remote-endpoint trust failures produce actionable host-owned errors rather than indefinite pending drafts.
- [ ] The adapter does not treat an Ollama URL, model name, workspace session ID, or worker ID as authentication.
- [ ] Model output remains an unapplied authoring draft and receives no special execution or installation path.
- [ ] Automated tests use a local HTTP fake for API responses and failures and do not require an Ollama installation.
- [ ] When a user-supplied Ollama endpoint is available, production browser validation generates one application and applies one UI-only interface revision through Ollama, then stops all preview processes.
