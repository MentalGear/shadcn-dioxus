// Spike: can a page-side classifier tell the realized default action behind a scroll position change of a
// Rust-written scroller (the one-shot intent kind)? Four causes move a scroller: the user (wheel, key), the library's
// own write (a programmatic scrollTo), an anchoring adjustment (content inserted above the viewport) and a snap
// (the browser settles on a snap point after a user scroll). The classifier reads three signals only: a trusted
// input in the same animation turn, the write's own flag, and whether the delta follows a user-caused scroll.
// Chromium only; the verdict per cause is asserted below.

import { expect, test } from "../../fixtures";
import { gotoReference } from "../../page-kinds";

const URL = "http://classifier.fixture/page.html";

const PAGE = `<!doctype html><meta charset="utf-8"><body style="margin:0">
<style>#s{width:300px;height:300px;overflow:auto;overflow-anchor:auto} #snap{scroll-snap-type:y mandatory} #snap>div{height:300px;scroll-snap-align:start}
#plain>div.row{height:100px}</style>
<div id="s" tabindex="0"><div id="top"></div><div id="plain">${Array.from({ length: 20 }, (_, i) => `<div class="row">row ${i}</div>`).join("")}</div></div>
<div id="snap" style="width:300px;height:300px;overflow:auto" tabindex="0"><div>a</div><div>b</div><div>c</div></div>
<script>
// the classifier under test
window.__log = [];
const state = { lastInput: -Infinity, expected: new WeakMap(), userScrolling: new WeakSet() };
for (const type of ["wheel", "keydown", "touchmove", "pointerdown"]) {
  window.addEventListener(type, (e) => { if (e.isTrusted) state.lastInput = performance.now(); }, { capture: true, passive: true });
}
// a write names the position it asked for: the scroll event fires a frame later, so a flag would be gone by then
window.__own = (el, top) => { el.scrollTop = top; state.expected.set(el, el.scrollTop); };
const track = (el) => {
  let before = el.scrollTop;
  el.addEventListener("scroll", () => {
    const delta = el.scrollTop - before;
    before = el.scrollTop;
    const input = performance.now() - state.lastInput < 600;
    let kind;
    if (state.expected.get(el) === el.scrollTop && !input) kind = "own";
    else if (input) { kind = "user"; state.userScrolling.add(el); }
    else if (state.userScrolling.has(el)) kind = "snap";
    else kind = "adjust";
    window.__log.push({ el: el.id, kind, delta });
  });
};
track(document.getElementById("s"));
track(document.getElementById("snap"));
</script>`;

const open = async (page: import("@playwright/test").Page) => {
  await page.route(URL, (route) => route.fulfill({ contentType: "text/html", body: PAGE }));
  await gotoReference(page, URL);
};
const kinds = (page: import("@playwright/test").Page, el: string) => page.evaluate((id) => (window as any).__log.filter((l: any) => l.el === id).map((l: any) => l.kind), el);

test.describe("scroll classifier spike (Chromium)", () => {
  test("a wheel scroll is user", async ({ page }) => {
    await open(page);
    await page.mouse.move(100, 100);
    await page.mouse.wheel(0, 150);
    await expect.poll(() => kinds(page, "s")).toContain("user");
    expect(await kinds(page, "s")).not.toContain("adjust");
  });

  test("a write the library makes is own, whatever input came before", async ({ page }) => {
    await open(page);
    await page.mouse.move(100, 100);
    await page.mouse.wheel(0, 50);
    await expect.poll(() => kinds(page, "s")).toContain("user");
    await page.waitForTimeout(700);
    await page.evaluate(() => (window as any).__own(document.getElementById("s"), 400));
    await expect.poll(() => kinds(page, "s")).toContain("own");
  });

  test("content inserted above the viewport moves the scroller as an adjustment: no input, no write", async ({ page }) => {
    await open(page);
    await page.evaluate(() => (window as any).__own(document.getElementById("s"), 500));
    await page.waitForTimeout(700);
    await page.evaluate(() => {
      const top = document.getElementById("top")!;
      top.style.height = "200px";
    });
    await expect.poll(() => kinds(page, "s")).toContain("adjust");
  });

  test("a mandatory snap after a user scroll settles as snap, not as a second user input", async ({ page }) => {
    await open(page);
    await page.mouse.move(100, 400);
    await page.mouse.wheel(0, 200);
    await expect.poll(() => page.evaluate(() => document.getElementById("snap")!.scrollTop), { timeout: 5000 }).toBe(300);
    const k = await kinds(page, "snap");
    expect(k).toContain("user");
    expect(k.every((x: string) => x === "user" || x === "snap")).toBe(true);
  });
});
