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
//! events, the interpreter's DOM writes. See `dev-docs/research/row-329-ssr-first-render-parity-spike.md`.

use crate::components::DEMOS;
use crate::ComponentType;
use dioxus::prelude::*;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Rounds of `render_immediate` allowed before a demo counts as never settling.
const MAX_ROUNDS: usize = 12;

/// The two frames of one component.
pub(crate) struct Frames {
    /// The markup the server sends.
    pub server: String,
    /// The markup after the first client tick, effects included.
    pub client: String,
    /// Rounds of `render_immediate` that wrote mutations before the dom went quiet.
    pub rounds: usize,
}

/// The root every demo sits under on the site: the app's own i18n context (`App` in `main.rs`).
#[derive(Props, Clone, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
struct RootProps {
    /// The demo under test.
    demo: fn() -> Element,
}

#[component]
fn Root(props: RootProps) -> Element {
    use dioxus_i18n::prelude::*;
    use unic_langid::langid;
    use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_locale((langid!("en-US"), include_str!("i18n/en-US.ftl")))
            .with_locale((langid!("fr-FR"), include_str!("i18n/fr-FR.ftl")))
            .with_locale((langid!("es-ES"), include_str!("i18n/es-ES.ftl")))
            .with_locale((langid!("de-DE"), include_str!("i18n/de-DE.ftl")))
    });
    let Demo = props.demo;
    rsx! { Demo {} }
}

