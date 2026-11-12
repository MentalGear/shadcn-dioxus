# Build environment and the fast loop

Read this before you run cargo or dx, build the SSG, start `dx serve`, or run Playwright.

- **Build environment** ([`lane-rules.md`](../dev-docs/lane-rules.md), "Build environment"): every `CARGO_TARGET_DIR` is
  an **absolute, isolated** path, never the shared repo-root `target/` (a relative one breaks `dx build --ssg`, a shared
  one contaminates base paths); build the SSG only through `scripts/build-ssg.sh`; lane cargo runs with
  `CARGO_INCREMENTAL=0`; lanes that share or copy a target touch their sources before every cargo or dx run; worktrees
  symlink or `pnpm install --frozen-lockfile` the `node_modules`, never `pnpm add`/`update` or `npm install`.
- **The fast loop** ([`lane-rules.md`](../dev-docs/lane-rules.md), "The fast loop", owner 2026-10-21): iterate on
  `dx serve` (own warm target copy, own port, `scripts/dev-wait.sh` after each edit), verify with the diff-based
  `cd playwright && pnpm run test:changed` on one `lane-ssg` build, and leave the full suite to the merge batch.
- If you are about to trust a `dx serve` result, read
  [`dev-docs/dx-serve-hot-reload.md`](../dev-docs/dx-serve-hot-reload.md) first: a plain HTTP 200 does not prove your
  edit is served, and a new component directory needs `preview/build.rs` rewritten, not touched.
- Functional specs may run on debug SSG builds; timing specs only on release
  ([`dev-loop.md`](../dev-docs/dev-loop.md), "Running tests").
