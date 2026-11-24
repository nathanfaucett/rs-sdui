# SDUI agent guide

## Read first

- `README.md`: workspace architecture, application model, and public API.
- `CODING_STANDARDS.md`: required review rules for every change.
- `packages/browser-example/README.md`: browser sessions, transports, reconnection, and local processes.

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

After `cargo update`, inspect Clippy output carefully for duplicate dependency versions. Follow the dependency rules in `CODING_STANDARDS.md`.

## Final review

Read `CODING_STANDARDS.md` before reviewing the diff. Report the validation actually run and any remaining warnings or risks.
