//! Named behaviours: the decisions that must be made on the page's thread, inside an event's
//! dispatch, because a message to Rust would arrive after the browser committed the event's default
//! action (backlog row 434 step 3). A behaviour is NOT an interpreter and NOT a predicate language:
//! it is a name, typed parameters, and one pure Rust `decide(config, memory, snapshot, event) ->
//! Decision` in the `decision-kernel` crate (`decision-kernel/src/<kind>.rs`, row 509). The crate is
//! called three ways: as a direct call on a renderer with no page (Blitz), as a wasm module the
//! bridge calls synchronously inside the event's dispatch on a renderer with a page, and by
//! `cargo test`. `ts/behaviours/<kind>.ts` holds no decision: it registers the generated adapter.
//!
//! * [`Behaviour`] has one variant per named behaviour. Today there is one, [`Behaviour::Fixture`], a
//!   test fixture no component uses; a real behaviour (`hotkey`, `scrollLock`, `otpFilter`, ...) is
//!   added by a lane that ports one block, by adding a variant, its kernel function (registered in
//!   `decision-kernel/src/kinds.rs`), its TS module and its vectors.
//! * **Parity by generation.** `cargo test` writes `ts/behaviours/behaviours.gen.ts` (the kinds, one
//!   interface per variant, the declarations, and from the kernel's `Config`, `Snapshot` and `Event`
//!   input types the adapter's event-field and attribute-read lists and the kernel's kind table with
//!   its hash) from this enum and the kernel's table and fails on drift; a TS module is typed against
//!   the generated union, and `bun test` asserts the registry's keys equal the generated kinds.
//!   The input types are the one definition of their wire keys, so a renamed key is red here and
//!   never a behaviour that silently stops deciding in a page. Regenerate with
//!   `DX_BLESS=1 cargo test -p dioxus-primitives behaviours`, then `scripts/build-kernel.sh` and
//!   `scripts/build-js-bundle.sh`.
//! * **Placement.** A target inside a Dioxus element and an event Dioxus supports is an rsx handler
//!   (it decides synchronously on every renderer). A window, document or outside target, an event
//!   Dioxus lacks, or the capture phase is a behaviour. A continuous stream (scroll, resize,
//!   pointermove, wheel) is `listen` / `observe` / `sample`, never a vdom handler.
//! * **Declared, not chosen by the page.** [`Info`] states each variant's target, phase and
//!   passivity; `passive` is derived from the behaviour (it is `false` only for a behaviour that can
//!   cancel), never a free option. A decision's `prevent` / `stop` are honoured only when `passive`
//!   is `false`.
//! * **Owned by the Rust scope, ref-counted.** [`use_behaviour`](super::use_behaviour) installs on
//!   mount and the session's teardown removes it; the page keeps one DOM listener per (target, type,
//!   capture, passive) with a reference count, so a blocking listener exists only while something
//!   needs it. One behaviour of a kind per target: a second install of the same kind on the same
//!   target replaces the first.
//! * **Epoch and fail-open.** Every install carries an [`Epoch`](super::Epoch): an install older than
//!   one the target already saw is dropped (a restarted session's late command cannot undo its
//!   successor), an equal one renews. A variant may declare a lease, and a non-passive one always has one
//!   ([`CANCEL_LEASE_MS`] unless it names its own): the page removes the listener
//!   itself when Rust stops renewing, so a hung or dead Rust side leaves the page's native behaviour
//!   (WCAG 2.1.2), and the session renews while it lives.

use decision_kernel::{table_hash, KINDS as KERNEL_KINDS};
use serde::{Deserialize, Serialize};

use super::events::EventKind;

/// A report from the page that a kind could not decide: the kernel lacks the kind, could not read the
/// request, trapped, or never loaded. The page answers no decision (the browser keeps its native
/// action) and sends this once per install over the scope's channel, so Rust can tear the install down
/// or resync it (a gesture half done, attributes set by earlier decisions).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StandDown {
    /// The behaviour kind that stood down.
    pub kind: String,
    /// Why.
    pub reason: StandDownReason,
}

