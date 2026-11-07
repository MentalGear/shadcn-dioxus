`initial_focus` as a function of how the popover was opened. The line inside the panel reads the open's modality from the page and says where focus went.

- **Keyboard** (Enter or Space on the button): focus lands on the second field. Closing returns focus to "Return here".
- **Mouse** (click): focus lands on the first field. Text fields always show their ring, so the landing is visible.
- **Touch and pen**: focus lands on the panel, so no software keyboard opens and nothing rings.
