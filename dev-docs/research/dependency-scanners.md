# Dependency scanners and criteria review (researched 2026-10-23, read-only)

Sources: crates.io API (versions/licences/dates), cargo-deny and cargo-vet books, pnpm docs, OSV-Scanner docs, Socket and
The Hacker News write-ups of the 2026-08-20 crate compromise, the Cargo min-publish-age RFC 3923. GitHub API was blocked in
this session, so "maintenance health" below is crates.io release recency only (no issue/commit stats). Items marked
(verify) were not confirmed in docs and need a 10-minute spike before the gate is written.

## Decisions first

1. Adopt cargo-deny now as the one tool for four classes: bans (build-script and capability allowlists), sources, licences,
   advisories (pinned snapshot). One gate: `scripts/check-cargo-deny.sh`; config `deny.toml` (+ `supply-chain/deny.shipped.toml`).
2. Adopt cargo-vet second, after deny is green, as the machine-checked trust register for the lock closure. Exact
   (crate, version) keys make "new crate", "new version", "new transitive dep" red by construction, which deny's name-level
   allowlist cannot. Gate: `scripts/check-cargo-vet.sh` running `cargo vet --locked`.
3. Do NOT put OSV-Scanner, cargo-audit, geiger, crev, supply-chain, auditable, Socket, Dependabot or Renovate in
   `run-gates.sh`. Reasons per tool below. For pnpm use a static gate (`scripts/check-pnpm-hardening.sh`, no network).
4. Time-varying facts (advisories newer than the pin, yanked, deleted versions, publisher changes) live in a SEPARATE
   networked step, `scripts/supply-chain-refresh.sh`, that writes committed snapshot files. The gate only reads snapshots, so
   the same tree always gives the same verdict. A refresh may turn the gate red; that is the intended signal.
5. The register is machine-checked config (`deny.toml`, `supply-chain/*.toml`) plus a small `supply-chain/register.toml` for the
   judgement fields neither tool can hold (outcome, reach class, evidence). Prose only carries rationale.
6. Part B: criteria are right in shape; seven concrete changes in the last section (closure not direct deps, users' closure not
   the workspace, lifting hides code from advisories, range floors, time-varying split, ratchets, cooldown for users).

## Facts about this repo today (measured with `cargo metadata --locked --offline`)

- Workspace lock: 972 packages, 966 from crates.io, 0 git sources, 414 external crates, 73 with a build.rs, 42 proc-macros.
- 25 crate names resolve to 2+ versions, mostly `windows-*` (4 `windows-sys`, 3 `winnow`, 2 `rand`, 2 `thiserror`).
- Top licences all permissive (MIT/Apache, Unicode-3.0, Zlib, ISC, Unlicense); no MPL since `dirs` went (row 217).
- No `deny.toml`, no `supply-chain/`, none of cargo-deny/audit/vet/osv installed. Cargo/rustc 1.97.0 (no rust-toolchain file).
- The workspace closure is the docs site (desktop and native features included). A Button-only wasm user app is ~167 crates
  (row 217). Budgets must be measured on the users' closure, not on this 414.
- pnpm 10.28.0 both folders, `minimumReleaseAge: 43200` (30 d), `onlyBuiltDependencies: []`; no `trustPolicy`; 0 tarball/git
  resolutions in either lock.

## Part A: scanner evaluation

Columns: class caught | offline/db | install and own footprint | licence, health | noise | fit.

- cargo-deny 0.20.2 (2026-07-09, EmbarkStudios; 32 direct deps; MIT OR Apache-2.0; 6.7M downloads).
  Classes: advisories (vulns, unmaintained, unsound, yanked), bans (denylist, allowlist via `allow`, duplicates
  `multiple-versions`, wildcards, per-crate `features`, `bans.build`: `allow-build-scripts` name list, native-executable
  and script detection with SHA-256 bypass), licenses (SPDX allow list, `exceptions`, `clarify` hash-bound), sources
  (`unknown-registry`/`unknown-git` default WARN, `allow-registry`, `allow-git`, `required-git-spec`).
  Offline: `--disable-fetch`/`--offline`; then `maximum-db-staleness` (default P90D) is enforced, so staleness is a built-in
  failure. Db is a git clone under `db-path` (default `$CARGO_HOME/advisory-dbs`). Exit code is a per-check bitset
  (advisories 1, bans 2, licenses 4, sources 8). Install: `cargo install --locked cargo-deny@0.20.2` (minutes, once; MSRV 1.88)
  or prebuilt release binary (verify availability from this container). Noise: unmaintained advisories on dioxus
  desktop/gtk transitive crates will be real; set `unmaintained = "workspace"` and reasoned `ignore` with `expiry`.
  Fit: best. Deterministic given tree + db pin + config. Note `bans.build` stresses it is not a malware detector.
