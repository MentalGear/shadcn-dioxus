import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";

/**
 * The kinds of page a spec navigates to, one helper each (backlog row 427): `scripts/check-spec-goto-hydrated.sh`
 * fails every other `.goto(`, so a spec says which kind it is looking at in code, and the helper checks its own claim.
 *
 *   - `gotoHydrated`    the interactive app (`hydration.ts`): waits for `html[data-hydrated]`.
 *   - `gotoPrerendered` the app, but the spec must NOT wait for hydration: it reads the prerendered markup or head
 *                       (JavaScript off, wasm aborted on purpose), samples the first paint or the stylesheets before
 *                       hydration, observes hydration itself, or settles by its own wait after the navigation
 *                       (`waitUntil: "commit"`). Nothing is awaited beyond what `page.goto` itself awaits (`opts`,
 *                       Playwright's default `load`). Returns the response, as `page.goto` does.
 *   - `gotoReference`   a page that is NOT the app: a vendored W3C reference page (`file://`), a route-fulfilled host,
 *                       a static fixture or `about:blank`. It never sets `html[data-hydrated]`, so there is nothing to
 *                       wait for; a locator on it is the reference's own markup, not demo copy.
 */
export async function gotoPrerendered(page: Page, url: string, opts?: Parameters<Page["goto"]>[1]) {
  if (/^(?:about:|file:)/i.test(url)) {
    throw new Error(`gotoPrerendered: ${url} is not the app; a reference or blank page goes through gotoReference`);
  }
  return page.goto(url, opts);
}

/** Origins the app is served from for this run (the base URL every spec and config reads). */
function appOrigins(): string[] {
  const out = new Set<string>();
  for (const raw of [BASE_URL, process.env.PLAYWRIGHT_BASE_URL]) {
    if (!raw) continue;
    try {
      out.add(new URL(raw).origin);
    } catch {
      /* not a URL: nothing to compare */
    }
  }
  return [...out];
}

export async function gotoReference(page: Page, url: string, opts?: Parameters<Page["goto"]>[1]) {
  let origin: string;
  try {
    origin = new URL(url).origin;
  } catch {
    throw new Error(`gotoReference: ${url} is not an absolute URL`);
  }
  if (appOrigins().includes(origin)) {
    throw new Error(`gotoReference: ${url} is under the app's origin (${origin}); the app's pages go through gotoHydrated or gotoPrerendered`);
  }
  return page.goto(url, opts);
}
