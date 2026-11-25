# A2UI transport bindings: AG-UI, A2A, and MCP

Research date: 2026-11-25  
Status: exploration only. No implementation or package changes are proposed for immediate execution.

## Executive decision

**Recommended order: A2A, then scoped MCP support, then AG-UI only if a concrete target requires it.**

- **A2A first:** it has the only normative mapping found for the complete A2UI v1.0 message union, including bidirectional function RPC and transport metadata. A thin mapping is realistic, but task recovery and A2A version compatibility need a spike.
- **MCP second:** support the documented native resource/tool flow. Do not claim complete A2UI v1.0 RPC or `sendDataModel` support until those mappings exist. Reuse `@a2ui/catalog-mcp` where its application model fits.
- **AG-UI third:** the official integration is useful but does not map the complete v1.0 union. Existing middleware and CopilotKit code already owns activity reconciliation and action handling. Avoid a parallel implementation without a target use case.

A2UI requires ordered delivery, message framing, metadata support, and transport-level end-of-turn signaling: [A2UI v1.0 transport contract](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/docs/a2ui_protocol.md).

## Comparison

| Area | A2A | MCP | AG-UI |
| --- | --- | --- | --- |
| Official mapping | Normative A2UI extension | A2UI guides and catalog; several integration patterns | Official middleware and CopilotKit integration |
| Six agent-to-renderer message types | Complete | UI operations documented; complete RPC mapping not found | UI operations; complete RPC unsupported in current CopilotKit integration |
| Four renderer-to-agent message types | Complete | `action` and `error`; other mappings unknown | Primarily `action`; complete union unsupported |
| Standard envelope | A2A `DataPart`; `data` is an A2UI message array | MCP resources and `CallToolResult` embedded resources | `ACTIVITY_SNAPSHOT`, `activityType: "a2ui-surface"` |
| Function RPC | A2UI `functionCallId` is preserved | No official mapping to A2UI function RPC found | No complete official mapping |
| End of turn | Stream close or terminal/interrupted task state | JSON-RPC response to the request | `RUN_FINISHED` or `RUN_ERROR`; truncation is distinct |
| Renderer capabilities | `message.metadata.a2uiRendererCapabilities`, every turn | Legacy initialize or per-call `_meta.a2ui`; current MCP requires per-request metadata | Catalog/schema context; no standard `a2uiRendererCapabilities` mapping found |
| `a2uiRendererDataModel` | `message.metadata`, every message when enabled | Exact placement not found | No official mapping found |
| Recovery | Resubscribe and task snapshots; missed events remain possible | Depends on negotiated MCP revision | No HTTP SSE resumption; start a new run |
| Existing official TS integration | Generic A2A SDK; no A2UI-specific TS adapter found | `@a2ui/catalog-mcp` and MCP Apps SDK | `@ag-ui/a2ui-middleware`, CopilotKit renderer |
| Fit with current `Transport` | Best after metadata and lifecycle additions | Suitable only for a scoped resource/tool adapter | Limited and requires activity reconciliation |

## Current repository constraints

The relevant local contracts are:

- `packages/web/src/transport.ts`: `Connection.messages` yields only `AgentToRendererMessage`; `send()` accepts only `RendererToAgentMessage` and returns `void`.
- `packages/web/src/connect.ts`: one fresh official `MessageProcessor` connects to one transport. Processing errors escape the outer async loop and end consumption.
- `MessageProcessor.getRendererCapabilities()` and `getRendererDataModel()` already expose the data needed by A2A and MCP. The current transport callback does not expose the processor or a metadata context to `send()`.
- The package is renderer-independent and side-effect-free. SDK-dependent adapters must stay in separate subpaths with optional peer dependencies.

The A2A `DataPart` array can still feed the existing `AsyncIterable` one message at a time. Transport envelopes do not need to reach `MessageProcessor`. Metadata, turn lifecycle, recovery, and processing-error policy need binding-specific APIs or shared extensions.

## 1. A2A

### Verified mapping

The extension URI is **`https://a2ui.org/a2a-extension/a2ui/v1.0`**.

Agents and renderers exchange A2UI messages in A2A `DataPart` objects:

- `metadata.mimeType = "application/a2ui+json"`.
- `data` contains an **array** of standard A2UI messages.
- Receivers process array entries sequentially. The array is not a transaction. An invalid entry should be reported without discarding later entries.
- Separate list schemas cover all six agent-to-renderer and four renderer-to-agent A2UI message types. This preserves `functionCallId` and the complete function RPC union.

Source: [A2UI v1.0 A2A extension](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/extensions/a2a/docs/a2ui_extension_specification.md).

Metadata rules:

