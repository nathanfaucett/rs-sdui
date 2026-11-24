# Unified application and agent authoring

Date: 2026-11-23

## Decision

Unify the workspace at the **versioned catalog and standard A2UI message stream**, not at `Application<S>` and not at generated Rust source.

Support two authoring paths:

1. A developer writes typed Rust state, views, transitions, and effects. `a2ui-app` lowers them to standard A2UI messages.
2. A live agent uses an official A2UI inference format. An agent compiler validates the output and lowers it to the same standard A2UI messages.

Both paths must use the same catalog IDs and schema revisions, transport, renderer, and renderer-to-agent envelopes. A live agent must not generate and compile Rust. A2UI deliberately exchanges data rather than executable code.[4]

Static or pre-authored A2UI is also an official use case. The current typed application path therefore complements agent-generated UI; it is not a competing protocol.[1]

## Official agent flow

The official pipeline is:[1][4]

1. Load one or more catalog JSON documents.
2. Generate prompt instructions and grammar rules from their component schemas, function signatures, and `instructions`.
3. Give the generated A2UI skill and application tools to the model.
4. Parse and validate the model output with an Agent SDK compiler.
5. Emit ordered `AgentToRendererMessage` values.
6. Let the official renderer construct and update surfaces.
7. Return user actions and function messages to the surface owner.

The model does not always write wire JSON directly. The official TypeScript Agent SDK supports **Direct JSON** and **Express**. Express is a compact functional inference format; the official compiler expands it into standard wire messages and chooses component IDs. Both formats support streamed responses.[5]

The official project lists Agent SDKs for TypeScript, Python, and Dart. It lists no Rust Agent SDK. The TypeScript package is `@a2ui/agent`; its current source README describes catalog configuration, prompt generation, Direct JSON and Express parsing, streaming, renderer-state tracking, and function-RPC processing.[4][5]

**Reuse rule:** use an official Agent SDK for Express and its complete agent pipeline. Do not create a workspace-specific inference DSL. A pure Rust implementation should start with standard Direct JSON and official conformance fixtures only if a Rust-only agent is a real requirement.

## Protocol facts that affect the design

### Streaming and updates

A2UI is a stream of distinct JSON envelopes. `createSurface` establishes a surface. `updateComponents` adds or replaces component definitions. `updateDataModel` changes data at a JSON Pointer. `deleteSurface` removes the surface. Initial components and data can instead be inline in `createSurface`.[2]

Progressive rendering is valid: a surface can be created before its root or data arrives, and component references can temporarily be missing. A validator for an agent stream must therefore track surface state; it cannot require every message to contain a complete tree.[2]

The transport owns ordered delivery, message framing, metadata, and optional return messaging. A2UI does not mandate WebSocket, SSE, HTTP, AG-UI, A2A, MCP, or another transport.[2]

The agent controls the granularity of its own `updateComponents` and `updateDataModel` messages. The structural and data diffs in `a2ui-app` apply to Rust-owned snapshots; they must not buffer or rewrite an agent's valid incremental stream.

### Catalogs and capabilities

A catalog is the shared contract for components, functions, validation schemas, and LLM instructions. The renderer advertises supported catalog IDs through transport metadata or a transport handshake. The agent selects a surface default with `createSurface.catalogId`; v1.0 can mix catalogs with component `@catalogId` and function `catalogId`. These capabilities are not new A2UI message kinds.[2][3]

The same catalog files must feed Rust builder generation, agent prompt generation, outgoing validation, and renderer registration. A catalog ID is an identifier, not a URL that the protocol fetches.[2]

### Actions and local input state

Input controls update the renderer-local data model without sending a message for each edit. An agent action resolves its `userMessage` and `context`, then returns an `action` envelope with `surfaceId`, `sourceComponentId`, and resolved values. Optional `sendDataModel` synchronization uses transport metadata.[2]

A Rust `.on(...)` callback is one possible owner of an action. An LLM turn is another. The JSON event is the same in both cases. Rust callbacks themselves are not A2UI data and must remain outside the serialized view.

### Functions

A2UI v1.0 has standard bidirectional function RPC:

- `callAgentFunction` and `agentFunctionResponse`.
- `callRendererFunction` and `rendererFunctionResponse`.

Catalog declarations and runtime checks control caller permissions and user-activation requirements. Functions avoid sending executable code.[2]

The official TypeScript `RendererToAgentMessageProcessor` handles agent functions and dispatches actions, errors, and renderer-function responses to callbacks. Unhandled actions can start a model turn. The workspace's `a2ui-function::RendererMessageProcessor` follows this model and leaves ordinary actions and errors to their application or agent owner.[5]

