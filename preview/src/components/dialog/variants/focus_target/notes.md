`initial_focus` and `final_focus` as functions of how the dialog was opened. The line inside the panel reads the open's modality from the page and says where focus went.

- **Keyboard** (Enter or Space on the button): focus lands on the second field and a ring shows, because rings follow the keyboard. Closing with Escape or Cancel returns focus to "Return here".
- **Mouse** (click): focus lands on Cancel, but no ring is painted: focus rings are keyboard-only by design, so the move is invisible. Press Tab and the ring appears on the focused button.
- **Touch** (tap): focus lands on the panel, so no software keyboard opens and nothing rings.
- **Pen**: the author selector matches nothing, so the built-in policy decides; like touch, focus lands on the panel.
