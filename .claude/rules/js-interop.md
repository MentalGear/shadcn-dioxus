---
paths:
  - "primitives/src/**"
---
JavaScript interop: Rust first; browser APIs only through the bridge (`primitives/src/bridge/`); no new JS in Rust
strings; JS volume can only shrink. Read `dev-docs/js-interop.md` before writing interactive behaviour, and
`agent-rules/code-rules.md` for the comment and history rules.
