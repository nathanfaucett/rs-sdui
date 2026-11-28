# 13: Add the WebLLM provider

**What to build:** A user can run a compatible WebLLM model locally in the browser and use it for application generation and interface editing through the common provider interface.

**Blocked by:** 10: Generate JavaScript applications with an LLM; 11: Edit application interfaces with an LLM

**Status:** ready-for-agent

- [ ] WebLLM runs in a dedicated provider worker, separate from the root realm and generated application workers, and satisfies the common provider interface.
- [ ] The host checks WebGPU support and reports unsupported hardware or browser capabilities before starting a model download.
- [ ] Model selection exposes relevant download size and cache status where WebLLM provides them; download, initialization, generation, and cancellation have visible host-owned status.
- [ ] Model downloads and cache use WebLLM's supported browser mechanisms without a repository-owned server or committed model assets.
- [ ] Cancellation and provider failures stop or abandon the pending authoring request without changing installed applications or selected interfaces.
- [ ] Application generation and interface editing use the same authoring contexts, validation, preview, apply, and rollback paths as the fake provider.
- [ ] Model output remains an unapplied authoring draft and never executes in the provider worker or root workspace realm.
- [ ] WebLLM requires no API credential, and no provider credential or client capability is exposed to generated application workers.
- [ ] Automated tests mock the WebLLM adapter boundary for capability, progress, cancellation, malformed output, and failure cases without downloading a model.
- [ ] When compatible WebGPU hardware is available, production browser validation loads a user-selected model, generates one application, applies one UI-only interface revision, and releases provider and preview resources.
