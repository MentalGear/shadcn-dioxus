# Merge drivers and the install check

Read this when you merge lane branches, or add, rename or change a component.

- **Merge drivers (row 400):** `dev-docs/backlog.md`, `dev-docs/backlog-done.md` and `playwright/durations.json` are
  generated or derived and merge through drivers named in `.gitattributes` (by row id, by spec entry), so merging lane
  branches into a batch no longer conflicts on them. Git reads a driver only from the clone's config: the SessionStart
  hook runs `scripts/session-setup.sh`, which runs `scripts/setup-merge-drivers.sh` (run it by hand in any other clone;
  worktrees share the config). `check-merge-drivers` keeps the files, the registration command and the behaviour in
  step.
- **Install check:** run `scripts/registry-install-smoke.sh` whenever a `component.json` or a component's
  `component.rs`/`mod.rs`/`style.css` changes, or a component is added or renamed (not in `run-gates.sh`: a cold check
  takes minutes and several GB; green is `registry-install-smoke: OK -- N/N components installed`).