- cargo-audit 0.22.2 (2026-06-05, rustsec project; 12 deps; Apache-2.0 OR MIT; 13.7M downloads). Only advisories, lock-based,
  `--no-fetch/--db/--stale`. Strict subset of deny's advisories check on the same db. Choose deny, not both.
- cargo-vet 0.10.3 (2026-10-22, Mozilla; 35 deps; Apache-2.0/MIT; 0.69M downloads; active, released yesterday).
  Class: integrity and trust. Every (crate, exact version) in the graph must be audited (`safe-to-deploy`/`safe-to-run`), covered
  by an imported audit (Google, Mozilla, Bytecode Alliance feeds through `[imports.*]` URLs in `config.toml`, central
  registry.toml), a `trusted` publisher entry (mandatory end date), or an `[exemptions]` entry with exact version and
  notes. Files: `supply-chain/config.toml`, `audits.toml`, `imports.lock` (snapshot of imported audits).
  Offline: `cargo vet --locked` skips fetching imports and uses `imports.lock`; refresh = run without `--locked`, commit the
  lock diff. That is the "refresh step commits a snapshot" pattern natively. Caveat: an exemption means "trusted unreviewed";
  with 414 crates most entries start as exemptions, so the value is change detection plus imports, not review depth.
  Noise: every `cargo update` needs `cargo vet regenerate exemptions` or imports; acceptable because our lock changes
  deliberately. Verify: `--frozen`/offline cargo-metadata behaviour and `--filter-graph` to scope to the shipped closure.
- cargo-crev 0.27.1 (2026-04-12; 39 deps; MPL-2.0 OR MIT OR Apache-2.0; 0.11M downloads). Web-of-trust reviews on a
  proof-repo network; sparse coverage, needs network and an identity. Reject (cargo-vet imports cover the same ground with
  organisations that exist).
- cargo-geiger 0.14.0 (2026-10-12; 19 deps; Apache-2.0 OR MIT; MSRV 1.97). Counts unsafe per crate. Not pass/fail; nearly every
  dioxus/wasm-bindgen dep has unsafe. Use only as manual CODE-risk evidence for a register entry; never a gate. Cheap own
  gate instead: `#![forbid(unsafe_code)]` presence in our first-party crates (verify none needs unsafe).
- cargo-supply-chain 0.3.8 (2026-10-15, rust-secure-code; 13 deps; Apache-2.0 OR MIT OR Zlib). Lists crate publishers/owners from
  crates.io data (`update` fetches data, then offline queries). Informational; only useful in the refresh step to snapshot
  publisher sets (see "ownership change" below). Not a gate.
- cargo-auditable 0.7.7 (2026-10-02; 8 deps; MIT OR Apache-2.0). Embeds the dep list in built binaries so scanners can audit
  artefacts. Relevant only if the `shadcn-dioxus` CLI is released as a binary: `cargo auditable build` in the release script.
  No gate.
- OSV-Scanner (Google, Go binary, Apache-2.0; not on crates.io). Reads `Cargo.lock` and `pnpm-lock.yaml`. Offline:
  `--offline-vulnerabilities --download-offline-databases`, db per ecosystem from the `osv-vulnerabilities` bucket
  `<ECOSYSTEM>/all.zip`, dir via `OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY`. No bans/licence/source policy. The only
  one-tool answer for the JS lockfiles; but JS is dev tooling with no install scripts, 30-day age and exact pins, so the
  residual (known CVE in a dev tool) is low. Optional non-gating manual run; revisit if JS ever ships.
- RustSec advisory-db (rustsec.org; Rust Secure Code WG). It is the data under deny and audit; also exported as OSV and
  GitHub Advisory DB. Pin by git commit SHA (below). It lags brand-new malware: the Aug 2026 incident advisory was first
  worded "no evidence of use" and revised later.
