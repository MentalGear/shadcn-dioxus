/**
 * Controls are not text (dev-docs/backlog.md row 229; gate `scripts/check-theme-baseline.sh`, arm 4).
 *
 * The owner: "interactive elements like btns etc. should not allow user text selection". The theme has ONE
 * zero-specificity rule (`dx-components-theme.css`, "Controls are not text") giving every `dx-` button,
 * `[role=button]`, `summary` and menu-item/option/tab/switch/checkbox/radio/slider role
 * `-webkit-user-select: none; user-select: none`. It deliberately does NOT reach text fields, `spinbutton`
 * date segments, the `[tabindex]` catch-all (prose, code) or `a[href]`.
 *
 *   1. computed style: on a sweep of component pages EVERY `dx-` control the rule names computes `none` (no
 *      per-component list to keep in step), and the Safari spelling is declared on the same rule;
 *   2. behaviour: a double-click and a drag that starts on a button / tab / accordion trigger leave an empty
 *      selection, and Select All (Ctrl+A) on the page skips the control labels but keeps the prose;
 *   3. the other side: inputs, textareas, prose, code and links stay selectable (computed style + a real
 *      selection).
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8147 npx playwright test --config=ssg.local.config.ts user-select.spec.ts
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { baseUrlOr } from "./base-url";
import { gotoHydrated } from "./hydration";

const BASE = baseUrlOr("http://127.0.0.1:8080");

async function open(page: Page, path: string) {
  await page.route(
    (url) => !["127.0.0.1", "localhost"].includes(url.hostname),
    (route) => route.abort(),
  );
  await gotoHydrated(page, `${BASE}${path}`);
}

const DX = '[class^="dx-"], [class*=" dx-"]';
/** The theme rule's list, as a selector: what MUST compute `user-select: none`. */
const CONTROLS = [
  "button",
  '[role="button"]',
  "summary",
  ...["menuitem", "menuitemcheckbox", "menuitemradio", "option", "tab", "switch", "checkbox", "radio", "slider"].map(
    (r) => `[role="${r}"]`,
  ),
]
  .flatMap((s) => [`${s}[class^="dx-"]`, `${s}[class*=" dx-"]`])
  .join(", ");

/**
 * Row 331: 36 pages -> 12. Chosen by a greedy set cover, not by taste: on a debug SSG build every page of the old
 * sweep was probed for the (CONTROLS clause, first `dx-` class) pairs it renders, 38 distinct pairs in all, and
 * these 12 pages carry every one of them (picked most-uncovered-first, ties by the old list's order). A page not
 * listed adds no clause/class pair the listed ones lack, so a component sheet that re-declares `user-select` on
 * its control still fails here. The old 36 took 77 s; the gate (`check-theme-baseline.sh`, arm 4) cannot see a
 * `user-select: text` override, a main.css or inline style, or an element the `[class^="dx-"]` selector misses.
 */
const SWEEP = [
  "form", "carousel", "calendar", "slider", "menubar", "combobox", "toggle", "tag_group", "toolbar", "command",
  "navigation_menu", "color_picker",
];

for (const name of SWEEP) {
  test(`every control on /component/${name}/ computes user-select: none`, async ({ page }) => {
    await open(page, `/component/${name}/`);
    const bad = await page.evaluate((sel) => {
      return [...document.querySelectorAll<HTMLElement>(sel)]
        .filter((el) => getComputedStyle(el).userSelect !== "none")
        .map((el) => `${el.tagName.toLowerCase()}${el.getAttribute("role") ? `[role=${el.getAttribute("role")}]` : ""}.${String(el.className).split(/\s+/)[0]}`);
    }, CONTROLS);
    expect(bad, "controls that stayed text-selectable").toEqual([]);
    const count = await page.locator(CONTROLS).count();
    expect(count, "the page has controls to check").toBeGreaterThan(0);
  });
}

test("the Safari spelling is declared on the same theme rule", async ({ request }) => {
  const html = await (await request.get(`${BASE}/`)).text();
  const href = html.match(/href="([^"]*dx-components-theme[^"]*\.css)"/)?.[1] ?? html.match(/href="([^"]*main[^"]*\.css)"/)?.[1];
  expect(href, "a theme stylesheet link in the prerendered head").toBeTruthy();
  // The theme is bundled into the site's CSS; find any stylesheet that holds the rule.
  const sheets = [...html.matchAll(/href="([^"]+\.css)"/g)].map((m) => m[1]);
  let found = false;
  for (const s of sheets) {
    const css = await (await request.get(new URL(s, `${BASE}/`).toString())).text();
    if (/-webkit-user-select:\s*none;?\s*user-select:\s*none/.test(css)) found = true;
  }
  expect(found, "a stylesheet declares -webkit-user-select: none beside user-select: none").toBe(true);
});

