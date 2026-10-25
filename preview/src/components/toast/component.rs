use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::spinner::Spinner;
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::toast::{
    self, use_toast_action, Toast, ToastActionButtonProps, ToastCloseButtonProps,
    ToastContentProps, ToastDescriptionProps, ToastProps, ToastTitleProps,
};
use std::time::Duration;

#[component]
fn StyledToast(props: ToastProps) -> Element {
    let base = attributes!(div { class: "dx-toast", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        Toast {
            id: props.id,
            index: props.index,
            revision: props.revision,
            action: props.action,
            title: props.title,
            description: props.description,
            toast_type: props.toast_type,
            on_close: props.on_close,
            permanent: props.permanent,
            duration: props.duration,
            attributes: merged,
            // `data-toast-body` marks the one box whose height is the toast's content: the primitive
            // measures it and sizes the toast and the pile from it (`--toast-height`).
            div { class: "dx-toast-body", "data-toast-body": "",
                if props.toast_type == ToastType::Loading {
                    // Hidden from assistive tech: the toast's own live region announces the title,
                    // and the spinner's `role="status"` would be a second announcement.
                    span { class: "dx-toast-spinner", aria_hidden: "true",
                        Spinner {}
                    }
                }
                ToastContent {
                    ToastTitle {}
                    ToastDescription {}
                }
                ToastActionButton {}
                ToastCloseButton {}
            }
        }
    }
}

/// The toast's action: the library's own `Button`, wired to the primitive's action handle, so
/// activation (click, Enter, Space) runs the callback and dismisses in every skin.
#[component]
fn ToastActionButton(props: ToastActionButtonProps) -> Element {
    let Some(action) = use_toast_action() else {
        return rsx! {};
    };
    rsx! {
        span { class: "dx-toast-action",
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Sm,
                attributes: props.attributes,
                onclick: move |_| action.activate(),
                {action.label()}
            }
        }
    }
}

#[component]
fn ToastContent(props: ToastContentProps) -> Element {
    let base = attributes!(div { class: "dx-toast-content" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        toast::ToastContent { attributes: merged, {props.children} }
    }
}

#[component]
fn ToastTitle(props: ToastTitleProps) -> Element {
    let base = attributes!(div { class: "dx-toast-title" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        toast::ToastTitle { attributes: merged, children: props.children }
    }
}

#[component]
fn ToastDescription(props: ToastDescriptionProps) -> Element {
    let base = attributes!(div { class: "dx-toast-description" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        toast::ToastDescription { attributes: merged, children: props.children }
    }
}

#[component]
fn ToastCloseButton(props: ToastCloseButtonProps) -> Element {
    let base = attributes!(button { class: "dx-toast-close" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        toast::ToastCloseButton { attributes: merged, children: props.children }
    }
}

// Non-markup API types, re-exported so demos compose them through this wrapper.
pub use dioxus_primitives::toast::{
    PromiseMessages, ToastId, ToastPosition, ToastType, ToastUpdate,
};

#[component]
pub fn ToastProvider(
    /// How long a toast without its own duration stays up. Defaults to 5 seconds; `None` keeps such
    /// toasts up until they are closed.
    #[props(default = ReadSignal::new(Signal::new(Some(Duration::from_secs(5)))))]
    default_duration: ReadSignal<Option<Duration>>,
    /// The most toasts kept at once. Past it, the oldest non-permanent toast is removed first.
    /// Defaults to 10.
    #[props(default = ReadSignal::new(Signal::new(10)))] max_toasts: ReadSignal<usize>,
    /// How many toasts the pile shows; the rest wait hidden until a slot frees (default 3).
    #[props(default = ReadSignal::new(Signal::new(3)))]
    visible_toasts: ReadSignal<usize>,
    /// Where the toaster sits: a top or bottom edge, at the start, centre or end (default `BottomEnd`).
    #[props(default)]
    position: ReadSignal<ToastPosition>,
    /// Renders each toast. Defaults to the themed toast.
    #[props(default)] render_toast: Option<Callback<toast::ToastPropsWithOwner, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The page content the toasts appear over.
    children: Element,
) -> Element {
    let render_toast = render_toast.unwrap_or_else(|| {
        Callback::new(|p: toast::ToastPropsWithOwner| rsx! { StyledToast { ..p } })
    });
    let base = attributes!(div { class: "dx-toast-container" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        // dev-docs/backlog.md row 32: this `document::Link` merely appends a
        // `<link rel="stylesheet">` to the document head -- it does not
        // reorder anything already there.
        // `primitives/src/toast.rs`'s own baseline stylesheet
        // (`ensure_toast_base_styles`) is injected by a `use_effect` in
        // `ToastProvider`'s *rendered* primitive, which only runs after
        // this wrapper's initial render commits its elements (including
        // this `Link`) to the DOM. So this themed sheet's `<link>` lands in
        // the head before the primitive's `<style>` tag arrives, and
        // `.dx-toast-container[popover]`/`.dx-toast` win the cascade over
        // the baseline's zero-specificity rules.
        document::Link { rel: "stylesheet", href: asset!("/src/components/toast/style.css") }
        toast::ToastProvider {
            default_duration,
            max_toasts,
            visible_toasts,
            position,
            render_toast,
            attributes: merged,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_primitives::toast::{use_toast, ToastOptions};

    #[component]
    fn TriggerToast() -> Element {
        let toast_api = use_toast();
        use_hook(move || {
            toast_api.success(
                "Saved".to_string(),
                ToastOptions::new()
                    .description("Everything synced")
                    .permanent(true),
            );
        });

        rsx! {}
    }

    #[test]
    fn styled_toast_preserves_primitive_fallback_children() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                ToastProvider {
                    TriggerToast {}
                }
            }
        });
        dom.rebuild_in_place();
        dom.mark_all_dirty();
        dom.render_immediate_to_vec();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("Saved"));
        assert!(html.contains("Everything synced"));
        assert!(html.contains('\u{00d7}') || html.contains("&#215;") || html.contains("&times;"));
    }
}
