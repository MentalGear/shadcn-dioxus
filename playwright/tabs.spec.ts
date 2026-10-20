import { test, expect } from "./fixtures";
import { type Locator, type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/tabs/`);
  let activeTab = page.locator('[role="tabpanel"][data-state="active"]:not(#component-preview-frame)')
    .filter({ hasText: /^Tab \d Content$/ });
  let tab1Button = page.getByRole("tab", { name: "Tab 1" });
  let tab2Button = page.getByRole("tab", { name: "Tab 2" });
  let tab3Button = page.getByRole("tab", { name: "Tab 3" });
  // Clicking the right arrow should focus the next tab trigger
  await tab1Button.click();
  await page.keyboard.press("ArrowRight");
  await expect(tab2Button).toBeFocused();

  // Clicking enter should activate the focused tab
  await page.keyboard.press("Enter");
  await expect(activeTab).toContainText("Tab 2 Content");

  // Clicking right twice more should bring us back to the first tab
  await page.keyboard.press("ArrowRight");
  await expect(tab3Button).toBeFocused();
  await page.keyboard.press("ArrowRight");
  await expect(tab1Button).toBeFocused();

  // Clicking each tab should activate it
  await tab3Button.click();
  await expect(activeTab).toContainText("Tab 3 Content");
  await tab2Button.click();
  await expect(activeTab).toContainText("Tab 2 Content");
  await tab1Button.click();
  await expect(activeTab).toContainText("Tab 1 Content");
});

test.describe("active tab keeps its color under the pointer (calendar-state lane sweep)", () => {
  // Same bug class as calendar/style.css's day hover (see calendar.spec.ts):
  // `.dx-tabs-trigger:hover` used to carry higher specificity than
  // `.dx-tabs-trigger[data-state="active"]`, so clicking a tab -- which
  // leaves the pointer resting on it -- silently dimmed its text to the
  // plain hover tint instead of the active color. Runs in both themes: this
  // one (unlike calendar's) reproduced in light mode too, since
  // --secondary-color-1 and --secondary-color-3 differ in both.
  for (const dark of [false, true]) {
    test(`clicking a tab keeps its active text color while still hovered (${dark ? "dark" : "light"} mode)`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/tabs/${dark ? "?dark_mode=true" : ""}`);
      const tab2Button = page.getByRole("tab", { name: "Tab 2" });
      const tab3Button = page.getByRole("tab", { name: "Tab 3" });

      // Tab colours now transition (Nova tabs, row 111), so every read waits for the
      // trigger's running animations to finish: a read taken mid-fade is an interpolated
      // colour, not either end state.
      const settledColor = (tab: typeof tab2Button) =>
        tab.evaluate(async (el) => {
          await Promise.all(el.getAnimations().map((a) => a.finished.catch(() => undefined)));
          return getComputedStyle(el).color;
        });

      // Click Tab 2 -- Playwright's .click() leaves the pointer on it.
      await tab2Button.click();
      await expect(tab2Button).toHaveAttribute("data-state", "active");
      expect(await tab2Button.evaluate((el) => el.matches(":hover"))).toBe(true);
      const activeAndHoveredColor = await settledColor(tab2Button);

      // Move the pointer off and read the "true" active color, plus the resting color of a
      // tab that stayed inactive throughout (Tab 3).
      await page.locator("body").hover({ position: { x: 0, y: 0 } });
      const activeAtRestColor = await settledColor(tab2Button);
      const inactiveAtRestColor = await settledColor(tab3Button);

      expect(activeAndHoveredColor).toBe(activeAtRestColor); // active color survives hover
      // Hover and active both resolve to `foreground` now, so "not the plain hover tint" can
      // no longer be observed; what must still hold is that the active tab is not rendered
      // in the inactive (muted) colour.
      expect(activeAndHoveredColor).not.toBe(inactiveAtRestColor);
    });
  }
});

