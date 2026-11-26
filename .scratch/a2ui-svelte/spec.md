# Spec: A2UI Svelte Component Library

Date: 2026-11-26  
Status: proposed

## 1. Summary

Build a Svelte 5 renderer and a complete A2UI v1.0 Basic Catalog. Use `@ui-sdk/svelte` for accessible behavior and Tailwind CSS with shadcn-style theme tokens for presentation.

Each catalog component has one code-owned Zod 4 definition. Its schema defines the wire properties. TypeScript maps the same schema to the Svelte view properties. A build-time emitter generates `catalog.json` for agents, Rust `sdui-codegen`, validation, and npm publication.

The browser loads that JSON through `Catalog.fromJson()`. The current `@a2ui/web_core` then derives its own Zod 3 runtime schemas. Zod 4 is used only for authoring and generation. Application code never authors Zod 3 schemas and never passes Zod 4 schemas to the SDK binder.

The official processor, binder, resolver, and Basic functions remain authoritative. This project adds no second protocol runtime.

“UI SDK” means `@ui-sdk/svelte` and `@ui-sdk/core` from the supplied reference. It does not mean `bits-ui` or `shadcn-svelte`. The visual API follows shadcn conventions: composed parts, Tailwind utilities, CSS theme variables, and no style properties added to A2UI messages.

## 2. Goals

- Implement all 18 Basic components and all 14 declared Basic functions.
- Support literals, data bindings, function results, actions, checks, static children, and list templates.
- Use the latest stable Zod 4 for all project-owned authoring schemas.
- Infer Svelte view properties from the same schemas used to emit the catalog.
- Reject accidental `any` in public component property types. Use `unknown` only where the protocol has no narrower type.
- Let applications create independent catalogs with the same definition pattern.
- Support mixed catalogs and duplicate component names in one surface without a global name registry.
- Keep `@sdui/web` renderer-independent.
- Keep Zod 4 and authoring code out of browser runtime entry points.
- Publish deterministic catalog JSON, Svelte sources, declarations, and compiled Tailwind utilities.
- Prove the renderer in the existing browser example with production builds.

## 3. Non-goals

- No Svelte implementation of A2UI expression parsing, data binding, validation checks, action dispatch, or Basic functions.
- No custom A2UI messages, fields, themes, compatibility transforms, or transport behavior.
- No Zod 3 project-owned authoring schemas and no Valibot conversion layer.
- No direct Zod 4 input to `GenericBinder`, `ComponentApi`, `spec`, or `Catalog` constructors.
- No duplicate handwritten Svelte prop interfaces for catalog components.
- No runtime download of catalog views or executable code.
- No general-purpose standalone shadcn component API. These views render A2UI nodes. Child properties contain resolved A2UI nodes, not Svelte snippets.
- No arbitrary `class`, `style`, HTML, event handler, or URL properties from an agent unless a catalog explicitly declares them.

## 4. Main decisions

### 4.1 Use Zod 4 for authoring and generation

Use Zod `4.6.7`, the latest stable release verified for this spec. Use `z.infer`, `z.input`, `z.output`, and `z.toJSONSchema()` with Draft 2020-12 output.

Pin the development version for repeatable JSON output. The authoring interface can declare a compatible Zod 4 peer range. Upgrade deliberately and rerun all catalog parity tests.

The current latest `@a2ui/web_core` is 0.16.1 and still depends on Zod 3. Its binder inspects Zod 3 classes and internals. Zod 4 moves internals from `_def` to `_zod.def` and removes `ZodEffects`. Direct Zod 4 integration is therefore not compatible.

The JSON Schema document is the supported seam between our Zod 4 authoring module and the current SDK runtime.

### 4.2 Load generated JSON at runtime

For each generated catalog:

1. Call `Catalog.fromJson(document)` to get SDK-compatible runtime schemas.
2. Pair each loaded component with its code-owned Svelte view.
3. Preserve each loaded JSON definition with official `copyLoadedDefinition(source, target)`.
4. Install official or application-owned executable function implementations.
5. Use `copyWith` to produce the final runtime `Catalog`.

A view-extended component has the shape of `ComponentApi` plus `view`. `A2uiNode` selects `node.impl.view`. Copying the loaded definition ensures that a later `toJson()` still emits the original document rather than regenerating a different schema.

This approach supports code-owned and JSON-loaded catalogs through the same runtime adapter. No sidecar name registry is needed.

### 4.3 Use official SDK runtime modules

Use:

