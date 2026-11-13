<div align="center">
  <h1>🎲 Dioxus Primitives 🧱</h1>
  <p><strong>Accessible, unstyled, foundational components for Dioxus.</strong></p>
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

Dioxus primitives is an ARIA-accessible, unstyled, foundational component library for Dioxus based on Radix Primitives. It defines the structure and behavior of components without imposing any specific styles. You can bring your own design patterns and styles to create a consistent look and feel for your application.

If you are looking for a starting point to develop a styled component library for your application, you can use our shadcn-style [dioxus components](https://dioxuslabs.github.io/components) or a community library.

## Cargo features

An app compiles only the crates its components use. `default` is `router`; `dx components add` always passes `--no-default-features` and then each component's own `features`.

| Feature | Enables | Extra crates |
|---|---|---|
| `native` | the Blitz arm (no JavaScript) of the overlay and listener code instead of the browser-engine one; also turns on `dioxus/native` | the Blitz renderer |
| `router` | `navbar` (needs `dioxus-router`) | about 40 |
| `calendar` | `calendar` and `date_picker` (their public types are `time::Date` and friends) | `time` and 5 more |
| `color-picker` | `color_picker` (its props use `palette::Hsv` / `Srgb<u8>`) | `palette` and 11 more |

Browser engines need no feature: a browser, a desktop or mobile webview and an SSG or fullstack server render the
browser-engine arm by default, and the `time` crate's `wasm-bindgen` feature (calendar) comes with the wasm target. A Blitz
app enables `dioxus-primitives/native` in the same feature that enables `dioxus/native`; a build where the two disagree
fails to compile, with a message that names the fix. `web` is an inert feature kept so older manifests still build.

## License

This project is dual licensed under the [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE) licenses.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository, by you, shall be licensed as MIT or Apache 2.0, without any additional terms or conditions.
