# rs-a2ui

State-driven Rust authoring for the [official A2UI v1.0 specification](https://a2ui.org/specification/v1.0-a2ui/).

Write owned Rust state, explicit transitions, and fluent catalog views. Run the same application on a server, in WASM, or on another Rust target. The output is always standard A2UI JSON. Transport and rendering stay outside the library.

This workspace is not an A2UI Core SDK, Agent SDK, or renderer. Official A2UI SDKs perform renderer-side validation, expression evaluation, local input state, function execution, and rendering.

## Crates

| Crate              | Purpose                                                                                                                                |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| `a2ui-app`         | Owned state machines, declarative view mounting, automatic state data models, action decoding, component diffs, and surface lifecycle. |
| `a2ui-author`      | Transport-independent A2UI v1.0 messages, components, bindings, actions, and function calls.                                           |
| `a2ui-codegen`     | `build.rs` generation of typed component builders and function helpers from catalog JSON Schema files.                                 |
| `a2ui-view`        | Declarative element trees, child composition, typed event callbacks, and async effects.                                                |
| `a2ui-basic`       | Generated helpers for the official v1.0 Basic Catalog. Its catalog file is copied verbatim from `a2ui-project/a2ui`.                   |
| `examples/browser` | A complete local Rust/WASM application with Vite, TypeScript, and the official `@a2ui/web_core` and `@a2ui/lit` packages.              |

No third-party Rust A2UI implementation is used.

## Application model

The interface follows two useful Rust UI patterns:

- GPUI-style owned state and fluent builders.
- Leptos-style reusable view functions: state in, UI out.

Standard events and their state transitions are declared together in the view. No application trait, message enum, reducer, action registry, or action-conversion impl is required.

```rust
use a2ui_app::Application;
use a2ui_author::{ActionMessage, Event};
use a2ui_basic::v1::{self, Button, Column, Text};
use a2ui_view::{Element, EventExt};
use serde::Serialize;

#[derive(Clone, Default, Serialize)]
struct Counter {
    value: i64,
}

// Reusable component: typed props in, a declarative view out.
fn value_label(value: i64) -> impl Into<Element> {
    Text::new(value.to_string())
}

fn counter_view(state: &Counter) -> Element {
    Column::new((
        value_label(state.value),
        Button::new(
            Text::new("Increment"),
            Event::new("increment").on(|state: &mut Counter, _action| state.value += 1),
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

The view is a pure description. The runtime generates all component IDs, flattens the tree into the standard A2UI list, creates the surface, and serializes state as its root data model. State must implement `Serialize` and produce a JSON object. Use `#[serde(skip)]` for clients or other non-data fields.

Component functions can return a concrete generated type, `impl Into<Element>`, or `Element` when branches need type erasure. Tuples hold heterogeneous children. Arrays and `Vec` hold homogeneous children. `Children` can also collect an iterator of generated or erased views. No component macro is required.

`Event::context` keeps standard A2UI expressions and data bindings in the JSON. The official renderer evaluates them before sending the action. `.on(...)` attaches a Rust-only callback; neither the callback nor any custom action metadata is serialized.

Use `.on_effect(...)` for I/O and other side effects. The effect returns a closure that applies its result to state, so ordinary errors work with `?`:

```rust,ignore
fn load_button(client: Arc<Client>) -> impl Into<Element> {
    Button::new(
        Text::new("Load"),
        Event::new("load").on_effect(move |_state, _action| {
            let client = client.clone();
            Effect::new(async move {
                let value = client.load().await?;
                Ok(move |state: &mut Counter| state.value = value)
            })
        }),
    )
}
```

The action handler can also update state before returning `Effect::new(...)`. Native effects are `Send`; WASM effects are local. `handle()` drives the future but does not spawn it. Choose Tokio, another executor, a local WASM task, or direct `.await` in the application.

Serialize each returned `AgentToRendererMessage` in order. Send it through a local call, WebSocket, SSE, HTTP, or another application-owned transport. This workspace intentionally has no transport abstraction or transport dependency.

### Emission and state rules

- `start()` validates the declarative view and emits one complete `createSurface`. Serialize it directly to a file, stream, or transport.
- `handle()` runs a named action. An action error, state-serialization error, or invalid next view rolls back Rust state. External side effects cannot roll back.
- Each completed action emits a root `updateDataModel`, even if the Rust view is unchanged. A2UI inputs can change renderer-local data without notifying Rust. The application state is authoritative after each action.
- Only added or changed component definitions are sent in `updateComponents`.
- Removed component definitions are left unreachable in the renderer map. A2UI has no component-deletion message. This avoids unnecessary surface resets and focus loss.

- `handle()` accepts standard `action` envelopes and rejects actions for another surface.
- `close()` emits `deleteSurface`. A later `start()` is valid.

One `Application` owns one fixed surface ID and catalog. Use separate instances with unique surface IDs for multiple surfaces. Capture clients and other external resources in action/effect callbacks instead of storing them in cloned UI state.

The action loop does not interpret `callAgentFunction`, `rendererFunctionResponse`, or renderer errors. Applications that need those flows can use the standard types in `a2ui-author` directly. `a2ui-app` does not evaluate expressions or execute functions.

## Author a surface without the runtime

```rust
use a2ui_author::{DataBinding, Event};
use a2ui_basic::v1::{self, Button, Column, Text};

let message = v1::surface("main").root(Column::new((
    Text::new("A2UI from Rust"),
    Button::new(
        Text::new("Submit"),
        Event::new("submit").context("name", DataBinding::new("/form/name")),
    ),
)));

let standard_a2ui_json = serde_json::to_value(message.into_message())?;
```

The result is a standard `createSurface` message with `version: "1.0"`.

## Generate helpers for your catalogs

Add `a2ui-author`, `a2ui-view`, and `serde_json` as dependencies, and `a2ui-codegen` as a build dependency.

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

The generator supports A2UI v1.0 only. Complex schema shapes without a small safe Rust mapping accept `serde_json::Value`; the generator does not invent replacement semantics. Optional function arguments use the standard `FunctionCall::arg()` builder.

## Browser example

```sh
cd examples/browser/typescript
npm install
npm run dev
```

The example implements a submit/reset state machine. The official Lit renderer sends a standard action to the local WASM application. Rust returns standard A2UI updates to the official `MessageProcessor`. The local call is the only transport used; replace the callback to use a remote transport.

Useful checks:

```sh
npm test        # official MessageProcessor validates the complete native state loop
npm run check   # builds WASM and checks TypeScript
npm run build   # production WASM + Vite build
```

Official npm SDK `0.15.0` currently rejects the current schema's `@checks` field and uses older unprefixed component-envelope keys. The browser example avoids those fields. No compatibility transform is applied. Use a renderer and catalog that implement the same v1.0 schema revision.

The optional `@a2ui/markdown-it 0.2.2` package declares `web_core ^0.14`, so it cannot be installed cleanly with SDK `0.15.0`. Plain text renders, but Markdown formatting is unavailable until the official packages align.

## Validation

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

## Scope

These crates provide application state, authoring, and code generation. They do not provide catalog negotiation, a network transport, an async executor, renderer-side expression evaluation, inference compilation, or native rendering.
