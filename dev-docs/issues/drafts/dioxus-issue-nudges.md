# Draft: two comment nudges for existing Dioxus issues

**Status:** drafted 2026-09-03, not filed. Both issue numbers below (**#2962**, **#4319**) are carried over from `dev-docs/backlog.md` row 14's existing text, which predates this drafting session. **Neither number is independently corroborated anywhere in this repo's own tracking**: the running list of Dioxus framework friction at the end of this file (merged here from the root `complaints.md` by row 266) documents the underlying *behavior* each comment below describes (the ancestor-traversal gap, the need for a typed dialog-close/open API) but cites no GitHub issue number for either of these two behaviours, and no source file in `primitives/src/` references either number. Per this task's own instruction: **do not treat #2962/#4319 as verified — confirm both numbers against Dioxus's actual issue tracker before posting either comment**, and if either is wrong, search `DioxusLabs/dioxus`'s issues for the correct one (search terms below) rather than guessing a replacement.

---

**Target repo:** `DioxusLabs/dioxus`.

## Comment 1 — issue **#2962** (`onclose` event) — number unverified, confirm before posting

**Suggested search if #2962 turns out wrong:** issues mentioning `<dialog>`, `onclose`, or the native `close`/`cancel` DOM events in the context of Dioxus's event system.

> We hit exactly this gap building native `<dialog>` support for our modal `Dialog`/`AlertDialog` components (`MentalGear/dioxus-components`, a fork). Since there's no typed `onclose` (or `oncancel`) event exposed on `dialog` elements, we worked around it with a small `document::eval` bridge: `use_dialog_close_sync` (`primitives/src/lib.rs`) opens an eval channel, adds a native `close` event listener to the `<dialog>` element by id from the JS side, and forwards a signal back into Rust via `dioxus.send`/`await dioxus.recv()` whenever that native event fires — closing the loop so a `showModal()`-driven dialog dismissed by the browser itself (Escape's default `cancel` action, a `method="dialog"` form submission, or any other close not initiated by our own code) doesn't leave our `open` signal stranded at `true`. It works, but it's more moving parts than a first-class `onclose`/`oncancel` prop on `dialog` elements would need: an eval channel, a cleanup-on-unmount `send`, and a listener registered/torn down by hand.
>
> A typed `onclose`/`oncancel` handler on `dialog` elements (mirroring how other native events already get typed Dioxus handlers) would let us delete this bridge entirely and just write `dialog { onclose: move |_| set_open.call(false) }`. Happy to share the exact construction (`use_dialog_close_sync`, `use_dialog_open_driver`) if it's useful as a concrete "here's what a user has to build without this" reference, or to test against an experimental implementation if one exists.

## Comment 2 — issue **#4319** (dialog methods) — number unverified, confirm before posting

**Suggested search if #4319 turns out wrong:** issues about calling native `HTMLDialogElement` methods (`showModal()`, `close()`, `show()`) from Dioxus, or about binding a `dialog` element's `open` attribute declaratively alongside imperative modal control.

> Same area as the `onclose` gap (issue #2962, linked above): calling `showModal()`/`close()` on a `<dialog>` element currently has to go through a `document::eval` bridge rather than a typed method call. Our construction (`use_dialog_open_driver`, `primitives/src/lib.rs`) drives this from a Rust `open` signal: an effect re-runs `document::eval` with a small inline script that looks the element up by id, checks its `.open` DOM property, and calls `showModal()`/`close()` only when that property disagrees with the signal (calling either method redundantly throws `InvalidStateError`, so the guard is load-bearing, not defensive-only).
>
> One trap worth flagging concretely, since it cost us real debugging time and might be useful signal for whatever a typed API's design ends up being: we found that binding a `dialog` element's `open` attribute **declaratively** (`dialog { open: some_signal }`) in the same build as also calling `showModal()` imperatively **doesn't** produce a runtime error or a warning — it silently makes `showModal()` a no-op, because Dioxus commits the declarative `open` attribute during render, before any effect driving `showModal()` gets a chance to run, so by the time an imperative call checks `dialog.open` it's already `true` and the call is skipped. If a typed method API (`dialog_ref.show_modal()`, or similar) ships, it'd be worth either documenting this interaction explicitly or making the two mechanisms mutually exclusive at the type level, since the failure mode (dialog looks "open" per its attribute, but `showModal()`'s actual modal behavior — top layer, focus trap, `::backdrop` — never engaged) is easy to miss in testing and only shows up as "the dialog doesn't trap focus" rather than a hard error.
>
> A typed `dialog_ref.show_modal()`/`.close()` (or a `use_dialog_ref`-style hook returning handles to both) would let us drop the eval bridge the same way a typed `onclose` would for the paired issue.

