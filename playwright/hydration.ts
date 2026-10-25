import type { Page, Request } from "@playwright/test";

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
 * hydration. The page's `data-hydrated` attribute (below) is the one hydration-ready signal.
 *
 * The construction: `preview/src/main.rs`'s root `App` component
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
 * redirects client-side once mounted (`ComponentDemo` in preview/src/site/demos.rs, `ComponentBlockDemo` in
 * preview/src/site/block_demo.rs). The
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
  opts?: Parameters<Page["goto"]>[1] & {
    /** Bound for the `data-hydrated` wait (default: none of its own, the test's timeout). */
    hydratedTimeout?: number;
  },
): Promise<void> {
  const { hydratedTimeout, ...gotoOpts } = opts ?? {};
  const open = trackOpenRequests(page);
  try {
    await page.goto(url, { waitUntil: "networkidle", ...gotoOpts });
    await page.waitForSelector('html[data-hydrated="true"]', { state: "attached", timeout: hydratedTimeout });
    if (/[?&]name=/.test(url)) {
      const timeout = gotoOpts.timeout;
      await page.waitForURL((u) => !/\/component\/(block\/?)?$/.test(u.pathname), { timeout });
      await page.waitForFunction(() => !document.querySelector(".dx-component-demo-redirect"), null, { timeout });
    }
  } catch (error) {
    throw withOpenRequests(error, open.report());
  } finally {
    open.stop();
  }
}

/**
 * Row 257: a `networkidle` that never arrives says only "Timeout exceeded", so the request that kept the network busy
 * is never named. Every request of
 * every frame is recorded while `gotoHydrated` runs; on a failure the ones that neither finished nor failed are listed
 * under the error. That includes the TEST timeout: when it fires, the pending `goto`/`waitForSelector` rejects, the
 * error passes through here, and the report shows it beside Playwright's "Test timeout ... exceeded".
 */
function trackOpenRequests(page: Page) {
  const started = Date.now();
  const open = new Map<Request, number>();
  const onRequest = (r: Request) => open.set(r, Date.now());
  const onDone = (r: Request) => open.delete(r);
  page.on("request", onRequest);
  page.on("requestfinished", onDone);
  page.on("requestfailed", onDone);
  return {
    report(): string {
      const now = Date.now();
      const rows = [...open].map(([r, at]) => {
        const frame = r.frame();
        const where = frame === page.mainFrame() ? "main frame" : `frame ${frame.url() || frame.name() || "?"}`;
        return `  ${r.method()} ${r.resourceType()} (${where}, open ${now - at} ms): ${r.url()}`;
      });
      const spent = `${now - started} ms into gotoHydrated`;
      return rows.length
        ? `gotoHydrated: ${rows.length} requests still open (${spent}):\n${rows.join("\n")}`
        : `gotoHydrated: no request was open (${spent}), so the network was not what it waited on`;
    },
    stop() {
      page.off("request", onRequest);
      page.off("requestfinished", onDone);
      page.off("requestfailed", onDone);
    },
  };
}

function withOpenRequests(error: unknown, report: string): unknown {
  if (error instanceof Error) {
    error.message = `${error.message}\n\n${report}`;
    return error;
  }
  return new Error(`${String(error)}\n\n${report}`);
}
