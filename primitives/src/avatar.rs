//! Defines the [`Avatar`] component and its subcomponents, which manage user profile images with fallback options.

use std::time::Duration;

use dioxus::{document, prelude::*};
use dioxus_attributes::attributes;

use crate::interval::use_timeout_while;
use crate::{merge_attributes, use_id_or, use_unique_id};

/// Represents the different states an Avatar can be in.
///
/// The fallback follows Base UI: it is shown in every state but [`AvatarState::Loaded`] (so
/// while the picture is still loading it is the picture's stand-in, and there is never a blank
/// frame between the two), and only a loaded picture replaces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarState {
    /// Initial loading state
    Loading,
    /// Image loaded successfully
    Loaded,
    /// Error loading the image
    Error,
    /// No image source provided
    Empty,
}

#[derive(Clone)]
struct AvatarCtx {
    // State
    state: Signal<AvatarState>,
    has_fallback_child: Signal<bool>,
    has_image_child: Signal<bool>,

    // Callbacks
    on_load: Option<EventHandler<()>>,
    on_error: Option<EventHandler<()>>,
    on_state_change: Option<EventHandler<AvatarState>>,
}

fn set_avatar_state(mut ctx: AvatarCtx, state: AvatarState) -> bool {
    if *ctx.state.peek() == state {
        return false;
    }

    ctx.state.set(state);
    if let Some(handler) = &ctx.on_state_change {
        handler.call(state);
    }

    true
}

fn mark_avatar_loaded(ctx: AvatarCtx) {
    if set_avatar_state(ctx.clone(), AvatarState::Loaded) {
        if let Some(handler) = &ctx.on_load {
            handler.call(());
        }
    }
}

fn mark_avatar_error(ctx: AvatarCtx) {
    if set_avatar_state(ctx.clone(), AvatarState::Error) {
        if let Some(handler) = &ctx.on_error {
            handler.call(());
        }
    }
}

/// The props for the [`Avatar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    /// Callback when image loads successfully
    #[props(default)]
    pub on_load: Option<EventHandler<()>>,

    /// Callback when image fails to load
    #[props(default)]
    pub on_error: Option<EventHandler<()>>,

    /// Callback when the avatar state changes
    #[props(default)]
    pub on_state_change: Option<EventHandler<AvatarState>>,

    /// Additional attributes for the avatar element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the Avatar component, which can include AvatarImage and AvatarFallback
    pub children: Element,
}

/// # Avatar
///
/// A component that displays a user profile image with fallback options.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::avatar::{Avatar, AvatarFallback, AvatarImage};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Avatar {
///             aria_label: "Basic avatar",
///             AvatarImage {
///                 src: "https://avatars.githubusercontent.com/u/66571940?s=96&v=4",
///                 alt: "ealmloff user avatar",
///             }
///             AvatarFallback { class: "dx-avatar-fallback", "EA" }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Avatar`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the avatar. Possible values are `loading`, `loaded`, `error`, or `empty`.
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    // Internal state tracking
    let state = use_signal(|| AvatarState::Empty);
    let has_fallback_child = use_signal(|| false);
    let has_image_child = use_signal(|| false);

    // Create context for child components
    use_context_provider(|| AvatarCtx {
        state,
        has_fallback_child,
        has_image_child,
        on_load: props.on_load,
        on_error: props.on_error,
        on_state_change: props.on_state_change,
    });

    // The built-in "??" stand-in (used only when no `AvatarFallback` was given) is for a picture that
    // failed or was never given; while loading, the root's `data-state="loading"` placeholder shows.
    let show_fallback =
        use_memo(move || matches!(state(), AvatarState::Error | AvatarState::Empty));

    let owned = attributes!(span {
        role: "img",
        "data-state": match state() {
            AvatarState::Loading => "loading",
            AvatarState::Loaded => "loaded",
            AvatarState::Error => "error",
            AvatarState::Empty => "empty",
        },
    });
    // The loading placeholder pulses: paused while off-screen or in a hidden tab
    // (`crate::activity`); an avatar that is not loading subscribes to nothing.
    let motion = crate::activity::use_motion_when(move || state() == AvatarState::Loading);
    let merged = merge_attributes(vec![props.attributes.clone(), owned, motion.attributes()]);

    rsx! {
        span {
            ..merged,

            // Children (which may include AvatarImage and AvatarFallback)
            {props.children}

            // Default fallback if no AvatarFallback is provided and fallback should be shown
            if show_fallback() && !has_fallback_child() && has_image_child() {
                span {
                    style: "display: flex; align-items: center; justify-content: center; width: 100%; height: 100%;",
                    "??"
                }
            }
        }
    }
}

