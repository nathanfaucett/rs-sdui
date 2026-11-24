# rs-a2ui

State-driven Rust authoring for the [official A2UI v1.0 specification](https://a2ui.org/specification/v1.0-a2ui/).

Write owned Rust state, explicit transitions, and fluent catalog views. Run the same application on a server, in WASM, or on another Rust target. The output is always standard A2UI JSON. Transport and rendering stay outside the library.

This workspace is not a full A2UI Core SDK, Agent SDK, or renderer. Official A2UI SDKs perform renderer-side expression evaluation, local input state, function execution, and rendering. The workspace provides offline function-schema validation and bidirectional function RPC, but no inference compiler.

## Rust crates

| Crate          | Purpose                                                                                                                                                            |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `a2ui`         | Protocol types, owned state machines, declarative views, generated Basic Catalog helpers, and browser lifecycle exports. Enable `function` for validation and RPC. |
| `a2ui-codegen` | `build.rs` generation of typed component builders and function helpers from catalog JSON Schema files.                                                             |
| `a2ui-derive`  | Proc-macro implementation of the `A2uiData` derive exported by `a2ui`.                                                                                             |

No third-party Rust A2UI implementation is used.

## TypeScript packages

| Package                    | Purpose                                                                                            |
| -------------------------- | -------------------------------------------------------------------------------------------------- |
| `@rs-a2ui/web`             | Renderer-independent async transports and SharedWorker synchronization for standard A2UI messages. |
| `@rs-a2ui/browser-example` | A complete Rust/WASM todo application with two side-by-side official Lit renderers.                |

All TypeScript, WASM, and npm packages live under `packages/`. The helper does not select a rendering library.

## Application model

The interface follows two useful Rust UI patterns:

- GPUI-style owned state and fluent builders.
- Leptos-style reusable view functions: state in, UI out.

Standard events and their state transitions are declared together in the view. No application trait, message enum, reducer, action registry, or action-conversion impl is required.

```rust
use a2ui::basic::v1::{self, Button, Column, Text};
use a2ui::{A2uiData, ActionMessage, Application, DataBinding, Element, Event, EventExt};
use serde::Serialize;

#[derive(A2uiData, Clone, Default, Serialize)]
struct Counter {
    value: i64,
}

// Reusable component: typed props in, a declarative view out.
fn value_label(path: DataBinding) -> impl Into<Element> + 'static {
    Text::new(path)
}

fn counter_view(_state: &Counter) -> Element {
    Column::new((
        value_label(Counter::paths().value()),
        Button::new(
            Text::new("Increment"),
            Event::new("increment").on(|state: &Counter, _action| Counter {
                value: state.value + 1,
            }),
        ),
    ))
    .into()
}

let mut app = Application::new(
    "counter",
    v1::CATALOG_ID,
    Counter::default(),
    counter_view,
);

let initial_messages = app.start()?;
let action: ActionMessage = serde_json::from_str(&incoming_json)?;
let update_messages = app.handle(action).await?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Derive `A2uiData` on state, nested structs, and scalar enums. Structs provide `Type::paths()`; enums act as scalar fields. Generated bindings follow Serde names, and removed Rust fields fail at compile time. Collection fields use `.index(index)`. The derive follows Serde `rename`, `rename_all`, `flatten`, `skip`, and `skip_serializing`. It currently accepts named, non-generic structs and enums. Use typed paths for application state; reserve `DataBinding::new` for relative list-template bindings and wire-level protocol code.

The view is a pure description. The runtime generates all component IDs, flattens the tree into the standard A2UI list, creates the surface, and serializes state as its root data model. State must implement `Serialize` and produce a JSON object. Use `#[serde(skip)]` for clients or other non-data fields.

Component functions can return a concrete generated type, `impl Into<Element>`, or `Element` when branches need type erasure. Tuples hold heterogeneous children. Arrays and `Vec` hold homogeneous children. `Children` can also collect an iterator of generated or erased views. No component macro is required.

`Event::context` keeps standard A2UI expressions and data bindings in the JSON. The official renderer evaluates them before sending the action. `.on(...)` attaches a pure Rust transition from the current state and action to the next state. Neither the callback nor any custom action metadata is serialized.

Handlers are scoped by the standard `surfaceId`, `sourceComponentId`, and event name. Different components can reuse the same event name; the runtime does not rewrite or manually namespace it. An action with the right name but the wrong source component is rejected. For list actions, put a stable item ID in `Event::context` instead of encoding it in the event name. This keeps stale actions from affecting another row after the list changes.

Use `.on_effect(...)` to declare an event that performs I/O and returns the next state. The workspace does not provide an HTTP client or a `Client` type; those belong to the application. This is the browser example's actual effect:

```rust,ignore
fn load_examples_button() -> Button {
    Button::new(
        Text::new("Load example tasks"),
        Event::new("load_examples").on_effect(|state: &TodoApp, _| {
            let state = state.clone();
            Effect::new(async move {
                let examples = load_examples().await?;
                state.with_loaded(examples)
            })
        }),
    )
}
```

`load_examples()` uses browser `fetch` on WASM and reads the same bundled JSON on native targets. `with_loaded()` validates the result and returns the next `TodoApp`. See the complete implementations in `packages/browser-example/rust/src/load.rs` and `packages/browser-example/rust/src/ui.rs`; this path is covered by native tests and browser validation. An application with a shared API client can capture its `Arc` in the same way, but the library does not prescribe that dependency.

The callback and future are Rust-only. A2UI JSON contains the standard event, not the I/O operation. `handle()` awaits the effect, validates the next state and view, then commits both. Native effects are `Send`; WASM effects are local. `handle()` does not spawn work or select an executor. Choose Tokio, another executor, a local WASM task, or direct `.await` in the application.

### Complete JSON tree

Serialize `start()` to inspect the entire UI:

```rust
let complete_a2ui_json = serde_json::to_string_pretty(&app.start()?)?;
```

A2UI represents the tree as a flat component list with generated ID references. For the counter above, the initial message is:

```json
[
  {
    "version": "1.0",
    "createSurface": {
      "surfaceId": "counter",
      "catalogId": "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json",
      "components": [
        {
          "id": "root",
          "component": "Column",
          "children": ["root_0", "root_1"]
        },
        {
          "id": "root_0",
          "component": "Text",
          "text": { "@path": "/value" }
        },
        {
          "id": "root_1",
          "component": "Button",
          "child": "root_1_0",
          "action": { "event": { "name": "increment" } }
        },
        { "id": "root_1_0", "component": "Text", "text": "Increment" }
      ],
      "dataModel": { "value": 0 }
    }
  }
]
```

The complete visual tree and state are JSON. Rust transitions and I/O callbacks are application behavior and intentionally have no A2UI JSON representation. The exact multi-component wire shape is also asserted in `crates/a2ui/tests/app/application.rs`.

Incrementing this counter emits only:

```json
[
  {
    "version": "1.0",
    "updateDataModel": {
      "surfaceId": "counter",
      "path": "/value",
      "value": 1
    }
  }
]
```

No component update is needed because the existing text binding reads the new value.

Serialize each returned `AgentToRendererMessage` in order. Send it through a local call, WebSocket, SSE, HTTP, or another application-owned transport. The Rust application runtime has no transport dependency. `@rs-a2ui/web` provides browser transport adapters without selecting a renderer.

### Emission and state rules

- `start()` validates the declarative view and emits one complete `createSurface`. Serialize it directly to a file, stream, or transport.
- `snapshot()` clones the current surface into one complete `createSurface` for a newly connected renderer. It does not rerun the view or change lifecycle state.
- `handle()` runs a named action. An action error, state-serialization error, or invalid next view rolls back Rust state. External side effects cannot roll back.
- Changed state emits path-scoped `updateDataModel` messages. Arrays and scalar values update at their exact paths.
- Removing an object key replaces the smallest parent object because A2UI has no key-deletion message. Other changes do not send the root model.
- Bound paths are refreshed after each completed action. A2UI inputs can change renderer-local data without notifying Rust, so these small updates restore application-owned values.
- Unchanged, unbound state emits no data-model messages.
- Only added or changed component definitions are sent in `updateComponents`.
- Removed component definitions are left unreachable in the renderer map. A2UI has no component-deletion message. This avoids unnecessary surface resets and focus loss.

- `handle()` accepts standard `action` envelopes and rejects actions for another surface.
- `close()` emits `deleteSurface`. A later `start()` is valid.

One `Application` owns one fixed surface ID and catalog. Use separate instances with unique surface IDs for multiple surfaces. Capture clients and other external resources in action/effect callbacks instead of storing them in cloned UI state.

`Application::handle` intentionally accepts only standard `action` envelopes. Enable the `function` feature and use `a2ui::function::RendererMessageProcessor` for function RPC. Its `Dispatch::Unhandled` variant returns ordinary actions and errors to the application or agent loop. `Application` does not evaluate expressions.

## Catalog functions

Generated function builders have typed required arguments, typed fluent optional arguments, and conversions based on `returnType`, `allowedCallers`, and `requiresUserActivation`:

```rust
use a2ui::CheckRule;
use a2ui::basic::v1::{self, Button, Text};

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

Use renderer functions for formatting, checks, and local actions. Use `Event` transitions for authoritative application state. Use agent functions for discrete backend operations without an LLM turn.

The optional `a2ui::function` module validates complete function calls offline against the official common schema and each catalog. `AgentFunctions::register` deserializes resolved arguments into a Rust type and validates the return JSON. `RendererFunctionCalls` correlates `functionCallId`, validates results, and supports optional timeouts, explicit cancellation, surface cancellation, and disposal. No executor or transport is selected.

```rust,ignore
let catalogs = a2ui::function::CatalogSet::from_json(catalog_json)?;
let mut processor = a2ui::function::RendererMessageProcessor::new(catalogs);
processor.agent_functions_mut().register(
    catalog_id,
    "checkInventory",
    |args: InventoryArgs, context| async move {
        check_inventory(args, context).await
    },
)?;

match processor.process(renderer_message).await {
    a2ui::function::Dispatch::Handled(messages) => send(messages),
    a2ui::function::Dispatch::Unhandled(a2ui::RendererToAgentMessage::Action(action)) => {
        send(app.handle(action).await?);
    }
    a2ui::function::Dispatch::Unhandled(message) => handle_other(message),
}
```

## Author a surface without the runtime

```rust
use a2ui::basic::v1::{self, Button, Column, Text};
use a2ui::{A2uiData, Event};
use serde::Serialize;

#[derive(A2uiData, Serialize)]
struct Form {
    name: String,
}

#[derive(A2uiData, Serialize)]
struct State {
    form: Form,
}

let message = v1::surface("main").root(Column::new((
    Text::new("A2UI from Rust"),
    Button::new(
        Text::new("Submit"),
        Event::new("submit").context("name", State::paths().form().name()),
    ),
)));

let standard_a2ui_json = serde_json::to_value(message.into_message())?;
```

The result is a standard `createSurface` message with `version: "1.0"`.

## Generate helpers for your catalogs

Add `a2ui` and `serde_json` as dependencies, and `a2ui-codegen` as a build dependency.

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    a2ui_codegen::generate(&[
        a2ui_codegen::Catalog::new("basic", "catalogs/basic.json"),
        a2ui_codegen::Catalog::new("travel", "catalogs/travel.json"),
    ])
}
```

```rust
// src/generated.rs
include!(concat!(env!("OUT_DIR"), "/a2ui_catalogs.rs"));
```

```rust
// src/lib.rs: exports only
pub mod generated;
pub use generated::*;
```

Each catalog gets its own module, `CATALOG_ID`, `surface()` constructor, typed nested component builders, and a `functions` module. Required properties are constructor arguments; optional properties are fluent methods. Child properties accept other generated or composed views. Multiple catalogs can be used on one surface with the standard `@catalogId` and `catalogId` fields.

The generator supports A2UI v1.0 only. Complex schema shapes without a small safe Rust mapping accept `serde_json::Value`; the generator does not invent replacement semantics. Required function arguments are constructor parameters and optional arguments are typed fluent methods. Generated conversions respect the declared return type, caller permissions, and user-activation requirements. Raw `FunctionCall` remains the low-level escape hatch.

## Browser example

```sh
pnpm install
pnpm run dev
```

The todo page implements add, edit, complete/reopen, delete, filter, clear completed, and async example loading. It uses renderer `formatString`, `openUrl`, and `@checks` for reactive text, local navigation, and immediate whitespace-title validation. Rust still validates all submitted titles.

The page starts with two configurable application instances. **Add instance** creates more. Each instance selects a WASM SharedWorker or the native Rust WebSocket server and a session ID. Instances with the same backend and session ID share one runtime; a different ID creates independent state. Both backends serialize actions and effects, broadcast ordered updates, and send `snapshot()` to late clients. Renderer-local edits remain local until an action, as A2UI specifies.

WASM is necessary because the Rust application executes inside that browser worker. `a2ui` owns the reusable FFI on WASM targets; the example declares only `a2ui::export_application!(crate::application, crate::TodoApp)`. `@rs-a2ui/web` owns worker synchronization. A server-hosted Rust application does not use the browser binding.

`packages/web` defines the renderer-independent `Transport`/`Connection` seam and SharedWorker lifecycle helpers. Every connection exposes inbound messages as an `AsyncIterable` and accepts outbound renderer messages through `send()`. `SharedWorkerTransport`, `WebSocketTransport`, and `LocalStreamTransport` implement the same contract. The wire contains standard A2UI messages only; session IDs belong in worker names and WebSocket URLs. The example owns its Lit adapter in `packages/browser-example/src/mount-a2ui.ts`; other applications can use any rendering library.

A reconnection must use a fresh renderer mount and message processor. The new connection sends `Application::snapshot()` before new updates. The official processor rejects `createSurface` for an existing surface ID, so sending a snapshot into the old processor is invalid. Each renderer's **Go offline**/**Go online** toggle demonstrates missed updates and snapshot recovery while the other renderer keeps the worker alive. A transport with durable sequence replay can replay missed updates instead; otherwise, a full snapshot is the recovery mechanism. Do not blindly replay outbound actions because their side effects can already have run. If the application owner also stops, its in-memory state is gone; recovery then requires application-owned persistence.

The `dev` and `preview` scripts start the Rust WebSocket server on a dynamic internal port and use the named `a2ui-browser` portless route. Vite proxies same-origin `/ws` connections to Rust. Portless supplies a dynamic `PORT` to Vite and prints the local `.localhost` URL. The standalone server listens on `127.0.0.1:8081`; override it with `A2UI_WS_ADDR`.

Useful checks:

```sh
pnpm run check   # builds WASM and checks TypeScript and Biome
pnpm run build   # production WASM + Vite build
```

The example uses official `@a2ui/web_core 0.16.1` and `@a2ui/lit 0.16.0` with the matching official v1.0 Basic Catalog. `@rs-a2ui/web` depends only on protocol types and schemas from `web_core`, not Lit or another renderer. No compatibility transform is applied. Keep the schema and SDK revisions aligned.

## Validation

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
```

## Scope

These crates provide application state, authoring, code generation, offline function validation, and bidirectional function RPC. They do not provide catalog negotiation, a network transport, an async executor, renderer-side expression evaluation, inference compilation, or native rendering.
