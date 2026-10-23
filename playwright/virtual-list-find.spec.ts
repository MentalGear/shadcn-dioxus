import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Backlog row 153 (prototype): a windowed VirtualList with `text_value` keeps every UNMOUNTED row's text
// in the DOM inside a `hidden="until-found"` placeholder. The browser's find-in-page can match it, and a
// reveal fires `beforematch` on that placeholder, which names the row; the list then scrolls the real row
// into view. Playwright cannot drive the browser's Ctrl+F bar; `window.find` searches the same text but
// does not reveal or fire `beforematch` (survey section 4, re-measured here), and a `#:~:text=` text
// fragment does both, so it stands in for the reveal path.

const URL = `${BASE_URL}/component/block/?name=virtual_list&variant=find_in_page`;
const list = (page: Page) => page.locator('[data-vl-demo="find-in-page"] [role="list"]');
const row = (page: Page, index: number) => list(page).locator(`[role="listitem"][data-virtual-index="${index}"]`);
const placeholder = (page: Page, index: number) => list(page).locator(`[data-find-index="${index}"]`);

/** Whether row `index` is mounted and fully inside the list's scrollport. */
async function rowInView(page: Page, index: number) {
  return list(page).evaluate((el, i) => {
    const r = el.querySelector(`[role="listitem"][data-virtual-index="${i}"]`);
    if (!r) return false;
    const a = r.getBoundingClientRect();
    const b = el.getBoundingClientRect();
    return a.top >= b.top - 1 && a.bottom <= b.bottom + 1;
  }, index);
}

