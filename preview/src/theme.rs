use crate::components::{
    button::{Button, ButtonSize, ButtonVariant},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHandle,
        DrawerTitle,
    },
    popover::{PopoverContent, PopoverRoot, PopoverTrigger},
    sidebar::{SidebarCtx, SidebarFooter, SidebarMenu, SidebarMenuButton, SidebarMenuItem},
    switch::Switch,
    toggle::ToggleVariant,
    toggle_group::{ToggleGroup, ToggleItem},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Moon, Palette, Sun};
use dioxus_primitives::ContentAlign;
use std::collections::HashSet;

const CHANNEL_NAME: &str = "dx-theme";

/// The site's persisted display preferences. Each is one `<html data-<key>>`
/// attribute AND one cookie (`dx_<key with - as _>`), so adding one here, to
/// [`PREPAINT_JS`]'s key list and to `theme-presets.css` is the whole job.
///
/// * `Mode` is `data-theme="dark|light"` PLUS shadcn's `class="dark"` on `<html>`, written together
///   by [`PREPAINT_JS`]'s one `apply_mode` and switched by `dx-components-theme.css` (absent
///   `data-theme` = system: `.dark` then follows `prefers-color-scheme`).
/// * `Base` / `Accent` / `Radius` are `data-theme-base` / `-accent` / `-radius`,
///   matched by the blocks in `preview/assets/theme-presets.css`
///   (absent = no preset = the library's own look).
/// * `PressFeedback` is `data-dx-press-feedback="off"`, read by the theme's "Press feedback" rule
///   in `dx-components-theme.css` (absent = on, the only other state; `off` is the only value the
///   snippet accepts for it).
/// * `Coarse` is `data-dx-coarse="on|off"`, read by the theme's coarse-sizing block in
///   `dx-components-theme.css` (absent = `auto`, which follows `(pointer: coarse)`; `on` and `off` are
///   the only other values the snippet accepts, and force the touch or the desktop sizes page-wide).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pref {
    Mode,
    Base,
    Accent,
    Radius,
    PressFeedback,
    Coarse,
}

impl Pref {
    const fn key(self) -> &'static str {
        match self {
            Pref::Mode => "theme",
            Pref::Base => "theme-base",
            Pref::Accent => "theme-accent",
            Pref::Radius => "theme-radius",
            Pref::PressFeedback => "dx-press-feedback",
            Pref::Coarse => "dx-coarse",
        }
    }
}

/// shadcn's base colours, in picker order: `(attribute value, label)`. Every slug must have a
/// `[data-theme-base="<slug>"]` block in `theme-presets.css` (and vice versa; see the tests below).
pub const BASES: &[(&str, &str)] = &[
    ("neutral", "Neutral"),
    ("stone", "Stone"),
    ("zinc", "Zinc"),
    ("gray", "Gray"),
    ("slate", "Slate"),
    ("mauve", "Mauve"),
    ("olive", "Olive"),
    ("mist", "Mist"),
    ("taupe", "Taupe"),
];

/// shadcn's accent themes: `[data-theme-accent="<slug>"]` blocks, plus `docs`: the docs site's own link blue,
/// not shadcn data, so the colour the site shows by default (the `Default` swatch has no colour of its own) can
/// be picked as an accent (backlog row 301).
pub const ACCENTS: &[(&str, &str)] = &[
    ("docs", "Docs blue"),
    ("amber", "Amber"),
    ("blue", "Blue"),
    ("cyan", "Cyan"),
    ("emerald", "Emerald"),
    ("fuchsia", "Fuchsia"),
    ("green", "Green"),
    ("indigo", "Indigo"),
    ("lime", "Lime"),
    ("orange", "Orange"),
    ("pink", "Pink"),
    ("purple", "Purple"),
    ("red", "Red"),
    ("rose", "Rose"),
    ("sky", "Sky"),
    ("teal", "Teal"),
    ("violet", "Violet"),
    ("yellow", "Yellow"),
];

/// Radius choices in rem. [`DEFAULT_RADIUS`] is the library default and is stored as "no attribute",
/// so there is exactly one way to be on it.
pub const RADII: &[&str] = &["0", "0.3", "0.5", "0.625", "0.75", "1"];
const DEFAULT_RADIUS: &str = "0.625";

