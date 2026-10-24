# Conformance harness — rule sources and calibration

What the a11y/behaviour harness tests against, where each rule comes from, and how the harness proves itself correct before it judges a component.

Design principle: **a failing test must be traceable to a rule someone else wrote.** If the rule's only author is us, it is an opinion, and it must be labelled as one.

---

## Three tiers, never blurred

| Tier | Source | Status | Why it binds |
|---|---|---|---|
| **1. APG** | W3C WAI-ARIA Authoring Practices | Standard | The project's **own stated contract** — `README.md` requires new primitives to adhere to it |
| **2. HTML** | WHATWG HTML Living Standard + WPT | Standard | Form submission and constraint validation are plain HTML semantics |
| **3. Radix-parity** | Radix UI behaviour | **Opinion** | Upstream never committed to it; useful where the standards are silent |

Keeping tier 3 separate is not pedantry. Upstream's README names shadcn for styling and APG for behaviour, and never mentions Radix. If Radix behaviours are filed as "conformance", an upstream maintainer can fairly reject the whole suite. Labelled as a second tier, they are a proposal.

## Calibration — the part that makes it trustworthy

Every rule runs against **two subjects**: the component under test, and a reference implementation that is known-correct. If the reference fails, the *test* is wrong, not the component.

| Tier | Reference subject | Cost |
|---|---|---|
| 1. APG | The pattern's own APG example page | Vendor the page — see below |
| 2. HTML | **The browser itself** — a native `<input>`/`<select>` in the same fixture | Free, and it cannot drift |
| 3. Radix | Radix demo, pinned | Avoid in CI |

Tier 2's reference is the strongest of the three: a native `<input type="radio" name="x">` must contribute `x=value` to a `FormData` by specification. No vendoring, no network, no third party. The current `oracle-focus-restore.spec.ts` uses a weaker, internal control — `Dialog`, chosen by us because it already worked. Upgrading tier 1 to an APG-page reference removes us from the loop entirely.

---

## Tier 1 — APG

Reachability verified (HTTP 200): `patterns/menu-button/examples/menu-button-actions/`, `patterns/combobox/examples/combobox-select-only/`, `patterns/radio/`.

Source repo: `w3c/aria-practices`. The example pages are self-contained static HTML/CSS/JS under a permissive W3C licence, so **vendor them** into `playwright/oracle/reference/` and pin the commit — CI must not depend on `w3.org` being up.

Two related suites, and what they are actually good for:

- **`w3c/aria-at`** — the most authoritative statement of what a keystroke must do, but it asserts *screen-reader speech output* and needs real AT (NVDA/JAWS/VoiceOver) driving a browser. Mine it for rules; it cannot run in Playwright.
- **`web-platform-tests/wpt`, `wai-aria/`** — tests the browser's accessibility-tree mapping, not component behaviour. Useful for assertion technique, not conformance.

## Tier 2 — HTML forms

This is where the most severe defect in `dev-docs/archive/fork-research/capability-gaps.md` sits: `RadioGroup` and `Select` declare `name`/`required`, document them as being for form submission, and never reference them in their render bodies.

### Normative text — all verified reachable (HTTP 200)

