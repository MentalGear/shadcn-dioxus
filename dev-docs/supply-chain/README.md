# Supply-chain and dependency security: the playbook

Status: written 2026-10-23 (owner ask, same day). Self-contained on purpose: a project that wants to lift this approach
should be able to read this file alone, then copy the files named in [Adopt this in your project](#adopt-this-in-your-project).
The repo's own interlinked docs ([`../README.md`](../README.md), [`../../CLAUDE.md`](../../CLAUDE.md),
[`../../SECURITY.md`](../../SECURITY.md)) point here; nothing here depends on them.

Every statement below is true of the repository as of the date above, or is marked **planned** or **unverified**. The controls
table is checked against the tree by [`check-supply-chain-doc.sh`](./check-supply-chain-doc.sh) (see
[Keeping this document true](#keeping-this-document-true)).

## The approach in one paragraph

Each supply-chain risk is turned into a small, deterministic, offline script that exits non-zero, and every script is a
gate in one list (`scripts/run-gates.sh`) that must be green before a commit. Nothing relies on a reviewer remembering. Where
a rule needs a human decision (an exception), it goes in a reasoned allowlist in the script, and an allowlist entry that no
longer matches anything fails the run, so exceptions cannot rot. Tool-enforced settings (pnpm) are used where the tool already
enforces the property. What is not yet controlled is listed as **planned** with the design, not hidden.

## Scope

Owner decision, 2026-10-23:

- **SECURITY** covers everything that builds or touches the project: the whole workspace `Cargo.lock` (the docs site, desktop
  and native features included), the detached crates' locks, build scripts, dev tooling (the two pnpm folders, `dx`, `bun`), the
  `shadcn-dioxus` CLI, the registry manifests (`component.json`) and the starter template.
- **PERFORMANCE** budgets (crate count, compile time, wasm bytes) are measured on the closure **users compile**
  (`dioxus-primitives` per feature on `wasm32-unknown-unknown` plus each component's `cargoDependencies`), not on the workspace
  lock. Measured 2026-10-23: the workspace lock has 972 packages (414 external crates, 73 with a `build.rs`, 42 proc-macros)
  while a Button-only wasm app compiles about 167 crates
  ([`research/dependency-scanners.md`](../research/dependency-scanners.md), "Facts about this repo today";
  [`backlog/217`](../backlog/done/217-dependency-audit-follow-ups-what-users-pay-for.md)).

## Threat model

### What we defend against

| # | Threat | Where it bites | Controls (ids in the table below) |
|---|---|---|---|
| T1 | A compromised or typosquatted release of a dependency (account takeover, a new malicious transitive crate). Reference case: the 2026-08-20 crates.io incident, where poisoned releases of `arrayref`, `internment` and `append-only-vec` only added a dependency on a typosquat (`proc-macro1`) whose `build.rs` ran at build time | our builds and users' builds | C01 C02 C03 C04 (partial); closure-level detection is **planned** (C16 C17) |
| T2 | A malicious `build.rs`, proc-macro or install script | any machine that builds or installs | JS: C09 (no dependency install scripts). Rust: our own `dioxus-primitives` has no `build.rs` (C07); transitive build scripts are **not** yet controlled (C16, planned) |
| T3 | Lockfile drift: a lock that no longer matches its manifest, or a manifest floor that differs from the version the gates ran on | repo, users | C01 C02 C10 |
| T4 | Unreviewed dependency growth: a crate or feature added to what users compile without a decision | users | C03 C04 C05 C06; budgets and a register are **planned** (C19 C20) |
| T5 | Generated code drifting from its source (a committed artifact edited by hand, or its source edited without a rebuild) | users (the shipped JS) | C07 |
| T6 | Untrusted markdown reaching the DOM as raw HTML | users' end users | C04 |
| T7 | A user's install pulling the component or the primitives from the wrong registry (a default registry that resolves a bare name elsewhere) | users | C08, C03 (git source rule) |
| T8 | What users compile differing from what we tested (an install missing a file, a crate, or a feature) | users | C06 C11 |
| T9 | Third-party code or assets fetched at runtime by the docs site (CDN fonts, hot-linked images) | visitors | C13 |
| T10 | A workflow running automatically on pushed or proposed code | the repository's CI | C15 (a side effect of the CI freeze, not its reason) |
| T11 | Skew between the registry checkout a user installs and the primitives crate they compile | users | C14 |

### What we do not defend against

Stated so nobody assumes otherwise.

- **A malicious or compromised maintainer, GitHub account, or the repository itself.** Gates run on code a maintainer writes;
  they do not review intent.
- **A malicious update to a dependency version already in `Cargo.lock`.** We pin exact bytes (crates.io versions are immutable
  and `Cargo.lock` carries checksums), but nothing reviews the code of a locked version. `cargo-vet` would add that record
  (planned, C17).
- **Known vulnerabilities in dependencies.** No advisory scanner runs today (no `cargo-deny`, `cargo-audit` or OSV-Scanner in
  the tree); closing this is **planned** (C16, C18).
- **Our lock never reaches users.** A user's `cargo` re-resolves every transitive crate within the caret ranges we declare.
  We control only what we declare: exact pins for two crates (C03) and floors equal to the tested version (C03). A cooldown
  in the starter's `.cargo/config.toml` (Cargo's `min-publish-age`, due in Rust 1.100) is **planned** and unverified (C21).
- **Toolchain and tool binaries.** `rustc`/`cargo`, `dx` (downloaded by `scripts/session-setup.sh` as the GitHub release binary of a pinned version, with no checksum
  verification there), `bun`, `node`, `pnpm` and the browsers are trusted as installed.
- **The CLI as a distributed binary.** The repository publishes no release artifact and no signature; users build
  `shadcn-dioxus` from source with `cargo install --git`.
- **Runtime security of a user's app** beyond the one component that parses untrusted text (C04).
- **crates.io, npm or GitHub infrastructure compromise.**
- **Availability and secrets management.** Nothing here handles secrets; the repository holds none that a gate could check.

## Controls

One row per control. **Status** is one of: `enforced` (exists and runs as a gate in `scripts/run-gates.sh`, or is enforced
by a tool on every install), `manual` (exists, run by a person at a defined moment, not in the gate list) or `planned`
(not built; the row names the design). **Runs in**: `run-gates` (a gate), `pnpm` (enforced by pnpm itself, no repo script
reads it), `session-start` (the bootstrap script), `config` (a setting in a file the platform honours), `manual`, or `none yet`. Paths in the script and config columns are backticked repository paths; the drift gate
requires every `enforced`/`manual` path to exist and every `planned` script to still not exist.

| ID | Control | Threat it stops | Enforcing script | Config | Runs in | Status | Adopt |
|---|---|---|---|---|---|---|---|
| C01 | Every committed `Cargo.lock` (root and the detached crates) resolves against its manifest with `cargo metadata --locked --offline` | T1 T3: a stale lock that a later build silently rewrites | `scripts/check-cargo-locks.sh` | `Cargo.lock`, `playwright/oracle/attr-synth/Cargo.lock`, `labs/carousel-lab/Cargo.lock` | run-gates | enforced | Copy the script; it loops `git ls-files '*Cargo.lock'`, no edits |
| C02 | Registry manifests name only the version the gates ran on: `component.json` versions are `x.y.z`/`^`/`~`/`=` with a floor equal to the `Cargo.lock` version | T3 T1: shipped floor differs from tested version (`time` shipped 0.3.44 while tests ran 0.3.47) | `scripts/check-cargo-dep-versions.sh` | `preview/src/components/*/component.json`, `Cargo.lock` | run-gates | enforced | Adapt the manifest glob; keep rule 1 |
| C03 | Exact pins and source rules in the same script: crates in `EXACT_ONLY` are written `=x.y.z`; an entry with neither `version` nor `git`, or with a foreign `git`, is red; a `dioxus-primitives` release `rev` is all-or-nothing | T1 T4 T7: a user's `cargo add` taking an untested or foreign release | `scripts/check-cargo-dep-versions.sh` | `preview/src/components/*/component.json`, `primitives/Cargo.toml` | run-gates | enforced | Edit the `EXACT_ONLY` and `REGISTRY_CRATES` tables at the top |
| C04 | The one component that parses untrusted text has a fixed crate set, an exact parser pin with `default-features = false`, a lock allowlist of the parser's dependencies, an allowlist of parser `Options`, no HTML renderer and no `dangerous_inner_html`; parsers are opt-in (only an allowlisted component may list one) | T6 T4 T1: model or user text reaching the DOM as raw HTML; a parser gaining a transitive dependency | `scripts/check-markdown-parser.sh` | `preview/src/components/markdown/component.json`, `preview/Cargo.toml`, `Cargo.lock` | run-gates | enforced | Copy if you ship a parser; edit `ALLOWED_OPTIONS`, `ALLOWED_LOCK_DEPS`, `MD_DIR` |
| C05 | Optional features are real: each optional feature of the primitives crate builds alone, with `web`, and all together, outside workspace feature unification | T4: a non-gated module quietly depending on a gated crate (`time`, `palette`) so users pay for it | `scripts/check-primitives-features.sh` | `primitives/Cargo.toml` | run-gates | enforced | Point it at your crate; features are read from the manifest; needs an absolute `CARGO_TARGET_DIR` |
| C06 | A component's directory, manifest and the crates its sources name are complete: every external crate a component's Rust files use is in `cargoDependencies` of the component or its `componentDependencies` closure | T8 T4: an install that needs a crate nobody declared, so the declared set understates what compiles | `scripts/check-installed-paths.sh` | `preview/src/components/*/component.json`, `component.json` | run-gates | enforced | Specific to the `dx components` layout; copy only with that layout |
| C07 | Generated JS is committed with a manifest of sha256 sums (source `.ts` and output `.js`); edit of either side without a rebuild is red; with the recorded `bun` version the gate also rebuilds and compares bytes. `dioxus-primitives` has no `build.rs` and no bundler crate runs in a user's build | T5 T2: drift between source and shipped artifact; a build-time code runner in every consumer's build | `scripts/check-js-bundle.sh`, `scripts/build-js-bundle.sh` | `primitives/src/js/bundle.manifest` | run-gates | enforced | Copy both scripts; set the two directories at the top and record your bundler version |
| C08 | Every spelling of the registry URL is the one constant: bare-name `componentDependencies` are rejected (dx resolves them in its default registry), `dioxus-primitives` entries carry the registry `git`, README, CLI and starter template agree | T7: an install silently reading upstream's registry or crate | `scripts/check-registry-url.sh` | `preview/src/main.rs`, `cli/src/registry.rs`, `templates/starter/Cargo.toml.liquid`, `templates/starter/Dioxus.toml` | run-gates | enforced | Only if you publish a `dx components` registry; edit the constant's location |
| C09 | No JS dependency may run an install script: `onlyBuiltDependencies: []` (pnpm 10 runs no dependency lifecycle scripts unless listed) | T2: a malicious `postinstall` | none (pnpm itself) | `playwright/pnpm-workspace.yaml`, `preview/pnpm-workspace.yaml` | pnpm | enforced | Copy the line into your `pnpm-workspace.yaml`; pnpm 10 or newer |
| C10 | Node dependencies install from the committed lock only: `pnpm install --frozen-lockfile`, never `add`/`update`/plain `install`; the session bootstrap does exactly that | T1 T3: a lock-rewriting install pulling a different build | `scripts/session-setup.sh` | `playwright/pnpm-lock.yaml`, `preview/pnpm-lock.yaml` | session-start | enforced | Copy the rule and the setup function; the rule is in `CLAUDE.md`, and no gate detects a lock-rewriting install made by hand |
| C11 | A throwaway `file://` registry copy of the tree, a starter app with no crates but `dioxus`, `dx components add` of every component, then `cargo check` | T8 T7: what users compile differs from what we tested; a manifest that omits a crate | `scripts/registry-install-smoke.sh` | `templates/starter/Cargo.toml.liquid` | manual | manual | Copy if you ship an installable registry; run before merging any manifest change; needs about 6 GB |
| C12 | Release pin: `scripts/stamp-release-rev.sh <sha>` writes one `rev` into every `dioxus-primitives` entry and the starter, refuses a commit whose primitives differ from the tree, `--check` reports disagreement | T11 T7: registry sources and primitives crate resolving to different commits | `scripts/stamp-release-rev.sh` | `preview/src/components/*/component.json`, `templates/starter/Dioxus.toml` | manual (release step) | manual | Copy if you publish git-sourced crates; adjust the stamped file list |
| C13 | The docs site loads no remote image, font or stylesheet: every image is bundled or inline, every font is self-hosted | T9: third-party content or a request to a third party; a remote CSS `@import` | `scripts/check-demo-remote-assets.sh` | `preview/index.html` | run-gates | enforced | Copy; edit the roots and the reasoned allowlist. It flags images, fonts, stylesheets, not scripts (`preview/index.html` holds two inline scripts and no external `src`; no gate states that) |
| C14 | Dependency inventory: per direct crate the locked version, `unsafe` lines, `build.rs`, proc-macro; transitive crate counts per primitives feature set (wasm and host); `--network` adds crates.io owners and last release. Read-only, writes nothing | evidence for a decision (T1 T2 T4): which crates carry code risk and what a feature costs | `scripts/dep-report.sh` | none | manual | manual | Copy; edit the `CRATES` list and the feature sets |
| C15 | Every workflow is `workflow_dispatch`-only (CI freeze); no workflow runs on push or pull request | T10: automatic execution of proposed code in CI | none | `.github/workflows/mutants.yml` | config | enforced | Keep `on: workflow_dispatch`; no gate asserts it today |
| C16 | `cargo-deny` over the whole lock and every detached lock: `bans` (build-script allowlist by crate name, denied capability crates, per-crate features, duplicate versions with reasons), `sources` (`unknown-registry`/`unknown-git = "deny"`), `licenses` (allow list), `advisories` against a pinned db | T1 T2 T4: new crate, new `build.rs`, foreign source, bad licence, known advisory | `scripts/check-cargo-deny.sh` | `deny.toml`, `supply-chain/deny.shipped.toml` | none yet | planned | See [backlog row 371](../backlog/371-dependency-evaluation-keep-lift-drop.md) and the research |
| C17 | `cargo-vet` as the trust register: every (crate, exact version) in the lock is audited, imported, trusted with an end date, or an exemption with notes; `cargo vet --locked` offline from a committed `imports.lock`; the exemption count may only fall | T1: a new crate, a new version, a new transitive dependency of an existing crate | `scripts/check-cargo-vet.sh` | `supply-chain/config.toml`, `supply-chain/audits.toml`, `supply-chain/imports.lock` | none yet | planned | After C16 is green; row 371 |
| C18 | Pinned advisory database plus a networked refresh step (`supply-chain-refresh.sh`, never in the gate list) that advances the pin, runs advisories, re-imports audits, and checks locked versions against the registry index for yanks | T1: known vulnerabilities and yanked or deleted versions, without making the gate time-dependent | `scripts/supply-chain-refresh.sh` | `supply-chain/advisory-db.rev` | none yet | planned | Row 371; the gate only reads snapshots so one tree always gives one verdict |
| C19 | `check-pnpm-hardening.sh`: both `pnpm-workspace.yaml` keep `minimumReleaseAge >= 43200`, `trustPolicy: no-downgrade`, `blockExoticSubdeps: true`, `onlyBuiltDependencies: []`; every `package.json` dependency is an exact version; no `tarball:`/git resolution in the lock; `packageManager` pnpm >= 10.21 | T1 T2: the JS settings and exact pins are today a convention and a yaml comment, not a gate | `scripts/check-pnpm-hardening.sh` | `playwright/pnpm-workspace.yaml`, `preview/pnpm-workspace.yaml`, `playwright/package.json`, `preview/package.json` | none yet | planned | Row 371; research E |
| C20 | Dependency register: one schema-checked TOML entry per direct dependency (outcome, reach class, version, evidence date, closure hash); a version bump re-opens the entry; transitive, `build.rs` and proc-macro counts are shrink-only ratchets; `lifted-from: crate@version` headers must have an entry | T4: closure growth and lifted code that no advisory scan can see | `scripts/check-supply-chain-register.sh` | `supply-chain/register.toml` | none yet | planned | Row 371; follow the repo's shrink-only baseline pattern |
| C21 | Users' cooldown: the starter's `.cargo/config.toml` sets Cargo's minimum publish age once Rust 1.100 is stable (expected 2026-11-12); the only control that would have covered the 86-minute window of the August 2026 attack | T1 for users | none | `templates/starter/.cargo/config.toml` | none yet | planned | Unverified: that older cargo ignores the unknown key; row 371 |
| C22 | This document stays true: the controls table is parsed, enforced and manual rows must name existing paths, planned scripts must not exist yet, and every script in the declared supply-chain set must appear in the table | drift of this playbook from the tree | `dev-docs/supply-chain/check-supply-chain-doc.sh`, `scripts/check-supply-chain-doc.sh` | `dev-docs/supply-chain/README.md`, `SECURITY.md` | run-gates | enforced | Copy the gate (it lives next to this file) with the table format; edit its CONFIG block (playbook path, `SECURITY.md`, gate-list file, `SUPPLY_CHAIN_SCRIPTS`). `scripts/check-supply-chain-doc.sh` is only this repo's three-line wrapper |

Counts (checked by the gate, which prints them): 13 enforced, 3 manual, 6 planned.

## Decisions recorded

- **No vendoring (`cargo vendor`), decided 2026-10-23** ([row 371](../backlog/371-dependency-evaluation-keep-lift-drop.md)). Vendored
  code needs the same security checks as the crate it copies; `Cargo.lock` checksums plus crates.io's immutable versions already
  pin the exact bytes; `cargo vet diff` reviews each update without copying source; a yanked version still builds from the
  lock; crates.io deletes only malware, which a vendored copy would keep. Lift (copy a small slice into our source) only
  where it protects users, and then scan the lifted code like a dependency: a `lifted-from: crate@version` header,
  advisory match, `unsafe` and licence checks (C20).
- **cargo-deny first, cargo-vet second** (research 2026-10-23). deny answers bans, sources, licences and advisories in one
  tool; vet adds exact-version trust that deny's name-level allowlist cannot. `cargo-audit` is a strict subset of deny's
  advisories check, so not both. OSV-Scanner, cargo-geiger, cargo-crev, Socket, Dependabot and Renovate are not in the gate
  list, with reasons per tool in [`research/dependency-scanners.md`](../research/dependency-scanners.md).
- **Gates are offline and deterministic.** Facts that change with the calendar (new advisories, yanks, publisher changes)
  live in a separate networked refresh step that writes a committed snapshot; the gate reads the snapshot (C18).
- **Ratchets, not budgets.** A budget with headroom creeps; a baseline that may only shrink, with reasoned allowlists and
  failing stale entries, does not (the repo's pattern in `check-cross-component-overrides.sh`, `check-props-docs.sh`).
- **The ladder for a dependency** (row 371, cheapest and most reversible first): drop, feature-gate, pin exactly, replace
  (std, `web-sys`, browser API), lift, keep. Time zones, colour science, parsers and the framework are vetoed from lifting;
  the veto does not exempt them from the code-risk controls (parsers get an exact pin, `default-features = false` and an opt-in
  component, C03 C04).
- **The CI freeze is not a security control.** It was set for cost ("off until further notice", row 33); the security effect
  (T10) is incidental, and the playbook says so rather than claiming a control.

## Planned work

Everything marked `planned` above comes from [backlog row 371](../backlog/371-dependency-evaluation-keep-lift-drop.md) (parked by
the owner 2026-10-23, nothing approved) and [`research/dependency-scanners.md`](../research/dependency-scanners.md) (which also
carries the recommended rollout order: cargo-deny with `bans`, `sources` and `licenses`; the advisory pin and refresh script;
cargo-vet; register and ratchets; the pnpm gate; the starter cooldown). The measurements behind it are in
[`research/dependency-evaluation.md`](../research/dependency-evaluation.md), re-runnable with `scripts/dep-report.sh`. Items the
research marked "(verify)" were not confirmed and need a spike before a gate is written.

**Not measured** (recorded in row 371): wasm bytes per crate, compile-time deltas, complete RustSec advisory lists, real
`unsafe` counts.

## Adopt this in your project

The minimal order. Each step is independently useful; stop where the cost stops paying.

1. **`SECURITY.md`** (root). Copy [`SECURITY.md`](../../SECURITY.md); change the repository name, the supported-versions
   table, and enable private vulnerability reporting in the repository settings (Settings, Code security).
2. **A gate list and a runner.** Copy [`scripts/run-gates.sh`](../../scripts/run-gates.sh)'s idea, not necessarily the file: one
   array of `name|command`, one entry point, a guard that fails when a `scripts/check-*.sh` is not in the list, and a summary
   that names the tree it ran on. Without the single list nothing below is enforced.
3. **Lock hygiene (C01).** Copy `scripts/check-cargo-locks.sh`; no edits. Commit every `Cargo.lock`.
4. **JS settings (C09, C10).** In each folder with a `package.json`: a `pnpm-workspace.yaml` with
   `minimumReleaseAge: 43200` (minutes, 30 days; pnpm 10.16 or newer) and `onlyBuiltDependencies: []`, exact versions in
   `package.json`, `"packageManager": "pnpm@10.x.y"`, and install only with `pnpm install --frozen-lockfile`. Planned
   hardening to add at the same time: `trustPolicy: no-downgrade` (pnpm 10.21 or newer), `blockExoticSubdeps: true`.
5. **Manifest floors and pins (C02, C03).** If you ship manifests that name crates and versions to other projects, copy
   `scripts/check-cargo-dep-versions.sh`; edit `EXACT_ONLY`, `PARSER_ALLOW` and `REGISTRY_CRATES` and the manifest glob.
   If you do not, the same rule applies to your own `Cargo.toml` floors.
6. **Generated files (C07).** If you commit generated code, copy `scripts/check-js-bundle.sh` and
   `scripts/build-js-bundle.sh`; set the source and output directories and the bundler version recorded in the manifest.
7. **Feature isolation (C05).** If a library has optional features, copy `scripts/check-primitives-features.sh`; point it
   at the crate manifest.
8. **Untrusted-input component (C04).** Only if you parse untrusted text: copy `scripts/check-markdown-parser.sh` as the
   pattern (fixed crate set, exact pin, allowlisted parser options, a lock allowlist for the parser's dependencies).
9. **Registry and install checks (C06, C08, C11, C12).** Only for a `dx components` or git-sourced registry; they are
   specific to that layout.
10. **No third-party runtime fetches (C13).** Copy `scripts/check-demo-remote-assets.sh`; edit the roots.
11. **Evidence (C14).** Copy `scripts/dep-report.sh` to get the numbers (crates, `unsafe`, `build.rs`) a dependency decision needs.
12. **The drift gate (C22).** Copy this file's table format and `check-supply-chain-doc.sh` (next to this file); edit the
    CONFIG block at its top (playbook path, `SECURITY.md`, gate-list file, `SUPPLY_CHAIN_SCRIPTS`); register it in your gate list.
13. **Then the planned layer, in the order of the research**: `cargo-deny` (C16), a pinned advisory database with a
    refresh step (C18), `cargo-vet` (C17), a register with ratchets (C20), the pnpm gate (C19).

What each file needs changed is in the **Adopt** column of the table; every script documents its rules in its own header comment.

## Extracting this into its own repo

The owner may move this to a repository of its own. It is built so that `git subtree split --prefix=dev-docs/supply-chain`
yields a usable root: everything new lives in this directory (this README and `check-supply-chain-doc.sh`), and the gate
reads every repository-specific path from one CONFIG block at its top.

**What moves** (already in `dev-docs/supply-chain/`): this playbook and the drift gate. Nothing else is moved by the split.

**What stays as this repository's wiring** (and is only *described* here, not moved):
- the enforcing scripts and tool settings in the controls table (`scripts/check-*.sh`, `scripts/registry-install-smoke.sh`,
  `scripts/stamp-release-rev.sh`, `scripts/dep-report.sh`, `scripts/session-setup.sh`, the `pnpm-workspace.yaml` files and
  the workflows): they are specific to this repository's layout (`dx components` registry, Dioxus primitives). In an
  extracted repo they would become the **reference implementations** the table points at, copied into a `reference/` folder
  or linked by commit, which is a later decision;
- `scripts/check-supply-chain-doc.sh`, the three-line wrapper that `scripts/run-gates.sh` lists, and the gate-list entry;
- the root `SECURITY.md` (each project keeps its own), the links from `CLAUDE.md`, `README.md`, `dev-docs/README.md` and
  backlog row 371, and `dev-docs/research/dependency-*.md` which this playbook cites.

**What to change after a split**: in the gate's CONFIG block set `SCD_GATE_LIST=""` (or the new repo's gate list), point
`SCD_SECURITY` at the new repo's policy, and list the new repo's own scripts in `SUPPLY_CHAIN_SCRIPTS`; the table's
script and config paths are repository-relative, so they are rewritten to the reference copies. Relative links in this
README that leave the directory (`../../SECURITY.md`, `../backlog/`, `../research/`, `../../CLAUDE.md`, `../README.md`) will
not resolve in the extracted repo and must be turned into absolute links to this repository or into plain text; they are
this repository's interlinking (the docs-map and backlog links), deliberately kept while the playbook lives here.

## Keeping this document true

`check-supply-chain-doc.sh` (next to this file; `scripts/check-supply-chain-doc.sh` is the wrapper the gate list names) is a gate (it needs no network and no build). It parses the controls table above and fails when:

- an `enforced` or `manual` row names a script or config path that does not exist;
- a `planned` row names a script that already exists (the row must be flipped to `enforced`);
- a script in the declared set `SUPPLY_CHAIN_SCRIPTS` in the gate's CONFIG block is not named by any row (a new control cannot land
  without documentation), or a script in that set no longer exists;
- a row has a status other than `enforced`, `manual` or `planned`, an id is repeated, or the required headings are missing;
- `SECURITY.md` is missing or does not link this playbook.

`check-supply-chain-doc.sh --self-test` mutates an in-memory copy of the table against a throwaway tree and requires
every mutation to turn red (and the unmutated table to stay green). The declared set is a list kept in the gate's CONFIG block, so adding a
supply-chain script means adding it to that list and to the table in the same commit.

Facts this document relies on and that were **not verified by a script**: the 2026-08-20 incident details and the planned
tools' behaviour come from [`research/dependency-scanners.md`](../research/dependency-scanners.md) (crates.io API and tool
documentation read 2026-10-23; GitHub and rustsec.org were blocked in that session); whether older cargo ignores the
`min-publish-age` key; that pnpm enforces `minimumReleaseAge` on a `--frozen-lockfile` install of an already-locked version
(the setting is documented for resolution; the lock itself is what a frozen install trusts).
