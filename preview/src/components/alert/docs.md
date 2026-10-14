The Alert component displays a short, callout-style message -- an inline status, warning, or tip. It renders `role="alert"` so assistive technology announces it as a live region.

## Component Structure

```rust
Alert {
    // Alert::class() controls the visual style. Defaults to `AlertVariant::Default`.
    variant: AlertVariant::Destructive,

    // An optional leading icon; the layout adapts automatically when one is present.
    CircleAlert {}
    AlertTitle { "Unable to process your payment." }
    AlertDescription { "Please verify your billing information and try again." }
}
```

## Action

`AlertAction` pins a control to the alert's top-end corner (`inset-inline-end`, so it mirrors in RTL), as in shadcn. The alert keeps 72px free on the end side while it has one, so the title and description wrap before they reach it. It is a plain `div`: put a small `Button` (`ButtonSize::Xs`), a link or a dismiss button in it.

```rust
Alert {
    AlertTitle { "Dark mode is now available" }
    AlertDescription { "Enable it under your profile settings to get started." }
    AlertAction {
        Button { size: ButtonSize::Xs, "Enable" }
    }
}
```
