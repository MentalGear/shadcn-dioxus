use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::avatar::{self, AvatarState};
use dioxus_primitives::merge_attributes;

/// The avatar's size, following shadcn's Nova style (`data-[size=sm]:size-6`, the
/// default `size-8`, `data-[size=lg]:size-10`).
///
/// The names keep the library's old spelling so existing call sites compile, but
/// the sizes moved to shadcn's scale:
///
/// | variant      | now            | shadcn `size` | before |
/// |--------------|----------------|---------------|--------|
/// | `ExtraSmall` | 24px (new)     | `sm`          | none   |
/// | `Small`      | 32px (default) | `default`     | 32px   |
/// | `Medium`     | 40px           | `lg`          | 48px   |
/// | `Large`      | removed        | none          | 64px   |
#[derive(Clone, Copy, PartialEq, Default)]
pub enum AvatarImageSize {
    ExtraSmall,
    #[default]
    Small,
    Medium,
}

impl AvatarImageSize {
    /// The value of `data-size` on the root, shadcn's own spelling.
    fn to_data_size(self) -> &'static str {
        match self {
            AvatarImageSize::ExtraSmall => "sm",
            AvatarImageSize::Small => "default",
            AvatarImageSize::Medium => "lg",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum AvatarShape {
    #[default]
    Circle,
    Rounded,
}

impl AvatarShape {
    fn to_class(self) -> &'static str {
        match self {
            AvatarShape::Circle => "dx-avatar-circle",
            AvatarShape::Rounded => "dx-avatar-rounded",
        }
    }
}

/// The props for the [`Avatar`] root component.
#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    /// Callback when image loads successfully.
    #[props(default)]
    pub on_load: Option<EventHandler<()>>,

    /// Callback when image fails to load.
    #[props(default)]
    pub on_error: Option<EventHandler<()>>,

    /// Callback when the avatar state changes.
    #[props(default)]
    pub on_state_change: Option<EventHandler<AvatarState>>,

    /// The avatar's size: `ExtraSmall`, `Small` (default) or `Medium`.
    #[props(default)]
    pub size: AvatarImageSize,

    /// The avatar's outline: see `AvatarShape` (default `Circle`).
    #[props(default)]
    pub shape: AvatarShape,