/// Applies every preference from its cookie to `<html>`, and defines `window.__dxTheme(key, value)`,
/// the one place a value is validated and written to / removed from an attribute (the live
/// cross-tab sync and [`set_pref`] reuse it).
///
/// The mode (backlog row 158 (a)) goes through `apply_mode` (`window.__dxApplyMode`), the ONLY code
/// that may write `data-theme` or the `dark` class: it sets `data-theme` and toggles `.dark` together,
/// and resolves "system" (no value) from `prefers-color-scheme`, re-resolving on a page-lifetime
/// change listener while no explicit mode is set. So a shadcn theme's `.dark { ... }` block, and
/// Tailwind's `dark:`, follow the very same switch as `data-theme`.
///
/// This text is embedded VERBATIM in `preview/index.html`'s `<head>` (a unit test below enforces it),
/// which is what makes it run before first paint: the SSG HTML is static, the server cannot read a
/// cookie, and everything in `AppLayout`'s effect only runs after hydration. With the attributes
/// already on `<html>` and `theme-presets.css` / `dx-components-theme.css` render-blocking in
/// `<head>`, the first paint is already in the chosen mode, colours and radius. [`theme_seed`] runs it
/// again for any host that has no such `index.html`.
pub const PREPAINT_JS: &str = "(function(){var K=['theme','theme-base','theme-accent','theme-radius','dx-press-feedback','dx-coarse'],R=document.documentElement,Q=window.matchMedia&&matchMedia('(prefers-color-scheme: dark)');function M(m){if(m)R.setAttribute('data-theme',m);else R.removeAttribute('data-theme');R.classList.toggle('dark',m?m==='dark':!!(Q&&Q.matches))}function A(k,v){if(K.indexOf(k)<0)return;var ok=!!v&&/^[a-z0-9.]{1,16}$/.test(v)&&(k!=='theme'||v==='dark'||v==='light')&&(k!=='dx-press-feedback'||v==='off')&&(k!=='dx-coarse'||v==='on'||v==='off');if(k==='theme')M(ok?v:null);else if(ok)R.setAttribute('data-'+k,v);else R.removeAttribute('data-'+k)}window.__dxApplyMode=M;window.__dxTheme=A;if(Q&&!window.__dxModeWatch){window.__dxModeWatch=1;try{/*listener-ok: lives as long as the page*/Q.addEventListener('change',function(){if(!R.hasAttribute('data-theme'))M(null)})}catch(e){}}try{var C=document.cookie.split(';');K.forEach(function(k){var n='dx_'+k.replace('-','_')+'=',v=null;C.forEach(function(c){c=c.trim();if(c.indexOf(n)===0)v=decodeURIComponent(c.slice(n.length))});A(k,v)})}catch(e){}})();(function(){var W=window,R=document.documentElement;if(W.__dxModality)return;W.__dxModality=1;function S(m){if(R.getAttribute('data-dx-modality')!==m)R.setAttribute('data-dx-modality',m)}var Q=W.matchMedia&&W.matchMedia('(any-pointer: coarse) and (hover: none)');if(!R.hasAttribute('data-dx-modality'))S(Q&&Q.matches?'touch':'mouse');var K={Shift:1,Control:1,Alt:1,Meta:1,AltGraph:1,CapsLock:1,Fn:1,FnLock:1,Hyper:1,Super:1,OS:1};function P(t){return t==='touch'||t==='pen'?t:'mouse'}try{/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('pointerdown',function(e){S(P(e.pointerType))},true);/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('keydown',function(e){if(!K[e.key])S('keyboard')},true);/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('click',function(e){if(e.detail===0){S('keyboard')}else if(R.getAttribute('data-dx-modality')==='keyboard')S(P(e.pointerType||(Q&&Q.matches?'touch':'mouse')))},true)}catch(x){}})()";

pub fn theme_seed() {
    _ = document::eval(&format!(
        r#"
        (function () {{
          if (window.__dx_theme_seeded) return;
          window.__dx_theme_seeded = true;

          {PREPAINT_JS};

          try {{
            const ch = new BroadcastChannel('{CHANNEL_NAME}');
            // listener-ok: page-lifetime -- one BroadcastChannel per page, seeded once (__dx_theme_seeded)
            ch.addEventListener('message', (event) => {{
              const data = event.data;
              if (data && typeof data.key === 'string') window.__dxTheme(data.key, data.value);
            }});
            window.__dx_theme_channel = ch;
          }} catch (_) {{}}
        }})();
        "#,
    ));
}

