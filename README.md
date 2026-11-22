# rs-a2ui

Rust authoring helpers for the [official A2UI v1.0 specification](https://a2ui.org/specification/v1.0-a2ui/).

The crates produce standard A2UI JSON messages. They do not define a protocol, inference format, transport, surface runtime, or renderer. Official A2UI SDKs perform validation, expression evaluation, state management, and rendering.

## Crates

| Crate | Purpose |
| --- | --- |
| `a2ui-author` | Transport-independent authoring types for A2UI v1.0 messages, components, bindings, actions, and function calls. |
| `a2ui-codegen` | `build.rs` generation of typed component builders and function helpers from catalog JSON Schema files. |
| `a2ui-basic` | Generated helpers for the official v1.0 Basic Catalog. Its catalog file is copied verbatim from `a2ui-project/a2ui`. |
| `examples/browser` | Rust-to-WASM authoring with a simple Vite + TypeScript app and the official `@a2ui/web_core` and `@a2ui/lit` packages. |

No third-party Rust A2UI implementation is used.

## Author a surface

```rust
use a2ui_author::{DataBinding, Event};
use a2ui_basic::v1::{self, Button, Column, Text};

let message = v1::surface("main").components([
    Column::new("root", ["heading", "submit"]).into(),
    Text::new("heading", "A2UI from Rust").into(),
    Text::new("submit_label", "Submit").into(),
    Button::new(
        "submit",
        "submit_label",
        Event::new("submit").context("name", DataBinding::new("/form/name")),
    )
    .into(),
]);

let standard_a2ui_json = serde_json::to_value(message.into_message())?;
```

The result is a standard `createSurface` message with `version: "1.0"`. Serialize messages directly into a local call, WebSocket, SSE, or another application-owned transport. This workspace intentionally has no transport abstraction or transport dependency.

## Generate helpers for your catalogs

Add `a2ui-author` and `serde_json` as dependencies, and `a2ui-codegen` as a build dependency.

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

Each catalog gets its own module, `CATALOG_ID`, `surface()` constructor, component builders, and a `functions` module. Multiple catalogs can be used on one surface with the standard `@catalogId` and `catalogId` fields.

The generator supports A2UI v1.0 only. Complex schema shapes without a small safe Rust mapping accept `serde_json::Value`; the generator does not invent replacement semantics. Optional function arguments use the standard `FunctionCall::arg()` builder.

## Browser example

```sh
cd examples/browser
npm install
npm run dev
```

Useful checks:

```sh
npm test        # official MessageProcessor validates Rust-authored messages
npm run check   # builds WASM and checks TypeScript
npm run build   # production WASM + Vite build
```

The app sends Rust-generated messages directly to the official `MessageProcessor`. Outbound standard A2UI messages go to one application callback. No network transport is assumed.

## Validation

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

## Scope

This is an authoring and code-generation foundation. It is not an A2UI Core SDK or Agent SDK. Use an official SDK for catalog negotiation, incoming message validation, reactive state, expression evaluation, inference compilation, and native rendering.
