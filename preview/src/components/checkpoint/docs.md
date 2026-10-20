The Checkpoint component marks a point in the conversation you can return to: a labeled separator after a turn with a Restore button. Restore keeps that turn and drops every message after it; the component only reports the click, because what restoring means (truncate a chat history, rewind a workspace) is up to the app.

## Component Structure

```rust
Checkpoint {
    label: "Before refactor",
    on_restore: move |_| messages.write().truncate(index + 1),
}
```

- `action` changes the visible button text (default "Restore"). `disabled` disables the button, for the point the thread is already at or while a restore runs.
- It is built from `Marker` (the `Separator` variant) and `Button`, and extra attributes go to the `Marker`.

## Why a component and not only a recipe

shadcn/ui documents Checkpoint as a recipe: a separator `Marker` whose content holds a label and an outline `Button`. This is that recipe, with the one part that is easy to get wrong made impossible: `label` is required and the button's accessible name is built from it ("Restore checkpoint: Before refactor"), so a thread with many checkpoints never has a row of buttons that are all just "Restore". The visible word stays the first word of the name (label in name). If you need a different layout, build it from `Marker` directly.

## Accessibility

- The row is not `role="separator"`: that would hide the label from assistive tech. The rule lines are decorative.
- The Restore button is a real `<button>` named after its checkpoint.
