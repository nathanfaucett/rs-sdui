# 04: Share and recover runtime sessions

**What to build:** A user selects a session ID when opening a Todo window. Windows with the same backend and session synchronize through one worker runtime, while different sessions stay independent. Closing and reopening a window recovers the current snapshot when its runtime remains alive.

**Blocked by:** 02: Open a dynamically loaded Todo window

**Status:** ready-for-agent

- [ ] The root SDUI interface validates and stores the selected session ID for each new window.
- [ ] Runtime identity includes the backend ID, canonical absolute module URL, and session ID, with no session data added to A2UI messages.
- [ ] Two windows with matching runtime identities share one Todo application, and add or complete actions in either window update both.
- [ ] A window with a different session ID maintains independent Todo state.
- [ ] Every window keeps a separate processor, so multiple Todo surfaces with the same surface ID do not conflict.
- [ ] Closing one shared-session window leaves the other window and runtime operational.
- [ ] Reopening the same backend and session creates a fresh processor and connection and applies the current worker snapshot without duplicate-surface errors.
- [ ] Recovery does not replay outbound actions or renderer-local input edits.
- [ ] Tests cover runtime-key construction, worker-name handling, shared updates, independent state, surface isolation, and snapshot recovery.
- [ ] Production Playwriter validation demonstrates shared synchronization, independent sessions, close, and reopen without an application server.
