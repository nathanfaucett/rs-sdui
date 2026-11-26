# 03: Implement display and layout components

**What to build:** Add the Basic display, media, and layout definitions and views: `Text`, `Image`, `Icon`, `Video`, `AudioPlayer`, `Row`, `Column`, `List`, `Card`, and `Divider`.

**Blocked by:** 02: Build the Svelte surface renderer

**Status:** ready-for-agent

- [ ] Every component has one code-owned schema and a view typed from that schema.
- [ ] `Text` supports safe simple Markdown, both variants, inherited colors, and stale-render protection.
- [ ] `Image`, `Video`, and `AudioPlayer` implement every property and sanitize all media URLs.
- [ ] `Icon` covers every declared icon name and custom path data without raw SVG injection.
- [ ] `Row`, `Column`, and `List` implement all alignment, direction, weight, static-child, and template-child behavior.
- [ ] `Card` renders one child and `Divider` renders both axes with accessible semantics.
- [ ] Static Tailwind maps cover every variant and compiled package CSS contains all used utilities.
- [ ] Shared accessibility, hidden state, missing children, and progressive updates work for every component.
- [ ] Component and accessibility tests cover all properties, dynamic updates, media security, and keyboard-relevant behavior.
- [ ] Emitted definitions for these components deep-equal the official Basic JSON sections.
