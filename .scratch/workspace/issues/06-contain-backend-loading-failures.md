# 06: Contain backend loading failures

**What to build:** A user receives a visible error in the affected window when a backend cannot load or initialize. The root workspace and all healthy windows remain usable. Closing and reopening the failed window retries with clean renderer state.

**Blocked by:** 03: Add and remove WASM backend definitions

**Status:** ready-for-agent

- [ ] Missing modules, malformed worker metadata, missing application exports, and WASM initialization failures reach the affected window through host or transport lifecycle handling.
- [ ] Backend failures are not encoded as custom A2UI messages or envelopes.
- [ ] The affected surface slot displays useful loading and failure states without claiming that the root Rust application knows actual transport status.
- [ ] A failed module load rejects or closes the window's processing lifecycle instead of leaving an indefinite pending mount.
- [ ] Failure in one window does not close, reset, or block healthy windows or root workspace controls.
- [ ] Failed mounts release their connections, processors, listeners, and slot attachments exactly once.
- [ ] Closing and reopening a failed window uses a fresh processor and permits a corrected backend or recovered module to succeed.
- [ ] Tests use local fakes for initialization and transport failures and require no failure-test server.
- [ ] Production browser validation demonstrates a failed Todo load alongside a healthy Todo window and verifies successful retry after correction.
