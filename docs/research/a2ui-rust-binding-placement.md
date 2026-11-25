# A2UI Rust binding placement

Research date: 2026-11-25  
Status: addendum to `a2ui-transport-bindings.md`. Exploration only; no implementation is proposed for immediate execution.

## Recommendation — inference

**Keep the prior order: A2A, then scoped MCP, then deferred AG-UI.** The standalone `a2ui` crate improves the implementation boundary. It does not change protocol coverage.

A2A remains the only official mapping found for all six agent-to-renderer and four renderer-to-agent variants. Native Rust now has official A2A and MCP SDKs. No official Rust A2UI SDK or transport binding appears in the upstream A2UI package list.

Add native Rust binding crates beside `a2ui`; keep browser SDK adapters in `@sdui/web`. Do not add transport features to `a2ui` or re-export bindings through `sdui`.

The independent `ag-ui-a2ui` crate is a reuse candidate, not an official binding or a drop-in replacement. Its own protocol types would duplicate the local type graph, and its released A2A helper has the framing mismatch described below.

## Verified local facts

- `a2ui` depends only on Serde and `serde_json`. Its crate documentation explicitly excludes application state, declarative views, transport, and rendering: `crates/a2ui/Cargo.toml`, `crates/a2ui/src/lib.rs:1-23`.
- `AgentToRendererMessage` exports `CreateSurface`, `UpdateComponents`, `UpdateDataModel`, `DeleteSurface`, `CallRendererFunction`, and `AgentFunctionResponse`. `RendererToAgentMessage` exports `Action`, `CallAgentFunction`, `RendererFunctionResponse`, and `Error`: `crates/a2ui/src/message.rs:458-467,605-612`.
- `DataModel` is a transparent JSON object attached to one surface through `CreateSurface.data_model`. It is not a versioned map of surface IDs. `Metadata { extensions }` appears on `CreateSurface` and `ActionEvent`: `crates/a2ui/src/message.rs:24-64,45-49,469-482`.
- The public exports contain no dedicated `AgentCapabilities`, `RendererCapabilities`, versioned `RendererDataModel`, A2A `DataPart`, turn/task, run, or lifecycle types: `crates/a2ui/src/lib.rs:15-23`.
- `sdui` depends on `a2ui` but does not re-export its types. This is explicit in `README.md:18` and visible in `crates/sdui/src/lib.rs` and `crates/sdui/Cargo.toml`.
- Repository rules require standard A2UI v1.0 messages, transport independence, no custom envelopes or compatibility transforms, and reuse of official or maintained SDKs: `CODING_STANDARDS.md:5-17`.

## Verified SDK and upstream facts

### A2A