- `Catalog.fromJson`, `Catalog.copyWith`, and `copyLoadedDefinition` for runtime catalog assembly.
- `MessageProcessor` for message and surface state.
- `NodeResolver` for resolved component trees.
- `ResolvedBinding`, `WritableBinding`, and `isWritable` for values and writes.
- `BASIC_FUNCTIONS` or `createBasicCatalogFunctions` for Basic functions.
- `createFunctionImplementation` and `AgentFunction` for application-owned functions.
- `sanitizeMediaUrl` for media URLs.

Use `Catalog.toJson()` only for runtime round-trip checks. It is not the Zod 4 authoring generator. Use `validationSchema` only for payload validation, never as the catalog document.

### 4.4 Add a small A2UI JSON Schema emitter

Zod 4 emits standard JSON Schema, but A2UI catalogs also require:

- Exact external `common_types.json` references.
- Component discriminators and ordered component maps.
- `Checkable` and other `allOf` mixins.
- Local `$defs`, `anyComponent`, and `anyFunction`.
- Function `returnType`, caller, and activation metadata.
- Conditional property schemas such as date/time bounds.
- Required ordering, closure keywords, and deterministic output.

Add an authoring-only annotation module backed by a `WeakMap`. It stores these A2UI emission rules beside Zod 4 schemas without changing their inferred types. The emitter uses `z.toJSONSchema()` and applies those rules through its override and final document assembly.

Provide narrow Zod 4 helpers for `DataBinding`, `FunctionCall`, dynamic primitives, `DynamicValue`, `Child`, `ChildList`, `Action`, `CheckRule`, and `Checkable`. Their types must:

- Require `path` or `@path` for a binding.
- Require `call` or `@call` for a function call.
- Preserve optional `catalogId` and `args`.
- Avoid accidental `any`, including `unknown[]` for unconstrained arrays.
- Keep binding and function-call branches distinguishable in resolved property types.

The emitter must reject unsupported or unrepresentable constructs. It must not silently replace them with `{}`.

### 4.5 Keep component schemas canonical

Do not generate TypeScript schemas from `catalog.json` during the build. That reverses the required source of truth.

The Basic package contains Zod 4 schemas for all 18 components. The official catalog is a conformance fixture, not the source of project-owned component properties.

The build may use official `basicCatalog.toJson()` as a template for Basic document metadata and official function declarations. It replaces all component definitions and `anyComponent` references from the Zod 4 definitions. The official runtime `basicCatalog` is never imported by a browser entry point.

## 5. Package architecture

```mermaid
flowchart TD
    Author[Zod 4 component definitions]
    Emit[Build-time catalog emitter]
    JSON[catalog.json]
    Load[Catalog.fromJson]
    Views[View-extended runtime catalog]
    App[Application]
    Web[@sdui/web]
    Processor[@a2ui/web_core MessageProcessor]
    Surface[A2uiSurface]
    Resolver[NodeResolver]
    Node[A2uiNode]
    UISDK[@ui-sdk/svelte]
    Rust[sdui-codegen]

    Author --> Emit
    Emit --> JSON
    JSON --> Load
    Load --> Views
    App --> Web
    Web --> Processor
    Views --> Processor
    Processor --> Surface
    Surface --> Resolver
    Resolver --> Node
    Node --> UISDK
    JSON --> Rust
```

### 5.1 `@sdui/a2ui-svelte`

Location: `packages/a2ui-svelte`

This package owns:

- `A2uiSurface` and `A2uiNode`.
- Zod 4 `defineComponent`, `defineFunction`, and `defineCatalog` authoring helpers.
- Typed wire helpers and resolved view-property types.
- Runtime catalog assembly from generated JSON and views.
- Conversion from resolved SDK properties to view properties.
- Signal subscriptions and resolver cleanup.
- The Node-only deterministic catalog JSON writer.

Public entry points:

| Entry               | Responsibility                                     | Runtime dependencies        |
| ------------------- | -------------------------------------------------- | --------------------------- |
| `.`                 | `A2uiSurface`, `A2uiNode`, and renderer types      | Svelte and `@a2ui/web_core` |
| `./authoring`       | Zod 4 definitions, annotations, and property types | Zod 4; no DOM               |
| `./runtime-catalog` | Generated JSON to executable Svelte catalog        | `@a2ui/web_core`; no Zod 4  |
| `./catalog-json`    | Node-only emit and write helpers                   | Zod 4 and Node filesystem   |

The root and runtime entries do not import the authoring or Node entries.

### 5.2 `@sdui/a2ui-svelte-basic`

