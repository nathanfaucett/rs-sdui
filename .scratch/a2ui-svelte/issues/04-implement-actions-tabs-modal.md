# 04: Implement actions, tabs, and modal

**What to build:** Add `Button`, `Tabs`, and `Modal` with UI SDK behavior, complete schema definitions, resolved children, local UI state, validation, and accessible interaction.

**Blocked by:** 02: Build the Svelte surface renderer

**Status:** ready-for-agent

- [ ] `Button` implements all variants, arbitrary catalog children, checks, disabled state, and direct action invocation.
- [ ] Button action promises are handled without losing user activation or creating duplicate dispatches.
- [ ] `Tabs` supports dynamic titles and children, keyboard navigation, selection clamping, and local selected-index state.
- [ ] `Modal` uses its trigger and content children, supports focus management and closing, and preserves the correct node state.
- [ ] A changed `node.id` remounts local state; ordinary property updates preserve it.
- [ ] Shared accessibility attributes override inferred names and descriptions correctly.
- [ ] Component tests cover mouse and keyboard operation, checks, dynamic children, shrinking tabs, and cleanup.
- [ ] Emitted definitions for these components deep-equal the official Basic JSON sections.
