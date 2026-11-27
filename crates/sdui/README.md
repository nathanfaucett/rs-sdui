# `sdui` application guide

This guide covers the detailed `sdui` application contracts. See the [workspace quick start](../../README.md#application-quick-start) for a complete counter example and catalog generation setup.

## State and views

Derive `A2uiData` on state, nested structs, and scalar enums. Structs provide `Type::paths()`; enums act as scalar fields. Generated bindings follow Serde names, and removed Rust fields fail at compile time. Collection fields use `.index(index)`. The derive follows Serde `rename`, `rename_all`, `flatten`, `skip`, and `skip_serializing`. It currently accepts named, non-generic structs and enums. Use typed paths for application state; reserve `DataBinding::new` for relative list-template bindings and wire-level protocol code.

The view is a pure description. The runtime generates all component IDs, flattens the tree into the standard A2UI list, creates the surface, and serializes state as its root data model. State must implement `Serialize` and produce a JSON object. Use `#[serde(skip)]` for clients or other non-data fields.

Component functions can return a concrete generated type, `impl Into<Element>`, or `Element` when branches need type erasure. Tuples hold heterogeneous children. Arrays and `Vec` hold homogeneous children. `Children` can also collect an iterator of generated or erased views. No component macro is required.

## Events and effects

`Event::context` keeps standard A2UI expressions and data bindings in the JSON. The official renderer evaluates them before sending the action. `.on(...)` attaches a pure Rust transition from the current state and action to the next state. Neither the callback nor any custom action metadata is serialized.

Handlers are scoped by the standard `surfaceId`, `sourceComponentId`, and event name. Different components can reuse the same event name; the runtime does not rewrite or manually namespace it. An action with the right name but the wrong source component is rejected. For list actions, put a stable item ID in `Event::context` instead of encoding it in the event name. This keeps stale actions from affecting another row after the list changes.

Use `.on_effect(...)` to declare an event that performs I/O and returns the next state. The workspace does not provide an HTTP client or a `Client` type; those belong to the application. Capture an application-owned client or its `Arc` in the effect callback.

The callback and future are Rust-only. A2UI JSON contains the standard event, not the I/O operation. `handle()` awaits the effect, validates the next state and view, then commits both. Native effects are `Send`; WASM effects are local. `handle()` does not spawn work or select an executor. Choose Tokio, another executor, a local WASM task, or direct `.await` in the application.

## Complete JSON tree

Serialize `start()` to inspect the entire UI:

```rust
let complete_a2ui_json = serde_json::to_string_pretty(&app.start()?)?;
```

A2UI represents the tree as a flat component list with generated ID references. For the [counter example](../../README.md#application-quick-start), the initial message is:

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

The complete visual tree and state are JSON. Rust transitions and I/O callbacks are application behavior and intentionally have no A2UI JSON representation. The exact multi-component wire shape is also asserted in `crates/sdui/tests/app/application.rs`.

## Emission and state rules

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

`Application::handle` intentionally accepts only standard `action` envelopes. Enable the `function` feature and use `sdui::function::RendererMessageProcessor` for function RPC. Its `Dispatch::Unhandled` variant returns ordinary actions and errors to the application or agent loop. `Application` does not evaluate expressions.

## Catalog functions

Generated function builders have typed required arguments, typed fluent optional arguments, and conversions based on `returnType`, `allowedCallers`, and `requiresUserActivation`:

```rust
use a2ui::CheckRule;
use crate::generated::basic::{self, Button, Text};

let price = Text::new(
    basic::functions::format_currency("USD", Booking::paths().price())
        .decimals(2)
        .grouping(true),
);
let required = CheckRule::new(basic::functions::required(Booking::paths().price()));
let help = Button::new(
    Text::new("Documentation"),
    basic::functions::open_url("https://a2ui.org/"),
);
```

Use renderer functions for formatting, checks, and local actions. Use `Event` transitions for authoritative application state. Use agent functions for discrete backend operations without an LLM turn.

The optional `sdui::function` module validates complete function calls offline against the official common schema and each catalog. `AgentFunctions::register` deserializes resolved arguments into a Rust type and validates the return JSON. `RendererFunctionCalls` correlates `functionCallId`, validates results, and supports optional timeouts, explicit cancellation, surface cancellation, and disposal. No executor or transport is selected.

```rust,ignore
let catalogs = sdui::function::CatalogSet::from_json(catalog_json)?;
let mut processor = sdui::function::RendererMessageProcessor::new(catalogs);
processor.agent_functions_mut().register(
    catalog_id,
    "checkInventory",
    |args: InventoryArgs, context| async move {
        check_inventory(args, context).await
    },
)?;

match processor.process(renderer_message).await {
    sdui::function::Dispatch::Handled(messages) => send(messages),
    sdui::function::Dispatch::Unhandled(a2ui::RendererToAgentMessage::Action(action)) => {
        send(app.handle(action).await?);
    }
    sdui::function::Dispatch::Unhandled(message) => handle_other(message),
}
```

Complex catalog schema shapes without a small safe Rust mapping accept `serde_json::Value`; the generator does not invent replacement semantics. Generated conversions respect the declared return type, caller permissions, and user-activation requirements. Raw `FunctionCall` remains the low-level escape hatch. Multiple catalogs can be used on one surface with the standard `@catalogId` and `catalogId` fields.

## Author a surface without the runtime

```rust
use a2ui::Event;
use crate::generated::basic::{self, Button, Column, Text};
use sdui::{A2uiData, SurfaceExt};
use serde::Serialize;

#[derive(A2uiData, Serialize)]
struct Form {
    name: String,
}

#[derive(A2uiData, Serialize)]
struct State {
    form: Form,
}

let message = basic::surface("main").root(Column::new((
    Text::new("A2UI from Rust"),
    Button::new(
        Text::new("Submit"),
        Event::new("submit").context("name", State::paths().form().name()),
    ),
)));

let standard_a2ui_json = serde_json::to_value(message.into_message())?;
```

The result is a standard `createSurface` message with `version: "1.0"`.