/// The reasons of a [`StandDown`] (the page's `KernelStandDown` reasons, kebab-case).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StandDownReason {
    /// The kernel module has no such kind (a stale module, or a kind the generated table lacks).
    MissingKind,
    /// The kernel could not read the request (it answered `u32::MAX`) or the page could not encode it.
    UnreadableRequest,
    /// The kernel trapped on this kind (the page re-instantiates it; three traps stand the kind down).
    Trap,
    /// The kind trapped three times in this page and decides no more.
    KindStoodDown,
    /// The kernel failed to load (blocked, not a module, the wrong ABI or kind table).
    KernelFailed,
    /// No kernel was ever loaded in this page.
    KernelAbsent,
}

/// A message from a behaviour's install: what its `decide` sends (`T`), or a [`StandDown`] report. The
/// report is an object with `kernel: "stand-down"`, which no decision's `send` carries.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Inbound<T> {
    /// The kernel could not decide.
    Report(StandDownMessage),
    /// A decision's `send`.
    Message(T),
}

/// The wire form of a [`StandDown`]: `{"kernel": "stand-down", "kind": .., "reason": ..}`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StandDownMessage {
    /// Always `"stand-down"`: what tells a report from a decision's `send`.
    #[serde(rename = "kernel")]
    _kernel: StandDownTag,
    /// The report.
    #[serde(flatten)]
    pub report: StandDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum StandDownTag {
    #[serde(rename = "stand-down")]
    StandDown,
}

/// A named behaviour and its parameters. The wire form is `{"kind": "<kebab-case name>", ...params}`
/// (camelCase parameter names), which is what `behaviours.gen.ts` types.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum Behaviour {
    /// The test fixture: on `event` it sets `attr` to `value` on the host element and tells Rust,
    /// unless the host carries `data-dx-fixture-block`. With `cancel` it also cancels the event (and is
    /// therefore declared non-passive). It proves install, epoch, lease, ref-counting and teardown end
    /// to end and has no user.
    Fixture {
        /// The event to act on.
        event: EventKind,
        /// The attribute to set (`data-*` or `aria-*`).
        attr: String,
        /// Its value.
        value: String,
        /// Listen in the capture phase.
        #[serde(default)]
        capture: bool,
        /// Cancel the event's default action (makes the listener non-passive).
        #[serde(default)]
        cancel: bool,
        /// Fail-open lease in milliseconds, renewed while the session lives.
        #[serde(default)]
        lease_ms: Option<u32>,
    },
}

/// Which target kind a behaviour listens on; the page checks it against the resolved target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeclaredTarget {
    /// `window`.
    Window,
    /// `document`.
    Document,
    /// An element (the command's target).
    Element,
}

/// What a behaviour declares about its listener: sent with the install, and the only source of the
/// page's phase and passivity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Declared {
    /// The target kind.
    pub target: DeclaredTarget,
    /// The event.
    pub event: EventKind,
    /// The capture phase.
    pub capture: bool,
    /// `false` only for a behaviour that cancels events.
    pub passive: bool,
    /// The fail-open lease; always set for a non-passive behaviour (see [`CANCEL_LEASE_MS`]).
    pub lease_ms: Option<u32>,
}

/// The lease of a behaviour that cancels events and declares none of its own: the page removes the
/// listener this long after Rust last renewed it, and the session renews at half of it. Sized above
/// one desktop render plus its acknowledgement, so a healthy session never lapses.
pub const CANCEL_LEASE_MS: u32 = 3000;

/// A behaviour that can cancel an event always has a lease (WCAG 2.1.2: a hung Rust side must not leave
/// the page unable to take its native action), by construction rather than by remembering to declare one.
fn lease_for(passive: bool, declared: Option<u32>) -> Option<u32> {
    if passive {
        declared
    } else {
        Some(declared.unwrap_or(CANCEL_LEASE_MS))
    }
}

/// The static description of a variant, for the generator and the gates.
#[derive(Clone, Copy, Debug)]
pub struct Info {
    /// The wire `kind`.
    pub kind: &'static str,
    /// The parameters: wire name and TypeScript type (`EventName` is the generated event union).
    pub params: &'static [(&'static str, &'static str)],
}

