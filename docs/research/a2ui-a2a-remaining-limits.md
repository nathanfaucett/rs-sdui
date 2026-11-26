# A2A remaining limits: durable history and dependency branches

Research date: 2026-11-26
Status: research with an implemented store-factory follow-up; no durable backend included.

## Findings

**Verified:** `a2a-server-lf 0.5.3` provides `TaskStore` and `PushConfigStore` traits and in-memory implementations. It provides no durable store. Durable storage, identity scoping, restart reconciliation, retention, and retry policy are application-owned. A durable `TaskStore` restores `GetTask`/`ListTasks` data; it does not restore active executions or replay transient stream events.

**Verified dependency result:** the current official releases retain `base64` 0.22.1 and 0.23.1. The server graph also retains `tower-http` 0.6.11 and 0.7.1. Their SemVer requirements cannot resolve to one branch. No compatible newer official A2A release was available when checked.

**Recommendation — inference:** add an application-owned, identity-scoped durable `TaskStore` behind the implemented `TaskStoreFactory`. Use it together with durable `SessionStore`; neither store replaces the other. Reconcile orphaned non-terminal tasks at startup and do not automatically replay actions or function calls.

Keep the official dependency releases and do not suppress duplicate-version checks. Reassess the branches when the official SDK or its transitive dependencies provide compatible releases. TLS feature changes do not address either pair.

