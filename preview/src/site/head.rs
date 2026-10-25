use super::*;

/// Self-hosted Latin-subset variable fonts (SIL OFL, licence beside them), Geist 29 KB + Geist Mono 23 KB.
pub(crate) const GEIST_WOFF2: Asset = asset!("/assets/fonts/Geist-Latin-Variable.woff2");
pub(crate) const GEIST_MONO_WOFF2: Asset = asset!("/assets/fonts/GeistMono-Latin-Variable.woff2");
pub(crate) const LATIN_RANGE: &str = "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD";

/// The head links every route shares, in one place so the three route roots
/// cannot drift apart.
///
/// The Geist fonts are self-hosted (row 162): no request leaves the origin, and the preloads and the
/// `@font-face` rules are written here from the same `asset!` URLs (they cannot be in `main.css`, which
/// does not rewrite `url()` to the hashed names). Never `@import` a font from `main.css`: a stylesheet
/// whose `@import` is still loading is not applied at all (see the top of
/// `assets/main.css` and `playwright/oracle/tier2-html/global-stylesheet.spec.ts`).
///
/// **No font-swap layout shift (hard load, CLS summed):** with `display=swap` the
/// page paints in the fallback face and re-wraps when Geist arrives (0.117 on /component/form/). Two
/// things, together: the two LATIN woff2 files are `preload`ed from the head so they are ready by first
/// paint, and the faces say `font-display: optional`, so a face that is not ready in time keeps the
/// fallback for that view instead of swapping late. `crossorigin` is required on a font preload or the
/// browser fetches the file twice. `check-css-delivery.sh` holds this pair.
#[component]
pub(crate) fn GlobalHead() -> Element {
    // The `og:image` URL: the 512 px icon, absolute (see `SITE_ORIGIN`). `asset!()` already carries the
    // build's `--base-path`, so a root-relative path gets only the origin put in front of it.
    let icon = asset!("/assets/icon-512.png").to_string();
    let og_image = if icon.starts_with('/') {
        format!("{SITE_ORIGIN}{icon}")
    } else {
        icon
    };
    // `@font-face` cannot live in `main.css`: `asset!` hashes the font file names and does not rewrite a
    // stylesheet's `url()`, so the rules are written here from the same `asset!` URLs as the preloads.
    // `font-display: optional` is the CLS fix (see above); the tokens that name the families are in main.css.
    let font_face_css = format!(
        "@font-face{{font-family:Geist;font-style:normal;font-weight:100 900;font-display:optional;src:url({GEIST_WOFF2}) format(\"woff2\");unicode-range:{LATIN_RANGE}}}\
         @font-face{{font-family:\"Geist Mono\";font-style:normal;font-weight:100 900;font-display:optional;src:url({GEIST_MONO_WOFF2}) format(\"woff2\");unicode-range:{LATIN_RANGE}}}"
    );
    rsx! {
        // Icons go through `asset!()` (a static `<link>` in `index.html` is not prefixed with the base
        // path). The PNG comes first and the SVG last, so an engine that understands both takes the SVG,
        // which follows the browser's light/dark scheme itself (`@media (prefers-color-scheme)` inside
        // the file; it does not follow the site's own `data-theme` toggle).
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "32x32",
            href: asset!("/assets/favicon-32.png"),
        }
        document::Link {
            rel: "icon",
            r#type: "image/svg+xml",
            sizes: "any",
            href: asset!("/assets/favicon.svg"),
        }
        document::Link {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: asset!("/assets/apple-touch-icon.png"),
        }
        document::Meta { name: "description", content: SITE_DESCRIPTION }
        document::Meta { property: "og:site_name", content: SITE_NAME }
        document::Meta { property: "og:title", content: SITE_NAME }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:description", content: SITE_DESCRIPTION }
        document::Meta { property: "og:image", content: "{og_image}" }
        document::Meta { name: "twitter:card", content: "summary" }
        document::Meta { name: "twitter:title", content: SITE_NAME }
        document::Meta { name: "twitter:description", content: SITE_DESCRIPTION }
        document::Meta { name: "twitter:image", content: "{og_image}" }
        // Self-hosted fonts (row 162): the two preloads and the `@font-face` rules name the SAME hashed
        // `asset!` URLs, so the preload is always the file the face asks for. No request leaves the origin.
        document::Link {
            rel: "preload",
            r#as: "font",
            r#type: "font/woff2",
            crossorigin: "anonymous",
            href: GEIST_WOFF2,
        }
        document::Link {
            rel: "preload",
            r#as: "font",
            r#type: "font/woff2",
            crossorigin: "anonymous",
            href: GEIST_MONO_WOFF2,
        }
        document::Style { "{font_face_css}" }
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        // Optional effects (`dx-scroll-fade`, `dx-shimmer`) the chat kit (attachment, marker) uses.
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-effects.css") }
        // Docs-only: shadcn's theme customizer as `[data-theme-*]` blocks. Never part of the
        // installable theme (`component.json` ships only the file above); see the file's header.
        document::Link { rel: "stylesheet", href: asset!("/assets/theme-presets.css") }
    }
}
