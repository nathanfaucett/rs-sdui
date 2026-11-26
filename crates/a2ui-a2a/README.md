# a2ui-a2a

Native Rust bindings for **official A2UI v1.0 over official A2A v1 JSON-RPC**.

The crate maps standard A2UI messages to A2A `DataPart` arrays with `metadata.mimeType = "application/a2ui+json"`. It adds no custom envelope, message field, compatibility transform, or parallel protocol. `Application<S>` remains transport-independent.

## Features

Default features are empty.

- `client`: `A2uiClient` and the official `a2a-client-lf` JSON-RPC transport.
- `server`: `SduiExecutor`, `SduiRequestHandler`, `SessionStore`, and `TaskStoreFactory` with official `a2a-server-lf` types.

## Server contract

Construct `SduiExecutor` with an application factory and optional `SessionStore`. Build `SduiRequestHandler` with its Agent Card capabilities and an identity resolver.

The identity resolver receives official `ServiceParams` and the requested tenant. It must:

1. Authenticate the credential.
2. Authorize the requested tenant.
3. Return a `SessionIdentity` containing the canonical principal and tenant, or an `A2AError`.

Do not return a principal copied from an unverified header. The resolver runs once per request. The selected handler pins that identity for execution and cancellation.

Principal, canonical tenant, and `contextId` form the application `SessionKey`. Principal and tenant also isolate official task history. Task stores and execution entries cannot cross those identity boundaries. At most 1024 identity handlers are retained; no active handler is evicted.

Use official `jsonrpc_router` and `agent_card_router` from `a2a-server-lf`. The Agent Card extension URI is `https://a2ui.org/a2a-extension/a2ui/v1.0`.

## Task stores

`InMemoryTaskStoreFactory` is the default. To provide another task-history implementation:

1. Implement the official `a2a_server::task_store::TaskStore` trait.
2. Implement `TaskStoreFactory` and return the store as `Arc<dyn TaskStore>`.
3. Pass the factory to `SduiRequestHandler::with_task_store_factory`.

The factory runs once for each new verified principal/tenant handler. Bind that `SessionIdentity` to the store's persistence scope; official store methods do not receive request credentials. Return the same scoped store when recreating handlers to retain history. Initialize slow database resources before request routing and share their handles.

Task updates must be atomic, and `begin_cancel` must atomically check and transition state. Store errors propagate through official A2A error handling. Cancellation waits for yielded events to settle after either a successful or failed store operation; settlement is not a durability acknowledgment.

A durable task store restores task reads and history. It does not restore active executions, subscribers, transient streams, or pending RPC. Reconcile orphaned nonterminal tasks during application startup. Do not replay actions or function calls automatically.

## Session stores

`InMemorySessionStore` is the default. Implement `SessionStore<S>` for another lifetime or durability policy.

- `get_or_create` returns a live session or restores one through `Session::restore`.
- `save` atomically persists application state and the last timestamp.
- Do not persist initial messages, the RPC outbox, or pending function calls.
- Restore state into an unstarted `Application<S>` and a fresh `RendererMessageProcessor`.
- `Session::restore` emits one authoritative initial surface and an empty RPC outbox.

A failed save restores unsent initial messages and queued renderer calls. The next turn emits an authoritative surface when required.

`SessionStore` durability does not make the official A2A task store durable. The default A2A task-history factory is in memory. A durable restart/recovery deployment must provide both stores through their respective traits.

## Client and recovery

Use `A2uiClient::with_interceptor` for official authentication interceptors and `with_tenant` for the tenant declared by the selected A2A interface.

The client supports unary and streaming starts and sends, task subscription, cancellation, task listing, and snapshots. `snapshot()`:

- requires every task in the context to be terminal;
- deduplicates task and artifact IDs;
- compares two complete task-history reads and rejects changing history;
- excludes `callRendererFunction` and `agentFunctionResponse`, so recovery never replays RPC.

Apply a snapshot only to a fresh processor. Do not append it to stream events already applied to that processor. `decode_task` and `decode_tasks` have the same restriction.

Cancellation retains buffered UI output, waits for yielded events to settle, and publishes the official canceled task before releasing the execution stream. Cancellation is serialized per executor. Side effects that already ran are not rolled back or retried.

## Current limits

- Native Rust only.
- External A2A peer interoperability has not been validated.
- No MCP or AG-UI adapter.
- No built-in durable A2A task-history implementation; applications provide one through `TaskStoreFactory`.