test.describe("a double-click or a drag on a control selects nothing", () => {
  const selection = (page: Page) => page.evaluate(() => (window.getSelection()?.toString() ?? "").trim());

  for (const [path, locator, what] of [
    ["/component/button/", "button.dx-button:visible", "Button"],
    ["/component/tabs/", '[role="tab"]:visible', "Tab"],
    ["/component/accordion/", "button.dx-accordion-trigger:visible", "Accordion trigger"],
  ] as const) {
    test(`${what}: dblclick and drag`, async ({ page }) => {
      await open(page, path);
      const target = page.locator(locator).first();
      await expect(target).toBeVisible();
      const label = (await target.innerText()).trim();
      await target.scrollIntoViewIfNeeded();
      const box = (await target.boundingBox())!;

      // A browser never starts a selection on a button itself; the visitor's real gesture is a drag that
      // starts in the empty space beside the control (or in text above it) and sweeps across it.
      const y = box.y + box.height / 2;
      await page.mouse.move(Math.max(1, box.x - 40), y);
      await page.mouse.down();
      await page.mouse.move(box.x + box.width / 2, y, { steps: 8 });
      await page.mouse.move(box.x + box.width + 40, y + 2, { steps: 8 });
      await page.mouse.up();
      expect(await selection(page), `${what} "${label}": a drag sweeping across the control`).not.toContain(label);

      await page.mouse.click(box.x - 40, y); // collapse
      await page.mouse.dblclick(box.x + box.width / 2, y);
      expect(await selection(page), `${what} "${label}": double-click`).not.toContain(label);
    });
  }

  test("Select All keeps the prose and skips the control labels", async ({ page }) => {
    await open(page, "/component/button/");
    const label = (await page.locator("button.dx-button:visible").first().innerText()).trim();
    const prose = (await page.locator("main h1, #main-content h1").first().innerText()).trim();
    expect(label.length).toBeGreaterThan(0);
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+A");
    const all = await page.evaluate(() => window.getSelection()?.toString() ?? "");
    expect(all, "prose is selected").toContain(prose);
    // The first Button demo's label may also appear as prose elsewhere (docs text); the controls themselves
    // contribute nothing, so count the control's own text nodes that the selection kept.
    const kept = await page.evaluate((sel) => {
      const s = window.getSelection()!;
      return [...document.querySelectorAll<HTMLElement>(sel)].filter((el) => s.containsNode(el, true) && el.innerText.trim() && s.toString().includes(el.innerText.trim()) && getComputedStyle(el).userSelect !== "none").length;
    }, CONTROLS);
    expect(kept).toBe(0);
  });
});

test.describe("text fields, prose, code and links stay selectable", () => {
  test("input and textarea select their text", async ({ page }) => {
    for (const [path, sel] of [
      ["/component/input/", "input.dx-input:visible"],
      ["/component/textarea/", "textarea.dx-textarea:visible"],
    ] as const) {
      await open(page, path);
      const field = page.locator(sel).first();
      await field.fill("hello selectable world");
      expect(await field.evaluate((el) => getComputedStyle(el).userSelect), `${path} computed`).not.toBe("none");
      await field.dblclick({ position: { x: 12, y: 8 } });
      const selected = await field.evaluate((el: HTMLInputElement) => el.value.slice(el.selectionStart ?? 0, el.selectionEnd ?? 0));
      expect(selected.length, `${path}: a double-click selects a word`).toBeGreaterThan(0);
    }
  });

  test("a native select, prose, code and links are not user-select: none", async ({ page }) => {
    await open(page, "/component/native_select/");
    expect(
      await page.locator("select.dx-native-select:visible, select[class*='dx-']:visible").first().evaluate((el) => getComputedStyle(el).userSelect),
    ).not.toBe("none");

    await open(page, "/docs");
    const result = await page.evaluate(() => {
      const us = (el: Element | null) => (el ? getComputedStyle(el).userSelect : "missing");
      const main = document.querySelector("main, #main-content")!;
      return {
        paragraph: us(main.querySelector("p")),
        link: us(main.querySelector("a[href]:not(button *)")),
        code: us(main.querySelector("code, pre")),
        focusableProse: us(main.querySelector('[tabindex]:not([tabindex="-1"]):not(button):not(a):not(input)')),
      };
    });
    expect(result.paragraph).not.toBe("none");
    expect(result.link).not.toBe("none");
    expect(result.code).not.toBe("none");
    expect(result.focusableProse === "missing" || result.focusableProse !== "none").toBe(true);

    // And a real selection: triple-click a docs paragraph.
    const p = page.locator("main p, #main-content p").filter({ hasText: /.{60,}/ }).first();
    await p.scrollIntoViewIfNeeded();
    await p.click({ clickCount: 3 });
    expect((await page.evaluate(() => window.getSelection()?.toString() ?? "")).trim().length).toBeGreaterThan(10);
  });
});

