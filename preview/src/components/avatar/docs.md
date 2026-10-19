The Avatar components display a user's profile picture or fallback initials. Use the composable `Avatar`, `AvatarImage`, and `AvatarFallback` primitives when you need full control, or `ImageAvatar` for the common image-with-fallback case.

## Component Structure

```rust
Avatar {
    aria_label: "Jane Doe",
    AvatarImage {
        src: "https://example.com/avatar.png",
        alt: "Jane Doe",
    }
    AvatarFallback { "JD" }
}
```

```rust
ImageAvatar {
    src: "https://example.com/avatar.png",
    alt: "Jane Doe",
    on_state_change: |state: AvatarState| { /* image is loading/loaded/failed */ },
    "JD"
}
```

## Sizes

`size: AvatarImageSize` follows shadcn's Nova avatar (`size-6` / `size-8` / `size-10`) and is emitted as `data-size` on the root with shadcn's own values (`sm`, `default`, `lg`). The initials in the fallback follow it (`text-xs` in the small size, `text-sm` otherwise).

| `AvatarImageSize` | Size | shadcn `size` |
|---|---|---|
| `ExtraSmall` | 24px | `sm` |
| `Small` (default) | 32px | `default` |
| `Medium` | 40px | `lg` |

```rust
ImageAvatar { size: AvatarImageSize::Medium, src: "https://example.com/avatar.png", alt: "Jane Doe", "JD" }
```

**Breaking change (Nova migration).** The names are the old ones, the scale is shadcn's. `Small` is unchanged (32px, still the default), so most call sites need nothing. `Medium` went from 48px to 40px, `Large` (64px) is gone because shadcn's scale ends at 40px (use `Medium`, or set `width`/`height` on the avatar yourself for a larger one), and `ExtraSmall` (24px) is new. The `dx-avatar-sm|md|lg` classes no longer exist: select a size with `[data-size="sm|default|lg"]`, or override `--dx-avatar-size` on the root.

The fallback is always `muted` with `muted-foreground` text, and a 1px `border` ring is painted over the picture, as in Nova.

## Loading and the fallback

The fallback follows Base UI: it is rendered in every state except `loaded` (while the picture is loading, after it failed, and when there is no picture) and is replaced only by a picture that has loaded, so there is no blank frame between the two. The prerendered page holds both the `<img>` and the fallback, and hydration starts from the same state, so nothing mismatches.

`AvatarFallback { delay: 600 }` (`ImageAvatar { fallback_delay: 600 }`) holds the fallback back for that many milliseconds, so a picture that loads within it never flashes the fallback. While it is held back the avatar shows its loading placeholder, and the prerendered page has no fallback for it (the timer exists only in the browser). An avatar with no `AvatarFallback` shows a pulsing placeholder while it loads.

```rust
Avatar { aria_label: "Jane Doe",
    AvatarImage { src: "https://example.com/avatar.png", alt: "Jane Doe" }
    AvatarFallback { delay: 600, "JD" }
}
```

## Motion

While the image is loading and there is no fallback standing in for it, the placeholder pulses (`opacity`, 1.5 s, forever). The pulse is paused whenever the avatar is off-screen or the tab is hidden (`dioxus_primitives::activity::use_motion_when`), and an avatar that is not loading subscribes to nothing. Override `animation` on `.dx-avatar[data-state="loading"]` to change or remove it. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Badge

`AvatarBadge` is a status dot or icon on the avatar's bottom-end corner (`absolute right-0 bottom-0`, mirrored in RTL), ringed in the page background. It is a child of `Avatar`, in the same place as `AvatarFallback`. By default it is `primary` with `primary-foreground` ink; override `background-color` for a status colour. Its size follows the avatar: 8px in the small avatar (an icon is hidden there), 10px by default and 12px in the large one, the icon always 8px. The avatar root no longer clips (a clip would cut the badge, which sits outside the circle); the picture and the fallback round themselves to the root's shape.

```rust
Avatar { aria_label: "Avery Lin, online",
    AvatarImage { src: "https://example.com/avatar.png", alt: "" }
    AvatarFallback { "AL" }
    AvatarBadge { style: "background-color: var(--dx-success);" }
}
```

The badge is decorative: put the status in the avatar's `aria_label`.

## Group

`AvatarGroup` stacks avatars with a -8px overlap (`-space-x-2`) and rings each one in the page background, so the overlap reads as a cut. End it with `AvatarGroupCount` for a "+N" tile (or an icon); the tile is `muted`, round, and takes the size of the avatars in the group (24, 32 or 40px). Give the group an `aria_label` to name it.

```rust
AvatarGroup { aria_label: "Team",
    ImageAvatar { src: "https://example.com/a.png", alt: "Avery Lin", "AL" }
    ImageAvatar { src: "https://example.com/b.png", alt: "Casey Park", "CP" }
    AvatarGroupCount { "+3" }
}
```
