//! SSR markup vs the markup of the first client frame, natively (backlog row 329 spike).
//!
//! A hydrating page goes through two frames the visitor sees: the server's markup (painted before the
//! wasm loads) and, after the wasm has hydrated, the markup once `use_effect`, `use_hook` writes and the
//! first signal writes have run. Anything that differs between the two is a repaint, and often a
//! re-layout (rows 204, 215, 252, 306). A native `VirtualDom` runs the same component code, so both
//! frames can be rendered inside `cargo test`:
//!
//! - **server frame**: `rebuild_in_place` and nothing else, which is what `dioxus-server` does before it
//!   serialises (`dioxus-server-0.7.9/src/ssr.rs:211`; effects and non-suspense tasks do not run there,
//!   `dioxus-core-0.7.9/src/virtual_dom.rs:640`).
//! - **client frame**: the same `rebuild_in_place` (the hydration render), then `render_immediate` until a
//!   round writes no mutation. `render_immediate` calls `process_events`, which polls the tasks and runs
//!   every queued effect once no scope is dirty (`virtual_dom.rs:518-535`), then re-renders the scopes
//!   those effects dirtied, i.e. what `dioxus-web` does on its first animation frame after hydrating.
//!
//! Both are rendered with `dioxus_ssr::render` (no hydration markers) and compared after renaming the
//! generated `dxc-N` / `dxc-cN` ids by first appearance, because the real client reuses the server's ids
//! (`use_server_cached`) while a native run allocates from the one process counter.
//!
//! What this can see is what the component's own Rust does between the two frames. What it cannot see is
//! anything that only the browser produces: layout, measurement, `document::eval` replies, web-sys
//! events, the interpreter's DOM writes. See `dev-docs/archive/research/row-329-ssr-first-render-parity-spike.md`.

use crate::components::DEMOS;
use crate::ComponentType;
use dioxus::prelude::*;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Rounds of `render_immediate` allowed before a demo counts as never settling.
const MAX_ROUNDS: usize = 300;

/// The two frames of one component.
pub(crate) struct Frames {
    /// The markup the server sends.
    pub server: String,
    /// The markup after the first client tick, effects included.
    pub client: String,
    /// Rounds of `render_immediate` that wrote mutations before the dom went quiet.
    pub rounds: usize,
}

/// The root every demo sits under on the site (the i18n demos bring their own, `crate::i18n::LocaleDemo`).
#[derive(Props, Clone, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
struct RootProps {
    /// The demo under test.
    demo: fn() -> Element,
}

#[component]
fn Root(props: RootProps) -> Element {
    let Demo = props.demo;
    rsx! { Demo {} }
}

/// Runs the dom the way the browser's first animation frames do: `render_immediate` again and again
/// (each one runs the effects that are due, then re-renders what they dirtied) until three rounds in a
/// row write nothing -- one quiet round is not enough, because a round that only re-runs a dirty scope
/// can write no mutation while the effects queued behind it have not run yet. Returns the number of
/// rounds that wrote mutations, at most [`MAX_ROUNDS`].
pub(crate) fn settle(dom: &mut VirtualDom) -> usize {
    let (mut active, mut quiet) = (0, 0);
    while active < MAX_ROUNDS && quiet < 3 {
        if dom.render_immediate_to_vec().edits.is_empty() {
            quiet += 1;
        } else {
            quiet = 0;
            active += 1;
        }
    }
    active
}

/// The root of a whole page: the memory history the server gives each request
/// (`dioxus-server-0.7.9/src/ssr.rs:177`) and the site's `Route` router.
#[derive(Props, Clone, PartialEq)]
struct RouteRootProps {
    /// The URL to render, as `Route::to_string` writes it.
    path: String,
}

#[component]
fn RouteRoot(props: RouteRootProps) -> Element {
    use dioxus::history::{History, MemoryHistory};
    let path = props.path.clone();
    use_hook(|| {
        provide_context(
            std::rc::Rc::new(MemoryHistory::with_initial_path(path)) as std::rc::Rc<dyn History>
        )
    });
    rsx! { Router::<crate::Route> {} }
}

/// Renders the page at `path` both ways.
pub(crate) fn route_frames(path: &str) -> Frames {
    frames_of(|| {
        VirtualDom::new_with_props(
            RouteRoot,
            RouteRootProps {
                path: path.to_string(),
            },
        )
    })
}

/// Renders `app` both ways.
pub(crate) fn frames(app: fn() -> Element) -> Frames {
    frames_of(|| VirtualDom::new_with_props(Root, RootProps { demo: app }))
}

