import type { Page } from "@playwright/test";

/**
 * Navigate and wait until the page is actually interactive, not merely
 * loaded. `waitUntil: "networkidle"` alone is NOT sufficient under the SSG
 * lane: a prerendered page's markup (buttons, the carousel's slides, ...)
 * already exists in the HTML the server sent, before the wasm bundle has
 * finished loading, parsing and hydrating -- Dioxus's event delegation only
 * attaches once hydration walks the tree, and a DOM event issued before
 * that (a click, a `mouseenter`) is dropped, not queued, because nothing is
 * listening yet. "networkidle" only says the network went quiet; it says
 * nothing about whether hydration itself has completed, and a fast-booting
 * release build can go idle (no more in-flight requests) while still mid
 * hydration. This repo has no other hydration-ready signal -- every SSG
 * spec today either accepts the race (and, in carousel.spec.ts's case,
 * silently lost clicks/hovers issued right after goto) or gets lucky on
 * timing.
 *
 * The fix, by construction: `preview/src/main.rs`'s root `App` component
 * runs a `use_effect` with no signal reads -- so it fires exactly once,
 * and only after a render has actually committed to the DOM -- that sets
 * `document.documentElement.dataset.hydrated = "true"`. `use_effect`
 * bodies only ever run client-side, after wasm has booted and hydration
 * has attached listeners, so this attribute cannot appear before the page
 * is truly interactive. `gotoHydrated` waits for it after the usual
 * `networkidle` goto, giving every SSG interaction test one real signal
 * instead of network-quiet-as-a-proxy-for-hydrated.
 *
 * LEGACY REDIRECT SHELL. A `?name=` URL (`/component/?name=x&`, `/component/block/?name=x&variant=y&`) is
 * the one static file a plain file host has for it: a name-agnostic "Loading component..." shell that
 * redirects client-side once mounted (`ComponentDemo`/`ComponentBlockDemo` in preview/src/main.rs). The
 * flag is set by that shell, so for such a URL the helper also waits for the redirect to land on the
 * real route. This is how a non-Block component's variant page
 * (`/component/block/<name>/<variant>/`, prerendered for Block components only) is reached on a static
 * host; a spec names a component page by its path route (`check-spec-goto-hydrated.sh`).
 *
 * Under `dx serve` (client-rendered, no SSR/hydration step), the same
 * `use_effect` still runs after the first client render, so this helper
 * works there too -- it just resolves almost immediately since there is no
 * separate hydration pass to wait through.
 */
export async function gotoHydrated(
  page: Page,
  url: string,
  opts?: Parameters<Page["goto"]>[1],
): Promise<void> {
  await page.goto(url, { waitUntil: "networkidle", ...opts });
  await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });
  if (/[?&]name=/.test(url)) {
    const timeout = opts?.timeout;
    await page.waitForURL((u) => !/\/component\/(block\/?)?$/.test(u.pathname), { timeout });
    await page.waitForFunction(() => !document.querySelector(".dx-component-demo-redirect"), null, { timeout });
  }
}