- Socket.dev. SaaS scanner; Rust support announced 2025-07-31 (package search free; SBOM needs `Cargo.toml`+`Cargo.lock` and
  contacting them; GitHub app/CLI for Rust unconfirmed). Its AI scanner flagged the malicious `proc-macro1` crate at 07:29 UTC
  on 2026-08-20, 14 min after the first poisoned `arrayref` publish (07:15), before crates.io deleted anything (08:41).
  Network, nondeterministic, proprietary: not a gate; use as a human lookup before adding a direct dependency.
- Dependabot: `cooldown` (default-days default 3; separate semver-major/minor/patch days; not applied to security updates);
  cargo listed as supported but per-ecosystem cooldown support not confirmed from the extracted page. Opens PRs, needs
  Actions-adjacent plumbing under the CI freeze, adds PR noise. Not adopted. Enabling passive Dependabot security
  ALERTS in repo settings costs nothing (owner action; not a gate).
- Renovate: has `minimumReleaseAge` and `osvVulnerabilityAlerts`; hosted or self-run service, PR-based. Not adopted for the
  same reasons; also another thing to trust.
- pnpm own settings (docs, versions): `minimumReleaseAge` v10.16 (minutes; default 1440 only from v11, 0 before; applies to
  transitive); `minimumReleaseAgeExclude` names v10.16, patterns 10.17, versions 10.19; `trustPolicy: no-downgrade` v10.21
  (install fails if a package's trust evidence is weaker than an earlier release's; `trustPolicyExclude` 10.22,
  `trustPolicyIgnoreAfter` 10.27); `blockExoticSubdeps` v10.26 default true (only direct deps may be git/tarball);
  `trustLockfile` v11.3. `pnpm audit` queries the registry live: network, daily-changing, so advisory only. The repo is on
  10.28.0, so `trustPolicy` and explicit `blockExoticSubdeps: true` can be added today.
- Newer (2026): Cargo min publish age (RFC 3923, stabilisation merged 2026-08-28, on beta, due in Rust 1.100 on 2026-11-12):
  `[registry] global-min-publish-age = "7 days"`; skips young releases, FAILS rather than falls back (so yanking safe
  versions cannot steer onto a malicious one); versions already in `Cargo.lock` are kept; env
  `CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow` overrides one invocation; not for git/path deps. The 86-minute window of
  the August attack is far below any such cooldown. Not usable in gates on 1.97 until Rust 1.100.

Why the August 2026 incident matters (arrayref 0.3.10, internment 0.8.7, append-only-vec 0.1.9; account compromise): the
poisoned releases only ADDED a dependency on `proc-macro1` (typosquat) whose build.rs ran at build time, and the attacker
yanked 0.3.5-0.3.9 of arrayref so Cargo's yank warning pointed at 0.3.10. It was invisible at the direct-dependency level.
Which gates catch it: a closure-level new-crate check (deny `bans.allow`, vet), a new-build.rs check
(`allow-build-scripts`), a version-keyed check (vet), and a cooldown. A direct-dependency register alone catches none.

## Recommended wiring