fn frames_of(make: impl Fn() -> VirtualDom) -> Frames {
    // Tasks may hold a tokio timer (`primitives/src/sleep.rs`), which panics on its first poll outside a runtime.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("tokio runtime");
    let _guard = rt.enter();

    let mut server = make();
    server.rebuild_in_place();
    let server = dioxus_ssr::render(&server);

    let mut client = make();
    client.rebuild_in_place();
    let rounds = settle(&mut client);
    let client = dioxus_ssr::render(&client);

    Frames {
        server,
        client,
        rounds,
    }
}

/// Renames `dxc-N` / `dxc-cN` ids by order of first appearance (`dxc-#0`, `dxc-#1`, ...).
fn canonical_ids(html: &str) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find("dxc-") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 4..];
        let digits_from = usize::from(after.starts_with('c'));
        let digits = after[digits_from..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if digits == 0 {
            out.push_str("dxc-");
            rest = after;
            continue;
        }
        let id = &rest[at..at + 4 + digits_from + digits];
        let n = seen.iter().position(|s| s == id).unwrap_or_else(|| {
            seen.push(id.to_string());
            seen.len() - 1
        });
        out.push_str(&format!("dxc-#{n}"));
        rest = &rest[at + 4 + digits_from + digits..];
    }
    out.push_str(rest);
    out
}

/// Attributes that wire elements to each other or to the keyboard and so cannot change what is painted
/// or where a box lands: dropped by [`paint_view`]. Every other attribute is kept, `data-*` included,
/// because the stylesheets key on those, and so is `aria-label` (an accessible name is content, row 306).
const WIRING_ATTRIBUTES: &[&str] = &[
    "id",
    "for",
    "tabindex",
    "aria-controls",
    "aria-labelledby",
    "aria-describedby",
    "aria-owns",
    "aria-activedescendant",
    "data-dx-motion-key",
];

/// What the comparison looks at: [`paint_view`], then the generated ids renamed ([`canonical_ids`]) --
/// in that order, so an id that only a dropped element carried cannot shift the numbering.
pub(crate) fn view(html: &str) -> String {
    canonical_ids(&paint_view(html))
}

/// `html` without its `<option ...>...</option>` elements.
fn without_options(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find("<option") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        rest = after
            .find("</option>")
            .map_or("", |end| &after[end + "</option>".len()..]);
    }
    out.push_str(rest);
    out
}

/// `html` as it is painted: the [`WIRING_ATTRIBUTES`] removed from every start tag, and the `<option>`
/// elements gone (the hidden native `<select>` that Select, SelectMulti and friends keep for form
/// values and reset is filled from the option registry in a client effect and has no box; a visible
/// trigger's text is not an `<option>`, so a missing selected label still shows, row 306).
pub(crate) fn paint_view(html: &str) -> String {
    let html = &without_options(html);
    let b = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'<' || !b.get(i + 1).is_some_and(u8::is_ascii_alphabetic) {
            // Copy up to the next `<` (always at least the byte at `i` when that byte is a `<`).
            let from = i + usize::from(b[i] == b'<');
            let next = html[from..].find('<').map_or(b.len(), |n| from + n);
            out.push_str(&html[i..next]);
            i = next;
            continue;
        }
        // A start tag: copy the name, then each attribute unless it is wiring.
        let name_end = (i + 1..b.len())
            .find(|&j| b[j].is_ascii_whitespace() || b[j] == b'>' || b[j] == b'/')
            .unwrap_or(b.len());
        out.push_str(&html[i..name_end]);
        let mut j = name_end;
        loop {
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= b.len() {
                break;
            }
            if b[j] == b'>' {
                out.push('>');
                j += 1;
                break;
            }
            if b[j] == b'/' {
                out.push('/');
                j += 1;
                continue;
            }
            let start = j;
            while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'=' | b'>' | b'/')
            {
                j += 1;
            }
            let attr = &html[start..j];
            if b.get(j) == Some(&b'=') {
                j += 1;
                if b.get(j) == Some(&b'"') {
                    j += 1;
                    j = html[j..].find('"').map_or(b.len(), |n| j + n + 1);
                } else {
                    while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                }
            }
            if !WIRING_ATTRIBUTES.contains(&attr) {
                out.push(' ');
                out.push_str(&html[start..j]);
            }
        }
        i = j;
    }
    out
}

