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

`FieldLabel` renders the themed `Label` (`<label class="dx-label dx-field-label">`) and retunes it through `Label`'s slot variables (see the Label docs): `--dx-label-line-height` always (shadcn's `FieldLabel` is `Label` plus `leading-snug`), and `--dx-label-color` while the field is `invalid`. `Field`'s stylesheet never declares `font-size`, `line-height`, or `color` on the label itself, so the label looks the same on every route regardless of which stylesheet loads first.

## Invalid state

`invalid` on a `Field` turns the label, description and `FieldError` text destructive. The control itself is invalid when it carries `aria-invalid="true"` (the attribute assistive technology reads), exactly as in shadcn, so set both: `Input`, `Textarea`, `NativeSelect`, `InputGroup`, `InputOtp`, `Checkbox`, `RadioItem`, `Switch`, `Toggle` and `Button` then draw the one shared invalid recipe, a destructive edge plus a destructive ring (`--dx-ring-invalid*` in the theme), and keep it while focused.

## Choice card

A `FieldLabel` that wraps a `Field` renders as a "choice card": bordered and padded (the chrome is drawn on the wrapped `Field`), and the whole card is the click target for the control inside. When any descendant control is checked (`data-state="checked"`: `RadioItem`, `Switch` and `Checkbox` all emit it) the card takes a primary-tinted background and primary border, and it shows a focus ring while a control inside has keyboard focus.

```rust
FieldLabel { html_for: "plan-pro",
    Field { orientation: FieldOrientation::Horizontal,
        FieldContent {
            "Pro"
            FieldDescription { "For growing businesses." }
        }
        RadioItem { id: "plan-pro", value: "pro".to_string(), index: 0usize }
    }
}
```