test.describe("windowed find-in-page placeholders (row 153 prototype)", () => {
  test("unmounted rows are placeholders: findable text, no layout, hidden from the accessibility tree", async ({ page }) => {
    await gotoHydrated(page, URL);
    await expect(row(page, 0)).toBeVisible();
    const mounted = await list(page).getByRole("listitem").count();
    expect(mounted).toBeLessThan(60);
    // Every row has a placeholder, mounted or not: the layer never changes while the list scrolls.
    await expect(placeholder(page, 7776)).toHaveAttribute("hidden", "until-found");
    await expect(placeholder(page, 7776)).toHaveText("Row 7777 of 10000");
    await expect(placeholder(page, 0)).toHaveCount(1);
    expect(await list(page).locator("[data-find-index]").count()).toBe(10_000);
    // Not in the accessibility tree (until-found content is hidden)...
    expect(await list(page).ariaSnapshot()).not.toContain("Row 7777 of 10000");
    // ...but findable: window.find searches until-found content (the plain windowed list's equivalent
    // check in virtual_list.spec.ts is false), and scrolls the container to the match, so the window
    // follows and mounts the real row.
    expect(await page.evaluate(() => (window as unknown as { find: (s: string) => boolean }).find("Row 7777 of 10000"))).toBe(true);
    await expect.poll(() => rowInView(page, 7776)).toBe(true);
  });

  test("a beforematch on a placeholder scrolls the real row into view", async ({ page }) => {
    await gotoHydrated(page, URL);
    await expect(row(page, 0)).toBeVisible();
    // What the browser does on a find-in-page reveal, minus the find bar Playwright cannot open.
    await placeholder(page, 7776).evaluate((el) => {
      el.removeAttribute("hidden");
      el.dispatchEvent(new Event("beforematch", { bubbles: true }));
    });
    await expect.poll(() => rowInView(page, 7776)).toBe(true);
  });

  test("a revealed placeholder is hidden again, and the real row is marked as the match", async ({ page }) => {
    await gotoHydrated(page, URL);
    await expect(row(page, 0)).toBeVisible();
    // The browser removes `hidden` when it reveals a match. Left revealed, the placeholder's text
    // (and the find bar's active-match highlight on it) sat over the real row, offset by the
    // row's padding: measured with the real Ctrl+F bar in headed Chromium 153. Hidden again in
    // the next frame, find-next still walks on in order, forwards and back (same probe).
    const reveal = (index: number) =>
      placeholder(page, index).evaluate((el) => {
        el.removeAttribute("hidden");
        el.dispatchEvent(new Event("beforematch", { bubbles: true }));
      });
    await reveal(7776);
    await expect(placeholder(page, 7776)).toHaveAttribute("hidden", "until-found");
    await expect.poll(() => rowInView(page, 7776)).toBe(true);
    // The real row says it holds the match, so the styled layer can show it.
    await expect(row(page, 7776)).toHaveAttribute("data-find-match", "true");
    await reveal(7786);
    await expect(row(page, 7786)).toHaveAttribute("data-find-match", "true");
    await expect(list(page).locator("[data-find-match]")).toHaveCount(1);
  });

  test("the find bar's highlight is not painted on a placeholder", async ({ page }) => {
    await gotoHydrated(page, URL);
    await expect(row(page, 0)).toBeVisible();
    // An engine without `::search-text` has no rule to check: its highlight is the UA's and cannot be styled.
    const supported = await page.evaluate(() => CSS.supports("selector(::search-text)"));
    test.skip(!supported, "::search-text needs Chromium 144+");
    // `::search-text` (Chromium 144+): the active match is the placeholder, which never lines up
    // with the real row's own text, so its highlight is transparent; the real row carries
    // `data-find-match` instead.
    // Computed `::search-text` does not report the UA's own highlight colours, but its `color`
    // follows the element's unless a rule sets it, so give the element a colour of its own first.
    const paint = await placeholder(page, 7776).evaluate((el) => {
      (el as HTMLElement).style.color = "rgb(255, 0, 0)";
      const s = getComputedStyle(el, "::search-text");
      return { background: s.backgroundColor, color: s.color };
    });
    expect(paint).toEqual({ background: "rgba(0, 0, 0, 0)", color: "rgba(0, 0, 0, 0)" });
  });

  test("a text-fragment link reveals the placeholder and the list follows", async ({ page }) => {
    // A real browser reveal: the text fragment is searched when the document loads, so it can only
    // find placeholders that exist by then. They mount after hydration, so this records which of the
    // two happens on this engine instead of asserting the reveal.
    const fired: number[] = [];
    await page.exposeFunction("__dxBeforematch", (i: number) => fired.push(i));
    await page.addInitScript(() => {
      document.addEventListener(
        "beforematch",
        (e) => (window as unknown as { __dxBeforematch: (i: number) => void }).__dxBeforematch(
          Number((e.target as HTMLElement).dataset.findIndex),
        ),
        true,
      );
    });
    await gotoHydrated(page, `${URL}#:~:text=Row%207777%20of%2010000`, { waitUntil: "networkidle" });
    await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });
    await page.waitForTimeout(1500);
    const inView = await rowInView(page, 7776);
    console.log(`text fragment after hydration: beforematch fired for ${JSON.stringify(fired)}, row 7777 in view: ${inView}`);
    if (fired.includes(7776)) expect(inView).toBe(true);
  });

  test("cost: DOM nodes and JS heap with placeholders, 10,000 and 100,000 rows", async ({ page, browserName }) => {
    test.skip(browserName !== "chromium", "CDP Performance.getMetrics");
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Performance.enable");
    const metrics = async () => {
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
      const { metrics: m } = await cdp.send("Performance.getMetrics");
      const heap = m.find((x) => x.name === "JSHeapUsedSize")!.value;
      // Elements in THIS document (CDP's "Nodes" also counts detached nodes of the previous page);
      // each placeholder adds one text node on top.
      const nodes = await page.evaluate(() => document.getElementsByTagName("*").length);
      return { nodes, heapMB: heap / 1e6 };
    };
    await gotoHydrated(page, URL);
    await expect(row(page, 0)).toBeVisible();
    const at10k = await metrics();
    const scroll20 = () =>
      list(page).evaluate(async (el) => {
        const t0 = performance.now();
        for (let i = 0; i < 20; i++) {
          el.scrollTop += 400;
          await new Promise((r) => requestAnimationFrame(r));
        }
        return performance.now() - t0;
      });
    const scrollMs10k = await scroll20();
    const t0 = Date.now();
    await page.getByRole("button", { name: "100000 rows", exact: true }).click();
    await expect(placeholder(page, 99_999)).toHaveCount(1, { timeout: 30_000 });
    const mountMs = Date.now() - t0;
    const at100k = await metrics();
    // Every row has its placeholder (a single parent receiving 65,536+ new children in one edit
    // kept only the last n mod 65536 of them in Dioxus 0.7.9; the layer nests them in chunks).
    expect(await list(page).locator("[data-find-index]").count()).toBe(100_000);
    // A window move with 100,000 placeholders re-renders only the chunks it touches.
    const scrollMs = await scroll20();
    // The same scroll on the plain windowed demo (100,000 rows, no placeholders) as the baseline.
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=virtual_list&variant=windowed`);
    const plain = page.locator('[data-vl-demo="windowed"] [role="list"]');
    await expect(plain.getByRole("listitem").first()).toBeVisible();
    const plainNodes = (await metrics()).nodes;
    const plainMs = await plain.evaluate(async (el) => {
      const t0 = performance.now();
      for (let i = 0; i < 20; i++) {
        el.scrollTop += 400;
        await new Promise((r) => requestAnimationFrame(r));
      }
      return performance.now() - t0;
    });
    console.log(`plain windowed 100k rows: ${plainNodes} nodes; 20 scroll frames ${plainMs.toFixed(0)} ms`);
    console.log(
      `find placeholders: 10k rows ${at10k.nodes} nodes / ${at10k.heapMB.toFixed(1)} MB heap / 20 scroll frames ${scrollMs10k.toFixed(0)} ms; ` +
        `100k rows ${at100k.nodes} nodes / ${at100k.heapMB.toFixed(1)} MB heap / mounted in ${mountMs} ms; 20 scroll frames ${scrollMs.toFixed(0)} ms`,
    );
    expect(at100k.nodes - at10k.nodes).toBeGreaterThan(85_000);
    expect(at100k.nodes - at10k.nodes).toBeLessThan(100_000);
  });
});