A. `scripts/check-cargo-deny.sh` (new; add to run-gates and the CLAUDE.md sentence). Steps: require `cargo-deny` at the
   version in `supply-chain/tools.toml` else exit 1 with the install line (fail closed, tool missing is not green);
   ensure advisory db at the pinned commit; run
   `cargo deny --locked --offline --manifest-path <m> check bans licenses sources` and
   `cargo deny ... check advisories --disable-fetch` for the root manifest and for each detached lock (reuse
   check-cargo-locks' `git ls-files '*Cargo.lock'` loop); then a stricter run on `primitives/Cargo.toml` with
   `supply-chain/deny.shipped.toml`, `--target wasm32-unknown-unknown`, once per feature set that check-primitives-features
   already enumerates (verify `[graph]` `features`/`no-default-features` keys). Fails when: any new crate name not in
   `bans.allow` (shipped config), any build.rs crate not in `allow-build-scripts`, a denied capability crate
   (`reqwest`, `hyper`, `ureq`, `openssl-sys`, `native-tls`, `curl`, ... with `wrappers` naming the only parent allowed),
   a `bans.features` violation (tokio `fs`/`net`/`process`), a duplicate version not in a reasoned `skip` list,
   a wildcard requirement, a non-crates.io source or any git source other than the one reasoned `allow-git`
   (set `unknown-registry`/`unknown-git = "deny"`; defaults warn), a licence outside `allow`, an unused `allow` entry
   (`unused-allowed-license = "deny"`), an advisory in the pinned db (unmaintained scoped to `workspace`, ignores need
   `reason` and `expiry`), or db staleness over `maximum-db-staleness = "P45D"`.
   deny.toml sections: `[graph]`, `[advisories]` (db-path `.cache/advisory-db`, `yanked = "warn"`), `[licenses]`,
   `[bans]`, `[bans.build]`, `[sources]`.
B. Advisory db pinning: `supply-chain/advisory-db.rev` holds `<40-hex sha> <commit date>`. The gate clones/fetches that
   exact sha into the gitignored `.cache/advisory-db` only when the cache is missing or at another sha (cold container needs
   network once; no network and no cache = red with a clear message, never a silent skip), then runs with `--disable-fetch`.
   `scripts/supply-chain-refresh.sh` (human or `/loop`, networked, never in run-gates): advance the pin, run
   `cargo deny check advisories`, `cargo vet` (unlocked, rewrites `imports.lock`), and a yanked/deleted-version check of
   every locked version against the crates.io index; print findings; the diff of the rev file, `imports.lock` and snapshots
   is the commit. Why not network at gate time: results would change with the calendar, and the remote container is ephemeral.
   (Verify: whether deny accepts a `file://` db-url so the snapshot could be committed instead.)
C. `scripts/check-cargo-vet.sh`: `cargo vet --locked`. Fails on any (crate, version) in the lock without audit, import,
   trusted-publisher entry (with unexpired `end`) or exemption. Layout: `supply-chain/config.toml`
   (imports: Google, Mozilla, Bytecode Alliance; exemptions with `notes` stating outcome and the register id),
   `audits.toml` (our own audits of dependencies we keep as direct, in the shipped closure), `imports.lock` (committed).
   Initial `cargo vet init` creates ~414 exemptions; imports shrink them. Ratchet: a check that the exemption count may
   only fall (baseline file, the repo's usual pattern).
D. `supply-chain/register.toml`: one table per direct dependency of `primitives`, `component.json` `cargoDependencies`, `cli`:
   `outcome` (ladder), `reach` (shipped-to-users / users-binary (the CLI) / repo-only / dev-tool), `blast` (components
   pulling it, derived), `version`, `measured_at`, `measure_hash`. `check-supply-chain-register.sh` (can live inside
   check-cargo-dep-versions.py) fails on: direct dep without entry, entry whose `version` differs from the lock (re-open on bump),
   a new direct dep, `measure_hash` (hash of `cargo tree -e normal --target wasm32 -p dioxus-primitives -F <feature>`
   output per feature set) different from the recorded one (closure growth), counts over the ratcheted baseline
   (transitive crates, build.rs crates, proc-macros), and a `lifted-from:` header in the tree without a register entry.
E. `scripts/check-pnpm-hardening.sh` (static): both `pnpm-workspace.yaml` have `minimumReleaseAge >= 43200`, `trustPolicy:
   no-downgrade`, `blockExoticSubdeps: true`, `onlyBuiltDependencies: []`; every `package.json` dependency is exact; lock
   has no `tarball:`/`type: git` resolutions; `packageManager` pnpm >= 10.21. Replaces the manual comment in the yaml.
F. What it replaces: nothing outright. It moves the markdown-parser lock allowlist (check-markdown-parser) and
   check-cargo-dep-versions rules 2 and 4 toward `bans.deny`/`allow`, which could then be deleted from Python later; keep
   check-cargo-locks (lock freshness, a precondition of both tools), check-cargo-dep-versions (component.json is outside
   both tools' view), check-primitives-features, check-js-bundle.
G. Tool supply chain: both tools install `--locked` at the exact version recorded in `supply-chain/tools.toml`, added to
   `scripts/session-setup.sh`; do not take a release younger than 14 days (same principle as pnpm's 30).

## Part B: review of the criteria

Verdict: the split (CODE vs CHANNEL), the ladder, blast-radius weighting and "measured by a script" are right. These
changes make it hold by construction.

1. Register the CLOSURE, not only direct deps. The August attack changed no direct dep; it added one transitive crate with a
   build.rs. "New build.rs/proc-macro in the closure" is already in the draft; extend to "any new crate name or any new
   version in the lock" (cargo-vet does this exactly; deny `allow` does it by name). The direct-dep register is then the
   judgement layer on top, not the integrity layer.
2. Measure the shipped closure, not the workspace. Our lock (414 crates, 73 build.rs) is the docs site. What users pay for
   and trust is `primitives` per feature on wasm32 (~167 for Button-only) plus each `component.json` `cargoDependencies`.
   Reach classes need "users' app" vs "CLI binary users run" vs "repo-only" vs "dev tool"; budgets and strict bans apply only
   to the first two. The CLI runs with the developer's fs access: highest CODE reach.
3. Our lock never reaches users. Exact pins protect only what we declare. Users' cargo re-resolves every transitive crate
   within caret ranges, and `primitives/Cargo.toml` has `tokio = "1"`, `gloo-timers = "0.3"`, `serde = "1.0"` whose floors
   are not the tested lock version (only `time`/`palette` are). Extend check-cargo-dep-versions rule 1 to primitives'
   own `[dependencies]` (floor == lock version), and for CHANNEL risk ship users the control: a cooldown in the starter's
   `.cargo/config.toml` (`[registry] global-min-publish-age`, once Rust 1.100 is stable on 2026-11-12; verify that older
   cargo ignores the unknown key) and document it. This is the only measure that would have covered the 86-minute window.
4. Lifting trades one risk for another and the draft does not price it. A lifted slice vanishes from RustSec/cargo-vet
   visibility: advisories on upstream no longer reach us, and licence attribution becomes our job. Require a
   `// lifted-from: crate@version sha256` header and register entry, a gate that greps the pinned advisory-db for that crate
   name (new advisory after the lifted version = red), and a licence-notice check. Add to the ladder note: "lift only when
   we will also watch upstream".
5. Split gate checks by determinism. Tree-deterministic: build.rs/proc-macro/new-crate/source/licence/duplicate/count budgets.
   Time-varying: new advisory, yanked, deleted version (the attacker's yanks steered resolution), ownership change. The draft
   puts "new advisory" in the same gate; unpinned it flips red with no code change and flakes offline. Pin the db (B above);
   keep yanked/ownership in the refresh step with committed snapshots. Ownership change has no offline signal: snapshot
   publisher sets with cargo-supply-chain in refresh, or use vet `trusted` entries with mandatory expiry.
6. Budgets should be ratchets, not numbers. A budget with headroom creeps up; the repo's own pattern is shrink-only
   baselines with reasoned allowlists and stale-entry failure (cross-component-overrides, props-docs). Use that for transitive
   count, duplicate `skip` list, `ignore` and exemption counts. Do not gate wasm bytes (needs a full build, toolchain-
   dependent, slow); report bytes in refresh output and gate the deterministic proxy (crate count, build.rs count).
7. cargo-vet vs hand register: vet is the better construction for TRUST (version-keyed, expiring, shareable imports,
   machine-checked, snapshot built in). It cannot hold outcome/reach/evidence or measure cost, so keep the small
   `register.toml`. Be honest about vet's assurance: exemptions are "trusted unreviewed"; real audits are worth writing
   only for the few crates we keep in the shipped closure. Prose register: reject; schema-checked TOML only.

Smaller notes: `bans.build` bypass checksums and `allow-build-scripts` are by crate name; a new build.rs content in an
allowlisted crate passes deny and is caught only by vet's version key. `yanked = "deny"` is nondeterministic; keep `warn`
in the gate. The VETO list (parsers, tz, colour, framework) should say it exempts from LIFT, not from CODE-risk controls:
parser components already get exact pin, `default-features = false` and an opt-in component (row 213, E2); keep that bar
for any new veto-listed crate. The one allowed git source is `dioxus-primitives` for users (REGISTRY_GIT_URL); the repo itself
has none, so `unknown-git = "deny"` costs nothing.

Rollout order: (1) deny.toml with `bans` + `sources` + `licenses` and a reasoned `skip` baseline, measured against today's
lock; (2) advisory pin and refresh script; (3) vet init with exemptions and imports; (4) register.toml + ratchets; (5) pnpm
hardening gate; (6) starter cooldown after Rust 1.100. Spike first (verify items above, in a scratch worktree with the tools
installed via `cargo install --locked`).