/// Sets (or, with `None`, clears) one preference: attribute now, cookie for the next page load,
/// and a broadcast so other tabs and the demo iframes follow live. `value` is only ever one of the
/// slugs above; anything else is dropped rather than interpolated into script text.
pub fn set_pref(pref: Pref, value: Option<&str>) {
    let key = pref.key();
    let js_value = match value {
        Some(v) if !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') => {
            format!("'{v}'")
        }
        _ => "null".to_string(),
    };

    _ = document::eval(&format!(
        r#"
        (function () {{
          const key = '{key}';
          const value = {js_value};
          const cookieName = 'dx_' + key.replace('-', '_');

          function getCookie(name) {{
            const prefix = name + '=';
            for (let p of document.cookie.split(';')) {{
              p = p.trim();
              if (p.startsWith(prefix)) return decodeURIComponent(p.slice(prefix.length));
            }}
            return '';
          }}

          if (window.__dxTheme) window.__dxTheme(key, value);
          if (getCookie(cookieName) === (value || '')) return;

          document.cookie = value
            ? cookieName + '=' + encodeURIComponent(value) + '; path=/; max-age=31536000; samesite=lax'
            : cookieName + '=; path=/; max-age=0; samesite=lax';

          try {{
            const ch = window.__dx_theme_channel;
            if (ch && typeof ch.postMessage === 'function') {{
              ch.postMessage({{ key, value }});
            }} else {{
              const tmp = new BroadcastChannel('{CHANNEL_NAME}');
              tmp.postMessage({{ key, value }});
              tmp.close();
            }}
          }} catch (_) {{}}
        }})();
        "#
    ));
}

pub fn set_theme(dark_mode: bool) {
    set_pref(Pref::Mode, Some(if dark_mode { "dark" } else { "light" }));
}

#[component]
pub fn DarkModeToggle() -> Element {
    // Two buttons, one per mode, swapped by CSS (`.dx-dark-mode-only` / `.dx-light-mode-only`) so the
    // SSG output is right before hydration; the click only asks `set_pref`, never writes `data-theme`.
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Icon,
            class: "dx-dark-mode-toggle dx-dark-mode-only",
            onclick: move |_| set_theme(false),
            r#type: "button",
            aria_label: "Enable light mode",
            DarkModeIcon {}
        }
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Icon,
            class: "dx-dark-mode-toggle dx-light-mode-only",
            onclick: move |_| set_theme(true),
            r#type: "button",
            aria_label: "Enable dark mode",
            LightModeIcon {}
        }
    }
}

#[component]
fn DarkModeIcon() -> Element {
    rsx! {
        Moon {}
    }
}

#[component]
fn LightModeIcon() -> Element {
    rsx! {
        Sun {}
    }
}

/// The picker's four settings, mirrored from the `<html data-theme-*>` attributes for `aria-pressed` and
/// the selected ring. The attributes stay the single source of truth (so presets need no JS to apply and
/// the first paint is already right, see [`PREPAINT_JS`]); these signals are only ever read back from
/// them, on mount and whenever a panel opens ([`ThemeState::sync`]).
///
/// One per trigger: the header's [`ThemePicker`] and the phone menu's [`ThemePickerSidebarFooter`] each
/// own one and hand it to the shared [`ThemePickerPanel`], so the panel itself never has to fetch its
/// own state after it mounts (which would paint one frame of "Default" selected before the read-back).
#[derive(Clone, Copy, PartialEq)]
struct ThemeState {
    base: Signal<String>,
    accent: Signal<String>,
    radius: Signal<String>,
    /// `"off"` or empty (on).
    press: Signal<String>,
    /// `"on"`, `"off"` or empty (auto).
    coarse: Signal<String>,
}

impl ThemeState {
    /// Re-reads the three attributes: another tab, the cookie or the pre-paint script may have changed
    /// them since this component last looked.
    fn sync(self) {
        let (mut base, mut accent, mut radius, mut press, mut coarse) =
            (self.base, self.accent, self.radius, self.press, self.coarse);
        spawn(async move {
            let mut eval = document::eval(
                r#"const r = document.documentElement;
                dioxus.send([r.getAttribute('data-theme-base') || '', r.getAttribute('data-theme-accent') || '', r.getAttribute('data-theme-radius') || '', r.getAttribute('data-dx-press-feedback') || '', r.getAttribute('data-dx-coarse') || ''].join('|'));"#,
            );
            if let Ok(state) = eval.recv::<String>().await {
                let mut parts = state.split('|');
                base.set(parts.next().unwrap_or_default().to_string());
                accent.set(parts.next().unwrap_or_default().to_string());
                radius.set(parts.next().unwrap_or_default().to_string());
                press.set(parts.next().unwrap_or_default().to_string());
                coarse.set(parts.next().unwrap_or_default().to_string());
            }
        });
    }