impl Behaviour {
    /// The wire `kind`.
    pub fn kind(&self) -> &'static str {
        self.info().kind
    }

    /// The variant's static description. No wildcard arm: a new variant must describe itself here.
    pub fn info(&self) -> Info {
        match self {
            Behaviour::Fixture { .. } => Info {
                kind: "fixture",
                params: &[
                    ("event", "EventName"),
                    ("attr", "string"),
                    ("value", "string"),
                    ("capture", "boolean"),
                    ("cancel", "boolean"),
                    ("leaseMs", "number | null"),
                ],
            },
        }
    }

    /// What the listener is declared to be (target, event, phase, passivity, lease).
    pub fn declared(&self) -> Declared {
        match self {
            Behaviour::Fixture {
                event,
                capture,
                cancel,
                lease_ms,
                ..
            } => Declared {
                target: DeclaredTarget::Element,
                event: *event,
                capture: *capture,
                passive: !*cancel,
                lease_ms: lease_for(!*cancel, *lease_ms),
            },
        }
    }

    /// The page's module for this behaviour (`ts/behaviours/<kind>.ts`, bundled), evaluated once per
    /// webview before the first install of the kind.
    pub fn module(&self) -> &'static str {
        match self {
            Behaviour::Fixture { .. } => include_str!("../js/behaviours/fixture.js"),
        }
    }

    /// One value per variant, for the generator and the vectors' coverage. A test fails when this
    /// list and the enum disagree.
    pub fn samples() -> Vec<Behaviour> {
        vec![Behaviour::Fixture {
            event: EventKind::Click,
            attr: "data-dx-fixture".into(),
            value: "on".into(),
            capture: false,
            cancel: false,
            lease_ms: None,
        }]
    }
}

