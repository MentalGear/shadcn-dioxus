The Field component lays out a label, its control, and optional description/error text together, so building an accessible form is mostly composition rather than manual markup. It builds on this repo's native form participation: every themed control already mirrors its state onto a real, hidden native input, so `Field` only supplies layout and label/description/error wiring.

## Component Structure

```rust
FieldSet {
    FieldLegend { "Profile" }
    FieldGroup {
        Field {
            FieldLabel { html_for: "username", "Username" }
            Input { id: "username" }
            FieldDescription { "This is your public display name." }
        }
        Field { invalid: true,
            FieldLabel { html_for: "email", "Email" }
            Input { id: "email", "aria-invalid": "true" }
            FieldError { "Enter a valid email address." }
        }
        FieldSeparator {}
        // Horizontal orientation: the control sits beside its label.
        Field { orientation: FieldOrientation::Horizontal,
            Checkbox { id: "marketing" }
            FieldContent {
                FieldLabel { html_for: "marketing", "Marketing emails" }
                FieldDescription { "Receive emails about new products." }
            }
        }
    }
}
```

## Styling the label

`FieldLabel` renders the themed `Label` (`<label class="dx-label dx-field-label">`) and retunes it through `Label`'s slot variables (see the Label docs): `--dx-label-line-height` always (shadcn's `FieldLabel` is `Label` plus `leading-snug`), `--dx-label-color` while the field is `invalid`, and `--dx-label-opacity` while it is `disabled`. `Field`'s stylesheet never declares `font-size`, `line-height`, or `color` on the label itself, so the label looks the same on every route regardless of which stylesheet loads first.

## Required marker

`FieldLabel { required: true }` draws a `*` in the destructive ink after the label text (shadcn's `FieldLabel` has no such option; its demos draw one by hand). The marker is `aria-hidden`, so put `required` on the control too: that is what screen readers announce and what the form enforces. See the `input` page's `required` variant.

## Invalid state

`invalid` on a `Field` turns the label, description and `FieldError` text destructive. The control itself is invalid when it carries `aria-invalid="true"` (the attribute assistive technology reads), exactly as in shadcn, so set both: `Input`, `Textarea`, `NativeSelect`, `InputGroup`, `InputOtp`, `Checkbox`, `RadioItem`, `Switch`, `Toggle` and `Button` then draw the one shared invalid recipe, a destructive edge plus a destructive ring (`--dx-ring-invalid*` in the theme), and keep it while focused.

## Disabled state

`disabled` on a `Field` (shadcn's `data-disabled` on `Field`) dims every `FieldLabel` inside it to `--dx-opacity-disabled`; the description keeps its ink. It is styling only, so give the control its own `disabled` as well: that is what takes it out of the tab order and the submitted form.

```rust
Field { disabled: true,
    FieldLabel { html_for: "workspace", "Workspace" }
    Input { id: "workspace", value: "acme-inc", disabled: true }
}
```

## Choice card

A `FieldLabel` that wraps a `Field` renders as a "choice card": bordered and padded (the chrome is drawn on the wrapped `Field`), and the whole card is the click target for the control inside. When any descendant control is checked (`data-state="checked"`: `RadioItem`, `Switch` and `Checkbox` all emit it) the card takes a primary-tinted background and border (Nova: 5% fill and 30% edge, 10% and 20% in dark), a muted hover fill, and it shows a focus ring while a control inside has keyboard focus.

```rust
FieldLabel { html_for: "plan-pro",
    Field { orientation: FieldOrientation::Horizontal,
        FieldContent {
            "Pro"
            FieldDescription { "For growing businesses." }
        }
        RadioItem { id: "plan-pro", value: "pro".to_string(), }
    }
}
```