/// The props for the [`AvatarFallback`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    /// How long to wait, in milliseconds, before the fallback may show (Base UI's `delay`). A
    /// picture that loads within it never flashes the fallback. `None` (the default) shows the
    /// fallback straight away, including in the server-rendered markup; with a delay the server
    /// markup has no fallback (the timer only exists in the browser), and the root's loading
    /// placeholder stands in until it passes. Read once, when the fallback mounts.
    #[props(default)]
    pub delay: Option<u32>,
    /// Additional attributes for the fallback element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the AvatarFallback component, typically text or an icon
    pub children: Element,
}

/// # AvatarFallback
///
/// A component that stands in for the picture: it is rendered in every state except `loaded`
/// (so while the picture is loading, if it fails, or if there is none) and replaced by the picture
/// once it has loaded, as in Base UI. Give it a `delay` to hold it back for a moment so a fast
/// load never shows it.
///
/// This component must be used inside an [`Avatar`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::avatar::{Avatar, AvatarFallback, AvatarImage};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Avatar {
///             aria_label: "Basic avatar",
///             AvatarImage {
///                 src: "https://avatars.githubusercontent.com/u/66571940?s=96&v=4",
///                 alt: "ealmloff user avatar",
///             }
///             AvatarFallback { delay: 600, class: "dx-avatar-fallback", "EA" }
///         }
///     }
/// }
/// ```
#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let mut ctx: AvatarCtx = use_context();

    // Mark that a fallback child is provided
    use_effect(move || {
        ctx.has_fallback_child.set(true);
    });

    // The delay gate. With no delay it starts open (so the server markup holds the fallback);
    // with one it starts closed on the server and the client alike (no hydration mismatch) and
    // opens when the timer fires, which only runs in the browser.
    let mut delay_passed = use_signal(|| props.delay.is_none());
    let waiting = use_memo(move || !delay_passed());
    let delay = Duration::from_millis(u64::from(props.delay.unwrap_or(0)));
    use_timeout_while(waiting, delay, move || delay_passed.set(true));

    // Everything but a loaded picture shows the fallback (Base UI).
    let show_fallback = use_memo(move || (ctx.state)() != AvatarState::Loaded);

    if !show_fallback() || !delay_passed() {
        return rsx!({});
    }

    rsx! {
        span { ..props.attributes, {props.children} }
    }
}

/// The props for the [`AvatarImage`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AvatarImageProps {
    /// The id of the image element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// The image source URL
    pub src: String,

    /// Alt text for the image
    #[props(default)]
    pub alt: Option<String>,

    /// Additional attributes for the image element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # AvatarImage