/// `PascalCase` of a kebab-case kind (`scroll-lock` -> `ScrollLock`).
fn pascal(kind: &str) -> String {
    kind.split('-')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// The text of `ts/behaviours/behaviours.gen.ts`.
pub fn render_gen() -> String {
    let samples = Behaviour::samples();
    let mut out = String::new();
    out.push_str(
        "// GENERATED by `cargo test -p dioxus-primitives` from `src/bridge/behaviours.rs` (row 434 step 3). Do not edit:\n\
         // `DX_BLESS=1 cargo test -p dioxus-primitives behaviours` rewrites it, and the test fails when it drifts.\n\
         // The kinds, parameters and declarations of the Rust `Behaviour` enum, for the TS behaviour modules.\n\n",
    );
    let events: Vec<String> = EventKind::ALL
        .iter()
        .map(|e| format!("\"{}\"", e.wire()))
        .collect();
    out.push_str(&format!(
        "export type EventName = {};\n\n",
        events.join(" | ")
    ));
    // KINDS is the kernel's table in wire-index order: a kind's position here is the index the page passes
    // to `dx_decide`. A test holds this list equal to the `Behaviour` kinds.
    let kinds: Vec<String> = KERNEL_KINDS
        .iter()
        .map(|k| format!("\"{}\"", k.name))
        .collect();
    out.push_str(&format!(
        "export const KINDS = [{}] as const;\nexport type Kind = (typeof KINDS)[number];\n\n",
        kinds.join(", ")
    ));
    for b in &samples {
        let info = b.info();
        out.push_str(&format!(
            "export interface {}Behaviour {{\n",
            pascal(info.kind)
        ));
        out.push_str(&format!("  kind: \"{}\";\n", info.kind));
        for (name, ty) in info.params {
            out.push_str(&format!("  {name}: {ty};\n"));
        }
        out.push_str("}\n");
    }
    let union: Vec<String> = samples
        .iter()
        .map(|b| format!("{}Behaviour", pascal(b.kind())))
        .collect();
    out.push_str(&format!(
        "\nexport type Behaviour = {};\n\n",
        union.join(" | ")
    ));
    out.push_str("/** What each kind declares at its sample parameters (the real declaration depends on them). */\n");
    out.push_str("export const DECLARED = {\n");
    for b in &samples {
        let d = b.declared();
        out.push_str(&format!(
            "  \"{}\": {{ target: \"{}\", capture: {}, passive: {} }},\n",
            b.kind(),
            match d.target {
                DeclaredTarget::Window => "window",
                DeclaredTarget::Document => "document",
                DeclaredTarget::Element => "element",
            },
            d.capture,
            d.passive
        ));
    }
    out.push_str("} as const;\n\n");
    let list = |keys: &[&str]| {
        let quoted: Vec<String> = keys.iter().map(|k| format!("\"{k}\"")).collect();
        format!("[{}]", quoted.join(", "))
    };
    out.push_str(&format!(
        "/** The hash of the kernel's kind table: the page refuses a kernel module whose `dx_kinds` differs. */\nexport const KERNEL_HASH = {};\n\n",
        table_hash()
    ));
    out.push_str("/** The kernel's kind table in wire-index order: the keys of each kind's `Config`, `Snapshot` (the attribute reads) and `Event` (the event fields), from the kernel's input types. */\n");
    out.push_str("export const KERNEL_KINDS = [\n");
    for k in KERNEL_KINDS {
        out.push_str(&format!(
            "  {{ kind: \"{}\", config: {}, reads: {}, fields: {} }},\n",
            k.name,
            list(k.config),
            list(k.reads),
            list(k.fields)
        ));
    }
    out.push_str("] as const;\n\n");
    out.push_str("/** What a behaviour module registers: the event fields and attribute reads of its kernel kind. */\n");
    out.push_str("export const ADAPTER = {\n");
    for k in KERNEL_KINDS {
        out.push_str(&format!(
            "  \"{}\": {{ fields: {}, reads: {} }},\n",
            k.name,
            list(k.fields),
            list(k.reads)
        ));
    }
    out.push_str("} as const;\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const GEN_PATH: &str = "src/ts/behaviours/behaviours.gen.ts";
    const GEN: &str = include_str!("../ts/behaviours/behaviours.gen.ts");
    const VECTORS: &str = include_str!("../../conformance/behaviours.json");
    const THIS_FILE: &str = include_str!("behaviours.rs");

    /// The generated file is exactly what the enum says; `DX_BLESS=1` rewrites it.
    #[test]
    fn the_generated_ts_is_current() {
        let want = render_gen();
        if std::env::var_os("DX_BLESS").is_some() {
            // cwd of a `cargo test -p dioxus-primitives` run is the crate directory
            std::fs::write(GEN_PATH, &want).expect("write behaviours.gen.ts");
            return;
        }
        assert!(
            GEN == want,
            "behaviours.gen.ts is stale: run `DX_BLESS=1 cargo test -p dioxus-primitives behaviours`, \
             then scripts/build-js-bundle.sh"
        );
    }

    /// `samples()` has one value per enum variant (counted from this file's source).
    #[test]
    fn samples_cover_every_variant() {
        let start = THIS_FILE.find("pub enum Behaviour {").expect("enum");
        let body = &THIS_FILE[start..start + THIS_FILE[start..].find("\n}\n").expect("end")];
        let variants = body
            .lines()
            .filter(|l| {
                l.starts_with("    ")
                    && !l.starts_with("     ")
                    && l.trim_start()
                        .chars()
                        .next()
                        .is_some_and(char::is_uppercase)
                    && l.trim_end().ends_with('{')
            })
            .count();
        let samples = Behaviour::samples();
        assert_eq!(
            samples.len(),
            variants,
            "samples() must list every variant once"
        );
        let kinds: std::collections::HashSet<_> = samples.iter().map(Behaviour::kind).collect();
        assert_eq!(kinds.len(), samples.len(), "kinds are unique");
    }

    /// A sample serialises to `kind` plus exactly the parameters `info()` names, and reads back.
    #[test]
    fn samples_serialise_to_their_declared_params() {
        for b in Behaviour::samples() {
            let value = serde_json::to_value(&b).expect("serialises");
            let mut keys: Vec<&str> = value
                .as_object()
                .expect("object")
                .keys()
                .map(String::as_str)
                .collect();
            keys.sort_unstable();
            let mut want: Vec<&str> = std::iter::once("kind")
                .chain(b.info().params.iter().map(|(name, _)| *name))
                .collect();
            want.sort_unstable();
            // `leaseMs: None` is serialised as null, so every key is present
            assert_eq!(keys, want, "{}", b.kind());
            assert_eq!(
                serde_json::from_value::<Behaviour>(value).expect("reads back"),
                b
            );
        }
    }

    /// Passive is derived: only a behaviour that cancels is non-passive.
    #[test]
    fn passive_is_derived_from_the_behaviour() {
        let fixture = |cancel| Behaviour::Fixture {
            event: EventKind::Keydown,
            attr: "data-x".into(),
            value: "y".into(),
            capture: true,
            cancel,
            lease_ms: None,
        };
        assert!(fixture(false).declared().passive);
        assert!(!fixture(true).declared().passive);
        assert!(fixture(true).declared().capture);
    }

    /// A behaviour that cancels always carries a lease, a passive one only when it asks for one.
    #[test]
    fn a_cancelling_behaviour_always_has_a_lease() {
        let fixture = |cancel, lease_ms| Behaviour::Fixture {
            event: EventKind::Keydown,
            attr: "data-x".into(),
            value: "y".into(),
            capture: true,
            cancel,
            lease_ms,
        };
        assert_eq!(
            fixture(true, None).declared().lease_ms,
            Some(CANCEL_LEASE_MS)
        );
        assert_eq!(fixture(true, Some(900)).declared().lease_ms, Some(900));
        assert_eq!(fixture(false, None).declared().lease_ms, None);
        assert_eq!(fixture(false, Some(900)).declared().lease_ms, Some(900));
    }

    /// Every kind has vectors, each vector's `behaviour` parses as a `Behaviour` of that kind, and its
    /// declared passivity agrees with whether the expected decision cancels (`bun test` runs the
    /// same file through the real `decide`).
    #[test]
    fn the_vectors_parse_and_cover_every_kind() {
        let all: Value = serde_json::from_str(VECTORS).expect("behaviours.json is JSON");
        let all = all.as_object().expect("an object keyed by kind");
        for b in Behaviour::samples() {
            let vectors = all.get(b.kind()).and_then(Value::as_array);
            assert!(
                vectors.is_some_and(|v| !v.is_empty()),
                "no vectors for {}",
                b.kind()
            );
        }
        for (kind, vectors) in all {
            for vector in vectors.as_array().expect("an array") {
                let parsed: Behaviour = serde_json::from_value(vector["behaviour"].clone())
                    .unwrap_or_else(|e| panic!("{kind}: {} does not parse: {e}", vector["name"]));
                assert_eq!(parsed.kind(), kind);
                let cancels = vector["expect"]["prevent"] == json!(true)
                    || vector["expect"]["stop"] == json!(true);
                if cancels {
                    assert!(
                        !parsed.declared().passive,
                        "{kind}: {} cancels but is declared passive",
                        vector["name"]
                    );
                }
            }
        }
    }

    /// The kernel's kinds and the `Behaviour` kinds are one set: a variant with no `decide` in the kernel, or a kernel
    /// kind with no variant, is red here (the gate also requires `decision-kernel/src/<kind>.rs`).
    #[test]
    fn the_kernel_and_the_behaviours_are_the_same_kinds() {
        let mut kernel: Vec<&str> = KERNEL_KINDS.iter().map(|k| k.name).collect();
        let mut variants: Vec<&str> = Behaviour::samples().iter().map(Behaviour::kind).collect();
        kernel.sort_unstable();
        variants.sort_unstable();
        assert_eq!(kernel, variants);
    }

    /// Every key a kernel kind reads from its `Config` is a key the Rust `Behaviour` sends: a rename on either side
    /// is red here instead of a behaviour that never decides in a page.
    #[test]
    fn the_kernel_reads_only_keys_the_wire_form_carries() {
        for spec in KERNEL_KINDS {
            let sample = Behaviour::samples()
                .into_iter()
                .find(|b| b.kind() == spec.name)
                .expect("a sample of the kind");
            let wire = serde_json::to_value(&sample).expect("serialises");
            for key in spec.config {
                assert!(
                    wire.get(key).is_some(),
                    "{}: the kernel reads `{key}` but the `Behaviour` does not send it",
                    spec.name
                );
            }
        }
    }

    /// Every event field a kernel kind collects is a field of the bridge's closed `EventField` table.
    #[test]
    fn the_kernel_collects_only_event_fields_the_bridge_has() {
        let table: std::collections::HashSet<&str> =
            super::super::events::EventField::ALL.iter().map(|f| f.wire()).collect();
        for spec in KERNEL_KINDS {
            for field in spec.fields {
                assert!(table.contains(field), "{}: `{field}` is not an EventField", spec.name);
            }
        }
    }

    /// A vector's `behaviour` is what Rust would send: parsing it as a `Behaviour` and serialising it again gives the
    /// same object, so the vectors test the page's real request and not a hand-trimmed one.
    #[test]
    fn the_vectors_carry_the_full_wire_form() {
        let all: Value = serde_json::from_str(VECTORS).expect("behaviours.json is JSON");
        for (kind, vectors) in all.as_object().expect("an object keyed by kind") {
            for vector in vectors.as_array().expect("an array") {
                let parsed: Behaviour = serde_json::from_value(vector["behaviour"].clone())
                    .unwrap_or_else(|e| panic!("{kind}: {} does not parse: {e}", vector["name"]));
                assert_eq!(
                    serde_json::to_value(&parsed).expect("serialises"),
                    vector["behaviour"],
                    "{kind}: {} is not the wire form",
                    vector["name"]
                );
            }
        }
    }

    /// A stand-down report parses as a report, a decision's `send` as the caller's own type, and a report is never
    /// mistaken for a message.
    #[test]
    fn a_stand_down_report_is_not_a_message() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Mine {
            behaviour: String,
        }
        let report: Inbound<Mine> = serde_json::from_value(
            json!({"kernel": "stand-down", "kind": "fixture", "reason": "trap"}),
        )
        .expect("a report");
        let Inbound::Report(report) = report else {
            panic!("a report, not a message")
        };
        assert_eq!(
            report.report,
            StandDown {
                kind: "fixture".into(),
                reason: StandDownReason::Trap
            }
        );
        let message: Inbound<Mine> =
            serde_json::from_value(json!({"behaviour": "fixture"})).expect("a message");
        assert_eq!(message, Inbound::Message(Mine { behaviour: "fixture".into() }));
        assert!(serde_json::from_value::<Inbound<Mine>>(
            json!({"kernel": "other", "kind": "fixture", "reason": "trap"})
        )
        .is_err());
    }

    /// Every reason a report can carry is a string the page's kernel throws (`ts/lib/kernel.ts`), and the other way.
    #[test]
    fn the_reasons_are_the_pages_reasons() {
        const PAGE: &str = include_str!("../ts/lib/kernel.ts");
        for (reason, name) in [
            (StandDownReason::MissingKind, "missing-kind"),
            (StandDownReason::UnreadableRequest, "unreadable-request"),
            (StandDownReason::Trap, "trap"),
            (StandDownReason::KindStoodDown, "kind-stood-down"),
            (StandDownReason::KernelFailed, "kernel-failed"),
            (StandDownReason::KernelAbsent, "kernel-absent"),
        ] {
            assert!(
                PAGE.contains(&format!("\"{name}\"")),
                "the page's kernel never reports `{name}`"
            );
            assert_eq!(
                serde_json::from_value::<StandDownReason>(json!(name)).expect("parses"),
                reason
            );
        }
        let in_page = PAGE
            .lines()
            .find(|l| l.contains("export type StandDownReason"))
            .expect("the page declares its reasons");
        assert_eq!(in_page.matches('"').count() / 2, 6, "six reasons on the page");
    }

    #[test]
    fn the_wire_form_is_kind_plus_camel_case_params() {
        assert_eq!(
            serde_json::to_value(Behaviour::samples().remove(0)).expect("serialises"),
            json!({"kind": "fixture", "event": "click", "attr": "data-dx-fixture", "value": "on",
                   "capture": false, "cancel": false, "leaseMs": null})
        );
        assert!(serde_json::from_value::<Behaviour>(json!({"kind": "eval", "code": "x"})).is_err());
        assert!(serde_json::from_value::<Behaviour>(
            json!({"kind": "fixture", "event": "not-an-event", "attr": "a", "value": "b"})
        )
        .is_err());
    }
}
