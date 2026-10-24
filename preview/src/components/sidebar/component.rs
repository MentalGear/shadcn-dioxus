use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::separator::Separator;
use crate::components::sheet::{
    Sheet, SheetDescription, SheetHeader, SheetSide, SheetTitle,
};
use crate::components::skeleton::Skeleton;
use crate::components::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;
use dioxus_icons::lucide::PanelLeft;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::js_listener::{use_js_listeners, JsListeners};
use dioxus_primitives::merge_attributes;
use dioxus_primitives::use_controlled;

// Delivery: every exported entry point in this file renders its own `document::Link` to `style.css`
// (dedup on (href, rel) makes that free; see checkbox/component.rs's header comment). The variant demos in
// `variants/{main,inset,floating}/mod.rs` always render the real themed wrapper (`SidebarProvider` /
// `Sidebar`) from this file, so that Link covers the sheet for them; each `Demo()` also links
// `variants/demo.css`, which holds only the demo's own decorative classes (team-switcher icon/info-block/
// chevron, `dx-sidebar-hide-on-collapse`).
//
// The screen-reader-only helper is `dx-sidebar-sr-only` (`pagination` keeps its own
// `dx-pagination-sr-only` copy); see `style.css` and `scripts/check-dx-class-prefix.sh`.

// constants
const SIDEBAR_WIDTH: &str = "16rem";
const SIDEBAR_WIDTH_MOBILE: &str = "18rem";
const SIDEBAR_WIDTH_ICON: &str = "3rem";
const SIDEBAR_KEYBOARD_SHORTCUT: &str = "b";
const MOBILE_BREAKPOINT: u32 = 768;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarState {
    #[default]
    Expanded,
    Collapsed,
}

impl SidebarState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarState::Expanded => "expanded",
            SidebarState::Collapsed => "collapsed",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

impl SidebarSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarSide::Left => "left",
            SidebarSide::Right => "right",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

impl SidebarVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarVariant::Sidebar => "sidebar",
            SidebarVariant::Floating => "floating",
            SidebarVariant::Inset => "inset",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarCollapsible {
    #[default]
    Offcanvas,
    Icon,
    None,
}

impl SidebarCollapsible {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarCollapsible::Offcanvas => "offcanvas",
            SidebarCollapsible::Icon => "icon",
            SidebarCollapsible::None => "none",
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct SidebarCtx {
    pub state: Memo<SidebarState>,
    pub side: Signal<SidebarSide>,
    pub is_mobile: Signal<bool>,
    // From use_controlled:
    open: Memo<bool>,
    set_open: Callback<bool>,
    // Mobile state:
    open_mobile: Signal<bool>,
}

impl SidebarCtx {
    /// Toggle the sidebar open/closed state
    pub fn toggle(&self) {
        if (self.is_mobile)() {
            let current = (self.open_mobile)();
            let mut open_mobile = self.open_mobile;
            open_mobile.set(!current);
        } else {
            self.set_open.call(!self.open());
        }
    }

    /// Set the mobile sidebar open state
    pub fn set_open_mobile(&self, value: bool) {
        let mut open_mobile = self.open_mobile;
        open_mobile.set(value);
    }

