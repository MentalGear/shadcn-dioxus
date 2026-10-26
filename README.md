<div align="center">
  <h1>shadcn-dioxus</h1>
  <p><strong>Accessible, customizable components for Dioxus.</strong></p>
</div>

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/dioxus-primitives">
    <img src="https://img.shields.io/crates/v/dioxus-primitives.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/dioxus-primitives">
    <img src="https://img.shields.io/crates/d/dioxus-primitives.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs -->
  <a href="https://docs.rs/dioxus-primitives">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

---

<br/>

shadcn-dioxus is a shadcn style component library for Dioxus built on top of the unstyled [Dioxus primitives](https://crates.io/crates/dioxus-primitives) library. The unstyled primitives serve as the foundation for building accessible and customizable UI components in Dioxus applications. The styled versions serve as a starting point to develop your own design system.

**Live preview:** browse the component gallery at [mentalgear.github.io/shadcn-dioxus](https://mentalgear.github.io/shadcn-dioxus/) — individual component pages live at `https://mentalgear.github.io/shadcn-dioxus/component/<name>/`.

> **Roadmap: the remaining JavaScript moves into Rust.** The components work today. A few browser behaviours (popover placement, scroll lock, focus on open, the carousel and the message scroller) still run as small JavaScript snippets inside the Rust code. They are being moved into Rust, with typed TypeScript only where the browser leaves no other way. Each port keeps the components' props and behaviour, and is measured against the code it replaces, so an app that uses the components does not change; a component you add again after a port gets the new version. The plan is [`dev-docs/js-interop.md`](./dev-docs/js-interop.md), tracked as [backlog row 434](./dev-docs/backlog/434-javascript-in-typescript-modules-not-rust-strings.md) and rows 444-467 (approved 2026-10-26) of the [backlog](./dev-docs/backlog.md).

## Getting started

First, explore the [component gallery](https://mentalgear.github.io/shadcn-dioxus/) to find the components you want to use.

Install the `shadcn-dioxus` command-line tool once. It copies a component's source and stylesheet into your app, adds the crates it needs to `Cargo.toml`, installs the components it depends on, and reads **this** registry by default, so no command below needs a `--git` flag:

```sh
cargo install --git https://github.com/MentalGear/shadcn-dioxus shadcn-dioxus --locked
```

```sh
shadcn-dioxus new myapp      # new project, from the starter template (no dx needed)
shadcn-dioxus init           # existing project (writes the registry into Dioxus.toml)
shadcn-dioxus add button
shadcn-dioxus update         # never overwrites files you edited (--force does)
```

`shadcn-dioxus list` shows everything available and `shadcn-dioxus remove <name>` uninstalls a component. The full reference (what `update` keeps and replaces, which registry is read, `--path`/`--git`/`--rev`) is in [`cli/README.md`](./cli/README.md).

### New project

```sh
shadcn-dioxus new myapp
cd myapp
shadcn-dioxus add button
```

The starter declares `mod components;` in `src/main.rs`, depends on `dioxus-primitives` from this repository and names this registry in its `Dioxus.toml`. After your first add, link the shared theme once in `App` with `document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }`.

### Existing project

Run `shadcn-dioxus init` once from the project root. It writes `[components.registry]` into `Dioxus.toml` (keeping your comments), creates `src/components`, declares `mod components;` in `src/main.rs`, copies the shared theme to `assets/dx-components-theme.css` and links it from `App`. It is safe to run again. Then `shadcn-dioxus add button`.

`shadcn-dioxus update` brings installed components up to the registry's latest. It records a hash of every file it writes, so a component with a file you edited is left unchanged and listed (the command exits non-zero); `--force` replaces it anyway.

### Without the CLI

The registry is also a plain `dx components` registry, if you would rather not install another tool. If you do not already have `dx`:

```
cargo install dioxus-cli
```

Any one of these works:

- **A new project from the starter template** (the same project `shadcn-dioxus new` makes):

  ```
  dx new myapp --template https://github.com/MentalGear/shadcn-dioxus --subtemplate templates/starter
  cd myapp
  dx components add button
  ```

  (Add `--yes` to skip the prompts.) The starter's `Dioxus.toml` names this registry, so the plain `dx components add button` reads it.

- **Name the registry once in your app's `Dioxus.toml`**, then use plain `dx components add <name>`:

  ```toml
  [components.registry]
  git = "https://github.com/MentalGear/shadcn-dioxus"
  ```

- **Pass the registry on every command:**

  ```
  dx components add button --git https://github.com/MentalGear/shadcn-dioxus
  ```

  `dx components list --git https://github.com/MentalGear/shadcn-dioxus` shows everything available. Keep the URL exactly as written, since the components that depend on each other (and on `dioxus-primitives`) name it too.

**Warning:** plain `dx components add <name>` without one of those setups reads dx's default registry (upstream `DioxusLabs/components`) and installs *upstream's* version of the component, not this one. The `--git` flag or the `Dioxus.toml` setting is what points it here.

Either way the first add creates a `components` folder in your project (if it doesn't already exist), adds the component files to it, adds the `dioxus-primitives` dependency to your `Cargo.toml`, and copies the shared theme to `assets/dx-components-theme.css`. With `dx`, declare the module with `mod components;` in your `main.rs` and link the theme once in your root component with `document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }`; `shadcn-dioxus init` does both for you.

**Text inside components.** A component does not take its host's text styling: its root resets line height, weight, letter spacing and the other inherited text properties to the theme, and keeps only the font family, colour and direction of your page. The line height it resets to is `--dx-root-line-height` (default `normal`). If your app sets another body leading, as Tailwind's preflight does (`1.5`), set it once so text inside components matches the page around them:

```css
:root { --dx-root-line-height: 1.5; }
```

## Motion

Animation speed, reduced motion and motion nobody can see (loops pause, timers stop) are handled by the theme and a few hooks; the tokens, the hooks and how to write an animated component are in [`dev-docs/motion.md`](./dev-docs/motion.md), and on the site under [Docs, "Motion"](https://mentalgear.github.io/shadcn-dioxus/docs/#motion).

## Contributing

### Project structure

This repository contains two main crates:
- `dioxus-primitives`: The core unstyled component library.
- `preview`: A Dioxus application that showcases the components from `dioxus-primitives` with shadcn-styled versions.

### Adding new components

If you want to add a new component, you should:
1. If there is any new interaction logic or accessibility features required, implement an unstyled component in the `dioxus-primitives` crate. When adding components to the primitives library, ensure:
    - It adheres to the [WAI-ARIA Authoring Practices for accessibility](https://www.w3.org/WAI/standards-guidelines/aria/).
    - All styling can be modified via props. Every element should spread attributes and children from the props
2. In the `preview` crate, create a styled version of the component using shadcn styles. This will serve as an example of how to use the unstyled component and serve as the styled version `shadcn-dioxus add` copies into projects.
3. Add tests in `playwright` to ensure the component behaves as expected.

### Testing changes

The fast loop, in the order you need it (the measured details are in
[`dev-docs/dev-loop.md`](./dev-docs/dev-loop.md) and
[`dev-docs/lane-rules.md`](./dev-docs/lane-rules.md#the-fast-loop-dx-serve-to-iterate-testchanged-to-verify-owner-2026-10-21-row-326)):

```sh
# once: the locked Node dependencies, with pnpm (never `pnpm add`/`update` here, it can pull a Playwright without a
# matching browser; versions younger than 30 days are refused and no install script runs, backlog row 359)
(cd playwright && pnpm install --frozen-lockfile) && (cd preview && pnpm install --frozen-lockfile)

# 1. iterate: dev server with hot reload; wait for YOUR edit to be built before you look
#    (an HTTP 200 proves nothing, see dev-docs/dx-serve-hot-reload.md)
(cd preview && dx serve --web --port 8080 > /tmp/dx-serve-8080.log 2>&1 &)
scripts/dev-wait.sh 8080 --log /tmp/dx-serve-8080.log

# 2. every gate before every commit: the static checks, fmt, stylelint, the three clippy arms, rustdoc, cargo test
#    (green = "ALL GREEN" and exit 0; the target dir must be an absolute path; --fast skips the cargo gates)
CARGO_TARGET_DIR=$PWD/target-gates scripts/run-gates.sh

# 3. the Playwright specs that cover YOUR diff, on a prerendered build (what users get, hydration included)
CARGO_TARGET_DIR=$PWD/target-ssg scripts/build-ssg.sh debug      # prints PUBLIC_DIR=...
scripts/serve-spa.py <PUBLIC_DIR> 8741 &
cd playwright && PLAYWRIGHT_BASE_URL=http://127.0.0.1:8741 pnpm run test:changed -- --workers=2
node tools/select.mjs --explain                                  # which specs, and why
```

The whole suite (about 3,600 tests) runs once per merge batch as duration-balanced chunks:
`playwright/tools/run-suite.sh --out <dir>`. A component change also needs the install check,
`scripts/registry-install-smoke.sh`. Unit tests alone: `cargo test -p dioxus-primitives`.

Most specs also run an [axe-core](https://github.com/dequelabs/axe-core) static accessibility scan (valid ARIA, accessible names, unique landmarks, contrast, heading order, …) via the shared `playwright/axe.ts` helper — see [`dev-docs/conformance-harness.md`](./dev-docs/conformance-harness.md), "axe (static rules)", for what it covers versus the behaviour oracles and its exclusion policy.

Local-only Playwright configs, for driving the suite against an
already-running server instead of letting Playwright's own `webServer` block manage one: `baseline.local.config.ts` (full-suite
runs), `xvfb.local.config.ts` (headed Chromium under a virtual X server, for
tests that need a real, space-reserving scrollbar rather than headless
Chromium's 0-width one), and `ssg.local.config.ts` (points at a plain static
file server serving the fullstack-SSG-prerendered build rather than the dev
server — see [`dev-docs/conformance-harness.md`](./dev-docs/conformance-harness.md),
"Hydration/deployment parity", for the full build-and-serve recipe this
covers, including `oracle/hydration-parity.spec.ts`). When running any of
these under `root` (as in a container), the touch/mobile-emulation oracle
specs need Chromium launched with `--no-sandbox` — see
`baseline.local.config.ts`'s `launchOptions` for the pattern.

Source-level guard scripts enforce conventions that would otherwise regress one component or change at a time. There is no CI: `scripts/run-gates.sh` runs all of them, and [`dev-docs/gates.md`](./dev-docs/gates.md) (generated from the scripts' headers) says what each one checks and why. For a quick pass without the cargo gates, `scripts/run-gates.sh --fast`.

### Running the preview

To test your changes, you can run the preview application. For a desktop build, use:

```sh
dx serve -p preview --desktop
```

or for the web build:

```sh
dx serve -p preview --web
```

### Deploying the preview / docs site

GitHub Pages is configured to serve straight from `main`'s `/docs` folder
— there is no CI build step. To publish a new preview build:

```sh
scripts/deploy-preview.sh
```

This builds the `preview` app in release mode and overwrites `/docs` with
the output. Review the result (`git status`, `git diff --stat -- docs`),
then commit and push to `main`.

## Security

Report a vulnerability privately as described in [`SECURITY.md`](./SECURITY.md); how the dependency and repo supply chain is defended (threat model, controls, how to copy the approach) is in [`dev-docs/supply-chain/README.md`](./dev-docs/supply-chain/README.md).

## License

This project is dual licensed under the [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE) licenses.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository, by you, shall be licensed as MIT or Apache 2.0, without any additional terms or conditions.
