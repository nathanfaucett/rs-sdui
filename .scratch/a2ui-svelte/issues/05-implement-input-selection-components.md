# 05: Implement input and selection components

**What to build:** Add `TextField`, `CheckBox`, `ChoicePicker`, `Slider`, and `DateTimeInput` with typed two-way renderer-local bindings, validation, and UI SDK controls.

**Blocked by:** 02: Build the Svelte surface renderer

**Status:** ready-for-agent

- [ ] `TextField` implements short, long, number, and obscured variants with the correct UI SDK parts.
- [ ] `CheckBox` reads and writes booleans without truthiness bugs.
- [ ] `ChoicePicker` implements exclusive and multiple selection, checkbox and chip displays, and case-insensitive filtering.
- [ ] Exclusive choices use zero- or one-item arrays on the wire; multiple choices preserve string arrays.
- [ ] `Slider` implements min, max, steps, decimal values, and numeric writes.
- [ ] `DateTimeInput` implements every date/time enable combination, bounds, and ISO 8601 conversion.
- [ ] Controls remain controlled by resolved props and read-only literals do not receive fake setters.
- [ ] Input setters update local data without sending actions; related bound views update immediately.
- [ ] Validation errors, `aria-invalid`, labels, descriptions, focus, and keyboard behavior pass tests.
- [ ] Emitted definitions for these components deep-equal the official Basic JSON sections.