    /// Additional attributes for the avatar element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The fallback content shown while the image is loading or if it fails to load.
    pub children: Element,
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let class = format!("{} {}", "dx-avatar", props.shape.to_class());
    let base = attributes!(span {
        class,
        "data-dx-root": "",
        "data-size": props.size.to_data_size(),
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        avatar::Avatar {
            on_load: props.on_load,
            on_error: props.on_error,
            on_state_change: props.on_state_change,
            attributes: merged,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AvatarImageProps {
    /// The `id` of the `<img>` element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// The image URL. While it loads (or if it fails) the sibling `AvatarFallback` shows.
    pub src: String,

    /// Alternative text for the image; leave empty only for a purely decorative avatar.
    #[props(default)]
    pub alt: String,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn AvatarImage(props: AvatarImageProps) -> Element {
    let base = attributes!(img {
        class: "dx-avatar-image",
        draggable: "false",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        avatar::AvatarImage {
            id: props.id,
            src: props.src,
            alt: props.alt,
            attributes: merged,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    /// Milliseconds to wait before the fallback may show (Base UI's `delay`): a picture that loads within it never
    /// flashes the fallback. Default: none, the fallback shows immediately (also in the prerendered page).
    #[props(default)]
    pub delay: Option<u32>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// What shows while the image loads or after it fails: initials, an icon.
    pub children: Element,
}

#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let base = attributes!(span {
        class: "dx-avatar-fallback",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        avatar::AvatarFallback {
            delay: props.delay,
            attributes: merged,
            {props.children}
        }
    }
}

/// The props for the [`ImageAvatar`] convenience component.
#[derive(Props, Clone, PartialEq)]
pub struct ImageAvatarProps {
    /// The image source URL.
    pub src: String,

    /// The image alt text.
    #[props(default)]
    pub alt: String,

    /// Callback when image loads successfully.
    #[props(default)]
    pub on_load: Option<EventHandler<()>>,

    /// Callback when image fails to load.
    #[props(default)]
    pub on_error: Option<EventHandler<()>>,

    /// Callback when the avatar state changes.
    #[props(default)]
    pub on_state_change: Option<EventHandler<AvatarState>>,

    /// The avatar's size: `ExtraSmall`, `Small` (default) or `Medium`.
    #[props(default)]
    pub size: AvatarImageSize,

    /// The avatar's outline: see `AvatarShape` (default `Circle`).
    #[props(default)]
    pub shape: AvatarShape,

    /// Additional attributes for the avatar element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Milliseconds to hold the fallback back while the image loads (see [`AvatarFallback`]'s `delay`).
    #[props(default)]
    pub fallback_delay: Option<u32>,

    /// The fallback content shown while the image is loading or if it fails to load.
    pub children: Element,
}

#[component]
pub fn ImageAvatar(props: ImageAvatarProps) -> Element {
    // axe `role-img-alt` (docs/backlog.md row 34's own round): the root
    // `Avatar` renders `role="img"` (`avatar.rs`) and gets its accessible
    // name from a caller-supplied `aria_label`, same as this crate's own
    // doc example -- this convenience wrapper forwarded `alt` only to the
    // inner `<img>` (via `AvatarImage`), never to the outer `role="img"`
    // span, so every caller that used `alt` instead of separately also
    // setting `aria_label` (every dashboard email-client avatar; the
    // sidebar's user avatar) rendered an unnamed `role="img"`. Default the
    // root's `aria_label` from `alt` -- merged caller-wins, so an explicit
    // `aria_label` in `props.attributes` still overrides it. Contributed
    // only when `alt` is non-empty: an `aria-label=""` is still an empty
    // accessible name (no better than omitting it), and would additionally
    // shadow any real name the caller supplies some other way.
    let base: Vec<Attribute> = if props.alt.is_empty() {
        Vec::new()
    } else {
        attributes!(span { aria_label: "{props.alt}" })
    };
    let attributes = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        Avatar {
            on_load: props.on_load,
            on_error: props.on_error,
            on_state_change: props.on_state_change,
            size: props.size,
            shape: props.shape,
            attributes,
            AvatarImage {
                src: props.src,
                alt: props.alt,
            }
            AvatarFallback { delay: props.fallback_delay, {props.children} }
        }
    }
}

/// A status dot or icon at the avatar's bottom-end corner (shadcn's `AvatarBadge`: `absolute right-0
/// bottom-0`, `bg-primary text-primary-foreground`, a 2px ring in the page background). It must be a child
/// of [`Avatar`] -- the size comes from the root's `data-size`: 8px in the small avatar (an icon is hidden
/// there), 10px by default and 12px in the large one, with the icon at 8px. Colour a status by overriding
/// `background-color` (`style: "background-color: var(--dx-success)"`). A purely decorative mark; give the
/// avatar's `aria-label` the status if it matters.
#[component]
pub fn AvatarBadge(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    #[props(default)] children: Element,
) -> Element {
    let base = attributes!(span { class: "dx-avatar-badge" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        span { ..merged, {children} }
    }
}

/// An overlapping stack of [`Avatar`]s (shadcn's `AvatarGroup`: `flex -space-x-2`, each avatar ringed in
/// the page background so the overlap reads as a cut). End it with an [`AvatarGroupCount`] for the "+N"
/// tile. A plain `div`; pass `aria_label` to name the group.
#[component]
pub fn AvatarGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-avatar-group" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        div { ..merged, {children} }
    }
}

/// The "+N" tile at the end of an [`AvatarGroup`]: `muted` with `muted-foreground` text, ringed like the
/// avatars and sized to match them (it follows the group's avatars' `data-size`). Holds text or an icon.
#[component]
pub fn AvatarGroupCount(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-avatar-group-count" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/avatar/style.css") }
        div { ..merged, {children} }
    }
}