Location: `packages/a2ui-svelte-basic`

This package owns:

- The 18 Zod 4 component definitions.
- Their shadcn-styled Svelte views.
- A generated Basic document and view-extended runtime `Catalog`.
- Official Basic function implementations.
- Catalog metadata and official instruction text.
- Generated `catalog.json` and compiled `styles.css`.

The root entry exports `basicCatalog`, `basicComponents`, and `CATALOG_ID` without importing Zod 4. Authoring definitions use a separate subpath. Component subpath exports allow an application to derive a smaller or extended catalog without importing every view.

## 6. Component definition interface

An authoring definition pairs a Zod 4 schema with a Svelte view:

```ts
const Text = defineComponent({
  name: "Text",
  schema: textSchema,
  view: TextView,
});
```

`defineComponent` preserves the exact schema type. It must not widen the schema to `z.ZodType`.

The schema contains catalog properties but omits envelope fields such as `id` and `component`. The emitter adds the component discriminator and standard envelope composition.

Two property types come from each schema:

| Type                     | Source                                  | Purpose                         |
| ------------------------ | --------------------------------------- | ------------------------------- |
| `WireProps<typeof Text>` | `z.input` of the Zod 4 schema           | Validate or construct A2UI JSON |
| `ViewProps<typeof Text>` | `z.output` plus the Svelte node mapping | Type the Svelte view            |

The view type is not `ResolveA2uiProps` unchanged. That SDK type describes `GenericBinder` output. `NodeResolver` also creates `ResolvedBinding` objects and live `ComponentNode` children.

The Svelte mapping performs these type and runtime conversions:

| SDK value                           | Svelte view value                            |
| ----------------------------------- | -------------------------------------------- |
| `ResolvedBinding<T>`                | `T`                                          |
| `WritableBinding<T>`                | `T` plus optional `set<Property>(value: T)`  |
| Read-only binding                   | Value only; no no-op setter                  |
| `NodeAction`                        | The same callable action                     |
| Single child reference              | `ComponentNode`                              |
| Child list                          | `ComponentNode[]`                            |
| Nested objects and arrays           | Recursive conversion, including tab children |
| Validation and accessibility values | Pass through with narrow types               |

The view receives no arbitrary transport or processor object. Internal node identity and metadata stay with `A2uiNode` unless a resolved property explicitly exposes them.

Type tests must cover representative property values, setters, actions, nested children, required fields, optional fields, invalid literals, and recursive absence of `any`.

## 7. Renderer interface and lifecycle

### 7.1 `A2uiSurface`

Input: `surface: SurfaceModel | undefined`.

The application owns `MessageProcessor`, catalog registration, transport, connection recovery, and surface selection. `A2uiSurface` creates one `NodeResolver` for its current surface. It does not receive a second catalog because the resolver must use the same catalog instance as the surface.

When the surface changes or becomes `undefined`, the component releases subscriptions and disposes the old resolver. `undefined` renders nothing.

### 7.2 `A2uiNode`

Input: `node: ComponentNode`.

- Select the view through `node.impl.view`. Do not dispatch through a global map keyed only by component name.
- Key each rendered instance by `node.id`.
- Preserve child nodes recursively through `A2uiNode`.
- Handle `pending`, `unknown-type`, `cyclic`, and disposed states without calling a catalog view.
- Convert resolved properties through one pure adapter. Do not create another `GenericBinder`.
- Stop adapter subscriptions on replacement or unmount. The resolver owns node and binder disposal.

Use Svelte `createSubscriber` with SDK `effect`, `getValue`, and `peekValue`. Do not call `setSignalImplementation`; it changes process-global SDK state and can break another renderer.

Local UI state belongs to the node instance. A changed `node.id` remounts stateful views. Tabs clamp selection when the tab list shrinks. Modal and filter state must not move to a different protocol instance.

Input views remain controlled by resolved properties. Setters update renderer-local data and do not emit an action. Actions run directly from user events and their promises are handled. This preserves user activation for `openUrl`.

## 8. Basic Catalog implementation matrix