/// Renders `app` both ways.
pub(crate) fn frames(app: fn() -> Element) -> Frames {
    // Tasks may hold a tokio timer (`primitives/src/sleep.rs`), which panics on its first poll outside a runtime.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("tokio runtime");
    let _guard = rt.enter();

    let mut server = VirtualDom::new_with_props(Root, RootProps { demo: app });
    server.rebuild_in_place();
    let server = canonical_ids(&dioxus_ssr::render(&server));

    let mut client = VirtualDom::new_with_props(Root, RootProps { demo: app });
    client.rebuild_in_place();
    let mut rounds = 0;
    while rounds < MAX_ROUNDS {
        let mutations = client.render_immediate_to_vec();
        if mutations.edits.is_empty() {
            break;
        }
        rounds += 1;
    }
    let client = canonical_ids(&dioxus_ssr::render(&client));

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

/// `html` with the [`WIRING_ATTRIBUTES`] removed from every start tag.
pub(crate) fn paint_view(html: &str) -> String {
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
    let prefix = server
        .bytes()
        .zip(client.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    // Back up to the start of the tag the first difference is in.
    let start = server[..prefix].rfind('<').unwrap_or(0);
    let (s, c) = (&server[start..], &client[start..]);
    let suffix = s
        .bytes()
        .rev()
        .zip(c.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let end_s = s.len() - suffix;
    let end_c = c.len() - suffix;
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
    Some((cap(&s[..end_s.max(0)]), cap(&c[..end_c.max(0)])))
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

/// What happened to one demo.
pub(crate) enum Outcome {
    /// Panicked natively (needs a browser or a router context).
    Panicked,
    /// Server and client frames are identical.
    Same,
    /// They differ.
    Differs {
        rounds: usize,
        server: String,
        client: String,
    },
}

pub(crate) fn run_all() -> BTreeMap<String, Outcome> {
    let mut out = BTreeMap::new();
    for (name, app) in demo_list() {
        let outcome = match catch_unwind(AssertUnwindSafe(|| frames(app))) {
            Err(_) => Outcome::Panicked,
            // A component that panics is caught by Dioxus and drawn as an error box in both frames.
            Ok(f)
                if f.server.contains("Encountered panic")
                    || f.client.contains("Encountered panic") =>
            {
                Outcome::Panicked
            }
            Ok(f) => match divergence(&paint_view(&f.server), &paint_view(&f.client)) {
                None => Outcome::Same,
                Some((server, client)) => Outcome::Differs {
                    rounds: f.rounds,
                    server,
                    client,
                },
            },
        };
        out.insert(name, outcome);
    }
    out
}

/// Why a demo's two frames differ, one reason per class. Every entry of [`BASELINE`] names one.
mod why {
    pub const SELECT_MIRROR: &str = "the hidden native `<select>` mirror (form value and reset) lists the options the registry \
        collected in a client effect; it has no box";
    pub const AVATAR: &str = "the image's load state goes `empty` -> `loading` in the first effect; the picture and the fallback \
        share one grid cell, so no box moves (row 204)";
    pub const CAROUSEL: &str = "slide count, and so the previous/next `disabled` state, is read from the mounted track in an \
        effect; the buttons keep their box";
    pub const NO_SETTLE: &str = "still re-rendering after the round cap natively (a scroll or measurement loop whose answer only a \
        browser gives); the frames are cut mid-loop, so the diff is not a verdict";
    pub const FIND_LAYER: &str = "the find-in-page layer is mounted by an effect; it is absolutely positioned and `aria-hidden`";
    pub const TAG_GROUP: &str = "OPEN, found by this test: the remove button is `disabled` with `aria-label=\"Remove item \"` \
        (no label) until an effect fills the registry -- the class of row 306, not yet fixed here";
    pub const RESIZABLE: &str = "OPEN, found by this test: the panels are `flex-basis: 0%` in the server markup and take their \
        share (50%, 40%) in an effect, so the first paint has collapsed panels";
}
use why::*;

/// Demos whose server and first-client frames differ in a paint-relevant way, each with its reason.
/// Shrink-only: a new divergence fails the test with its markup, and an entry that no longer
/// diverges fails it too, so the list can only get shorter by fixing something.
const BASELINE: &[(&str, &str)] = &[
    ("area_chart/interactive", SELECT_MIRROR),
    ("avatar/badge", AVATAR),
    ("avatar/group", AVATAR),
    ("avatar/main", AVATAR),
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
    ("chart/main", SELECT_MIRROR),
    ("command/scrollable", NO_SETTLE),
    ("data_table/main", SELECT_MIRROR),
    ("drag_and_drop_list/main", AVATAR),
    ("empty/avatar", AVATAR),
    ("form/main", SELECT_MIRROR),
    ("item/avatar", AVATAR),
    ("item/group", AVATAR),
    ("message_scroller/animation", SELECT_MIRROR),
    ("pie_chart/interactive", SELECT_MIRROR),
    ("resizable/main", RESIZABLE),
    ("resizable/rtl", RESIZABLE),
    ("resizable/vertical", RESIZABLE),
    ("select/rtl", SELECT_MIRROR),
    ("select/scroll", NO_SETTLE),
    ("select/sizes", SELECT_MIRROR),
    ("tag_group/main", TAG_GROUP),
    ("tag_group/multi", TAG_GROUP),
    ("tag_group/states", TAG_GROUP),
    ("virtual_list/find_in_page", FIND_LAYER),
];

/// Demos that cannot render natively at all, with the missing piece.
const UNRENDERABLE: &[(&str, &str)] = &[
    ("navbar/click_only", "needs a `RouterContext` (`Link`)"),
    ("navbar/main", "needs a `RouterContext` (`Link`)"),
    ("navbar/rtl", "needs a `RouterContext` (`Link`)"),
    ("top_layer/main", "needs a `RouterContext` (`Link`)"),
];

fn baseline_reason(name: &str) -> Option<&'static str> {
    BASELINE.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
}

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
                Outcome::Differs { rounds, .. } if (*rounds >= MAX_ROUNDS) != baseline_reason(name).is_some_and(|r| r == NO_SETTLE) && known => problems.push(format!(
                    "{name}: settled in {rounds} rounds, so its BASELINE reason (never settles, or not) is wrong."
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

    #[test]
    fn a_registry_filled_by_an_effect_is_seen() {
        let f = frames(registry_label);
        assert!(!f.server.contains("Alpha"), "{}", f.server);
        assert!(f.client.contains(r#"aria-label="Alpha""#), "{}", f.client);
        assert!(divergence(&paint_view(&f.server), &paint_view(&f.client)).is_some());
    }

    #[test]
    fn a_label_known_at_render_time_is_the_same_in_both_frames() {
        let f = frames(render_time_label);
        assert_eq!(paint_view(&f.server), paint_view(&f.client));
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
    fn wiring_attributes_are_dropped_and_the_rest_kept() {
        let html = r#"<button id="x" aria-controls="y" tabindex="-1" aria-label="A > B" data-state="on" disabled=true class="c">t</button>é<br/>"#;
        assert_eq!(
            paint_view(html),
            r#"<button aria-label="A > B" data-state="on" disabled=true class="c">t</button>é<br/>"#
        );
    }
}
