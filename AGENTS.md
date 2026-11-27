# SDUI agent guide

## Documentation

Read the guide for each area you change:

- Application state, views, functions, and emission: `crates/sdui/README.md`.
- Browser sessions, recovery, and local processes: `apps/browser-example/README.md`.
- Browser transport and worker APIs: `packages/web/README.md`.
- A2A identity, stores, cancellation, and recovery: `crates/a2ui-a2a/README.md`.

## Local validation

Run commands from the repository root. Run them sequentially: `wasm-pack` also invokes Cargo, so overlapping commands contend for Cargo locks and build output.

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
pnpm run check
pnpm run build
```

Run the subset that covers the changed code. Run all four for shared or cross-language changes. Do not report success with new warnings.

After Vite, worker, transport, or bundle changes, also validate the production build through `pnpm run preview` and Playwriter. The development server is not sufficient.

- Verify that `dist/assets/` contains an application-worker JavaScript asset and a browser WASM asset after worker changes.
- Verify that the production page displays its surfaces and synchronizes an action after shared-runtime changes.
- Use role and label selectors for inputs. Input values are not page text.
- Stop the preview processes after validation.

After `cargo update`, inspect Clippy output carefully for duplicate dependency versions. A lockfile can select a coherent older branch without changing public dependency ranges. Follow the dependency rules in `CODING_STANDARDS.md`.

## Pre-release

Before release, replace old APIs directly. Do not retain compatibility shims, legacy modules, or parallel API versions.

## Final review

Read `CODING_STANDARDS.md` before reviewing the diff. Report the validation actually run and any remaining warnings or risks.

## Local workflows

- Issues and specs: local Markdown files under `.scratch/`; see `docs/agents/issue-tracker.md`.
- Triage labels: use the five canonical labels without renames; see `docs/agents/triage-labels.md`.
- Domain documentation: use the single-context root `GLOSSARY.md`; see `docs/agents/domain.md`.