| Component       | Implementation                                                                                               |
| --------------- | ------------------------------------------------------------------------------------------------------------ |
| `Text`          | Safe simple Markdown; body and caption variants; inherited color                                             |
| `Image`         | Native image; fit and all size variants; accessible description                                              |
| `Icon`          | Static tree-shakable Lucide mapping for every declared name; custom `svgPath` uses only an SVG `d` attribute |
| `Video`         | Native video with controls and poster                                                                        |
| `AudioPlayer`   | Native audio with controls and description                                                                   |
| `Row`           | Static Tailwind flex maps for direction, justify, align, and weight                                          |
| `Column`        | Static Tailwind flex maps for direction, justify, align, and weight                                          |
| `List`          | Scrollable vertical or horizontal layout with template children                                              |
| `Card`          | Bordered shadcn-style container with one child                                                               |
| `Tabs`          | UI SDK Tabs with keyboard behavior and local selected index                                                  |
| `Modal`         | UI SDK Dialog with trigger, content, focus control, and close behavior                                       |
| `Divider`       | UI SDK Separator with horizontal and vertical axes                                                           |
| `Button`        | UI SDK Button with all variants, nested content, checks, and direct action invocation                        |
| `TextField`     | UI SDK Field with Input, Textarea, Number Input, or Password Input by variant                                |
| `CheckBox`      | UI SDK Checkbox with a boolean writable binding                                                              |
| `ChoicePicker`  | Radio, Checkbox, and Toggle Group patterns; multiple and exclusive selection; optional label filtering       |
| `Slider`        | UI SDK Slider with min, max, steps, and numeric writable binding                                             |
| `DateTimeInput` | UI SDK Date and Time Input parts; all enable flags; ISO 8601 conversion and bounds                           |

All components also implement the shared contract:

- `@accessibility` labels, descriptions, live state, and hidden state override inferred semantics.
- Checkable components display associated errors and set `aria-invalid` and error relationships.
- Controls handle `false`, `0`, empty strings, and empty arrays with `??`, not truthiness fallbacks.
- Missing child definitions show resolver placeholders and recover when definitions arrive.
- Weight affects flex layout without adding spacing to invisible containers.
- Text and icons inherit foreground color. Nested cards remain distinct through borders.

Use only static Tailwind class maps so the compiler sees every variant. Publish a compiled `styles.css` containing the utilities used by the package. Consumers must not need to scan `node_modules`. Consumers provide shadcn theme variables and declare the `ui-sdk` cascade layer before Tailwind as required by UI SDK.

## 9. Functions and security

Use the official Basic function implementations for `required`, `regex`, `length`, `numeric`, `email`, `formatString`, `formatNumber`, `formatCurrency`, `formatDate`, `pluralize`, `openUrl`, `and`, `or`, and `not`. Keep `@index` as a system function, not an extra published catalog function.

Security requirements:

- Sanitize `Image.url`, `Video.url`, `Video.posterUrl`, and `AudioPlayer.url` with the official helper.
- Parse Markdown with a maintained parser and sanitize generated HTML. Allow only the simple markup required by the catalog. Never emit raw agent HTML.
- Prevent stale asynchronous Markdown output from replacing newer text.
- Render custom icon data as a path attribute, never as raw SVG markup.
- Keep official `openUrl` scheme, user-activation, and tab-nabbing protections.
- Invoke actions only from user events. Input setters alone send no network message.
- Do not expose application DOM nodes, transport objects, or arbitrary callbacks to agent data.

## 10. Catalog generation and publication

The Zod 4 definitions are canonical. `catalog.json` is generated and must not be edited.

Commands:

- `generate:catalog`: emit `packages/a2ui-svelte-basic/catalog.json` from the Zod 4 definitions and Basic document template.
- `check:catalog`: regenerate in memory and fail on any difference without writing.
- `build`: generate the catalog, compile the package and styles, and run the catalog check.
- `prepack`: run the build and verify the package contents.

Check `catalog.json` into Git. This lets Cargo and `sdui-codegen` consume it on a clean checkout without running Node during a Rust build. The npm package includes `catalog.json`, `dist`, and `styles.css`, and exports `./catalog.json`.

After full parity, `apps/browser-example/rust/build.rs` uses this generated file instead of its application-owned copy. Update `BASIC-NOTICE` with the same upstream commit and license details. Do not make `cargo test` execute pnpm.

Catalog tests assert:

1. Generated JSON equals the official Basic fixture, including component order, function order, descriptions, references, mixins, and definitions.
2. Generated JSON equals official `basicCatalog.toJson()` in a development-only comparison.
3. The view-extended runtime catalog preserves the document through `toJson()`.
4. A `Catalog.fromJson(...).toJson()` round trip preserves the document.
5. `sdui-codegen` accepts the generated file.
6. Repeated generation is byte-identical.
7. All official functions remain executable and all 18 component schemas are present.
8. Browser entries contain neither Zod 4 nor authoring-only definitions.

The generic `./catalog-json` helper provides the same deterministic generation pattern to application-owned catalogs.