    fn reset(self) {
        let (mut base, mut accent, mut radius, mut press, mut coarse) =
            (self.base, self.accent, self.radius, self.press, self.coarse);
        base.set(String::new());
        accent.set(String::new());
        radius.set(String::new());
        press.set(String::new());
        coarse.set(String::new());
        set_pref(Pref::Base, None);
        set_pref(Pref::Accent, None);
        set_pref(Pref::Radius, None);
        set_pref(Pref::PressFeedback, None);
        set_pref(Pref::Coarse, None);
    }
}

fn use_theme_state() -> ThemeState {
    let state = ThemeState {
        base: use_signal(String::new),
        accent: use_signal(String::new),
        radius: use_signal(String::new),
        press: use_signal(String::new),
        coarse: use_signal(String::new),
    };
    use_effect(move || state.sync());
    state
}

/// shadcn-style theme customizer: colour swatches (base colour, accent) and a radius row.
///
/// Why swatches rather than our `Select`: the choice is two independent colour axes plus a radius, and
/// a colour is picked by looking at it. A listbox shows one option at a time behind a click, three of
/// them would crowd the header, and shadcn's own customizer is swatches too. The panel opens from one
/// icon button and every option is visible at once.
///
/// The swatch dots take their colours from the very same `[data-theme-*]` blocks the presets use
/// (`main.css`, `.dx-theme-swatch`), so no colour is written down twice.
///
/// # One panel, two presentations, one entry per page
///
/// [`ThemePickerPanel`] is the content and exists once. Where it opens is a layout question, so it is
/// answered by CSS rather than by measuring the window in Rust (which would render different markup
/// before and after hydration, and flash): this component, the header trigger that opens it as a
/// popover, is always rendered and `main.css` hides it at phone width (`<= 760px`, the header's own
/// breakpoint). At that width the picker opens as a bottom [`Drawer`] from a phone entry, and which
/// entry is a structural question this component answers, because it is the one thing every route's
/// header renders:
///
/// * a route with a navigation sheet (`DocsLayout`: Home, Docs, component pages) already hosts
///   [`ThemePickerSidebarFooter`] in it, always rendered and shown by `main.css` only at that width;
/// * a route without one (`/demos`, the `/charts/` gallery, "not found") has no sheet to put it in, so
///   the header itself renders [`ThemePickerPhoneTrigger`] next to the popover's trigger, shown by
///   `main.css` only at that width. Whether a sheet exists is the presence of the [`SidebarCtx`]
///   context, identical on the server and in the browser, so it cannot flash either.
///
/// So a header always has exactly one phone entry and a layout cannot lose it by being built without a
/// sidebar (backlog row 142), and exactly one trigger is ever visible, so exactly one panel is mounted.
#[component]
pub fn ThemePicker() -> Element {
    let state = use_theme_state();
    let has_navigation_sheet = try_consume_context::<SidebarCtx>().is_some();

    rsx! {
        PopoverRoot {
            is_modal: false,
            on_open_change: move |open| {
                if open {
                    state.sync();
                }
            },
            PopoverTrigger { class: "dx-theme-picker-trigger", aria_label: "Theme", title: "Theme",
                Palette { size: "24px" }
            }
            PopoverContent { class: "dx-theme-picker", align: ContentAlign::End,
                ThemePickerPanel {
                    state,
                    scope: "popover",
                    title: rsx! {
                        span { class: "dx-theme-picker-title", "Theme" }
                    },
                }
            }
        }
        if !has_navigation_sheet {
            ThemePickerPhoneTrigger {}
        }
    }
}

