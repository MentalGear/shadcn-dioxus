# Session start, fresh container, backlog rows

Read this in a fresh container where `dx` is missing, or before you edit a backlog row.

- [`dev-docs/STATUS.md`](../dev-docs/STATUS.md) holds the current round, what is merged and deployed, what runs, the
  next round and owner inputs pending; the main loop keeps it current. [`dev-docs/README.md`](../dev-docs/README.md) is
  the map of the docs.
- The backlog (`dev-docs/backlog.md` row N) is one file per row under `dev-docs/backlog/`; the index is generated: edit
  the row file, run `scripts/backlog-index.sh`, commit both.
- A fresh remote container is bootstrapped by the SessionStart hook (`scripts/session-setup.sh`: dx 0.7.9, wasm32, dx
  tools, `pnpm install --frozen-lockfile`, `cargo fetch`; ~8 s). If `dx` is missing, run the script by hand; never
  `cargo install dioxus-cli` (~21 min) or hand-seed the dx tools.