## 11. Custom catalog pattern

An application authors an independent catalog with Zod 4:

```ts
const MetricCard = defineComponent({
  name: "MetricCard",
  schema: metricCardSchema,
  view: MetricCardView,
});

const operationsDefinition = defineCatalog({
  catalogId: "https://example.com/catalogs/operations.json",
  title: "Operations Catalog",
  description: "Application-owned operations components.",
  instructions: operationsInstructions,
  components: [MetricCard],
  functions: operationsFunctionDefinitions,
});
```

The build emits `operations.json`. The runtime adapter loads it, attaches views and executable functions, and returns the official `Catalog` used by the processor.

The application registers Basic and Operations catalogs before connecting transport. A component-level `@catalogId` selects Operations where required. `A2uiNode` uses the resolved implementation object, so another `MetricCard` name in Basic or a third catalog cannot collide.

Custom function arguments and return types use Zod 4 during authoring. Runtime execution adapts the loaded SDK function schema through official `createFunctionImplementation`. Schema-only agent functions use the official `AgentFunction` pattern. No custom function protocol is added.

## 12. Validation strategy

### 12.1 Type and schema tests

- Use compile-time tests for every public view and wire property map.
- Assert exact primitive, enum, action, setter, child, and nested property types.
- Assert `ComponentProps` for each exported Svelte view matches its definition.
- Add negative tests for wrong values and invalid bindings.
- Recursively reject `any`, including `DynamicValue` arrays and nested function arguments.
- Test external references, mixins, local definitions, conditional schemas, and function metadata in the emitter.
- Test that unsupported Zod constructs fail with a useful path and message.
- Compare loaded SDK `scrapeComponentBehavior` results against official Basic behavior.

### 12.2 Renderer and component tests

- Test Zod 4 generation, runtime loading, view attachment, and document round trips.
- Test processor-to-surface-to-node integration through public SDK interfaces.
- Test inbound model changes, local setter changes, actions, checks, and function results.
- Test every component variant and every input mode.
- Test static children, list templates, relative paths, missing children, and cycles.
- Test surface replacement, reconnect, unmount, and node replacement for resource cleanup.
- Test keyboard, focus, accessible names, error relationships, and automated accessibility checks.
- Test that one catalog cannot replace a same-named view from another catalog.

### 12.3 Production validation

Use the existing browser example for integration. Add a Svelte mount adapter and a Basic gallery fixture without creating a second Rust application.

Run the required workspace commands sequentially:

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
pnpm run check
pnpm run build
```

Also:

- Run package type, schema, renderer, and component tests.
- Run `pnpm pack` for both packages and inspect their files.
- Verify production output contains the application worker and browser WASM assets.
- Verify browser entries exclude Zod 4, authoring code, and official Lit Basic component implementations.
- Validate two surfaces, one synchronized action, all gallery interactions, keyboard flows, and reconnection through `pnpm run preview` and Playwriter.
- Stop preview processes after validation.

## 13. Implementation sequence

1. [01: Create the Zod 4 catalog foundation](issues/01-create-typed-catalog-foundation.md)
2. [02: Build the Svelte surface renderer](issues/02-build-svelte-surface-renderer.md)
3. [03: Implement display and layout components](issues/03-implement-display-layout-components.md)
4. [04: Implement actions, tabs, and modal](issues/04-implement-actions-tabs-modal.md)
5. [05: Implement input and selection components](issues/05-implement-input-selection-components.md)
6. [06: Publish Basic and prove a custom catalog](issues/06-publish-basic-prove-custom-catalog.md)
7. [07: Integrate the browser example and validate production](issues/07-integrate-browser-validate-production.md)

Each issue is complete only when its tests pass without new warnings. The full feature is complete only when all acceptance criteria in this spec pass.

## 14. References

- Zod 4 JSON Schema: https://zod.dev/json-schema
- Zod 4 migration and internals: https://zod.dev/v4/changelog
- UI SDK Svelte: https://ui-sdk.dev/docs/svelte
- UI SDK composition and state: https://ui-sdk.dev/docs/svelte/parts-and-composition and https://ui-sdk.dev/docs/svelte/state-and-events
- A2UI v1.0 protocol: https://a2ui.org/specification/v1.0-a2ui/
- Basic implementation guide: https://a2ui.org/specification/v1.0-basic-catalog-implementation-guide/
- Existing Basic fixture: `apps/browser-example/rust/catalog.json`
- Existing processor mount: `apps/browser-example/src/mount-a2ui.ts`