Sources below use the official SDK publication commit [`9a8d77eb9458e4a7d3673d3e11f00c8536cf0318`](https://github.com/a2aproject/a2a-rs/tree/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318), matching the versions in `Cargo.toml:16-18` and `Cargo.lock:5-86`.

## 1. Current local boundary — verified

- Each verified principal/tenant receives a separate `DefaultRequestHandler`. `SduiRequestHandler::new` selects `InMemoryTaskStoreFactory`; `with_task_store_factory` accepts application-owned official `TaskStore` implementations: `crates/a2ui-a2a/src/server.rs`. The default store explicitly does not survive restart: [official in-memory store, lines 18-46](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/task_store/inmemory.rs#L18-L46).
- `SessionStore<S>` only creates/restores a `Session` and atomically saves application state plus its last timestamp. It must not save initial messages, the RPC outbox, or pending function calls: `crates/a2ui-a2a/src/session.rs:51-115`. Restore creates a fresh processor and an authoritative initial surface: `crates/a2ui-a2a/src/session.rs:69-90`.
- `A2uiClient::snapshot()` reads every task in a context with `history_length = 0` and `include_artifacts = true`, requires all tasks to be terminal, and compares two complete reads. It recovers A2UI output from artifacts, not task-message history, and excludes both RPC directions: `crates/a2ui-a2a/src/client.rs:183-257`, `crates/a2ui-a2a/src/task.rs:16-47`. Apply the result only to a fresh processor.
- The local proposal already identifies durable A2A history as separate from `SessionStore`: `docs/proposals/a2ui-bindings-for-sdui-applications.md:150-167`.

## 2. Official SDK contracts — verified

### `TaskStore` and `TaskVersion`

The [official `TaskStore` contract](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/task_store/store.rs#L7-L52) requires:

| Method                  | Required behavior for a durable implementation                                                                                                                                                                        |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `create(Task)`          | Atomically insert a new task and return its initial version. The in-memory reference rejects a duplicate task ID and starts at version 1.                                                                             |
| `update(Task)`          | Atomically replace the stored task snapshot and return the new version. Return `TASK_NOT_FOUND` when absent; `DefaultRequestHandler::save_task` relies on that code to fall back to `create`.                         |
| `get(task_id)`          | Return the complete task or `None`, within the caller's identity scope.                                                                                                                                               |
| `list(request)`         | Honor filters and `page_size`; return a continuation token whenever more tasks remain. Preserve complete artifacts when `include_artifacts = true`.                                                                   |
| `begin_cancel(task_id)` | In one atomic operation, reject terminal/missing tasks with the documented error codes or transition a non-terminal task to `Canceled` and return it. Do not implement this as an unprotected read followed by write. |

The in-memory reference increments the version on every update and cancellation, allows re-applying the same terminal state, and rejects a transition from a terminal state to a different state: [reference update/cancel behavior](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/task_store/inmemory.rs#L37-L89). Match these observable semantics unless an SDK change defines a different contract.

`TaskVersion` is a `u64` described as an optimistic-concurrency counter, but the trait has no expected-version argument. `get` returns `Task`, not `StoredTask`, and `save_task` discards returned versions: [trait](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/task_store/store.rs#L7-L39), [save path](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L216-L232). Therefore the current API does **not** provide caller-controlled compare-and-swap. A store must provide its own atomic write invariants; `TaskVersion` alone does not provide them.

For listing, the A2A v1 specification requires authorization scoping, status-timestamp-descending order, and cursor pagination: [specification §3.1.4](https://github.com/a2aproject/A2A/blob/a3045e4448710342bcd892ac3958ebb51769bd8d/docs/specification.md#314-list-tasks). The SDK trait owns the continuation scheme. Use a stable cursor with a unique tie-breaker; offset pages can skip or duplicate entries when writes occur. The in-memory ID sort/offset implementation is not a durable persistence blueprint: [reference list implementation](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/task_store/inmemory.rs#L91-L169).

### Request-handler construction

Official construction is `DefaultRequestHandler::new(executor, task_store)`, followed by optional push-store and capability builders: [official constructors](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L537-L613). The required local shape is therefore:

1. Construct one durable store scoped to the verified `SessionIdentity` for each identity handler.
2. Wrap it in the settlement decorator and pass that decorator to `DefaultRequestHandler::new`.
3. Apply the existing capabilities. Add a push store only if push notifications are supported.

The identity scope must be bound into the store instance or factory. `TaskStore` methods receive task IDs and list requests, not `ServiceParams` or a principal. A single unscoped database-backed object would lose the isolation currently supplied by separate in-memory stores. Database rows should include principal, canonical tenant, task ID, context ID, status, status timestamp, version, and the complete official `Task` value. These are persistence fields, not protocol additions.

**Resolved integration requirement:** `SettlementTaskStore` now wraps `Arc<dyn TaskStore>`. Its event counter tracks official-handler settlement, not durable acknowledgment. Success and failure both settle cancellation waiters; the official handler propagates store errors separately. Do not use that counter as proof that a terminal event is durable.

The official library exports only `InMemoryTaskStore` and `InMemoryPushConfigStore` as concrete stores; its manifest contains no database backend: [exports](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/lib.rs#L34-L41), [server manifest](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/Cargo.toml). The durable implementations and their migrations are application-owned.

### Push notification configuration is separate

`PushConfigStore` has independent `save`, `get`, `list`, `delete`, and `delete_all` methods. The in-memory implementation generates a missing config ID, permits replacement by ID, caps each task at 50 configs, and treats deletes as idempotent: [official push-store contract and reference](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/push/store.rs#L9-L138).

- Use `.with_push_config_store(store)` for the default `HttpPushSender`, or `.with_push_notifications(store, sender)` for a configured sender: [handler builders](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L586-L608).
- A durable `TaskStore` does not persist push configs. If push must survive restart, provide a separate identity-scoped durable `PushConfigStore` and protect stored credentials as secrets.
- Push delivery is not a durable event journal. The handler persists the task, then sends push notifications, then broadcasts to subscribers: [execution order](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L354-L409). The supplied sender performs one POST without retry; by default it logs delivery errors and returns success. `fail_on_error = true` instead aborts execution on an error: [sender behavior](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/push/sender.rs#L19-L159). Persisted configs do not replay notifications missed during downtime.
- A2A permits duplicate delivery and requires at-least-once delivery attempts; webhook receivers should process notifications idempotently: [specification §4.3.3](https://github.com/a2aproject/A2A/blob/a3045e4448710342bcd892ac3958ebb51769bd8d/docs/specification.md#433-push-notification-payload).

### Streaming, subscribers, and restart

The official handler has process-local execution and subscriber state:

- Each active execution has a Tokio broadcast buffer of 32 events and a latest task snapshot. Resubscription receives that snapshot followed only by newer events. A lagging subscriber receives an internal error and its stream ends; missed events are not replayed: [active execution state](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L14-L133), [subscription stream](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L302-L352).
- Dropping one client stream does not stop the active execution. A later subscription can rejoin while that execution remains registered.
- Finished executions are removed. Subscribing to a terminal task returns `UnsupportedOperation`; subscribing to a non-terminal stored task with no active execution returns `TaskNotFound`: [completion and subscription behavior](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L1048-L1074).
- Restart discards the execution registry, Tokio tasks, broadcast channels, and sequence numbers. The SDK contains no startup path that reconstructs them from `TaskStore`. Durable terminal tasks remain readable through `GetTask`/`ListTasks`; orphaned non-terminal tasks block the current `A2uiClient::snapshot()` and cannot be rejoined until the application reconciles them.

Recovery after restart must therefore use stored task/artifact snapshots, not a mixture of old stream events and a new snapshot. The application must define how orphaned non-terminal tasks become recoverable. Do not silently resume side-effecting work. A2A itself warns that reconnecting clients can miss status messages and that messages are not reliable delivery for critical information: [specification §3.7](https://github.com/a2aproject/A2A/blob/a3045e4448710342bcd892ac3958ebb51769bd8d/docs/specification.md#37-messages-and-artifacts).

### Transactions and idempotency

- Make each whole-task create/update and each `begin_cancel` atomic in the database. Preserve duplicate-create, not-found, terminal-state, and cancellation error codes. Use stable transactional pagination for complete snapshot reads.
- `DefaultRequestHandler` prevents duplicate active execution only inside one handler instance. A shared durable store does not supply a cross-process execution lock. Multiple replicas need single-writer routing per task or an application-owned coordination mechanism; otherwise whole-snapshot writes can lose updates.
- The SDK's send path checks `taskId` and `contextId`, but contains no `messageId` duplicate lookup: [task preparation](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L693-L768). A2A makes `SendMessage` idempotency optional and names `messageId` as the possible duplicate key: [specification §3.3.1](https://github.com/a2aproject/A2A/blob/a3045e4448710342bcd892ac3958ebb51769bd8d/docs/specification.md#331-idempotency). A durable `TaskStore` alone does not make retries safe. Do not automatically retry sends; if retries are required, add application-owned, standard-`messageId` deduplication at the request boundary with an atomic result record and a side-effect policy.
- Cancellation has explicit handler-level idempotency for an already-canceled task; other terminal states remain non-cancelable: [cancel handler](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L955-L1046).

### What `SessionStore` does not solve

| Concern                                                        | Durable `SessionStore` | Durable A2A `TaskStore`            |
| -------------------------------------------------------------- | ---------------------- | ---------------------------------- |
| Application state and last timestamp                           | Yes                    | No                                 |
| `GetTask`/`ListTasks`, task status/history, and A2UI artifacts | No                     | Yes                                |
| Active execution, subscriber registry, transient event replay  | No                     | No                                 |
| Pending RPC/outbox or blind action recovery                    | Explicitly excluded    | Not supplied                       |
| Push configs or notification replay                            | No                     | No; separate store/policy required |

There is also a crash-consistency boundary. `SduiExecutor` saves `SessionStore` before yielding task events: `crates/a2ui-a2a/src/server.rs:1044-1072`. `DefaultRequestHandler` subsequently persists each task event: [official event loop](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-server/src/handler.rs#L354-L409). A crash between those commits can leave application state ahead of A2A task history. Two independently durable stores do not make that pair atomic or exactly-once. Startup reconciliation and recovery authority must account for this window.

## 3. Duplicate dependency branches — verified

### Exact graph and requirements

`cargo tree -p a2ui-a2a --all-features -e normal --duplicates` reports only these two duplicate package names in the normal dependency graph:

| Locked branches     | Required by                                               | Declared requirement                          |
| ------------------- | --------------------------------------------------------- | --------------------------------------------- |
| `base64` 0.22.1     | `pbjson` 0.9.0 and `tonic` 0.14.9                         | `base64 = "0.22"`                             |
| `base64` 0.23.1     | `a2a-lf` 0.4.2, `reqwest` 0.13.5, and `hyper-util` 0.1.21 | `base64 = "0.23"`                             |
| `tower-http` 0.6.11 | `reqwest` 0.13.5                                          | `tower-http = "0.6.8"` with `follow-redirect` |
| `tower-http` 0.7.1  | `a2a-server-lf` 0.5.3                                     | `tower-http = "0.7"` with `cors`              |

Primary sources: [official A2A workspace dependency requirements](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/Cargo.toml), [`pbjson` 0.9.0 packaged manifest](https://docs.rs/crate/pbjson/0.9.0/source/Cargo.toml), [`tonic` 0.14.9 manifest](https://github.com/grpc/grpc-rust/blob/tonic-v0.14.9/tonic/Cargo.toml), [`reqwest` 0.13.5 manifest](https://github.com/seanmonstar/reqwest/blob/v0.13.5/Cargo.toml), and [`hyper-util` 0.1.21 manifest](https://github.com/hyperium/hyper-util/blob/v0.1.21/Cargo.toml).

`a2a-pb` 0.3.2 requires `pbjson` 0.9 and `tonic` 0.14, so both client and server networking pull in `base64` 0.22 as well as the core crate's 0.23 branch: [official `a2a-pb` manifest](https://github.com/a2aproject/a2a-rs/blob/9a8d77eb9458e4a7d3673d3e11f00c8536cf0318/a2a-pb/Cargo.toml). The client-only normal graph has only `tower-http` 0.6. The server adds its direct 0.7 branch, producing the second duplicate.

Cargo treats the 0.22 and 0.23 requirements as incompatible, and likewise 0.6.8 and 0.7. A lockfile update cannot select one release that satisfies both ranges. Adding a direct dependency or changing TLS backends cannot relax the original requirements. `reqwest` declares its native `tower-http` dependency without an optional feature gate, and both HTTP libraries declare `base64` independently of TLS selection.

Registry inspection with `cargo info` found no release newer than `a2a-lf` 0.4.2, `a2a-client-lf` 0.2.9, `a2a-server-lf` 0.5.3, `a2a-pb` 0.3.2, `tonic` 0.14.9, `pbjson` 0.9.0, or `reqwest` 0.13.5 when checked. Therefore no released upgrade currently removes these branches.

### Impact and safe options

- The verified cost is extra dependency compilation and two update/audit streams. Duplicate code can also increase binary size, but this investigation did not measure it. A duplicate version alone does not prove a runtime defect or a security vulnerability.
- A clean Clippy run does not prove dependency unification. Inspect `cargo tree` after dependency changes, as `CODING_STANDARDS.md:38-44` requires.
- Keep the current official releases for now. Do not downgrade A2UI/A2A support, patch the lockfile against package requirements, switch TLS solely for deduplication, or add a Clippy allowance.
- Recheck after official A2A upgrades. `base64` unification needs compatible releases of the upstream `pbjson`/`tonic` chain or an A2A release that no longer selects the 0.22 chain. `tower-http` unification needs a compatible `reqwest` release on 0.7 or an official server release using a single compatible branch.
- If the duplication becomes a measured release blocker, prefer reviewed upstream changes and a published official SDK release over a local fork. Any such upgrade requires all repository gates and another complete duplicate-graph review.

## 4. Recommended next steps — inference

1. Implement an application-owned durable store behind the new `TaskStoreFactory` seam. Preserve identity scoping and distinguish event settlement from durable-write success.
2. Implement whole-task transactional storage with stable cursor pagination, the reference terminal/cancel semantics, complete artifacts, retention aligned with session recovery, and cross-identity isolation tests.
3. Define startup reconciliation for non-terminal tasks and for the session-save/task-save crash window. Keep application state and A2A task history as distinct recovery inputs; never replay pending RPC or side-effecting actions blindly.
4. If push is enabled, add a separately scoped durable `PushConfigStore` and document delivery loss/duplicate behavior. Do not describe it as a durable notification queue.
5. Test restart after terminal tasks, restart with an orphaned working task, database failure before settlement, concurrent update/cancel, duplicate create/send behavior, pagination beyond 100 tasks, and same-named task/context IDs across two identities.
6. Keep the current duplicate branches under explicit review during official SDK upgrades. Do not start an independent fork only to remove them.

## 5. Implemented extension seam

- `TaskStoreFactory` returns `Arc<dyn a2a_server::task_store::TaskStore>` for a verified `SessionIdentity`. `InMemoryTaskStoreFactory` preserves the previous default.
- `SduiRequestHandler::with_task_store_factory` installs the application-owned factory. The store decorator accepts the official trait object.
- Public integration tests prove that a custom store retains terminal history across handler recreation and that write errors propagate without blocking cancellation settlement: `crates/a2ui-a2a/tests/task_store.rs`.
- No database backend, restart reconciliation policy, or durable push-configuration store was added.

## Unverified items and validation

- No database engine, schema, transaction manager, multi-replica coordinator, or reconciliation policy was selected or tested.
- No claim is made for exactly-once execution, active-task resume, durable stream replay, or safe automatic send retries.
- Compatibility of any future SDK store-contract or dependency change is unverified. External A2A peer interoperability also remains unverified: `crates/a2ui-a2a/README.md`.
- No claim is made that duplicate branches are harmless under every future release; only their current graph and requirements were verified.
- Research validation consisted of local source review, official released source/specification review, read-only `cargo tree`/`cargo info` inspection, and Git diff/status checks.
- Factory follow-up validation passed: focused custom-store tests, server-only `cargo check`, `cargo test --workspace --all-features`, `cargo clippy --workspace --all-targets --all-features`, `cargo fmt --all -- --check`, `pnpm run check`, and `pnpm run build`.
