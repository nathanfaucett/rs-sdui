# sdui

Server-driven UI (SDUI) tools built on the [official A2UI v1.0 specification](https://a2ui.org/specification/v1.0-a2ui/).

Write owned Rust state, explicit transitions, and fluent catalog views. Run the same application on a server, in WASM, or on another Rust target. The output is always standard A2UI JSON. Transport and rendering stay outside the library.

This workspace is not a full A2UI Core SDK, Agent SDK, or renderer. Official A2UI SDKs perform renderer-side expression evaluation, local input state, function execution, and rendering. The workspace provides offline function-schema validation and bidirectional function RPC, but no inference compiler, catalog negotiation, async executor, or native rendering.

## Rust crates

| Crate          | Purpose                                                                                                                                 |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `a2ui`         | Standard A2UI v1.0 component, dynamic-value, and message types.                                                                         |
| `a2ui-a2a`     | Optional native A2UI v1.0 bindings for official A2A v1 JSON-RPC clients and servers.                                                    |
| `sdui`         | Owned state machines, typed state bindings, declarative views, and browser lifecycle exports. Enable `function` for validation and RPC. |
| `sdui-codegen` | `build.rs` generation of typed component builders and function helpers from catalog JSON Schema files.                                  |
| `sdui-derive`  | Proc-macro implementation of the `A2uiData` derive exported by `sdui`.                                                                  |

Use `a2ui` alone for wire-level protocol code. `sdui` depends on `a2ui` but does not re-export its types. No third-party Rust A2UI implementation is used.

`sdui` does not include a component catalog. Applications own their catalog schemas and generate typed builders with `sdui-codegen`. The examples below use an application-owned `generated::basic` module.

## TypeScript packages

| Package     | Purpose                                                                                         |
| ----------- | ----------------------------------------------------------------------------------------------- |
| `@sdui/web` | Renderer-independent processor connections, async transports, and SharedWorker synchronization. |

Reusable TypeScript, WASM, and npm packages live under `packages/`. See the [`@sdui/web` guide](packages/web/README.md) for transport, worker, and reconnection APIs.

## Applications

| Application             | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `@sdui/browser-example` | A complete Rust/WASM todo application with two side-by-side official Lit renderers. |

Deployable and demo applications live under `apps/`.

## Application quick start

Use owned Rust state and reusable view functions. Standard events and their state transitions are declared together in the view. No application trait, message enum, reducer, action registry, or action-conversion impl is required.

```rust
use a2ui::{ActionMessage, DataBinding, Event};
use crate::generated::basic::{self, Button, Column, Text};
use sdui::{A2uiData, Application, Element, EventExt};
use serde::Serialize;

#[derive(A2uiData, Clone, Default, Serialize)]
struct Counter {
    value: i64,
}

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
    basic::CATALOG_ID,
    Counter::default(),
    counter_view,
);

let initial_messages = app.start()?;
let action: ActionMessage = serde_json::from_str(&incoming_json)?;
let update_messages = app.handle(action).await?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The view is a pure description. The runtime generates component IDs, flattens the tree into the standard A2UI list, creates the surface, and serializes state as its root data model.

Incrementing this counter emits only the changed state:

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

Serialize each returned `AgentToRendererMessage` in order. Send it through a local call, WebSocket, SSE, HTTP, or another application-owned transport. The Rust application runtime has no transport dependency.

See the [`sdui` application guide](crates/sdui/README.md) for typed paths, events, effects, the complete JSON tree, emission and rollback rules, catalog functions, and authoring without the runtime.

## Generate helpers for your catalogs

Add `a2ui`, `sdui`, and `serde_json` as dependencies, and `sdui-codegen` as a build dependency. Generated code refers directly to both `a2ui` and `sdui`.

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    sdui_codegen::generate(&[
        sdui_codegen::Catalog::new("basic", "catalogs/basic.json"),
        sdui_codegen::Catalog::new("travel", "catalogs/travel.json"),
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

Each catalog gets its own module, `CATALOG_ID`, `surface()` constructor, typed nested component builders, and a `functions` module. Required properties are constructor arguments; optional properties are fluent methods. Child properties accept other generated or composed views. The generator supports A2UI v1.0 only.

## Official A2A binding

`a2ui-a2a` connects an unmodified `Application<S>` to official A2A v1 JSON-RPC using standard A2UI messages in official A2A `DataPart` arrays. Its default feature set is empty; enable `client`, `server`, or both.

See the [`a2ui-a2a` guide](crates/a2ui-a2a/README.md) for identity, session and task stores, cancellation, recovery, and current limits.

## Browser example

```sh
pnpm install
pnpm run dev
```

The todo example provides selectable WASM SharedWorker and native Rust WebSocket backends, standard A2UI messages, and official Lit renderers. See the [browser example guide](apps/browser-example/README.md) for sessions, reconnection, local processes, and validation.

## Validation

Install the formatting and lint hooks with `pre-commit install`. Run them against every tracked file with `pre-commit run --all-files`.

See [local validation](AGENTS.md#local-validation) for workspace and production browser checks.