test.describe("Axe automated scan", () => {
  test("loaded (tab 1 active) has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tabs/`);
    await expectNoAxeViolations(page, "tabs: tab 1 active", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("tab 2 selected has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tabs/`);
    await page.getByRole("tab", { name: "Tab 2" }).click();
    // Scoped with the same `.filter(...)` the file's own "test" test uses
    // above -- this page's "Examples" section renders a second, unrelated
    // Tabs instance with its own active tabpanel, so the bare selector
    // resolves to two elements (a Playwright strict-mode violation).
    await expect(
      page
        .locator('[role="tabpanel"][data-state="active"]:not(#component-preview-frame)')
        .filter({ hasText: /^Tab \d Content$/ }),
    ).toContainText("Tab 2 Content");
    await expectNoAxeViolations(page, "tabs: tab 2 selected", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

/**
 * `hidden_until_found` (backlog row 144), driven through the `until_found` variant: Overview
 * (active), Specs and Reviews, inactive panels mounted as `hidden="until-found"`. A reveal by the
 * browser activates that panel's tab.
 *
 * Headless Chromium's `window.find` finds `until-found` text but does not fire `beforematch` for
 * it (only fragment navigation does), so the reveal is driven by a real `#fragment` navigation and
 * by a synthetic `beforematch`; real Ctrl+F takes the same path as the fragment and is checked by
 * hand in a headed browser.
 */
test.describe("hidden_until_found", () => {
  const URL = `${BASE_URL}/component/tabs/`;
  const PANEL_TEXT: Record<string, string> = {
    Overview: "A small amphibian",
    Specs: "Regenerates limbs",
    Reviews: "Rated four stars",
  };

  async function load(page: Page) {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const tab = (name: string) => page.getByRole("tab", { name, exact: true });
    // The docs page wraps every demo in its own Tabs (Preview/Code), whose active panel also
    // contains this text: scope to the innermost Tabs root, the demo's own.
    const root = page.locator(".dx-tabs", { has: tab("Overview") }).last();
    const panelOf = (name: string) => root.locator(':scope > [role="tabpanel"]', { hasText: PANEL_TEXT[name] });
    await expect(tab("Overview")).toHaveAttribute("aria-selected", "true");
    return { tab, panelOf };
  }

  /** Give the content inside a panel an id the page can navigate to with a `#fragment`. */
  async function markInside(panel: Locator, id: string) {
    await panel.evaluate((el, id) => {
      el.firstElementChild!.id = id;
    }, id);
  }

  async function expectActive(tab: (n: string) => Locator, panelOf: (n: string) => Locator, active: string) {
    for (const name of Object.keys(PANEL_TEXT)) {
      const isActive = name === active;
      await expect(tab(name)).toHaveAttribute("aria-selected", String(isActive));
      await expect(tab(name)).toHaveAttribute("data-state", isActive ? "active" : "inactive");
      await expect(panelOf(name)).toHaveAttribute("data-state", isActive ? "active" : "inactive");
      if (isActive) await expect(panelOf(name)).not.toHaveAttribute("hidden", /.*/);
      else await expect(panelOf(name)).toHaveAttribute("hidden", "until-found");
    }
  }

  test("inactive panels are mounted as hidden=until-found only when opted in", async ({ page }) => {
    const { tab, panelOf } = await load(page);
    await expectActive(tab, panelOf, "Overview");
    for (const name of ["Specs", "Reviews"]) {
      await expect(panelOf(name)).toHaveCount(1);
      expect(await panelOf(name).evaluate((el) => el.textContent)).toContain(PANEL_TEXT[name]);
    }

    // The default demo unmounts inactive panels: an empty, plain `hidden` element, no content.
    await expect(page.locator('[role="tabpanel"]', { hasText: "Tab 2 Content" })).toHaveCount(0);
    await expect(
      page.locator('[role="tabpanel"][data-state="inactive"][hidden=""]').filter({ hasNotText: /\S/ }).first(),
    ).toBeAttached();
    // And find-in-page reaches the opted-in text but not the unmounted text.
    expect(await page.evaluate(() => window.find("axolotl"))).toBe(true);
    expect(await page.evaluate(() => window.find("Tab 2 Content"))).toBe(false);
  });

  test("an inactive until-found panel is skipped but stays a real box taking no space", async ({ page }) => {
    const { panelOf } = await load(page);
    for (const name of ["Specs", "Reviews"]) {
      expect(
        await panelOf(name).evaluate((el) => {
          const s = getComputedStyle(el);
          const r = el.getBoundingClientRect();
          return {
            cv: s.contentVisibility,
            displayNone: s.display === "none",
            position: s.position,
            padding: s.paddingTop,
            border: s.borderTopWidth,
            w: r.width,
            h: r.height,
          };
        }),
      ).toEqual({ cv: "hidden", displayNone: false, position: "absolute", padding: "0px", border: "0px", w: 0, h: 0 });
    }
  });

  test("a #fragment into an inactive panel reveals it and activates its tab", async ({ page }) => {
    const { tab, panelOf } = await load(page);
    await markInside(panelOf("Specs"), "axolotl-target");
    await page.evaluate(() => {
      location.hash = "#axolotl-target";
    });

    await expectActive(tab, panelOf, "Specs");
    await expect(page.locator("#axolotl-target")).toBeVisible();
    await expect(tab("Specs")).toHaveAttribute("tabindex", "0");
    await expect(tab("Overview")).toHaveAttribute("tabindex", "-1");
  });

  test("a beforematch on an inactive panel activates its tab", async ({ page }) => {
    const { tab, panelOf } = await load(page);
    await panelOf("Reviews").dispatchEvent("beforematch");
    await expectActive(tab, panelOf, "Reviews");
  });

  test("a beforematch on the already-active panel changes nothing", async ({ page }) => {
    const { tab, panelOf } = await load(page);
    await panelOf("Overview").dispatchEvent("beforematch");
    await page.waitForTimeout(300);
    await expectActive(tab, panelOf, "Overview");
  });

  test("clicking tabs afterwards still swaps panels and restores hidden=until-found", async ({ page }) => {
    const { tab, panelOf } = await load(page);
    await panelOf("Specs").dispatchEvent("beforematch");
    await expectActive(tab, panelOf, "Specs");

    await tab("Overview").click();
    await expectActive(tab, panelOf, "Overview");
    await tab("Reviews").click();
    await expectActive(tab, panelOf, "Reviews");
  });

  test("listens exactly once per panel, through switches, and not at all after unmount", async ({ page }) => {
    const { tab, panelOf } = await load(page);
    const cdp = await page.context().newCDPSession(page);
    const beforematchListeners = async (expression: string) => {
      const { result } = await cdp.send("Runtime.evaluate", { expression });
      const { listeners } = await cdp.send("DOMDebugger.getEventListeners", { objectId: result.objectId! });
      return listeners.filter((l) => l.type === "beforematch").length;
    };
    await panelOf("Specs").evaluate((el) => {
      (window as any).__panel = el;
    });
    expect(await beforematchListeners("window.__panel")).toBe(1);

    for (let i = 0; i < 3; i++) {
      await tab("Specs").click();
      await expect(tab("Specs")).toHaveAttribute("aria-selected", "true");
      await tab("Overview").click();
      await expect(tab("Overview")).toHaveAttribute("aria-selected", "true");
    }
    expect(await beforematchListeners("window.__panel")).toBe(1);

    await page.locator('a[href="/component/kbd/?"]').first().click();
    await expect(page).toHaveURL(/\/component\/kbd\//);
    await expect.poll(() => page.evaluate(() => (window as any).__panel.isConnected)).toBe(false);
    expect(await beforematchListeners("window.__panel")).toBe(0);
  });
});

// The `line` variant (`data-variant="line"`; `TabsVariant::Ghost`, the DEMO/CODE tabs on every component page,
// is its old name and renders the same): a transparent list, and the active tab carries a 2px underline that
// moves with the selection.
test.describe("line variant: the active tab carries the underline", () => {
  const underline = (tab: Locator) =>
    tab.evaluate((el) => {
      const after = getComputedStyle(el, "::after");
      return { opacity: Number(after.opacity), height: after.height, content: after.content };
    });

  test("the underline follows the selected tab and the list has no fill", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button/`, { timeout: 20 * 60 * 1000 });
    const list = page.locator('[data-variant="line"] .dx-tabs-list').first();
    const demo = list.getByRole("tab", { name: "DEMO" });
    const code = list.getByRole("tab", { name: "CODE" });
    await expect(demo).toHaveAttribute("aria-selected", "true");

    await expect(list).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
    await expect(demo).toHaveCSS("box-shadow", "none");
    await expect.poll(() => underline(demo)).toMatchObject({ opacity: 1, height: "2px" });
    await expect.poll(() => underline(code)).toMatchObject({ opacity: 0, height: "2px" });

    await code.click();
    await expect(code).toHaveAttribute("aria-selected", "true");
    await expect.poll(() => underline(code)).toMatchObject({ opacity: 1 });
    await expect.poll(() => underline(demo)).toMatchObject({ opacity: 0 });
  });
});

// Vertical orientation (backlog row 155 group 5): shadcn's `orientation="vertical"`. The `vertical` variant is
// Account / Password / Notifications (default fill); `vertical_line` is Teams / Members / Roles (line variant).
test.describe("vertical orientation", () => {
  const URL = `${BASE_URL}/component/tabs/`;
  async function load(page: Page) {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const vlist = page.locator('[role="tablist"][aria-orientation="vertical"]');
    await expect(vlist).toHaveCount(2);
    return { vlist, list: vlist.first(), lineList: vlist.last() };
  }

  test("aria-orientation and data-orientation follow the prop; horizontal is the default", async ({ page }) => {
    const { list } = await load(page);
    await expect(list.locator("xpath=..")).toHaveAttribute("data-orientation", "vertical");
    const main = page.getByRole("tab", { name: "Tab 1" }).locator('xpath=ancestor::*[@role="tablist"]');
    await expect(main).toHaveAttribute("aria-orientation", "horizontal");
    await expect(main.locator("xpath=..")).toHaveAttribute("data-orientation", "horizontal");
  });

  test("the list is a column beside the panel, triggers fill its width start-aligned", async ({ page }) => {
    const { list } = await load(page);
    const root = list.locator("xpath=..");
    await expect(root).toHaveCSS("flex-direction", "row");
    await expect(list).toHaveCSS("flex-direction", "column");
    const geo = await list.evaluate((el) => {
      const l = el.getBoundingClientRect();
      const t = [...el.querySelectorAll('[role="tab"]')].map((n) => n.getBoundingClientRect());
      const panel = el.parentElement!.querySelector('[role="tabpanel"]:not([hidden])')!.getBoundingClientRect();
      return { l, t, panel, justify: getComputedStyle(el.querySelector('[role="tab"]')!).justifyContent };
    });
    // Stacked top to bottom, each as wide as the list's content box, list to the start side of the panel.
    expect(geo.t[1].top).toBeGreaterThan(geo.t[0].bottom - 1);
    expect(geo.t[2].top).toBeGreaterThan(geo.t[1].bottom - 1);
    expect(Math.abs(geo.t[0].width - geo.t[2].width)).toBeLessThan(1);
    expect(geo.t[0].width).toBeGreaterThan(geo.l.width - 8);
    expect(geo.panel.left).toBeGreaterThanOrEqual(geo.l.right - 1);
    expect(geo.justify).toBe("flex-start");
    // `h-fit`: the list is as tall as its triggers, not stretched to the panel.
    expect(geo.l.height).toBeLessThan(geo.t[2].bottom - geo.t[0].top + 8);
  });

  test("ArrowUp/ArrowDown move focus (looping), Home/End jump, ArrowLeft/ArrowRight do nothing", async ({ page }) => {
    const { list } = await load(page);
    const tab = (n: string) => list.getByRole("tab", { name: n, exact: true });
    await tab("Account").click();
    await page.keyboard.press("ArrowDown");
    await expect(tab("Password")).toBeFocused();
    await page.keyboard.press("ArrowDown");
    await expect(tab("Notifications")).toBeFocused();
    await page.keyboard.press("ArrowDown");
    await expect(tab("Account")).toBeFocused();
    await page.keyboard.press("ArrowUp");
    await expect(tab("Notifications")).toBeFocused();
    await page.keyboard.press("Home");
    await expect(tab("Account")).toBeFocused();
    await page.keyboard.press("End");
    await expect(tab("Notifications")).toBeFocused();

    await page.keyboard.press("ArrowRight");
    await expect(tab("Notifications")).toBeFocused();
    await page.keyboard.press("ArrowLeft");
    await expect(tab("Notifications")).toBeFocused();

    // Focus moved without selecting; Enter selects.
    await expect(tab("Account")).toHaveAttribute("aria-selected", "true");
    await page.keyboard.press("Enter");
    await expect(tab("Notifications")).toHaveAttribute("aria-selected", "true");
    await expect(list.locator("xpath=..").locator(':scope > [role="tabpanel"]:not([hidden])')).toHaveText(/Choose which events email you/);
  });

  test("a horizontal list ignores ArrowUp/ArrowDown", async ({ page }) => {
    await load(page);
    const tab1 = page.getByRole("tab", { name: "Tab 1" });
    await tab1.click();
    await page.keyboard.press("ArrowDown");
    await expect(tab1).toBeFocused();
    await page.keyboard.press("ArrowUp");
    await expect(tab1).toBeFocused();
  });

  test("the line variant draws a vertical 2px bar on the side facing the panel", async ({ page }) => {
    const { lineList } = await load(page);
    const bar = (name: string) =>
      lineList.getByRole("tab", { name, exact: true }).evaluate((el) => {
        const a = getComputedStyle(el, "::after");
        const t = el.getBoundingClientRect();
        // Pseudo-element geometry from its computed box: right edge relative to the trigger.
        return { opacity: Number(a.opacity), width: a.width, height: a.height, right: parseFloat(a.right), triggerW: t.width };
      });
    await expect(lineList).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
    await expect.poll(() => bar("Teams")).toMatchObject({ opacity: 1, width: "2px", right: -4 });
    expect((await bar("Teams")).height).toBe(
      (await lineList.getByRole("tab", { name: "Teams", exact: true }).evaluate((el) => `${el.getBoundingClientRect().height - 2}px`)),
    );
    await expect.poll(() => bar("Members")).toMatchObject({ opacity: 0, width: "2px" });
    await lineList.getByRole("tab", { name: "Members", exact: true }).click();
    await expect.poll(() => bar("Members")).toMatchObject({ opacity: 1 });
    await expect.poll(() => bar("Teams")).toMatchObject({ opacity: 0 });
  });

  test("a default Tabs inside the docs page's line DEMO/CODE tabs keeps its own variant (no leak)", async ({ page }) => {
    const { list } = await load(page);
    // The vertical demo is a `default` Tabs nested in the page's `line` Tabs: its list keeps the muted pill and
    // its active trigger the raised fill, and no line bar is drawn; the outer line list stays transparent.
    expect(await list.evaluate((el) => getComputedStyle(el).backgroundColor)).not.toBe("rgba(0, 0, 0, 0)");
    const active = list.getByRole("tab", { name: "Account", exact: true });
    expect(await active.evaluate((el) => getComputedStyle(el).backgroundColor)).not.toBe("rgba(0, 0, 0, 0)");
    expect(await active.evaluate((el) => getComputedStyle(el, "::after").backgroundColor)).toBe("rgba(0, 0, 0, 0)");
    const outer = page.locator('[data-variant="line"] .dx-tabs-list').first();
    await expect(outer).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
  });

  test("vertical tabs have no automatically detectable a11y issues", async ({ page }) => {
    await load(page);
    await expectNoAxeViolations(page, "tabs: vertical", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// The docs site's demo frame is itself a `TabContent` (`.dx-component-preview-frame.dx-tabs-content-themed`)
// inside a `line` Tabs. The panel's default frame (tabs/style.css, loaded after main.css) used to out-rank the
// frame's own border and radius on stylesheet order alone, so every demo on every component page lost its
// chrome. The frame is the docs preview card (736e9600, row 285): `rounded-2xl`, 40px padding, 288px tall, NO border
// and no dotted grid, filled with the soft `--dx-docs-preview-surface` (a mix of page and card) -- the Tabs panel
// default would give it a 1px border and the card colour.
test.describe("the docs demo frame keeps its own chrome", () => {
  test("border, radius, padding and height come from main.css, not from the Tabs panel default", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/spinner/`);
    const frame = page.locator(".dx-component-preview-frame").first();
    await expect(frame).toBeVisible();
    await expect(frame).toHaveCSS("border-top-width", "0px");
    await expect(frame).toHaveCSS("border-top-left-radius", "16px");
    await expect(frame).toHaveCSS("padding-top", "40px");
    await expect(frame).toHaveCSS("background-image", "none");
    // The fill is the docs surface token resolved in the frame's own scope, not the Tabs panel's card colour.
    const [fill, surface, card] = await frame.evaluate((el) => {
      const probe = (value: string) => {
        const p = document.createElement("div");
        p.style.backgroundColor = value;
        el.appendChild(p);
        const c = getComputedStyle(p).backgroundColor;
        p.remove();
        return c;
      };
      return [getComputedStyle(el).backgroundColor, probe("var(--dx-docs-preview-surface)"), probe("var(--dx-card)")];
    });
    expect(fill, "the frame is filled with --dx-docs-preview-surface").toBe(surface);
    expect(fill, "not the Tabs panel's card colour").not.toBe(card);
    expect((await frame.boundingBox())!.height).toBeGreaterThanOrEqual(288);
  });

  test("a default Tabs panel is still framed by the zero-specificity default", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tabs/`);
    const panel = page.locator('.dx-tabs[data-variant="default"] .dx-tabs-content-themed[data-state="active"]').first();
    await expect(panel).toBeVisible();
    await expect(panel).toHaveCSS("border-top-width", "1px");
    expect(await panel.evaluate((el) => getComputedStyle(el).backgroundColor)).not.toBe("rgba(0, 0, 0, 0)");
  });
});