/// The picker's phone entry for a page with no navigation sheet to host [`ThemePickerSidebarFooter`]:
/// an icon button in the header, where the popover's own trigger sits above the breakpoint, that opens
/// [`ThemePickerPanel`] as the same bottom [`Drawer`]. Rendered only by [`ThemePicker`] and only when
/// there is no [`SidebarCtx`]; `main.css` shows it at phone width and nowhere else. Where a navigation
/// sheet exists the header already spends that room on its sidebar trigger, which is why the sheet keeps
/// its own entry rather than both moving here.
#[component]
fn ThemePickerPhoneTrigger() -> Element {
    let state = use_theme_state();
    let mut open = use_signal(|| false);

    rsx! {
        button {
            class: "dx-theme-phone-trigger",
            r#type: "button",
            aria_label: "Theme",
            title: "Theme",
            aria_haspopup: "dialog",
            onclick: move |_| open.set(true),
            Palette { size: "24px" }
        }
        ThemePickerDrawer { state, open }
    }
}

/// The picker on phones, on a page with a navigation sheet: a "Theme" entry at the foot of the mobile
/// navigation sheet that opens [`ThemePickerPanel`] as a bottom [`Drawer`]. (A page without a sheet gets
/// [`ThemePickerPhoneTrigger`] in its header instead; see [`ThemePicker`] for which and why.)
///
/// The header has no room for the popover's trigger at phone width (it overlapped "Charts" and the
/// GitHub link) and the popover itself would open from a point near the middle of the screen, so on
/// phones the picker lives where the rest of the navigation does and opens as a bottom sheet, the
/// native phone idiom. Rendered inside the desktop sidebar too (the sidebar only becomes a sheet below
/// 768px, in Rust); `.dx-theme-sidebar-footer` is `display: none` above the 760px breakpoint, which is
/// what keeps it out of the accessibility tree there.
///
/// The drawer is a child of the sidebar sheet, so it is a modal opened on top of a modal. That is the
/// shape the dialog primitive supports (it ships a nested-dialog demo): the browser stacks the two
/// `<dialog>`s in the top layer, so Escape and the focus trap act on the topmost one only, the scroll
/// lock is one refcounted lock shared along the component chain, and closing the drawer returns focus
/// to this trigger inside the still-open sheet.
#[component]
pub fn ThemePickerSidebarFooter() -> Element {
    let state = use_theme_state();
    let mut open = use_signal(|| false);

    rsx! {
        SidebarFooter { class: "dx-theme-sidebar-footer",
            SidebarMenu {
                SidebarMenuItem {
                    SidebarMenuButton {
                        as: move |attributes: Vec<Attribute>| rsx! {
                            button {
                                r#type: "button",
                                aria_haspopup: "dialog",
                                onclick: move |_| open.set(true),
                                ..attributes,
                                Palette { size: "1rem", "aria-hidden": "true" }
                                span { "Theme" }
                            }
                        },
                    }
                }
            }
            ThemePickerDrawer { state, open }
        }
    }
}

/// The phone presentation: [`ThemePickerPanel`] in a bottom [`Drawer`], opened and closed through the
/// `open` signal its trigger owns ([`ThemePickerSidebarFooter`] or [`ThemePickerPhoneTrigger`]). It is
/// rendered next to that trigger, so closing it returns focus to the trigger that opened it.
#[component]
fn ThemePickerDrawer(state: ThemeState, open: Signal<bool>) -> Element {
    let mut open = open;

    rsx! {
        Drawer {
            open: open(),
            on_open_change: move |next| {
                open.set(next);
                if next {
                    state.sync();
                }
            },
            DrawerContent {
                DrawerHandle {}
                ThemePickerPanel {
                    state,
                    scope: "drawer",
                    title: rsx! {
                        DrawerTitle { "Theme" }
                    },
                    DrawerDescription { "Colours and corner radius for this site." }
                }
                DrawerFooter {
                    DrawerClose {
                        as: |attributes| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes, "Done" }
                        },
                    }
                }
            }
        }
    }
}

