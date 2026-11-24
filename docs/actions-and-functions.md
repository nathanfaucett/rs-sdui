# Actions and functions in the application model

Date: 2026-11-23. Updated after the function runtime and browser examples were implemented.

## Model

Use each A2UI interaction mechanism for its intended job:

| Mechanism                      | Use for                                                                          | Main benefit                                           |
| ------------------------------ | -------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `Event` plus a Rust transition | User intent that changes authoritative application state or starts an agent turn | Predictable next state and A2UI diffs                  |
| `Event` plus a Rust effect     | State-changing I/O owned by the application                                      | One async transition with rollback before commit       |
| Renderer function              | Formatting, validation, local context, and local user actions                    | Immediate response; no network or agent turn           |
| Agent function                 | One discrete backend query or operation with a typed result                      | No LLM turn; standard async pending and error behavior |
| `callRendererFunction`         | An agent-initiated client operation authorized by the catalog                    | Access to client capabilities without executable code  |

Events and functions have different result semantics. Function calls do not replace predictable application transitions.

## Official behavior

An interactive component has an `action`. In v1.0 its schema is a `oneOf`: exactly one `event` or one `functionCall`. There is no action-list or custom pipeline field.[1][3]

- `action.event` resolves `userMessage` and `context`, sends an `action` envelope, and lets the agent decide what UI or data updates to send.[1]
- `action.functionCall` invokes a catalog operation directly without an agent conversation turn.[1]

The renderer first looks for a local function implementation. If none exists, it sends `callAgentFunction` and waits for `agentFunctionResponse`. Functions can also run in dynamic properties and `@checks`. Reactive calls may share an in-flight call or cached result. Functions with side effects belong in actions, not reactive properties or checks.[1][2]

Dynamic properties re-evaluate when their bound inputs change. A failing check on a `Button` disables it. This gives immediate feedback but does not replace server-side validation.[1]

Catalog function metadata controls execution:[1][2]

- `allowedCallers`: `rendererOnly` by default, `agentOnly`, or `rendererOrAgent`.
- `requiresUserActivation`: restricts execution to a direct user action and requires `rendererOnly`.

`callRendererFunction` asks the client to execute a local function. The renderer returns a matching `rendererFunctionResponse`, including for a `void` result. Every response contains `functionCallId` and exactly one of `value` or `error`.[2][3]

## Implemented workspace support

### Generated builders

`a2ui-codegen` now generates a function-specific builder for every catalog function:

- Required arguments remain typed constructor parameters.
- Optional arguments are typed fluent methods.
- `From<Builder> for FunctionCall` keeps low-level RPC available.
- `returnType` controls conversions to `DynamicString`, `DynamicNumber`, `DynamicBoolean`, `DynamicValue`, and `Condition`.
- `void` has no dynamic-value conversion.
- `agentOnly` has no component action or dynamic-property conversion.
- `requiresUserActivation` permits only component-action conversion.

For example:

```rust
let price = Text::new(
    v1::functions::format_currency("USD", Booking::paths().price())
        .decimals(2)
        .grouping(true),
);
let required = CheckRule::new(v1::functions::required(Booking::paths().price()));
let help = Button::new(
    Text::new("Documentation"),
    v1::functions::open_url("https://a2ui.org/"),
);
```

The Basic Catalog integration tests assert exact JSON for formatting, dynamic conversion, checks, and function actions.

### Strict wire types

`a2ui-author` now enforces:

- Exactly one `value` or `error` in `FunctionResponse`, including `null` as a valid value.
- Explicit `catalogId` in both agent and renderer RPC calls.
- Unknown-field rejection for function calls and RPC payloads.
- Private RPC fields and fallible constructors, preventing invalid public construction.

Normal dynamic function calls can still omit `catalogId` when the surface default applies.

### Offline validation and execution

`a2ui-function` contains:

- `Catalog` and `CatalogSet` for offline JSON Schema validation with the official v1.0 common types.
- `AgentFunctions::register` for typed resolved arguments and serializable typed returns.
- Return-shape validation for every standard `returnType`.
- Caller-permission and user-activation checks.
- Standard unknown-function, invalid-call, and execution-error responses.

Validation errors do not include raw argument values.

### Bidirectional RPC

`RendererFunctionCalls` provides:

- Correlation by `functionCallId`.
- Duplicate-ID rejection.
- Optional per-call timeouts.
- Explicit and per-surface cancellation.
- Disposal and pending-future drop cleanup.
- Response return-type validation.

No executor is spawned and no transport is selected.

`RendererMessageProcessor` handles `callAgentFunction` and pending `rendererFunctionResponse` messages. It returns `Dispatch::Unhandled` for actions, renderer errors, and unknown responses. An application-level test proves that an unhandled action continues into `Application::handle` and produces its normal path-scoped update.

## Browser examples

The browser uses official `@a2ui/web_core 0.16.1` and `@a2ui/lit 0.16.0` with the matching Basic Catalog.

The todo page demonstrates:

- `formatString` for a reactive filter subtitle.
- `openUrl` for documentation without a Rust action.
- `regex("\\S", path)` in `@checks` on draft/edit fields and Add/Save buttons.
- Continued Rust validation and trimming at the trust boundary.
- Existing async state-changing effects.

`stream.test.mjs` delivers seven deterministic standard messages through the local `AsyncIterable` transport. It includes unresolved child references, `formatString`, `formatCurrency`, resolved event context, and small path updates. Official schema and processor tests validate the complete stream and action response.

## Remaining limits

- Function RPC requires an explicit resolved `catalogId`. Surface-default resolution and expression evaluation belong to the renderer/agent caller.
- `a2ui-function` validates function calls, not every component property or complete progressive component tree. The official renderer performs those checks.
- The protocol does not include user-activation proof in RPC messages. Agent-initiated calls that require activation are rejected; renderer-originated activation remains a renderer responsibility.
- Pending calls have no count limit. Applications can add their own quota if exposed to untrusted callers.
- Generated resolved-argument types are supplied by the application, not emitted from every catalog schema.

## Sources

1. [Official A2UI actions, events, and functions](https://a2ui.org/concepts/actions-and-functions/)
2. [Official A2UI v1.0 protocol specification](https://a2ui.org/specification/v1.0-a2ui/)
3. [Official A2UI v1.0 common types schema](https://a2ui.org/specification/v1_0/common_types.json)
4. [Official `@a2ui/agent` renderer-message and function processing](https://github.com/a2ui-project/a2ui/blob/main/typescript/agents/a2ui_agent/README.md)
