# 14: Add the direct browser LLM API provider

**What to build:** A user can configure an OpenAI-compatible LLM HTTP endpoint and model and use it directly from the browser for application generation and interface editing through the common provider interface.

**Blocked by:** 10: Generate JavaScript applications with an LLM; 11: Edit application interfaces with an LLM

**Status:** ready-for-agent

- [ ] The host adapter uses the endpoint's documented OpenAI-compatible chat-completion contract directly from the browser and requires no repository-owned proxy or application server.
- [ ] The user can configure the endpoint and model and supply an optional bearer credential for the host browser session.
- [ ] The credential remains in host-session memory and never enters root Rust state, A2UI messages, durable storage, generated source, application workers, logs, or validation artifacts.
- [ ] The configuration flow warns that direct browser access exposes the credential to that browser session and sends it to the configured provider.
- [ ] Application generation and interface editing use the same authoring contexts, validation, preview, apply, and rollback paths as the fake provider.
- [ ] Requests support cancellation and bounded waiting, with contained errors for HTTP failure, rate limits, timeouts, cancellation, and malformed responses.
- [ ] CORS, mixed-content, endpoint trust, and unsupported response-capability failures produce actionable host-owned errors.
- [ ] Model output remains an unapplied authoring draft and receives no special execution or installation path.
- [ ] Automated tests use a local HTTP fake and assert authorization handling, secret isolation, cancellation, HTTP failures, and malformed responses without a real credential.
- [ ] When a user-supplied compatible endpoint is available, production browser validation generates one application and applies one UI-only interface revision without recording the credential, then stops all preview processes.