## Before filing

- [ ] **Verify #2962 and #4319 actually exist and are about these topics** on `DioxusLabs/dioxus`'s issue tracker — neither number is corroborated in this repo's own docs or code, only carried over from `dev-docs/backlog.md` row 14's pre-existing text. If either number is wrong, search using the terms above and substitute the correct issue (or file new ones if none exist).
- [ ] Re-verify against the latest Dioxus release — either gap may already be closed.
- [ ] Confirm both issues are still open (not already closed/fixed) before posting either comment.
- [ ] Link a permalink to `primitives/src/lib.rs`'s `use_dialog_close_sync`/`use_dialog_open_driver` at the commit these comments are posted from (both currently `#[cfg(feature = "web")]`-gated private functions in that file, not part of the public API).

---

## Appendix: running list of Dioxus framework friction

This file's tracker of Dioxus issues found while developing this library (formerly the root `complaints.md`, merged by row 266).

#### Unused props don't emit unused warnings.
https://github.com/DioxusLabs/dioxus/issues/3919

#### Setting default of signal prop is verbose.
https://github.com/DioxusLabs/dioxus/issues/3920

It's verbose to set a `Signal` or `ReadSignal`'s default value through props.
```rust
#[derive(Props, Clone, PartialEq)]
pub struct SomeProps {

    // This sets bool to be false
    #[props(default)] 
    value: ReadSignal<bool>,

    // This is what I'd like, except it wants a ReadSignal
    #[props(default = true)] 
    value: ReadSignal<bool>,

    // Instead you have to do this:
    #[props(default = ReadSignal::new(Signal::new(true)))]
    value: ReadSignal<bool>,

    // Same for a regular signal:
    #[props(default = Signal::new(true))]
    value: Signal<bool>,
}
```

#### No way to know a component or element's parent, siblings, or children.

Some stuff relies on knowing their surrounding elements for proper behavior.

Take [radix-primitives' switch](https://github.com/radix-ui/primitives/blob/6e75e117977c9e6ffa939e6951a707f16ba0f95e/packages/react/switch/src/switch.tsx#L51) as an example. It detects when the switch is in a form and creates an input so that the switch's value bubbles with the form submit event.

This is also an issue with keybind navigation - we can give components ids to internally track them through a parent context, but how do we know which order they are in for navigation?

At a minimum, I need the ordering index. This could be a special prop similar to `children`.

#### Need Portals
Components should behave as if they are still a child of the parent of the "portaled" item. Same scope basically - context is still consumable as if it was a child.

Aka the component is still used in the same spot, and the portal only moves where that component is actually rendered. Portals can't have children or attributes:

```rust

#[component]
pub fn App() -> Element {
    let portal = use_portal();

    rsx! {
        div {
            // ... nested stuff
            PortalIn {
                portal,

                // Children of PortalIn becomes children of PortalOut.
                div {
                    h1 { "Alert Dialog!" }
                    p { "alert!!" }
                }
            }
        }

        div {
            // ... other nested stuff
            PortalOut { portal }
        }
    }
}

```

#### `From<Signal<T>>` Is Not Implemented For `Option<ReadSignal<T>>`

#### `From<T>` Is Not Implemented For `Option<ReadSignal<T>>`
`T` can already be converted to `Option<Signal<T>>` when provided thru props.
This however doesn't work for `Option<ReadSignal<T>>`.

#### Number Props Don't Type Infer
Normally Rust would automatically determine that a number should be of type thru inference but for props it doesn't work when the prop is a signal.

`index: ReadSignal<usize>,` fails
`index: usize,` works

```rust
SomeComponent {
    index: 1,
}
```

#### No `#[props(extends = MyPropsStruct)]`
https://github.com/DioxusLabs/dioxus/issues/3938

```rust
pub fn MyComp1(#[props(extends = MyComp2)] comp2_attr: Vec<Attribute>) -> Element {
    rsx! {
        MyComp2 {
            ..comp2_attr
        }
    }
}

```

#### Unable To Insert Attributes On `Element`.

Ideally there would be a way to pass attributes through a top-level component. Radix has the `asChild` prop which replaces their element with the user provided one. E.g.
```rs
rsx! {

    if as_child {
        p {
            aria_something: "abc",
            {children}
        }
    } else {
        {children} // Can't add `aria_something`
    }
}
```