// Row 294: a chart is a picture. `.dx-chart` carries the declaration for its whole subtree (axis labels, legend,
// tooltip, the arc labels of a pie), because the focusable chart root is a `[tabindex]` container the theme's
// control rule leaves selectable on purpose.
test.describe("a chart selects no text", () => {
  for (const path of ["/component/bar_chart/", "/component/pie_chart/", "/component/line_chart/", "/charts/area/"]) {
    test(`${path}: every part of every chart computes user-select: none`, async ({ page }) => {
      await open(page, path);
      const bad = await page.evaluate(() => {
        const out: string[] = [];
        for (const chart of document.querySelectorAll<HTMLElement>(".dx-chart")) {
          for (const el of [chart, ...chart.querySelectorAll<HTMLElement | SVGElement>("*")]) {
            if (getComputedStyle(el).userSelect !== "none") out.push(`${el.tagName.toLowerCase()}[${el.getAttribute("data-slot") ?? ""}]`);
          }
        }
        return [...new Set(out)];
      });
      expect(bad, "chart parts that stayed text-selectable").toEqual([]);
      expect(await page.locator(".dx-chart").count(), "the page has charts").toBeGreaterThan(0);
    });
  }

  test("a drag and a double-click across a chart select none of its labels, prose around it stays selectable", async ({ page }) => {
    await open(page, "/component/bar_chart/");
    const chart = page.locator(".dx-chart").first();
    await chart.scrollIntoViewIfNeeded();
    const labels = await chart.locator("svg text").evaluateAll((els) => els.map((el) => el.textContent ?? ""));
    expect(labels.length, "the chart has axis text").toBeGreaterThan(0);
    const box = (await chart.boundingBox())!;
    const y = box.y + box.height / 2;
    await page.mouse.move(Math.max(1, box.x - 20), box.y - 6);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2, y, { steps: 8 });
    await page.mouse.move(box.x + box.width + 20, box.y + box.height + 6, { steps: 8 });
    await page.mouse.up();
    const swept = (await page.evaluate(() => window.getSelection()?.toString() ?? "")).replace(/\s+/g, " ").trim();
    for (const label of labels.map((l) => l.trim()).filter(Boolean)) {
      expect(swept, `"${label}" was swept into a selection`).not.toContain(label);
    }
    await page.mouse.dblclick(box.x + box.width / 2, y);
    const word = (await page.evaluate(() => window.getSelection()?.toString() ?? "")).trim();
    expect(word, "a double-click on a chart selects no word").toBe("");

    // The docs prose above the demo is untouched.
    const p = page.locator("main p, #main-content p").filter({ hasText: /.{60,}/ }).first();
    expect(await p.evaluate((el) => getComputedStyle(el).userSelect)).not.toBe("none");
  });

  test("the iOS text callout is switched off on the chart in the shipped stylesheet", async ({ request }) => {
    const html = await (await request.get(`${BASE}/component/bar_chart/`)).text();
    const sheets = [...html.matchAll(/href="([^"]+\.css)"/g)].map((m) => m[1]);
    let found = false;
    for (const s of sheets) {
      const css = await (await request.get(new URL(s, `${BASE}/component/bar_chart/`).toString())).text();
      if (/\.dx-chart\s*\{[^}]*-webkit-touch-callout:\s*none[^}]*-webkit-user-select:\s*none[^}]*user-select:\s*none/s.test(css)) found = true;
    }
    expect(found, "the .dx-chart rule declares -webkit-touch-callout and the Safari user-select").toBe(true);
  });
});