///
/// A component that displays a user profile image. While it loads the [`AvatarFallback`] stands in
/// for it; if the image fails to load, it will stop rendering and the Avatar will switch to the error
/// state, which the fallback keeps covering.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::avatar::{Avatar, AvatarFallback, AvatarImage};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Avatar {
///             aria_label: "Basic avatar",
///             AvatarImage {
///                 src: "https://avatars.githubusercontent.com/u/66571940?s=96&v=4",
///                 alt: "ealmloff user avatar",
///             }
///             AvatarFallback { class: "dx-avatar-fallback", "EA" }
///         }
///     }
/// }
/// ```
#[component]
pub fn AvatarImage(props: AvatarImageProps) -> Element {
    let ctx: AvatarCtx = use_context();
    let mut current_src = use_signal(|| None::<String>);
    let image_id = use_id_or(use_unique_id(), props.id);
    let src = props.src.clone();
    let mut effect_ctx = ctx.clone();

    // Track the image source independently so source changes reset loading state before the
    // browser's image events report the final result.
    use_effect(use_reactive!(|src| {
        effect_ctx.has_image_child.set(true);

        if src.is_empty() {
            current_src.set(None);
            set_avatar_state(effect_ctx.clone(), AvatarState::Empty);
            return;
        }

        if current_src.peek().as_ref() != Some(&src) {
            current_src.set(Some(src.clone()));
            set_avatar_state(effect_ctx.clone(), AvatarState::Loading);
        }
    }));

    let watcher_src = props.src.clone();
    let watcher_ctx = ctx.clone();
    let watcher_current_src = current_src;
    // Reconcile cached or very fast image loads that can complete before Dioxus
    // delivers the synthetic load/error event.
    use_effect(use_reactive!(|watcher_src| {
        if watcher_src.is_empty() {
            return;
        }

        let image_id_value = image_id();
        let mut eval = document::eval(
            r#"
            const imageId = await dioxus.recv();
            const expectedSrc = await dioxus.recv();
            const image = document.getElementById(imageId);

            const matchesExpectedSrc = image && (
                image.getAttribute("src") === expectedSrc ||
                image.currentSrc === expectedSrc ||
                image.src === expectedSrc
            );

            if (!matchesExpectedSrc || !image.complete) {
                dioxus.send("pending");
            } else {
                dioxus.send(image.naturalWidth > 0 ? "loaded" : "error");
            }
            "#,
        );
        let _ = eval.send(image_id_value);
        let _ = eval.send(watcher_src.clone());

        let event_ctx = watcher_ctx.clone();
        let mut event_current_src = watcher_current_src;
        spawn(async move {
            let Ok(state) = eval.recv::<String>().await else {
                return;
            };

            let matches_current_src = event_current_src
                .peek()
                .as_ref()
                .map(|src| src == &watcher_src)
                .unwrap_or(true);

            if !matches_current_src {
                return;
            }

            match state.as_str() {
                "loaded" => {
                    event_current_src.set(Some(watcher_src.clone()));
                    mark_avatar_loaded(event_ctx.clone());
                }
                "error" => {
                    event_current_src.set(Some(watcher_src.clone()));
                    mark_avatar_error(event_ctx.clone());
                }
                _ => {}
            }
        });
    }));

    let load_src = props.src.clone();
    let load_ctx = ctx.clone();
    let mut load_current_src = current_src;

    let handle_load = move |_| {
        if load_src.is_empty() {
            return;
        }

        let matches_current_src = load_current_src
            .peek()
            .as_ref()
            .map(|src| src == &load_src)
            .unwrap_or(true);

        if matches_current_src {
            load_current_src.set(Some(load_src.clone()));
            mark_avatar_loaded(load_ctx.clone());
        }
    };

    let error_src = props.src.clone();
    let error_ctx = ctx.clone();
    let mut error_current_src = current_src;

    let handle_error = move |_| {
        if error_src.is_empty() {
            return;
        }

        let matches_current_src = error_current_src
            .peek()
            .as_ref()
            .map(|src| src == &error_src)
            .unwrap_or(true);

        if matches_current_src {
            error_current_src.set(Some(error_src.clone()));
            mark_avatar_error(error_ctx.clone());
        }
    };

    let show_image = !props.src.is_empty() && (ctx.state)() != AvatarState::Error;
    if !show_image {
        return rsx!({});
    }

    let owned = attributes!(img {
        style: "width: 100%; height: 100%; object-fit: cover;"
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        img {
            id: image_id,
            src: props.src.clone(),
            alt: props.alt.clone().unwrap_or_default(),
            onload: handle_load,
            onerror: handle_error,
            ..merged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// An `AvatarFallback` under a context whose status is `state`.
    #[component]
    fn FallbackIn(state: AvatarState) -> Element {
        let state = use_signal(|| state);
        let has_fallback_child = use_signal(|| false);
        let has_image_child = use_signal(|| false);
        use_context_provider(|| AvatarCtx {
            state,
            has_fallback_child,
            has_image_child,
            on_load: None,
            on_error: None,
            on_state_change: None,
        });
        rsx! {
            AvatarFallback { "FB" }
        }
    }

    fn rendered_in(state: AvatarState) -> bool {
        let mut dom = VirtualDom::new_with_props(FallbackIn, FallbackInProps { state });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom).contains("FB")
    }

    /// Base UI: the fallback is shown in every status but `loaded`, so there is no blank frame while the
    /// picture loads (the old rule removed it as soon as the status became `loading`).
    #[test]
    fn the_fallback_shows_in_every_state_but_loaded() {
        assert!(rendered_in(AvatarState::Empty));
        assert!(rendered_in(AvatarState::Loading));
        assert!(rendered_in(AvatarState::Error));
        assert!(!rendered_in(AvatarState::Loaded));
    }

    #[component]
    fn Plain() -> Element {
        rsx! {
            Avatar {
                AvatarImage { src: "/a.png", alt: "A" }
                AvatarFallback { "AB" }
            }
        }
    }

    #[component]
    fn Delayed() -> Element {
        rsx! {
            Avatar {
                AvatarImage { src: "/a.png", alt: "A" }
                AvatarFallback { delay: 500, "AB" }
            }
        }
    }

    /// The server markup holds both the picture and the fallback, so hydration (which starts in the same
    /// status) finds exactly what the server sent.
    #[test]
    fn the_server_markup_holds_the_picture_and_the_fallback() {
        let html = render(Plain);
        assert!(html.contains("<img"), "{html}");
        assert!(html.contains("AB"), "{html}");
    }

    /// With a `delay` the fallback is closed on the server and on the first client render alike (the timer
    /// only exists in the browser), so there is nothing to mismatch.
    #[test]
    fn a_delayed_fallback_is_not_in_the_server_markup() {
        let html = render(Delayed);
        assert!(html.contains("<img"), "{html}");
        assert!(!html.contains("AB"), "{html}");
    }
}