/// The middle of two strings once their common prefix and suffix (cut at tag boundaries) are removed,
/// each side capped, with a little context: what a reader needs to see what moved.
pub(crate) fn divergence(server: &str, client: &str) -> Option<(String, String)> {
    if server == client {
        return None;
    }
    let mut prefix = server
        .bytes()
        .zip(client.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !server.is_char_boundary(prefix) {
        prefix -= 1;
    }
    // Back up to the start of the tag the first difference is in.
    let start = server[..prefix].rfind('<').unwrap_or(0);
    let (s, c) = (&server[start..], &client[start..]);
    let suffix = s
        .bytes()
        .rev()
        .zip(c.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let (mut end_s, mut end_c) = (s.len() - suffix, c.len() - suffix);
    while !s.is_char_boundary(end_s) {
        end_s += 1;
    }
    while !c.is_char_boundary(end_c) {
        end_c += 1;
    }
    // Forward to the end of the tag the last difference is in.
    let end_s = s[end_s..].find('>').map_or(s.len(), |i| end_s + i + 1);
    let end_c = c[end_c..].find('>').map_or(c.len(), |i| end_c + i + 1);
    let cap = |t: &str| {
        if t.len() > 360 {
            let mut at = 360;
            while !t.is_char_boundary(at) {
                at -= 1;
            }
            format!("{} ...", &t[..at])
        } else {
            t.to_string()
        }
    };
    Some((cap(&s[..end_s]), cap(&c[..end_c])))
}

/// Every component variant demo, as `name/variant`.
fn demo_list() -> Vec<(String, fn() -> Element)> {
    let mut out = Vec::new();
    for demo in DEMOS {
        for variant in demo.variants {
            // Block demos are whole pages that need the router; they get their own tests.
            if demo.r#type == ComponentType::Block {
                continue;
            }
            out.push((format!("{}/{}", demo.name, variant.name), variant.component));
        }
    }
    out
}

/// What happened to one demo or page.
pub(crate) enum Outcome {
    /// Panicked natively (needs a browser-only piece).
    Panicked,
    /// Still writing mutations after [`MAX_ROUNDS`] rounds (an effect loop).
    Unsettled,
    /// Server and client frames are identical.
    Same,
    /// They differ.
    Differs { server: String, client: String },
}

fn classify(frames: std::thread::Result<Frames>) -> Outcome {
    match frames {
        Err(_) => Outcome::Panicked,
        // A component that panics is caught by Dioxus and drawn as an error box in both frames.
        Ok(f)
            if f.server.contains("Encountered panic") || f.client.contains("Encountered panic") =>
        {
            Outcome::Panicked
        }
        Ok(f) if f.rounds >= MAX_ROUNDS => Outcome::Unsettled,
        Ok(f) => match divergence(&view(&f.server), &view(&f.client)) {
            None => Outcome::Same,
            Some((server, client)) => Outcome::Differs { server, client },
        },
    }
}

pub(crate) fn run_all() -> BTreeMap<String, Outcome> {
    demo_list()
        .into_iter()
        .map(|(name, app)| {
            let outcome = classify(catch_unwind(AssertUnwindSafe(|| frames(app))));
            (name, outcome)
        })
        .collect()
}

/// Every page the site prerenders that is not a block's bare iframe route: the home page, the docs and
/// demos indexes, the charts gallery and its tabs, and one page per component.
fn route_list() -> Vec<String> {
    let mut paths: Vec<String> = ["/", "/docs", "/demos", "/charts/"]
        .map(String::from)
        .into();
    for kind in crate::charts_gallery::ChartKind::ALL {
        paths.push(format!("/charts/{}/", kind.slug()));
    }
    for demo in DEMOS {
        paths.push(format!("/component/{}/", demo.name));
    }
    paths
}

pub(crate) fn run_routes() -> BTreeMap<String, Outcome> {
    route_list()
        .into_iter()
        .map(|path| {
            let outcome = classify(catch_unwind(AssertUnwindSafe(|| route_frames(&path))));
            (path, outcome)
        })
        .collect()
}

/// Why a demo's two frames differ, one reason per class. Every entry of [`BASELINE`] names one.
mod why {
    pub const AVATAR: &str = "the image's load state goes `empty` -> `loading` in the first effect; the picture and the fallback \
        share one grid cell, so no box moves (row 204)";
    pub const CAROUSEL: &str = "slide count, and so the previous/next `disabled` state, is read from the mounted track in an \
        effect; the buttons keep their box";
    pub const FIND_LAYER: &str = "the find-in-page layer (`primitives/src/virtual/find.rs`, rows 153 and 309) is mounted after hydration \
        on purpose: it is one hidden placeholder per row or cell (the data-grid demo's is 16,000, the wrapping grid's 3,000), which the \
        server would ship as page weight and the client would hydrate for nothing the visitor sees; it is absolutely positioned, \
        `aria-hidden`, `pointer-events: none`, transparent and `hidden=\"until-found\"`, so it has no box in the flow and paints nothing";
    pub const TABLE_ROWCOUNT: &str = "`TableRows` hands its row count up to the `<table>` (`aria-rowcount`, an ARIA attribute, \
        no box) through the `TableScroller` context after the body renders, since the table renders first (row 343)";
    pub const CLOCK: &str = "the date comes from the clock, which is read after hydration (an effect), never while rendering: a render-time \
        date would be baked into the page as the build day and hydration keeps the server's text (row 377); the date's span \
        reserves the widest date text in a hidden `::before` sharing its grid cell, so the empty -> filled text moves no box";
    pub const ANCHOR_ID: &str = "a caller-supplied `id` on a popup's content reaches the trigger's `anchor-name` only after the content's \
        effect syncs it into the root context (`PopoverCtx::content_id` and its Tooltip/HoverCard twins); the server writes the generated \
        id. `anchor-name` is a custom ident that names an anchor and paints nothing, and the client's name is the one `position-anchor` resolves";
}
use why::*;

/// Demos whose server and first-client frames differ in a paint-relevant way, each with its reason.
/// Shrink-only: a new divergence fails the test with its markup, and an entry that no longer
/// diverges fails it too, so the list can only get shorter by fixing something.
const BASELINE: &[(&str, &str)] = &[
    ("avatar/badge", AVATAR),
    ("avatar/group", AVATAR),
    ("avatar/main", AVATAR),
    ("avatar/dropdown", AVATAR),
    ("avatar/states", AVATAR),
    ("carousel/align", CAROUSEL),
    ("carousel/api", CAROUSEL),
    ("carousel/autoplay", CAROUSEL),
    ("carousel/drag_free", CAROUSEL),
    ("carousel/drag_free_rtl", CAROUSEL),
    ("carousel/edge_fade", CAROUSEL),
    ("carousel/hidden_arrows", CAROUSEL),
    ("carousel/indicators", CAROUSEL),
    ("carousel/main", CAROUSEL),
    ("carousel/peek", CAROUSEL),
    ("carousel/rewind", CAROUSEL),
    ("carousel/rtl", CAROUSEL),
    ("carousel/sizes", CAROUSEL),
    ("carousel/spacing", CAROUSEL),
    ("carousel/vertical", CAROUSEL),
    ("carousel/virtual_loop", CAROUSEL),
    ("carousel/virtual_loop_rtl", CAROUSEL),
    ("carousel/virtual_many", CAROUSEL),
    ("drag_and_drop_list/main", AVATAR),
    ("date_picker/natural_language", CLOCK),
    ("dropdown_menu/account_switcher", AVATAR),
    ("empty/avatar", AVATAR),
    ("item/avatar", AVATAR),
    ("item/group", AVATAR),
    ("table/virtualized", TABLE_ROWCOUNT),
    ("table/reorder", TABLE_ROWCOUNT),
    ("virtual_grid/find_in_page", FIND_LAYER),
    ("virtual_list/find_in_page", FIND_LAYER),
];

/// Pages whose first client frame differs from the server page in a paint-relevant way: the pages that
/// show a demo of [`BASELINE`]. Shrink-only, like it.
const ROUTE_BASELINE: &[(&str, &str)] = &[
    ("/", AVATAR),
    ("/component/avatar/", AVATAR),
    ("/component/carousel/", CAROUSEL),
    ("/component/date_picker/", CLOCK),
    ("/component/drag_and_drop_list/", AVATAR),
    ("/component/empty/", AVATAR),
    ("/component/item/", AVATAR),
    ("/component/table/", TABLE_ROWCOUNT),
    ("/component/virtual_grid/", FIND_LAYER),
    ("/component/virtual_list/", FIND_LAYER),
];

/// Pages that differ only when primitives' `web` arm is compiled (preview's `web`, `desktop` and `server`
/// features turn it on; a bare `cargo test --workspace` does not). The `web` arm is the one that ships:
/// the popup roots there are real `<dialog>`s whose content effect syncs its id into the trigger's
/// `anchor-name` (`primitives/src/popover.rs:242,385`), and the dropdown menu page's account-switcher
/// avatar (`dropdown_menu/account_switcher`) reaches its load effect only in the web arm's markup. The plain
/// arm does neither, so these pages match there.
/// The outcome is a function of the compiled feature set, never of timing: 10 runs per arm agree.
const ROUTE_BASELINE_WEB: &[(&str, &str)] = &[
    ("/component/dropdown_menu/", AVATAR),
    ("/component/top_layer/", ANCHOR_ID),
];

/// The pages [`ROUTE_BASELINE`] and, when the `web` arm is compiled, [`ROUTE_BASELINE_WEB`] list.
fn route_baseline() -> Vec<(&'static str, &'static str)> {
    let web = cfg!(any(
        feature = "web",
        feature = "desktop",
        feature = "server"
    ));
    ROUTE_BASELINE
        .iter()
        .chain(web.then_some(ROUTE_BASELINE_WEB).into_iter().flatten())
        .copied()
        .collect()
}

/// Demos that cannot render natively at all, with the missing piece.
const UNRENDERABLE: &[(&str, &str)] = &[
    (
        "breadcrumb/router_link",
        "needs a `RouterContext` (`NavLink`)",
    ),
    ("navbar/click_only", "needs a `RouterContext` (`Link`)"),
    ("navbar/main", "needs a `RouterContext` (`Link`)"),
    ("navbar/rtl", "needs a `RouterContext` (`Link`)"),
    ("top_layer/main", "needs a `RouterContext` (`Link`)"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(names: impl Iterator<Item = String>) -> Vec<String> {
        let mut v: Vec<String> = names.collect();
        v.sort();
        v
    }

    /// The gate: every demo paints the same markup on the server as after the first client tick,
    /// except the demos [`BASELINE`] explains.
    #[test]
    fn demos_paint_the_same_before_and_after_the_first_client_tick() {
        let all = run_all();
        let mut problems = Vec::new();
        for (name, outcome) in &all {
            let known = BASELINE.iter().any(|(n, _)| n == name);
            match outcome {
                Outcome::Unsettled => problems.push(format!(
                    "{name}: still re-rendering after {MAX_ROUNDS} rounds; an effect loop."
                )),
                Outcome::Differs { server, client, .. } if !known => problems.push(format!(
                    "{name}: the server markup and the markup after the first client tick differ.\n  server: {server}\n  client: {client}\n  \
                     Fix the component so the server renders what the client's first frame will, or add the demo to BASELINE with the reason."
                )),
                Outcome::Same if known => problems.push(format!(
                    "{name}: no longer differs; delete its BASELINE entry."
                )),
                _ => {}
            }
        }
        for (name, _) in BASELINE {
            if !all.contains_key(*name) {
                problems.push(format!(
                    "{name}: BASELINE names a demo that does not exist."
                ));
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    /// The demos that panic natively are exactly the listed ones, so a demo that starts needing a
    /// browser-only piece is noticed instead of silently dropping out of the comparison.
    #[test]
    fn only_the_listed_demos_cannot_render_natively() {
        let got = sorted(
            run_all()
                .into_iter()
                .filter(|(_, o)| matches!(o, Outcome::Panicked))
                .map(|(n, _)| n),
        );
        let want = sorted(UNRENDERABLE.iter().map(|(n, _)| n.to_string()));
        assert_eq!(got, want);
    }

    /// The same gate over whole pages (the layouts, the router, the sidebar and every demo of the page
    /// together), which is what a visitor's first paint is.
    #[test]
    fn pages_paint_the_same_before_and_after_the_first_client_tick() {
        let all = run_routes();
        let mut problems = Vec::new();
        let baseline = route_baseline();
        for (path, outcome) in &all {
            let known = baseline.iter().any(|(n, _)| n == path);
            match outcome {
                Outcome::Panicked => problems.push(format!("{path}: panicked natively.")),
                Outcome::Unsettled => problems.push(format!(
                    "{path}: still re-rendering after {MAX_ROUNDS} rounds; an effect loop."
                )),
                Outcome::Differs { server, client } if !known => problems.push(format!(
                    "{path}: the server page and the page after the first client tick differ.\n  server: {server}\n  client: {client}\n  \
                     Fix the component, or add the page to ROUTE_BASELINE with the reason."
                )),
                Outcome::Same if known => {
                    problems.push(format!("{path}: no longer differs; delete its ROUTE_BASELINE entry."))
                }
                _ => {}
            }
        }
        for (path, _) in &baseline {
            if !all.contains_key(*path) {
                problems.push(format!(
                    "{path}: ROUTE_BASELINE names a page that is not rendered."
                ));
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    // ---- non-vacuity: the harness must see the divergences it exists for ----

    /// A label read from a registry that a client effect fills (row 306: Select and Combobox).
    fn registry_label() -> Element {
        let mut registry = use_signal(Vec::<(String, String)>::new);
        use_effect(move || registry.write().push(("a".into(), "Alpha".into())));
        let label = registry
            .read()
            .iter()
            .find(|(v, _)| v == "a")
            .map(|(_, l)| l.clone())
            .unwrap_or_default();
        rsx! { button { "aria-label": "{label}", "{label}" } }
    }

    /// The same markup with the label known at render time (row 306's fix: `item_to_label`).
    fn render_time_label() -> Element {
        let mut registry = use_signal(Vec::<(String, String)>::new);
        use_effect(move || registry.write().push(("a".into(), "Alpha".into())));
        let label = "Alpha";
        rsx! { button { "aria-label": "{label}", "{label}" } }
    }

    /// A box size set by a first-signal write in an effect (rows 215/252's class, if the value came
    /// from Rust rather than from a browser measurement).
    fn effect_sized_box() -> Element {
        let mut basis = use_signal(|| 0.0_f64);
        use_effect(move || basis.set(50.0));
        rsx! { div { style: "flex-basis: {basis}%;" } }
    }

    /// A text input whose `value` an effect fills (row 306's Combobox half: the input showed no label
    /// until hydration). The browser spec `first-paint-text` read `input.value`; this is that read.
    fn effect_valued_input() -> Element {
        let mut value = use_signal(String::new);
        use_effect(move || value.set("SvelteKit".into()));
        rsx! { input { r#type: "text", value: "{value}" } }
    }

    #[test]
    fn a_value_written_by_an_effect_is_seen() {
        let f = frames(effect_valued_input);
        assert!(f.client.contains(r#"value="SvelteKit""#), "{}", f.client);
        let (server, client) =
            divergence(&view(&f.server), &view(&f.client)).expect("the two frames differ");
        assert!(
            client.contains("SvelteKit") && !server.contains("SvelteKit"),
            "{server}\n{client}"
        );
    }

    #[test]
    fn a_registry_filled_by_an_effect_is_seen() {
        let f = frames(registry_label);
        assert!(!f.server.contains("Alpha"), "{}", f.server);
        assert!(f.client.contains(r#"aria-label="Alpha""#), "{}", f.client);
        assert!(divergence(&view(&f.server), &view(&f.client)).is_some());
    }

    #[test]
    fn a_label_known_at_render_time_is_the_same_in_both_frames() {
        let f = frames(render_time_label);
        assert_eq!(view(&f.server), view(&f.client));
    }

    #[test]
    fn a_first_signal_write_in_an_effect_is_seen() {
        let f = frames(effect_sized_box);
        assert!(f.server.contains("flex-basis: 0%"), "{}", f.server);
        assert!(f.client.contains("flex-basis: 50%"), "{}", f.client);
    }

    // ---- the normalisers ----

    #[test]
    fn generated_ids_are_renamed_by_first_appearance() {
        let a = canonical_ids(r#"<i id="dxc-41" aria-controls="dxc-c7"></i><b for="dxc-41">"#);
        let b = canonical_ids(r#"<i id="dxc-9" aria-controls="dxc-c0"></i><b for="dxc-9">"#);
        assert_eq!(a, b);
        assert_eq!(
            a,
            r##"<i id="dxc-#0" aria-controls="dxc-#1"></i><b for="dxc-#0">"##
        );
    }

    #[test]
    fn options_are_dropped_but_a_trigger_label_is_not() {
        let html = r#"<button><span>first</span></button><select name="a"><option value="a" selected=true>a</option><option value="b">b</option></select><p>x</p>"#;
        assert_eq!(
            paint_view(html),
            r#"<button><span>first</span></button><select name="a"></select><p>x</p>"#
        );
    }

    #[test]
    fn wiring_attributes_are_dropped_and_the_rest_kept() {
        let html = r#"<button id="x" aria-controls="y" tabindex="-1" aria-label="A > B" data-state="on" disabled=true class="c">t</button>é<br/>"#;
        assert_eq!(
            paint_view(html),
            r#"<button aria-label="A > B" data-state="on" disabled=true class="c">t</button>é<br/>"#
        );
    }
}