## Current workspace fit

| Area                        | Current state                                                                                                                                      | Consequence                                                                                                        |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Wire format                 | `a2ui-author` defines both message directions, including function RPC.                                                                             | This is already the shared seam.                                                                                   |
| Typed application authoring | `a2ui-view`, `a2ui-codegen`, and `a2ui-basic` produce standard components and events.                                                              | Keep this path for developers and coding agents writing Rust at development time.                                  |
| Application updates         | `a2ui-app` emits a complete initial surface, then small data-path and component diffs.                                                             | Keep for Rust-owned state snapshots. Do not impose it on agent streams.                                            |
| Action handling             | `Application::handle` accepts `ActionMessage` and runs a Rust callback from the current view.                                                      | It is application-specific, not a complete renderer-to-agent processor.                                            |
| Function return messages    | `a2ui-function` validates and dispatches agent calls and pending renderer responses. Ordinary actions and errors remain unhandled.                 | Compose it with `Application` or an agent loop.                                                                    |
| View validation             | `a2ui-app::view::validate` checks IDs, names, catalog presence, duplicates, and a complete root. `a2ui-function` validates function calls offline. | Component-tree and progressive-stream validation still belong to the official renderer or a future agent compiler. |
| Agent prompt/compiler       | Not implemented.                                                                                                                                   | Use `@a2ui/agent` where its runtime is acceptable. Do not reproduce Express unnecessarily.                         |
| Transport and capabilities  | Application-owned.                                                                                                                                 | Preserve this constraint.                                                                                          |

Serde deserialization alone is not a complete trust-boundary check. Model output needs the official envelope and catalog schemas, structural checks, and stateful lifecycle checks. Renderer validation remains necessary as well.

## Recommended architecture

```text
                    one versioned catalog set
                   /                       \
        Rust builder generation       Agent prompt + validation
                 /                             \
 Rust state -> Element -> snapshot diff         Model
                 \                         Direct JSON or Express
                  \                    official/spec-conformant compiler
                   \                       /
                    AgentToRendererMessage stream
                               |
                    application-owned transport
                               |
                    official A2UI MessageProcessor
                               |
                    native catalog component renderers

 RendererToAgentMessage -> route by owning surface and message kind:
 - Rust application transition or effect
 - agent/model turn
 - agent or renderer function RPC continuation
 - renderer error handling
```

Each surface needs one clear owner. Different Rust applications and agents can own different surfaces in one renderer without protocol extensions. The official Agent SDK also supports handling a local action and declining others so they can continue to the model.[5]

Do not add a universal Rust `Application` trait just to join these paths. A live agent can emit messages without an incoming action, while `Application<S>` currently returns a batch after `start`, `handle`, or `close`. The standard message streams are already the smaller and more capable interface.

## Implementation status and next steps

Completed:

- Official schema and browser SDK revisions are aligned without compatibility transforms.
- `stream.test.mjs` proves progressive Direct JSON delivery through an `AsyncIterable` transport, renderer functions, resolved action context, and scoped updates without an LLM.
- `a2ui-function` provides offline function validation, typed agent handlers, bidirectional RPC, timeouts, cancellation, and renderer-message dispatch.
- An application integration test proves that function processing and `Application` actions compose.

Next only when required:

1. For a real TypeScript/Node agent, adopt official `@a2ui/agent` with the same catalogs and compatible `@a2ui/web_core` revision.
2. For Rust-only live agents, design a Direct JSON compiler against official conformance fixtures. Reuse the official architecture for prompt generation, streaming validation, and persistent renderer state. Do not implement Express or an LLM-provider abstraction without a concrete need.
3. Add full progressive component validation if a Rust compiler needs it. Function-only validation is not a substitute.

This gives one rendering and integration experience without forcing human-written applications and live agents to have identical internal state models.

## Sources

1. [Official A2UI overview: agent pipeline, static payloads, compiler, renderer, and actions](https://a2ui.org/concepts/overview/)
2. [Official A2UI v1.0 protocol specification](https://a2ui.org/specification/v1.0-a2ui/)
3. [Official A2UI catalog concepts](https://a2ui.org/concepts/catalogs/)
4. [Official A2UI project and package inventory](https://github.com/a2ui-project/a2ui)
5. [Official `@a2ui/agent` README](https://github.com/a2ui-project/a2ui/blob/main/typescript/agents/a2ui_agent/README.md)