/// The picker's content: reset, base colour, accent, radius, press feedback, touch sizing. Rendered inside either the header's
/// `Popover` (`scope: "popover"`) or the phone `Drawer` (`scope: "drawer"`); `scope` is also
/// the `data-presentation` CSS hooks onto (larger touch targets in the drawer) and keeps the label ids
/// unique per presentation.
///
/// `title` is the heading in the head row (a `DrawerTitle` names the drawer's dialog, a plain span
/// does in the popover) and `children` render right under that row (the drawer's description), the two
/// kept together as one intro block.
#[component]
fn ThemePickerPanel(
    state: ThemeState,
    scope: &'static str,
    title: Element,
    children: Element,
) -> Element {
    let ThemeState {
        mut base,
        mut accent,
        mut radius,
        mut press,
        mut coarse,
    } = state;

    let base_label = label_of(BASES, &base.read());
    let accent_label = label_of(ACCENTS, &accent.read());

    rsx! {
        div { class: "dx-theme-picker-panel", "data-presentation": scope,
            div { class: "dx-theme-picker-intro",
                div { class: "dx-theme-picker-head",
                    {title}
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Xs,
                        r#type: "button",
                        onclick: move |_| state.reset(),
                        "Reset"
                    }
                }
                {children}
            }

            div { class: "dx-theme-picker-section",
                span { class: "dx-theme-picker-label", id: "dx-theme-base-label-{scope}",
                    "Base colour"
                    span { class: "dx-theme-picker-current", "{base_label}" }
                }
                ToggleGroup {
                    class: "dx-theme-swatches",
                    aria_labelledby: "dx-theme-base-label-{scope}",
                    pressed: Some(single(swatch_index(&base.read(), BASES))),
                    on_pressed_change: move |next: HashSet<usize>| {
                        // Index 0 is "Default"; the presets follow in list order.
                        match next.into_iter().next().and_then(|i| i.checked_sub(1)) {
                            None => {
                                base.set(String::new());
                                set_pref(Pref::Base, None);
                            }
                            Some(i) => {
                                let slug = BASES[i].0;
                                base.set(slug.to_string());
                                set_pref(Pref::Base, Some(slug));
                            }
                        }
                    },
                    ToggleItem {
                        index: 0usize,
                        class: "dx-theme-swatch",
                        aria_label: "Default base colour",
                        title: "Default",
                        span { class: "dx-theme-swatch-dot", "data-none": "true" }
                    }
                    for (i , (slug , label)) in BASES.iter().copied().enumerate() {
                        ToggleItem {
                            key: "{slug}",
                            index: i + 1,
                            class: "dx-theme-swatch",
                            aria_label: "{label}",
                            title: "{label}",
                            "data-theme-base": slug,
                            span { class: "dx-theme-swatch-dot" }
                        }
                    }
                }
            }

            div { class: "dx-theme-picker-section",
                span { class: "dx-theme-picker-label", id: "dx-theme-accent-label-{scope}",
                    "Accent"
                    span { class: "dx-theme-picker-current", "{accent_label}" }
                }
                ToggleGroup {
                    class: "dx-theme-swatches",
                    aria_labelledby: "dx-theme-accent-label-{scope}",
                    pressed: Some(single(swatch_index(&accent.read(), ACCENTS))),
                    on_pressed_change: move |next: HashSet<usize>| {
                        // Index 0 is "Default"; the presets follow in list order.
                        match next.into_iter().next().and_then(|i| i.checked_sub(1)) {
                            None => {
                                accent.set(String::new());
                                set_pref(Pref::Accent, None);
                            }
                            Some(i) => {
                                let slug = ACCENTS[i].0;
                                accent.set(slug.to_string());
                                set_pref(Pref::Accent, Some(slug));
                            }
                        }
                    },
                    ToggleItem {
                        index: 0usize,
                        class: "dx-theme-swatch",
                        aria_label: "Default accent",
                        title: "Default",
                        span { class: "dx-theme-swatch-dot", "data-none": "true" }
                    }
                    for (i , (slug , label)) in ACCENTS.iter().copied().enumerate() {
                        ToggleItem {
                            key: "{slug}",
                            index: i + 1,
                            class: "dx-theme-swatch",
                            aria_label: "{label}",
                            title: "{label}",
                            "data-theme-accent": slug,
                            span { class: "dx-theme-swatch-dot" }
                        }
                    }
                }
            }

            div { class: "dx-theme-picker-section",
                span { class: "dx-theme-picker-label", id: "dx-theme-radius-label-{scope}",
                    "Radius"
                    span { class: "dx-theme-picker-current", "rem" }
                }
                ToggleGroup {
                    class: "dx-theme-chips",
                    variant: ToggleVariant::Outline,
                    aria_labelledby: "dx-theme-radius-label-{scope}",
                    pressed: Some(single(chip_index(&radius.read()))),
                    on_pressed_change: move |next: HashSet<usize>| {
                        if let Some(value) = next.into_iter().next().and_then(|i| RADII.get(i).copied()) {
                            if value == DEFAULT_RADIUS {
                                radius.set(String::new());
                                set_pref(Pref::Radius, None);
                            } else {
                                radius.set(value.to_string());
                                set_pref(Pref::Radius, Some(value));
                            }
                        }
                    },
                    for (i , value) in RADII.iter().copied().enumerate() {
                        ToggleItem {
                            key: "{value}",
                            index: i,
                            title: if value == DEFAULT_RADIUS { "0.625 (default)" } else { value },
                            "{value}"
                        }
                    }
                }
            }

            div { class: "dx-theme-picker-section",
                div { class: "dx-theme-picker-toggle",
                    span {
                        class: "dx-theme-picker-label",
                        id: "dx-theme-press-label-{scope}",
                        "Press feedback"
                        span { class: "dx-theme-picker-current", "scale and wash while pressed" }
                    }
                    Switch {
                        checked: press.read().is_empty(),
                        aria_labelledby: "dx-theme-press-label-{scope}",
                        on_checked_change: move |on: bool| {
                            if on {
                                press.set(String::new());
                                set_pref(Pref::PressFeedback, None);
                            } else {
                                press.set("off".to_string());
                                set_pref(Pref::PressFeedback, Some("off"));
                            }
                        },
                    }
                }
            }

            div { class: "dx-theme-picker-section",
                span { class: "dx-theme-picker-label", id: "dx-theme-coarse-label-{scope}",
                    "Touch sizing"
                    span { class: "dx-theme-picker-current", "auto follows the device" }
                }
                ToggleGroup {
                    class: "dx-theme-chips",
                    variant: ToggleVariant::Outline,
                    aria_labelledby: "dx-theme-coarse-label-{scope}",
                    pressed: Some(single(coarse_index(&coarse.read()))),
                    on_pressed_change: move |next: HashSet<usize>| {
                        if let Some((value, _)) = next.into_iter().next().and_then(|i| COARSE_MODES.get(i).copied()) {
                            coarse.set(value.to_string());
                            set_pref(Pref::Coarse, if value.is_empty() { None } else { Some(value) });
                        }
                    },
                    for (i , (value , label)) in COARSE_MODES.iter().copied().enumerate() {
                        ToggleItem {
                            key: "coarse-{label}",
                            index: i,
                            title: match value {
                                "on" => "Touch sizes everywhere",
                                "off" => "Desktop sizes everywhere",
                                _ => "Follow the device's pointer",
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}

/// Touch sizing choices, `(attribute value, label)`: "" is `auto` and stored as "no attribute".
pub const COARSE_MODES: &[(&str, &str)] = &[("", "Auto"), ("on", "On"), ("off", "Off")];

/// Item index of the touch-sizing chips.
fn coarse_index(current: &str) -> usize {
    COARSE_MODES
        .iter()
        .position(|(v, _)| *v == current)
        .unwrap_or(0)
}

/// The one pressed item of a single-select [`ToggleGroup`]. The groups are controlled: `on_pressed_change`
/// writes the preference and the pressed set is read back from it, so pressing the pressed item again
/// (an empty next set) changes nothing, a radio rather than a toggle.
fn single(index: usize) -> HashSet<usize> {
    HashSet::from([index])
}

/// Item index of a swatch group: 0 is "Default" (no preset), the presets follow in list order.
fn swatch_index(current: &str, list: &[(&str, &str)]) -> usize {
    list.iter()
        .position(|(slug, _)| *slug == current)
        .map_or(0, |i| i + 1)
}

/// Item index of the radius chips: the default radius is stored as "no attribute" (empty).
fn chip_index(current: &str) -> usize {
    RADII
        .iter()
        .position(|r| {
            if *r == DEFAULT_RADIUS {
                current.is_empty()
            } else {
                *r == current
            }
        })
        .unwrap_or(0)
}

fn label_of(list: &[(&'static str, &'static str)], slug: &str) -> &'static str {
    list.iter()
        .find(|(s, _)| *s == slug)
        .map_or("Default", |(_, label)| label)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX_HTML: &str = include_str!("../index.html");
    const PRESETS_CSS: &str = include_str!("../assets/theme-presets.css");
    const MAIN_CSS: &str = include_str!("../assets/main.css");

    /// The pre-paint snippet exists twice (a Rust const the runtime evals, and the `<script>` in
    /// `index.html` that actually runs before first paint). They are the same text or the page flashes.
    #[test]
    fn index_html_embeds_the_prepaint_snippet_verbatim() {
        assert!(
            INDEX_HTML.contains(&format!("<script>{PREPAINT_JS}</script>")),
            "preview/index.html must contain `<script>` + theme::PREPAINT_JS + `</script>` exactly"
        );
        assert!(!PREPAINT_JS.contains("</script"));
    }

    /// The interaction-modality writer (`dioxus_primitives::modality::MODALITY_JS`, the one writer of
    /// `<html data-dx-modality>`) is part of the pre-paint snippet, so the attribute is right from the
    /// first frame and the open-focus policy and the ring tokens never see it missing.
    #[test]
    fn prepaint_snippet_embeds_the_modality_writer() {
        assert!(
            PREPAINT_JS.contains(dioxus_primitives::modality::MODALITY_JS),
            "PREPAINT_JS must contain dioxus_primitives::modality::MODALITY_JS verbatim"
        );
    }

    /// Every cookie key the snippet reads is a `Pref`, and vice versa.
    #[test]
    fn prepaint_snippet_handles_every_pref() {
        for pref in [
            Pref::Mode,
            Pref::Base,
            Pref::Accent,
            Pref::Radius,
            Pref::PressFeedback,
            Pref::Coarse,
        ] {
            assert!(
                PREPAINT_JS.contains(&format!("'{}'", pref.key())),
                "PREPAINT_JS does not list {}",
                pref.key()
            );
        }
    }

    fn blocks(attr: &str) -> Vec<String> {
        let needle = format!("[data-{attr}=\"");
        let mut out: Vec<String> = PRESETS_CSS
            .lines()
            .filter(|l| l.starts_with(&needle) && l.trim_end().ends_with('{'))
            .map(|l| {
                let rest = &l[needle.len()..];
                rest[..rest.find('"').unwrap()].to_string()
            })
            .collect();
        out.sort();
        out
    }

    fn sorted(list: &[&str]) -> Vec<String> {
        let mut v: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        v.sort();
        v
    }

    /// The picker's lists and the generated stylesheet cannot drift apart: a swatch with no block
    /// would silently do nothing, a block with no swatch would be unreachable.
    #[test]
    fn picker_lists_match_the_preset_stylesheet() {
        let bases: Vec<&str> = BASES.iter().map(|(s, _)| *s).collect();
        let accents: Vec<&str> = ACCENTS.iter().map(|(s, _)| *s).collect();
        assert_eq!(blocks("theme-base"), sorted(&bases));
        assert_eq!(blocks("theme-accent"), sorted(&accents));
        let non_default: Vec<&str> = RADII
            .iter()
            .copied()
            .filter(|r| *r != DEFAULT_RADIUS)
            .collect();
        let mut radius_blocks = blocks("theme-radius");
        radius_blocks.retain(|r| r != DEFAULT_RADIUS);
        assert_eq!(radius_blocks, sorted(&non_default));
    }

    /// Accent blocks override base blocks through the cascade, so they must come after every base block.
    #[test]
    fn accents_follow_bases_in_the_stylesheet() {
        let last_base = PRESETS_CSS.rfind("[data-theme-base=\"").unwrap();
        let first_accent = PRESETS_CSS.find("[data-theme-accent=\"").unwrap();
        assert!(last_base < first_accent);
    }

    /// The popover's trigger is hidden, the header's phone entry shown and the sidebar sheet's footer shown
    /// under ONE query in `main.css`. If they ever sat under different ones there would be a width with no
    /// "Theme" button, or with two.
    #[test]
    fn the_phone_entries_switch_under_one_breakpoint() {
        const QUERY: &str = "@media (width <= 760px) {";
        let popover = MAIN_CSS
            .find(".dx-popover:has(> .dx-theme-picker-trigger) {")
            .expect("main.css hides the popover's trigger on phones");
        let query = MAIN_CSS[..popover].rfind("@media").unwrap();
        assert!(
            MAIN_CSS[query..].starts_with(QUERY),
            "main.css hides the popover trigger under a different query than `{QUERY}`"
        );
        // The block is the query's own braces: it ends at the first line that is just `}`.
        let block = &MAIN_CSS[query..query + MAIN_CSS[query..].find("\n}\n").unwrap()];
        for (what, rule) in [
            (
                "shows the header's phone entry",
                "button.dx-theme-phone-trigger {\n    display: flex;",
            ),
            (
                "shows the sidebar sheet's footer",
                ".dx-sidebar-footer.dx-theme-sidebar-footer {\n    display: flex;",
            ),
        ] {
            assert!(
                block.contains(rule),
                "the `{QUERY}` block of main.css must also {what}"
            );
        }
    }
}
