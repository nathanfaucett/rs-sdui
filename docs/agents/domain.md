# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Layout

Use a single-context layout: one `GLOSSARY.md` at the repo root for all Rust crates and TypeScript packages.

## Before exploring, read these

- **`GLOSSARY.md`** at the repo root.

If this file does not exist, **proceed silently**. Do not flag its absence or suggest creating it upfront. The `/domain-modeling` skill creates it lazily when terms or decisions actually get resolved.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `GLOSSARY.md`. Do not drift to synonyms the glossary explicitly avoids.

If the concept you need is not in the glossary yet, either reconsider language the project does not use or note the gap for `/domain-modeling`.

## Flag conflicts

If your output contradicts an existing definition, surface the conflict explicitly. Do not silently override the glossary.