- Agent Card extension `params` contains the inner `1.0` agent capability object, without a version wrapper.
- Every renderer message carries the whole version-keyed `a2uiRendererCapabilities` object in `message.metadata`.
- Capabilities are **turn-scoped**. Omitting them means no A2UI for that turn. Extension activation alone does not advertise catalogs.
- Surfaces using `sendDataModel` place `a2uiRendererDataModel` in `message.metadata` on every message to their creating agent.
- `accepted_output_modes: ["a2ui"]` is explicitly non-standard.
- A2A `contextId` groups conversational work; `taskId` identifies a unit of work.

The A2A specification mandates generated event order and provides task resubscription and `GetTask` snapshots. It also warns that reconnecting clients can miss status messages. `SendMessage` is only optionally idempotent through `messageId`: [A2A specification](https://github.com/a2aproject/A2A/blob/a3045e4448710342bcd892ac3958ebb51769bd8d/docs/specification.md), [streaming guidance](https://github.com/a2aproject/A2A/blob/a3045e4448710342bcd892ac3958ebb51769bd8d/docs/topics/streaming-and-async.md).

### SDK and compatibility

Registry and source values observed on 2026-11-25:

- **`@a2a-js/sdk` 1.3.0**: [registry](https://registry.npmjs.org/@a2a-js/sdk), [package metadata](https://github.com/a2aproject/a2a-js/blob/a6077782c9e806359292684d2704acfde4d23d71/package.json).
- The client offers `sendMessage`, `sendMessageStream`, `getTask`, and `resubscribeTask`; streams are `AsyncGenerator`s. JSON-RPC and REST use `fetch`; gRPC is Node-only: [client transport](https://github.com/a2aproject/a2a-js/blob/a6077782c9e806359292684d2704acfde4d23d71/src/client/transports/transport.ts), [README](https://github.com/a2aproject/a2a-js/blob/a6077782c9e806359292684d2704acfde4d23d71/README.md).
- Import **`@a2a-js/sdk/client`**, not the root, to avoid server dependencies. Optional peers cover gRPC, Express, and databases. Its required runtime dependency is `jose`.
- **`@a2ui/agent` 0.2.1** is the TypeScript agent SDK, not an A2A binding: [registry](https://registry.npmjs.org/@a2ui/agent).
- The official A2UI A2A helper is Python-only and pins `a2a-sdk>=0.3.0,<0.4.0`: [pyproject.toml](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/python/a2ui_a2a/pyproject.toml).
- The JS SDK has opt-in A2A 0.3 compatibility and normalizes `X-A2A-Extensions` versus `A2A-Extensions`: [client header handling](https://github.com/a2aproject/a2a-js/blob/a6077782c9e806359292684d2704acfde4d23d71/src/client/multitransport-client.ts).

### Assessment and unknowns

A thin adapter is realistic. The browser can own a JSON-RPC/REST client when CORS and authentication permit it. An application server should own gRPC, webhook receivers, credentials, or server-side session state.

**Unknown:** the A2UI extension does not declare a base A2A protocol version. Test actual A2UI agents against both the JS 0.3 compatibility path and native A2A 1.0 before selecting a production default.

Required design work:

1. Provide processor-backed capabilities and data-model metadata on every renderer message.
2. Expose turn/task completion and error events outside the A2UI message stream.
3. Reconcile task history, artifacts, final task snapshots, and resubscription without duplicate A2UI messages.
4. Process every entry in a `DataPart` array even after an individual processing error. The current connector's outer catch may otherwise stop the whole connection.
5. Decide whether binding-specific sends should return a Promise. The official outbound listener supports Promises; the current generic `Connection.send()` does not surface acceptance or backpressure.

Do not replay actions or function calls after disconnects. Rebuild the processor from an authoritative task snapshot. Agent Card URLs, inline catalogs, data models, and remote agents require trust, authorization, CORS, SSRF, and privacy controls.

## 2. MCP

### Verified integration patterns

MCP is not one continuous A2UI stream. The A2UI project documents four integration patterns: [MCP integration overview](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/docs/site/src/content/docs/guides/mcp-overview.mdx).

The native-host flow is:

1. `resources/read` returns an `a2ui://` resource with MIME `application/a2ui+json`, usually containing `createSurface` and `updateComponents`.
2. `_meta.ui.resourceUri` links a tool or result to that resource.
3. `tools/call` returns embedded resources containing A2UI updates.
4. The client caches layouts, skips duplicate surface creation, and applies updates in content order.

The guide also defines:

- Legacy session capabilities at `initialize.params.capabilities.a2ui.rendererCapabilities`.
- Stateless capabilities at `tools/call.params._meta.a2ui.rendererCapabilities`.
- `a2ui_action` for the five required action fields.
- `a2ui_error` for renderer errors.

Source: [Serve A2UI layouts from MCP](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/docs/site/src/content/docs/guides/a2ui-over-mcp.mdx).

A JSON-RPC response ends each tool request. Request IDs correlate responses. A2UI still needs application-level sequencing of concurrent updates.

The MCP Apps pattern is separate: a `ui://` HTML resource embeds an A2UI renderer inside a sandboxed iframe. It uses `postMessage`, `ui/initialize`, tool notifications, and host-mediated `tools/call`: [A2UI in MCP Apps](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/docs/site/src/content/docs/guides/a2ui-in-mcp-apps.mdx), [MCP Apps specification](https://github.com/modelcontextprotocol/ext-apps/blob/ea3e9d0e624a20bc1c51310292f09d9b8ed17161/specification/2026-01-26/apps.mdx).

### Version boundary and recovery

The A2UI guide demonstrates MCP **2025-11-25**. That revision supports optional session IDs and `Last-Event-ID` replay: [2025-11-25 transports](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/2bbace557bbf2cb5d03d6f9b48905415cd7607b0/docs/specification/2025-11-25/basic/transports.mdx).

Current MCP **2026-07-28** uses self-contained requests and per-request capabilities. It removes protocol sessions and `Last-Event-ID` resumption: [versioning](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/2bbace557bbf2cb5d03d6f9b48905415cd7607b0/docs/specification/2026-07-28/basic/versioning.mdx), [Streamable HTTP](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/2bbace557bbf2cb5d03d6f9b48905415cd7607b0/docs/specification/2026-07-28/basic/transports/streamable-http.mdx).

Recovery therefore depends on the negotiated MCP revision. Rehydrate a fresh processor from authoritative resources and initial tool results. Do not blindly retry tools or replay A2UI events.

### SDK and gaps

Registry and source values observed on 2026-11-25:

- **`@a2ui/catalog-mcp` 0.1.1**: [registry](https://registry.npmjs.org/@a2ui/catalog-mcp), [package metadata](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/typescript/catalogs/mcp/package.json).
- It implements resource discovery, caching, message hydration, `callMcpTool`, and latest-call protection for `targetDataPath`: [implementation](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/typescript/catalogs/mcp/src/functions/callMcpTool.ts).
- It depends on legacy **`@modelcontextprotocol/sdk ^1.29.0`** and **`@modelcontextprotocol/ext-apps ^1.7.4`**, plus AJV, JMESPath, and Zod 3. It has a Lit peer dependency.
- Legacy SDK latest: **1.32.2**: [registry](https://registry.npmjs.org/@modelcontextprotocol/sdk).
- Current SDK packages: **`@modelcontextprotocol/client`, `@modelcontextprotocol/core`, and `@modelcontextprotocol/server` 2.4.0**: [client registry](https://registry.npmjs.org/@modelcontextprotocol/client), [server registry](https://registry.npmjs.org/@modelcontextprotocol/server).
- Current SDK roots are browser/Worker-neutral; stdio is isolated behind `./stdio`: [package layout](https://github.com/modelcontextprotocol/typescript-sdk/blob/5ee54ee30ddc00a914a3d981880f87fdce0e6246/docs/get-started/packages.md).
- **`@modelcontextprotocol/ext-apps` 2.0.4**: [registry](https://registry.npmjs.org/@modelcontextprotocol/ext-apps).

**Unknowns:** no official MCP mapping was found for `callAgentFunction`, `rendererFunctionResponse`, `callRendererFunction`, `agentFunctionResponse`, action extension metadata, or the exact `a2uiRendererDataModel` placement. Do not invent these envelopes.

### Assessment

The official catalog is application-level integration, not a thin `Transport`. Reuse it directly where possible. A separate adapter is justified only for the generic `a2ui_action`/`a2ui_error` flow or where the repository needs different instance ownership.

MCP Apps needs a different integration shape from native MCP. It belongs in an embedded application example or a separate entry point, not in the ordinary network `Transport` API. Its iframe has storage, CSP, Origin, and bundling constraints.

Choose the SDK and protocol revision from the target server before implementation. The current official A2UI catalog and guide use the legacy SDK line; a greenfield adapter should not select legacy behavior only because the guide predates MCP 2026-07-28.

Security requirements include tool consent, strict tool routing, Origin validation, session protection, payload limits, and privacy filtering. MCP Apps requires sandboxed iframes, CSP enforcement, tool allowlists, and host-audited messages.

## 3. AG-UI

### Verified mapping and limits

The official middleware converts fixed or streamed A2UI generation into:

- `ACTIVITY_SNAPSHOT`
- `activityType: "a2ui-surface"`
- `content.a2ui_operations` containing A2UI operations.

Snapshots replace by `messageId` and are intended to replay independently into a fresh renderer: [middleware source](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/middlewares/a2ui-middleware/src/index.ts), [snapshot merging](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/middlewares/a2ui-middleware/src/activity-snapshots.ts), [AG-UI activity semantics](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/docs/spec/1.0/events/activity.mdx).

The middleware runs on the **application server** and imports `node:crypto`. The browser owns `HttpAgent` and the renderer.

Its action path reads `forwardedProps.a2uiAction.userAction` and creates private user context containing `A2UI user action:\n` plus JSON. It does not define mappings for the complete outbound A2UI union: [middleware types](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/middlewares/a2ui-middleware/src/types.ts).

AG-UI requires ordered, complete run events and explicit terminal signaling. `RUN_FINISHED` and `RUN_ERROR` close runs. HTTP SSE has **no resumption**; a new run needs a new `runId`: [transport contract](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/docs/spec/1.0/basic/transports/index.mdx), [HTTP SSE](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/docs/spec/1.0/basic/transports/http-sse.mdx).

CopilotKit's current A2UI 1.0 support matrix rejects agent/renderer function RPC and does not support agent-function calls: [support matrix](https://docs.copilotkit.ai/generative-ui/components/from-a2ui#support-matrix). Structured actions use `forwardedProps.copilotkit.ui.metadata.extensions.copilotkit_action`: [state and actions](https://docs.copilotkit.ai/generative-ui/components/state-and-actions#send-a-click-to-the-agent).

Registry and source values observed on 2026-11-25:

- **`@ag-ui/client` and `@ag-ui/core` 1.0.5**: [client registry](https://registry.npmjs.org/@ag-ui/client), [core registry](https://registry.npmjs.org/@ag-ui/core).
- **`@ag-ui/a2ui-middleware` 0.0.16**: [registry](https://registry.npmjs.org/@ag-ui/a2ui-middleware).
- **`@ag-ui/a2ui-toolkit` 0.0.5**: [registry](https://registry.npmjs.org/@ag-ui/a2ui-toolkit).
- **`@copilotkit/a2ui-renderer` 1.90.2**: [registry](https://registry.npmjs.org/@copilotkit/a2ui-renderer).
- `@ag-ui/client` supports browser `fetch`, SSE, RxJS streams, and Zod 3/4 validation: [package metadata](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/sdks/typescript/packages/client/package.json).

### Assessment and unknowns

No official AG-UI mapping was found for `a2uiRendererCapabilities`, `a2uiRendererDataModel`, or `sendDataModel`.

An action-only adapter is possible, but it is not a thin forwarding adapter:

- Cumulative activity snapshots can repeat `createSurface` and components. The official renderer reconciles them; the repository's `MessageProcessor` rejects a duplicate surface.
- Activity `messageId` and surface ownership must drive replacement and duplicate suppression.
- Only the latest self-contained activity snapshots should replay into a fresh processor. Replaying all operations into an existing processor risks duplicate `createSurface` errors.
- The official integration already owns activity reconciliation, run history, private action handling, and generation recovery.

Defer a new adapter unless a concrete AG-UI target cannot use the official middleware/CopilotKit path. A full-v1.0 adapter would need an upstream protocol extension, not only local mapping.

## Proposed package shape

Keep `@sdui/web` root exports unchanged.

| Subpath | Proposed scope | Optional peers |
| --- | --- | --- |
| `@sdui/web/a2a` | A2A client adapter, `DataPart` mapping, task/turn events, metadata, and snapshots | `@a2a-js/sdk` |
| `@sdui/web/mcp` | Scoped native resource/tool adapter; no unsupported RPC claims | Selected MCP SDK line after compatibility decision |
| `@sdui/web/ag-ui` | Deferred; otherwise explicitly action-only activity adapter | `@ag-ui/client`, `@ag-ui/core`, `rxjs`, `zod` |

Do not export server middleware, CopilotKit, or `@a2ui/catalog-mcp` from the root. Applications should install those integrations directly. Keep MCP Apps separate from the native MCP adapter.

Start with binding-specific connector APIs. Add shared processor-backed metadata and turn-lifecycle types only when at least two bindings need the same contract. This avoids premature changes to every existing transport.

## Pre-implementation spikes

1. **A2A interoperability:** prove one JSON-RPC or REST round trip with all A2UI message classes, metadata, function RPC, terminal events, and task recovery against A2A 0.3 and 1.0 peers.
2. **Metadata ownership:** prove that capabilities and current renderer data models can be read at send time without a processor/transport initialization cycle.
3. **MCP target selection:** select native resources/tools versus MCP Apps and select the protocol/SDK revision from the actual host.
4. **AG-UI need:** identify a target that the official integration cannot serve before designing a new adapter.

No dependencies were installed and no builds were run for this research.