    /// Get the current open state (desktop)
    pub fn open(&self) -> bool {
        self.open.cloned()
    }
}

pub fn use_sidebar() -> SidebarCtx {
    use_context::<SidebarCtx>()
}

pub fn use_is_mobile() -> Signal<bool> {
    let mut is_mobile = use_signal(|| false);

    // The resize listener belongs to this component: `use_js_listeners` removes it when the
    // provider unmounts (it used to be reachable only through a `window.__sidebarResizeHandler`
    // global, so a second provider, or a remount before the drop ran, orphaned the first).
    use_js_listeners(move || {
        Some(
            JsListeners::install(&format!(
                r#"
                const checkMobile = () => window.innerWidth < {MOBILE_BREAKPOINT};
                listen(window, 'resize', () => dioxus.send(checkMobile()));
                dioxus.send(checkMobile());
                "#
            ))
            .on_message(move |mobile: bool| is_mobile.set(mobile)),
        )
    });

    is_mobile
}

#[component]
pub fn SidebarProvider(
    #[props(default = true)] default_open: bool,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let is_mobile = use_is_mobile();
    let side = use_signal(|| SidebarSide::Left);
    let open_mobile = use_signal(|| false);

    let (open, set_open) = use_controlled(open, default_open, on_open_change);

    let state = use_memo(move || {
        if open() {
            SidebarState::Expanded
        } else {
            SidebarState::Collapsed
        }
    });

    let ctx = SidebarCtx {
        state,
        side,
        is_mobile,
        open,
        set_open,
        open_mobile,
    };

    use_context_provider(|| ctx);

    use_js_listeners(move || {
        Some(
            JsListeners::install(&format!(
                r#"
                listen(window, 'keydown', (event) => {{
                    if (event.key === '{SIDEBAR_KEYBOARD_SHORTCUT}' && (event.metaKey || event.ctrlKey)) {{
                        event.preventDefault();
                        dioxus.send(true);
                    }}
                }});
                "#
            ))
            .on_message(move |_: bool| ctx.toggle()),
        )
    });

    let sidebar_style = format!(
        r#"--dx-sidebar-width: {SIDEBAR_WIDTH}; --dx-sidebar-width-mobile: {SIDEBAR_WIDTH_MOBILE}; --dx-sidebar-width-icon: {SIDEBAR_WIDTH_ICON}"#
    );

    let base = attributes!(div {
        class: "dx-sidebar-wrapper",
        "data-slot": "sidebar-wrapper",
        style: sidebar_style,
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        // Eager: the mobile branch mounts `Sheet` only below the breakpoint, so on a desktop first render its
        // `Link` would be inserted late (a runtime switch to mobile) -- not render-blocking.
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn Sidebar(
    #[props(default)] side: SidebarSide,
    #[props(default)] variant: SidebarVariant,
    #[props(default)] collapsible: SidebarCollapsible,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let ctx = use_sidebar();
    let mut ctx_side = ctx.side;
    if *ctx_side.peek() != side {
        ctx_side.set(side);
    }

    let is_mobile = ctx.is_mobile;
    let state = ctx.state;
    let open_mobile = ctx.open_mobile;

    if collapsible == SidebarCollapsible::None {
        // axe `region` (docs/backlog.md row 38): this was a plain `<div>`
        // with no landmark role at all, so its content (menu items,
        // footer) sat outside every landmark on the page -- surfaced both
        // by the block-route direct-visit case (`ComponentBlockDemo`,
        // `preview/src/main.rs`) and, independently, by the dashboard
        // email client's own embedded sidebar (`dashboard/views/
        // email_client/sidebar.rs`), which renders `Sidebar` directly and
        // was never routed through the block preview at all. `role:
        // "complementary"` (the `<aside>` landmark) fixes both call sites
        // at their one shared source; labelled so it stays unambiguous if
        // a future page ever has more than one.
        let base = attributes!(div {
            class: "dx-sidebar-static",
            "data-slot": "sidebar",
            "data-side": side.as_str(),
            role: "complementary",
            aria_label: "Sidebar",
        });
        let merged = merge_attributes(vec![base, attributes]);

        return rsx! {
            document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
            div { ..merged, {children} }
        };
    }

    if is_mobile() {
        let sheet_side = match side {
            SidebarSide::Left => SheetSide::Left,
            SidebarSide::Right => SheetSide::Right,
        };

        return rsx! {
            document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
            Sheet {
                open: open_mobile(),
                on_open_change: move |v| ctx.set_open_mobile(v),
                "data-side": sheet_side.as_str(),
                class: "dx-sidebar-sheet".to_string(),
                "data-sidebar": "sidebar",
                "data-slot": "sidebar",
                "data-mobile": "true",
                div { class: "dx-sidebar-sr-only",
                    SheetHeader {
                        SheetTitle { "Sidebar" }
                        SheetDescription { "Displays the mobile sidebar." }
                    }
                }
                div { class: "dx-sidebar-mobile-inner", {children} }
            }
        };
    }

    let collapsible_str = if state() == SidebarState::Collapsed {
        collapsible.as_str()
    } else {
        ""
    };

    let container_base = attributes!(div {
        class: "dx-sidebar-container",
        "data-slot": "sidebar-container",
    });
    let container_attrs = merge_attributes(vec![container_base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        // Eager: the mobile branch mounts `Sheet` only below the breakpoint, so on a desktop first render its
        // `Link` would be inserted late (a runtime switch to mobile) -- not render-blocking.
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        div {
            class: "dx-sidebar-desktop",
            "data-state": state().as_str(),
            "data-collapsible": collapsible_str,
            "data-variant": variant.as_str(),
            "data-side": side.as_str(),
            "data-slot": "sidebar",
            // axe `region` (docs/backlog.md row 38) -- see the identical
            // comment on the `collapsible == SidebarCollapsible::None`
            // branch above; this is the desktop-expanded/-collapsed
            // branch actually rendered by both known affected call sites
            // (the block route's `sidebar` demo and the dashboard email
            // client's sidebar), so it's the one that mattered for the
            // reported gap.
            role: "complementary",
            aria_label: "Sidebar",
            div { class: "dx-sidebar-gap", "data-slot": "sidebar-gap" }
            div {
                ..container_attrs,
                div {
                    class: "dx-sidebar-inner",
                    "data-sidebar": "sidebar",
                    "data-slot": "sidebar-inner",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn SidebarTrigger(
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    /// Replaces the default panel icon (e.g. a hamburger `Menu`). Give it the class
    /// `dx-sidebar-trigger-icon` to keep the trigger's icon sizing and RTL flip.
    #[props(default)]
    icon: Option<Element>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_sidebar();

    let base = attributes!(button {
        class: "dx-sidebar-trigger",
        "data-sidebar": "trigger",
        "data-slot": "sidebar-trigger",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            onclick: move |e| {
                if let Some(handler) = &onclick {
                    handler.call(e);
                }
                ctx.toggle();
            },
            attributes: merged,
            if let Some(icon) = icon {
                {icon}
            } else {
                PanelLeft {
                    class: "dx-sidebar-trigger-icon",
                    size: "1rem",
                }
            }
            span { class: "dx-sidebar-sr-only", "Toggle Sidebar" }
        }
    }
}

#[component]
pub fn SidebarRail(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    let ctx = use_sidebar();

    let base = attributes!(button {
        class: "dx-sidebar-rail",
        "data-sidebar": "rail",
        "data-slot": "sidebar-rail",
        aria_label: "Toggle Sidebar",
        title: "Toggle Sidebar",
    });
    // `tabindex: -1` is required behavior -- the rail is a pointer-only
    // affordance deliberately excluded from the tab order -- not a caller
    // default, so it is owned-wins.
    let owned = attributes!(button { tabindex: -1 });
    let merged = merge_attributes(vec![base, attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        button {
            onclick: move |_| ctx.toggle(),
            ..merged,
        }
    }
}

#[component]
pub fn SidebarInset(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(main {
        class: "dx-sidebar-inset",
        "data-slot": "sidebar-inset",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        main { ..merged, {children} }
    }
}

#[component]
pub fn SidebarHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-header",
        "data-slot": "sidebar-header",
        "data-sidebar": "header",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-content",
        "data-slot": "sidebar-content",
        "data-sidebar": "content",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-footer",
        "data-slot": "sidebar-footer",
        "data-sidebar": "footer",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarSeparator(
    #[props(default = true)] horizontal: bool,
    #[props(default = true)] decorative: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-separator",
        "data-slot": "sidebar-separator",
        "data-sidebar": "separator",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        Separator { horizontal, decorative, attributes: merged }
    }
}

#[component]
pub fn SidebarGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-group",
        "data-slot": "sidebar-group",
        "data-sidebar": "group",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarGroupLabel(
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-group-label",
        "data-slot": "sidebar-group-label",
        "data-sidebar": "group-label",
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
            div { ..merged,{children} }
        }
    }
}

#[component]
pub fn SidebarGroupAction(
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(button {
        class: "dx-sidebar-group-action",
        "data-slot": "sidebar-group-action",
        "data-sidebar": "group-action",
        // Same opt-out as `SidebarMenuButton` (see there).
        "data-style": "sidebar",
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
            button { ..merged,{children} }
        }
    }
}

#[component]
pub fn SidebarGroupContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-group-content",
        "data-slot": "sidebar-group-content",
        "data-sidebar": "group-content",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenu(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(ul {
        class: "dx-sidebar-menu",
        "data-slot": "sidebar-menu",
        "data-sidebar": "menu",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        ul { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenuItem(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(li {
        class: "dx-sidebar-menu-item",
        "data-slot": "sidebar-menu-item",
        "data-sidebar": "menu-item",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        li { ..merged, {children} }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(dead_code)]
pub enum SidebarMenuButtonVariant {
    #[default]
    Default,
    Outline,
}

impl SidebarMenuButtonVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarMenuButtonVariant::Default => "default",
            SidebarMenuButtonVariant::Outline => "outline",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(dead_code)]
pub enum SidebarMenuButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl SidebarMenuButtonSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarMenuButtonSize::Default => "default",
            SidebarMenuButtonSize::Sm => "sm",
            SidebarMenuButtonSize::Lg => "lg",
        }
    }
}

#[component]
pub fn SidebarMenuButton(
    #[props(default = false)] is_active: bool,
    #[props(default)] variant: SidebarMenuButtonVariant,
    #[props(default)] size: SidebarMenuButtonSize,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    #[props(default)] tooltip: Option<Element>,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    children: Element,
) -> Element {
    let ctx = use_sidebar();
    let is_mobile = ctx.is_mobile;
    let state = ctx.state;

    let base = attributes!(button {
        class: "dx-sidebar-menu-button",
        "data-slot": "sidebar-menu-button",
        "data-sidebar": "menu-button",
        "data-size": size.as_str(),
        "data-variant": variant.as_str(),
        "data-active": if is_active { "true" } else { "false" },
        // The ONE opt-out from another component's default trigger recipe (`:not([data-style])` in
        // dropdown_menu/ and collapsible/ style.css, the signal drawer/ and button/ use too): a sidebar
        // button is composed as `DropdownMenuTrigger as` / `CollapsibleTrigger as`, which render it
        // with that trigger's class beside `.dx-sidebar-*`, and the sidebar owns every property of its
        // own buttons (`:where()` recipes are not a contest it should have to win).
        "data-style": "sidebar",
    });
    let merged = merge_attributes(vec![base, attributes]);

    let Some(tooltip_content) = tooltip else {
        return if let Some(dynamic) = r#as {
            dynamic.call(merged)
        } else {
            rsx! {
                document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
                button { ..merged, {children} }
            }
        };
    };

    let hidden = state() != SidebarState::Collapsed || is_mobile();
    let sidebar_side = ctx.side;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        Tooltip {
            class: "dx-sidebar-tooltip",
            disabled: hidden,
            TooltipTrigger {
                as: move |tooltip_attrs: Vec<Attribute>| {
                    let final_attrs = merge_attributes(vec![tooltip_attrs, merged.clone()]);
                    let children = children.clone();
                    if let Some(dynamic) = &r#as {
                        dynamic.call(final_attrs)
                    } else {
                        rsx! { button { ..final_attrs, {children} } }
                    }
                },
            }
            TooltipContent {
                side: match sidebar_side() {
                    SidebarSide::Left => dioxus_primitives::ContentSide::Right,
                    SidebarSide::Right => dioxus_primitives::ContentSide::Left,
                },
                {tooltip_content}
            }
        }
    }
}

#[component]
pub fn SidebarMenuAction(
    #[props(default = false)] show_on_hover: bool,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(button {
        class: "dx-sidebar-menu-action",
        "data-slot": "sidebar-menu-action",
        "data-sidebar": "menu-action",
        "data-show-on-hover": if show_on_hover { "true" } else { "false" },
        // Same opt-out as `SidebarMenuButton` (see there).
        "data-style": "sidebar",
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
            button { ..merged,{children} }
        }
    }
}

#[component]
pub fn SidebarMenuBadge(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-menu-badge",
        "data-slot": "sidebar-menu-badge",
        "data-sidebar": "menu-badge",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenuSkeleton(
    #[props(default = false)] show_icon: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "dx-sidebar-menu-skeleton",
        "data-slot": "sidebar-menu-skeleton",
        "data-sidebar": "menu-skeleton",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        div {
            ..merged,
            if show_icon {
                Skeleton { class: "dx-sidebar-menu-skeleton-icon" }
            }
            Skeleton { class: "dx-sidebar-menu-skeleton-text", width: "70%" }
        }
    }
}

#[component]
pub fn SidebarMenuSub(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(ul {
        class: "dx-sidebar-menu-sub",
        "data-slot": "sidebar-menu-sub",
        "data-sidebar": "menu-sub",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        ul { ..merged, {children} }
    }
}

#[component]
pub fn SidebarMenuSubItem(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(li {
        class: "dx-sidebar-menu-sub-item",
        "data-slot": "sidebar-menu-sub-item",
        "data-sidebar": "menu-sub-item",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
        li { ..merged, {children} }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(dead_code)]
pub enum SidebarMenuSubButtonSize {
    Sm,
    #[default]
    Md,
}

impl SidebarMenuSubButtonSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            SidebarMenuSubButtonSize::Sm => "sm",
            SidebarMenuSubButtonSize::Md => "md",
        }
    }
}

#[component]
pub fn SidebarMenuSubButton(
    #[props(default = false)] is_active: bool,
    #[props(default)] size: SidebarMenuSubButtonSize,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(a {
        class: "dx-sidebar-menu-sub-button",
        "data-slot": "sidebar-menu-sub-button",
        "data-sidebar": "menu-sub-button",
        "data-size": size.as_str(),
        "data-active": if is_active { "true" } else { "false" },
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            document::Link { rel: "stylesheet", href: asset!("/src/components/sidebar/style.css") }
            a { ..merged, {children} }
        }
    }
}