| Section | Covers |
|---|---|
| [`form-control-infrastructure`](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html) | form owner, the `name` attribute, **submittable elements**, the constraint validation API (`willValidate`, `checkValidity()`, `reportValidity()`, `validity`) |
| [`forms`](https://html.spec.whatwg.org/multipage/forms.html) | the form submission algorithm, **constructing the entry list** |
| [`form-elements`](https://html.spec.whatwg.org/multipage/form-elements.html) | `<select>`, `<option>`, `<textarea>` |

The decisive concept is **submittable element**. Only `button`, `input`, `select`, `textarea` (and form-associated custom elements) can contribute to the entry list. A `div role="radio"` cannot — no matter what ARIA it carries. That is why the hidden-native-input pattern (`Checkbox`'s `BubbleInput`, Radix's "BubbleSelect") is not a hack but the only conforming approach available here: Dioxus renders plain DOM, not custom elements, so `ElementInternals.setFormValue()` — the modern alternative — is not on the table.

### Executable reference — WPT

`web-platform-tests/wpt`, path `html/semantics/forms/`. Sparse-clone it rather than the whole repo (32 MB vs. gigabytes):

```bash
git clone --filter=blob:none --no-checkout --depth 1 \
  https://github.com/web-platform-tests/wpt.git
cd wpt && git sparse-checkout init --cone
git sparse-checkout set html/semantics/forms && git checkout
```

Most relevant directories, with test counts as cloned:

| Path | Tests | Use |
|---|---|---|
| `form-submission-0/` | 33 | **`constructing-form-data-set.html`** is the entry-list reference |
| `constraints/` | 44 | `form-validation-checkValidity`, `-reportValidity`, `-validity-valueMissing`, … |
| `resetting-a-form/` | 6 | form reset restores initial values |
| `the-label-element/` | 14 | label→control association and activation |
| `the-select-element/` | 327 | `<select>` semantics |
| `the-input-element/` | 177 | `<input>` semantics |

**Important limit:** WPT tests the *browser's* implementation of the spec. These files cannot be pointed at our components — they assert on native elements. Their value is (a) an authoritative rule inventory and (b) proven technique: build a form, submit it, inspect the resulting entry list. Do not plan on running WPT as our suite.

### The rule checklist this yields

Each rule runs against our component *and* a native control in the same fixture:

1. A named control contributes `name=value` to `FormData` on submit.
2. An unchecked/unselected control contributes **nothing** — not an empty string.
3. A disabled control is barred from constraint validation and excluded from the entry list.
4. `required` with no value blocks submission and sets `validity.valueMissing`.
5. `checkValidity()` / `reportValidity()` / `willValidate` behave per spec.
6. Form reset restores the initial value.
7. A `<label>` association focuses and activates the control.
8. The `form` attribute associates a control rendered outside the `<form>` element.

Rules 1–4 alone would have caught the `RadioGroup` and `Select` defects.

### WCAG, for the messaging layer

Constraint validation covers whether a form *blocks*; WCAG covers whether a person can *recover*. Relevant and reachable: **3.3.1 Error Identification** (verified 200), 3.3.3 Error Suggestion, and 4.1.2 Name, Role, Value. These matter once validation exists — an error that is only conveyed by a red border fails 1.4.1/3.3.1 regardless of whether the form submits correctly.

## Tier 3 — Radix-parity, labelled as opinion

Behaviours no standard specifies, which mature libraries nonetheless implement: body scroll lock while a modal is open, `onCloseAutoFocus` semantics (restore focus to the trigger *unless* dismissal came from outside interaction), `aria-hidden` on background content, collision-aware repositioning.

File these as `tier3/` with the Radix source named in each test. They are proposals, and should read that way in a PR.

When a Radix behaviour looks idiosyncratic, reach for **bits-ui** (or another mature headless library) as an ad-hoc tie-breaker — read, never vendor — to distinguish a Radix quirk from a headless-library consensus. A second implementation is a second opinion, not a rule source, so it can strengthen a tier-3 proposal's framing ("de-facto consensus") but never becomes a standing oracle; policy details in `playwright/oracle/tier3-radix/README.md`.

---

---

## axe (static rules) — orthogonal to the three tiers, like hydration parity

Source: `axe-core`'s own rule set (`playwright/axe.ts`, wired through `@axe-core/playwright`), tagged to the WCAG success
criteria it implements (`wcag2a`/`wcag2aa`/`wcag21a`/`wcag21aa`) plus its curated `best-practice` rules. It is a different
*class* of check from the three tiers: tiers 1-3 test *behaviour* (does Escape move focus where APG says, does a required
control block submission); axe tests *static* rules on whatever DOM exists when it runs (valid ARIA role/attribute
combinations, accessible names, unique landmarks, label association, contrast, list/table structure, heading order). A
component can pass every behaviour oracle and still fail axe: a duplicate landmark, an `aria-required` on a `button`, an
icon-only button with no name. (axe has no rule that checks a bare `role="menu"` container for a name; it does check
`role="listbox"` via `aria-input-field-name`, so a menu name is guarded by construction, not by a scan.)

- **The helper** `expectNoAxeViolations(page, label, opts?)` in `playwright/axe.ts` runs `AxeBuilder` (scoped by
  `opts.include`), waits for the page's `<main>` and a level-one heading to attach first (a CSR wasm app's `load` can fire
  before the first frame, and axe would scan a bare `<html>`), and fails with a table of rule id, impact, help URL and every
  offending node. `opts.exclude` disables specific rules, and **each exclusion requires a non-empty `reason`**: the call
  throws otherwise, so an excluded rule is a documented false positive at that scan site, never a way to force green on a
  real defect. Exclusions are by region or rule at one site, not sitewide, with the exception of the exported constants
  (`CONTRAST_TRACKED_ELSEWHERE`, `EXCLUDE_VENDORED_CODE_HIGHLIGHT`), each naming the tracked row.
- **The two-state convention.** A component spec that reaches an open/expanded/selected state scans twice: once at rest,
  once after the spec's own existing "open it" step (no new fixture). A component with no such state scans once.
- **Fix by construction.** A finding is fixed in the primitive or the wrapper, with the accessible name derived from what
  the component already knows (a `role="menu"` popup labelled from its trigger's id, a `role="listbox"` only when the
  caller has not supplied `aria-label`/`aria-labelledby`, via the shared `has_own_accessible_name`), and merged through
  `merge_attributes`, never as a literal attribute beside a `..attributes` spread (see Rule 4 below).

---

## Hydration/deployment parity — orthogonal to the other three tiers

The tiers above ask "does this component *behave* correctly?" against a dev-server client. They cannot catch a defect that
exists only in the **deployed build shape**: fullstack SSG, prerendered by a host (non-wasm) server binary and then hydrated
by the wasm client. `playwright/oracle/hydration-parity.spec.ts` closes that gap. Its rule source is the contract that server
and client must render the same tree, plus Dioxus's hydration model, so it sits at `oracle/` top level, not under a tier.

**Why it exists:** the 2026-09-01 production incident ([`reference/platform-rationale.md`](./reference/platform-rationale.md)
Caveat 1): primitives split markup on `target_family = "wasm"`, false on the SSG server's host binary, so server prerender
and client hydration disagreed structurally and events were dead page-wide. `scripts/check-cfg-axis.sh` is the source-level
guard against that predicate; this oracle is the black-box guard for the deployed symptom. The two are complementary.

**Rule 4: the served HTML contains no start tag with a duplicated attribute name.** WHATWG HTML's duplicate-attribute parse
error keeps the *first* occurrence while the CSR/hydrated DOM path keeps the *last*, so a single element's attribute value
can disagree between lanes even when the tree shape agrees. The house rule it enforces: an explicit attribute followed by a
caller-`attributes` spread on the same element is merged (`merge_attributes`, caller wins), never left as two sequential
same-named attributes. For `style` on every anchored-content component (`TooltipContent`, `HoverCardContent`,
`DropdownMenuContent`, `MenubarContent`, `PopoverContent`, `SelectList`, `ComboboxList`) the construction is
`top_layer::anchored_content_attributes` (`primitives/src/top_layer.rs`), which folds the component's own `position-anchor`
binding and a caller's `style` into the one attribute `merge_attributes` carries. A preview-level themed wrapper can hit the
same class one layer up (`Label`'s literal `class` beside a caller `class`), so the rule also runs on **every prerendered
component page** (`SSG_SITE_DIR`), not only `/`: only SSR/SSG's raw text can show the duplicate, because the browser DOM
coalesces sequential duplicate `setAttribute` calls. **Rule 4b** asserts the caller's override wins (the `avatar` gallery
card: `aria_label: "Error avatar"` beats the wrapper's alt-derived default). The static guard is
`scripts/check-attr-spread-collision.sh`.

**The SSG lane, how to run it.** Build with `scripts/build-ssg.sh` (or `scripts/lane-ssg.sh` for a lane's own copy): an
absolute, isolated `CARGO_TARGET_DIR`, never the shared repo-root `target/`, and serve the output with a static server whose
root is the site directory ([`lane-rules.md`](./lane-rules.md), [`dev-loop.md`](./dev-loop.md)). Traps that each cost a run:

- **A root-served build takes no `--base-path`; the Pages build does.** Serve each at the mount it was built for, or every
  asset 404s and the unhydrated shell looks like dozens of real failures. Check with
  `grep -o '/<base>/' <public>/component/<name>/index.html | wc -l` (not `grep -c`: pages are minified, so it counts lines).
- **A debug SSG build's wasm traps at runtime** (`memory access out of bounds`, a failed `ws://…/_dioxus` handshake) while the
  markup and HTTP 200 look fine: geometry and `hydration-parity` read the markup and pass, interaction specs fail like a real
  regression. Use a release build for anything interactive.
- **Start the static server with `--directory`**, never `cd … && … &`, and confirm what it serves (`readlink /proc/<pid>/cwd`).
- **A bare spec filename substring-matches** `oracle/tier1-apg/carousel.spec.ts` too: pass the path.
- **Do not confirm a background build by tailing a shared log path**; poll the build's own output directory.

`playwright/ssg.local.config.ts` has no `webServer` entry (start the static server first); `oracle/hydration-parity.spec.ts`
hardcodes port 8090. The quickest manual check that a build landed on the web arm, not the native arm:

```bash
grep -o 'dx-toast[^>]*' <public>/index.html
# web arm (correct):    ...dx-toast-container-XXXXXXXX" ... popover="manual" ...
# native arm (the bug): ...dx-toast-container-XXXXXXXX" data-node-hydration="..." (no popover)
```

---

## Main-thread responsiveness — orthogonal to the other three tiers, like axe and hydration parity

A *performance* class of check, filed at `oracle/` top level for the same reason. Source:
[`reference/platform-rationale.md`](./reference/platform-rationale.md) §11 (kciter, *The Browser's Main Thread Is Expensive*)
and backlog row 45. `oracle/tier2-html/main-thread.spec.ts` plus its helper `playwright/main-thread.ts` install a
`PerformanceObserver` for `longtask` and event timing (`durationThreshold`) before each gesture and assert no task over 50 ms
and no interaction over 200 ms during open/close of each overlay and the form-control gestures (real pointer drags for Slider,
Resizable and Drawer). Normative sources: the W3C Long Tasks API (a long task exceeds 50 ms) and Event Timing API (16 ms
duration floor). Chromium-only (both APIs are), so the file skips elsewhere rather than reporting a passing zero. Every
gesture runs through `assertResponsive` (up to 3 attempts, first clear pass wins): a genuine block reproduces on every
attempt, a scheduling hiccup does not, and the thresholds are never lowered. Not `serial` mode, which would hide every
subject after the first red. `combobox`, `command` and `date_picker` `open` were red by design when it landed (real per-open
work, row 45); timing specs run on release builds only. **Scope locators**: an unscoped role query (`getByRole("list").first()`)
silently matches more once a page grows more instances, so scope through the demo's frame or class.

---

## Layout

```
playwright/oracle/
  tier1-apg/                 rules citing an APG pattern section
  tier2-html/                rules citing an HTML spec section
  tier3-radix/               labelled opinion
  hydration-parity.spec.ts   SSG-server/wasm-client markup parity (its own axis, no tier)
  reference/                 vendored APG example pages, pinned by commit (7e4034b)
  subjects/                  adapters: route + selectors per implementation
```

Each rule file names its source at the top. The run matrix is *rule × subject*; the output is a conformance matrix per
component, which doubles as the library's accessibility scorecard.

## Status

The current inventory (spec, tier, rule) is [`conformance-harness-inventory.md`](./conformance-harness-inventory.md). The dated
per-spec account of how each landed, the axe round's findings, the long SSG-lane trap notes and the 2026-08 measurement of the
existing suites are in [`archive/rounds/conformance-harness-narrative.md`](./archive/rounds/conformance-harness-narrative.md).
