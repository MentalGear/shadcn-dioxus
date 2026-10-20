A multi-step question flow for agent clarification prompts, onboarding and surveys (shadcn's `questionnaire`, 2026-08). One item shows at a time: single choice on native radios, multiple choice on native checkboxes, a freeform "other" answer, an explicit skip for optional items, previous/next/submit, required and external validation, controlled navigation, resume from saved answers, conditional items, optional letter or number answer shortcuts, and native `<form>` serialization.

## Component Structure

```rust
let items = vec![
    ItemDef::new("direction").required().choices(["delegation", "questions"]).freeform(),
    ItemDef::new("detail").choices(["focused", "complete"]),
];

Questionnaire { items, on_submit: move |event: FormEvent| save(event.values()),
    QuestionnaireProgress {}
    QuestionnaireItem { name: "direction",
        QuestionnaireTitle { "What should we prototype next?" }
        QuestionnaireDescription { "Choose a direction or write your own." }
        QuestionnaireChoices {
            QuestionnaireChoice { value: "delegation",
                span { "Delegation" }
                QuestionnaireChoiceDescription { "Show how work moves to a specialist." }
            }
            QuestionnaireChoice { value: "questions", "Question prompts" }
            QuestionnaireInput { aria_label: "Another answer", placeholder: "Type another answer…" }
        }
        QuestionnaireError {}
    }
    QuestionnaireItem { name: "detail", /* ... */ }
    QuestionnaireActions {
        QuestionnairePrevious {}
        QuestionnaireSkip {}
        QuestionnaireNext {}
        QuestionnaireSubmit {}
    }
}
```

**One list declares every item.** `items: Vec<ItemDef>` is the only place an item's behaviour lives: its name, `required()`, `multiple()`, `disabled(..)` (a conditional item), `invalid(..)` (an external error), its choices in render order (`choice`, `choices`, `disabled_choice`), its freeform answer (`freeform()`, `freeform_kind(InputKind::Email)`) and its saved answers (`default_choice`, `default_input`). `QuestionnaireItem` takes only a `name` and `QuestionnaireChoice` only a `value`, so the parts can never disagree with the list (shadcn warns about that at runtime; here it cannot happen). Render the choices in the list's order, because shortcuts follow it.

| `Questionnaire` prop | Default | |
| --- | --- | --- |
| `items` | required | The `ItemDef`s, in order. Recompute it from earlier answers for conditional items. |
| `default_item` | first item | The item to open on and to return to on reset (resume). |
| `item` / `on_item_change` | uncontrolled | Controlled active item: navigation calls `on_item_change` and waits for `item`. |
| `on_answer_change` | none | `AnswerChange { item, values, status }` after an answer changes. |
| `shortcuts` | none | `ShortcutMode::Letters` (`A`-`Z`) or `ShortcutMode::Numbers` (`1`-`9`). |
| `animate_height` | `true` | Glide the height between items with a CSS transition instead of snapping; off under `prefers-reduced-motion`. `false` keeps the snap. The Animate height demo flips it live. |
| `native_validation` | `false` | Validate filled answers with the browser (an invalid `email` input blocks Next). |
| `on_submit` | none | A submit that passed validation, with the `FormEvent` (`values()`, `get(name)`). |
| `on_reset` | none | After a `type="reset"` button restored the saved answers and item. |

The navigation buttons take `variant`, `size` (the `Button` props), `disabled` and children (their label). `use_questionnaire()` reads the progress, the active item and each item's status, for a custom progress indicator or navigation.

## Behaviour

- **Navigation.** Next validates the active item, then moves on and focuses the new item's `fieldset`. Previous needs no validation. Skip (shown only on an optional item) clears the item and moves on, or submits on the last item. Submit validates every item and returns to the first invalid one, focusing its answer.
- **Validation.** A required item needs an answer; an optional item needs an answer or an explicit skip. An item's error shows once validation was attempted and stays live: answering clears it, unchecking the last answer brings it back. `invalid(true)` is an external error (for example from a schema run in `on_submit`) that blocks the item until you clear it.
- **Freeform answer.** Typing selects the input; in a single-choice item it unchecks the radios. Choosing a radio keeps the text as a draft that is neither submitted nor validated.
- **Conditional items.** A `disabled` item leaves the progress count, the navigation and validation, and submits nothing.
- **Height glide.** `animate_height` pins the form's old and new height inline and lets a CSS `height` transition run between them, so items of different heights do not make the page jump. The Animate height demo has a switch to compare it with the snap; `data-animate-height` on the form shows the current state.
- **Resume and reset.** `default_item` and each item's defaults restore a saved draft. A reset button returns to them.

## Keyboard

| Key | |
| --- | --- |
| `ArrowDown` / `ArrowUp` | Move between an item's answers (wrapping). Radios also follow the browser's own radio group. An empty text input lets the arrows leave it. |
| `ArrowRight` / `ArrowLeft` | Next / previous item, from the item or a button (not from a radio or a text input). `ArrowRight` never skips an unanswered item. |
| `Enter` | On the selected answer: confirm (next, or submit on the last item). Never submits from an unselected answer. |
| `Command`/`Control` + `Enter` | Confirm from anywhere in the form. |
| `A`-`Z` or `1`-`9` | With `shortcuts`: select (or toggle) the answer with that key. |

Under RTL (a `DirectionProvider`, or the page's RTL toggle) `ArrowLeft` and `ArrowRight` swap, in `aria-keyshortcuts` too. Composition (IME) keystrokes and held-key repeats are ignored. The active item lists its keys in `aria-keyshortcuts`, each answer lists its own.

## Form submission

Every answer is a real form control, so the browser's `FormData` is the answer set: a checked radio or checkbox submits its value under the item's name, the selected freeform input submits its text, a skipped or disabled item submits nothing. `on_submit` receives that event. The form has `method="dialog"` and its submit is always `preventDefault`ed, so it never navigates (also before hydration).

## Accessibility

`QuestionnaireItem` renders a `fieldset` and `QuestionnaireTitle` its `legend`. The description and, while the item is invalid, the error (`role="alert"`) are joined to the `fieldset`'s `aria-describedby`; the item and its answers carry `aria-invalid`. Choices are native radios and checkboxes inside a `label`. Progress is a `progressbar` named "Questionnaire progress" with "Question 2 of 3" as its value text. Inactive items and buttons that do not apply are `hidden` and `inert`. A successful navigation focuses the new item; a failed validation focuses its first answer (or the selected freeform input).

Give `QuestionnaireInput` an accessible name with `aria_label`, `aria_labelledby` or a visible label: a placeholder is not a label.