- The official `a2aproject/a2a-rs` workspace targets **A2A v1**. Current crates are **`a2a-lf` 0.4.2** (library name `a2a`), **`a2a-client-lf` 0.2.9**, and **`a2a-server-lf` 0.5.3**. Optional protobuf/gRPC crates are `a2a-pb` 0.3.2 and `a2a-grpc` 0.3.11. They require Rust 1.85. Sources: [workspace README and dependency names](https://github.com/a2aproject/a2a-rs/blob/1a7ee4b2aa801da6c67975dbea467c49e42c8a5a/README.md), [workspace manifest](https://github.com/a2aproject/a2a-rs/blob/1a7ee4b2aa801da6c67975dbea467c49e42c8a5a/Cargo.toml), [core registry](https://crates.io/crates/a2a-lf/0.4.2), [client registry](https://crates.io/crates/a2a-client-lf/0.2.9), [server registry](https://crates.io/crates/a2a-server-lf/0.5.3).
- The client uses Tokio and reqwest; the server uses Axum; gRPC uses Tonic. No browser/WASM target is documented. The core type crate has no Tokio dependency, but that alone does not establish WASM support: [core manifest](https://github.com/a2aproject/a2a-rs/blob/1a7ee4b2aa801da6c67975dbea467c49e42c8a5a/a2a/Cargo.toml), [client manifest](https://github.com/a2aproject/a2a-rs/blob/1a7ee4b2aa801da6c67975dbea467c49e42c8a5a/a2a-client/Cargo.toml).
- The Rust workspace documents A2A v1, not the TypeScript SDK's opt-in A2A 0.3 compatibility mode. Do not transfer that TypeScript finding to Rust.

### MCP

- The official SDK is **`rmcp` 3.11.0**, requiring Rust 1.88. It implements MCP **2026-07-28** and documents compatibility with **2025-11-25** and earlier versions. Sources: [registry](https://crates.io/crates/rmcp/3.11.0), [official README](https://github.com/modelcontextprotocol/rust-sdk/blob/d6c3e2cfe55a63bf1a7d52bb6a0b26454a9aeef1/README.md).
- Relevant features include `client`, `server`, `transport-streamable-http-client-reqwest`, and `transport-streamable-http-server`. Default features enable `server`, macros, and Base64, so a scoped dependency should start with `default-features = false`: [manifest](https://github.com/modelcontextprotocol/rust-sdk/blob/d6c3e2cfe55a63bf1a7d52bb6a0b26454a9aeef1/crates/rmcp/Cargo.toml).
- The README describes a Tokio runtime and native stdio, child-process, HTTP, and in-process transports. `transport-worker` means an in-process worker, not a browser Web Worker. The manifest has WASM-specific Chrono configuration, but no end-to-end browser transport is documented. Treat browser support as unverified, not as absent or proven.

### AG-UI

- AG-UI's official repository lists TypeScript, Python, and .NET as its main SDKs and Rust as a **community SDK**. The existing community crates are **`ag-ui-core` 0.1.0** and **`ag-ui-client` 0.1.0**, last published on 2025-08-12. The client uses reqwest and Tokio; no browser/WASM target is documented. Sources: [official SDK table](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/README.md#sdks), [community core manifest](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/sdks/community/rust/crates/ag-ui-core/Cargo.toml), [community client manifest](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/sdks/community/rust/crates/ag-ui-client/Cargo.toml), [core registry](https://crates.io/crates/ag-ui-core/0.1.0), [client registry](https://crates.io/crates/ag-ui-client/0.1.0).
- A separate independent project publishes **`ag-ui` 1.0.0** and **`ag-ui-a2ui` 1.0.0**, requiring Rust 1.85. Its README says CI checks WASM feature builds; HTTP and Axum remain optional. `ag-ui-a2ui/a2a` provides JSON helpers without an A2A library dependency. Sources: [project README](https://github.com/KimSoungRyoul/ag-ui-rust/blob/e1728f609c6460786090d33a2e673733b8ba5696/README.md), [`ag-ui-a2ui` manifest](https://github.com/KimSoungRyoul/ag-ui-rust/blob/e1728f609c6460786090d33a2e673733b8ba5696/crates/ag-ui-a2ui/Cargo.toml), [`ag-ui` registry](https://crates.io/crates/ag-ui/1.0.0), [`ag-ui-a2ui` registry](https://crates.io/crates/ag-ui-a2ui/1.0.0).
- The independent SDK is not accepted upstream: [AG-UI PR #2778](https://github.com/ag-ui-protocol/ag-ui/pull/2778) was open and unmerged when checked. Its A2UI README also records three known multi-catalog schema-validation failures. These are status facts, not proof that the crate is unsuitable for every use.

### Official A2UI binding check

The official A2UI repository lists Python, TypeScript, Dart/Flutter, and Swift packages, but no Rust package or Rust A2A/MCP/AG-UI binding: [official package list](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/README.md#packages). The crates.io name **`a2ui` 0.3.1** belongs to an unrelated third-party renderer project, not the official A2UI project or this workspace: [registry](https://crates.io/crates/a2ui/0.3.1). `ag-ui-a2ui` is independent as described above.

## Mapping traps

### Facts

1. **Metadata namespaces differ.** Local `a2ui::Metadata` belongs inside `createSurface` or `action`. A2A puts `a2uiRendererCapabilities` and `a2uiRendererDataModel` in `Message.metadata`, and `mimeType` in `DataPart.metadata`. MCP uses request or result `_meta`. AG-UI uses run input, forwarded properties, and activity wrappers. Sources: [A2UI A2A extension](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/extensions/a2a/docs/a2ui_extension_specification.md), [MCP guide](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/docs/site/src/content/docs/guides/a2ui-over-mcp.mdx), [AG-UI middleware](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/middlewares/a2ui-middleware/src/index.ts).
2. **Data-model shapes differ.** A2A `a2uiRendererDataModel` is `{ version: "1.0", surfaces: { [surfaceId]: { ... } } }`. Serializing one local `DataModel` directly into that metadata loses the required version and surface key. Capabilities are version-keyed as `{ "1.0": ... }`, while A2A Agent Card extension `params` contain only the inner object: [renderer data-model schema](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/json/renderer_data_model.json), [renderer capabilities schema](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/json/renderer_capabilities.json), [A2A extension](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/extensions/a2a/docs/a2ui_extension_specification.md).
3. **A2A `DataPart.data` is an array.** Entries are ordered, are not transactional, and processing must continue after an individual entry error. The independent `ag-ui-a2ui` **1.0.0** `a2a::data_part` instead writes `"data": message` as one object. It therefore does not produce the currently required v1.0 shape in that helper: [official encoding rules](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/specification/v1_0/extensions/a2a/docs/a2ui_extension_specification.md#data-encoding), [independent helper](https://github.com/KimSoungRyoul/ag-ui-rust/blob/v1.0.0/crates/ag-ui-a2ui/src/adapters/a2a.rs).
4. **MCP is not a complete six-plus-four mapping.** The official guide documents resources/tools and `a2ui_action`/`a2ui_error`. It gives no mapping for `callRendererFunction`, `agentFunctionResponse`, `callAgentFunction`, or `rendererFunctionResponse`, nor for action extension metadata or the exact renderer-data-model placement. The `a2ui_action` tool carries the five standard action fields, not optional `userMessage` or `metadata`: [MCP guide](https://github.com/a2ui-project/a2ui/blob/a23b19c4f2102695c506adb7d1ded1f5f81a7688/docs/site/src/content/docs/guides/a2ui-over-mcp.mdx).
5. **AG-UI remains a subset and a different stream model.** Its middleware types list only `createSurface`, `updateComponents`, `updateDataModel`, and `deleteSurface`, plus a user-action path. They do not map the four function-RPC variants or renderer `error`. Cumulative `ACTIVITY_SNAPSHOT` values require `messageId`-based replacement and duplicate suppression, not blind forwarding into an existing processor: [middleware types](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/middlewares/a2ui-middleware/src/types.ts), [snapshot handling](https://github.com/ag-ui-protocol/ag-ui/blob/ed94ca4a3240527d3ef20fd0f0cc63501f44e469/middlewares/a2ui-middleware/src/index.ts). The independent `ag-ui-a2ui` AG-UI adapter also explicitly rejects batches containing function calls: [crate README](https://github.com/KimSoungRyoul/ag-ui-rust/blob/e1728f609c6460786090d33a2e673733b8ba5696/crates/ag-ui-a2ui/README.md#a2ui-10-function-calls).

### Design consequences — inference

- Keep transport metadata outside `a2ui::Metadata`; use binding-owned context types.
- Wrap per-surface models in the versioned, all-surfaces A2A payload at the binding boundary. Supply capabilities and data models on every required send.
- Reject unsupported MCP/AG-UI variants or unmapped optional fields. Do not drop them silently or invent envelopes.
- Do not reuse `ag-ui-a2ui/a2a` unchanged for this repository's v1.0-only, no-legacy scope. Any reuse spike must address its object-versus-array framing and its separate protocol types.
- Keep AG-UI activity reconciliation outside the generic `@sdui/web` message loop. Use a fresh processor for authoritative snapshots.

## Recommended crate and feature boundaries — inference

Names below are workspace working names, not claims of registry availability.

| Location            | Scope and dependencies                                                                                                                                                                                                                                                                                                       |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/a2ui`       | Keep Serde-only base A2UI v1.0 types and the complete two message unions. When A2A work starts, add the official capability and renderer-data-model payload types here. Add no A2A, MCP, AG-UI, async, HTTP, task/run, or renderer-state features.                                                                           |
| `crates/a2ui-a2a`   | Own full-union `DataPart` array mapping, Agent Card params, message/part metadata, turn/task events, recovery, and extension negotiation. Use `a2a-lf` for types. Make network support opt-in through `client` (`a2a-client-lf`), `server` (`a2a-server-lf`), and `grpc` (`a2a-grpc`/`a2a-pb`) features; no network default. |
| `crates/a2ui-mcp`   | Own only verified native resource/tool, `_meta`, action, and error flows. Use `rmcp` with default features off. Expose client/server and selected Streamable HTTP support as opt-in features. Do not expose a continuous full-union transport or MCP Apps claims.                                                            |
| `crates/a2ui-ag-ui` | Keep deferred. If a concrete target requires it, choose the community or independent SDK after an interoperability spike. Scope it to activity snapshots and actions unless upstream adds mappings; gate client/server dependencies separately.                                                                              |
| `crates/sdui`       | Leave unchanged. Applications depend directly on `a2ui` and any binding crate. Do not re-export protocol SDKs or bindings through `sdui`.                                                                                                                                                                                    |
| `@sdui/web`         | Keep root exports and generic transports unchanged. Put browser clients in the prior `/a2a`, `/mcp`, and deferred `/ag-ui` subpaths with optional peers. Keep MCP Apps separate. Processor-backed capabilities and renderer data models belong at this browser boundary.                                                     |

Keep A2A `DataPart`, Agent Card parameters, MCP `_meta`, AG-UI activities, and turn/task/run lifecycle types in their binding crates. Add the standard `AgentCapabilities`, `RendererCapabilities`, and `RendererDataModel` payload types to `a2ui` when A2A work starts: they are official A2UI v1.0 schemas, and A2A and MCP share their capability payload. Reuse the existing per-surface `DataModel` inside the versioned all-surfaces payload. Do not create a second shared protocol crate. The transport placement of these payloads stays binding-local.

Before implementation, run three spikes:

1. Prove the official Rust A2A client with array framing, all ten message variants, metadata, function RPC, terminal events, and task recovery. Test A2A 0.3 peers separately if required; the Rust SDK does not document that compatibility mode.
2. Compare that path with `ag-ui-a2ui/a2a`. Check the framing mismatch, v1.0-only behavior, type conversion cost, known upstream failures, and optional `jsonschema =0.61.2` duplication beside this workspace's 0.63.1 pin.
3. If direct browser Rust is required, prove the exact `wasm32-unknown-unknown` dependency and transport combination. Otherwise keep direct browser protocol clients in TypeScript and native clients in Rust.

No builds or tests were run for this documentation-only research. Registry and repository status was observed on 2026-11-25.
