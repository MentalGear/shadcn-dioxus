The `InputOtp` component is a one-time-passcode entry control: a row of single-character boxes backed by one real, focusable `<input>`. Typing, Backspace/Delete, the arrow keys, and pasting a full code all work through that one real input; `InputOtpSlot` boxes are a purely visual, `aria-hidden` overlay that mirrors its value and caret position.

## Component Structure

```rust
InputOtp {
    max_length: 6,
    aria_label: "One-time passcode",
    InputOtpGroup {
        InputOtpSlot { index: 0 }
        InputOtpSlot { index: 1 }
        InputOtpSlot { index: 2 }
    }
    InputOtpSeparator {}
    InputOtpGroup {
        InputOtpSlot { index: 3 }
        InputOtpSlot { index: 4 }
        InputOtpSlot { index: 5 }
    }
}
```

## Accepted characters

`pattern: OtpPattern` is the one prop that decides what the code may contain, and the `<input>`'s `inputmode`, its `pattern` attribute and the filter applied to every typed or pasted value are all derived from it, so the keyboard and the accepted characters cannot disagree.

| `OtpPattern` | Accepts | Keyboard | shadcn |
| --- | --- | --- | --- |
| `Digits` (default) | `0-9` | numeric | `REGEXP_ONLY_DIGITS` |
| `Alphanumeric` | `A-Z a-z 0-9` | text | `REGEXP_ONLY_DIGITS_AND_CHARS` |
| `Letters` | `A-Z a-z` | text | `REGEXP_ONLY_CHARS` |
| `Custom { class, inputmode }` | the character-class body `class`, e.g. `"A-Z0-9"` | `inputmode` | custom regex |

```rust
InputOtp { max_length: 6, pattern: OtpPattern::Alphanumeric, aria_label: "Code", /* slots */ }
InputOtp { max_length: 4, pattern: OtpPattern::Custom { class: "0-9a-f".into(), inputmode: OtpInputMode::Text }, /* slots */ }
```

Rejected characters are removed as they arrive (a pasted `A1-B2` into an alphanumeric code becomes `A1B2`). The browser applies `maxlength` to a paste before the filter runs, so a pasted code with separators may be truncated first; paste the bare code. Raw `inputmode` or `pattern` attributes passed to `InputOtp` are overridden by the typed prop: it is the only way to set them. Letters keep their case.

## Motion

The caret blinks (`opacity`, 1 s, forever) on the one focused slot only, i.e. while the user is typing into the field; it is visible by definition, so it is not gated (it is on `check-motion-gating.sh`'s allowlist). Override `animation` on `.dx-input-otp-slot[data-active="true"]::after` to change or remove it; reduced motion swaps in `--dx-motion-duration-reduced`. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
