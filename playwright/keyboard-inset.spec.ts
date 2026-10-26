/**
 * Row 227: modal panels follow the on-screen keyboard (`primitives/src/keyboard_inset.rs`).
 *
 * HOW THE KEYBOARD IS SIMULATED, and why. Headless Chromium has no on-screen keyboard, and no CDP
 * command shrinks ONLY the visual viewport: `Emulation.setDeviceMetricsOverride` /
 * `setVisibleSize` resize the layout viewport too (so `innerHeight - visualViewport.height` stays 0,
 * which is "no keyboard" to the helper), and `Emulation.setPageScaleFactor` shrinks the visual
 * viewport only by pinch-zooming (`scale !== 1`, which the helper deliberately ignores). iOS Safari
 * does exactly the one thing neither can do: `innerHeight` and the layout viewport keep their size
 * and `visualViewport.height` drops. So this spec STUBS `window.visualViewport` before any page
 * script runs: an `EventTarget` whose `height` is `innerHeight - <keyboard>`, whose `offsetTop` and
 * `scale` the test sets, and which dispatches a real `resize`/`scroll` event on every change. The
 * layout viewport is the real, untouched one -- the same shape as iOS -- so the CSS under test is
 * the shipped CSS acting on a real layout; what is NOT exercised is a real keyboard's own focus
 * scrolling, which needs the manual iPhone checklist of row 227.
 *
 * Bounds are the stub's visual viewport (`[offsetTop, offsetTop + height]`), never
 * `documentElement.clientHeight`/`innerHeight` (`scripts/check-spec-viewport-bound.sh`).
 */
import { test, expect } from "./fixtures";
import type { Page, Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnimationsSettled } from "./animations";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const PHONE = { width: 390, height: 844 };
test.use({ viewport: PHONE });

type Kb = { height?: number; offsetTop?: number; scale?: number };

/** Installs the visual-viewport stub. `__dxKb` is the test's handle on it. */
async function stubVisualViewport(page: Page) {
  await page.addInitScript(() => {
    const state = { kb: 0, offsetTop: 0, scale: 1 };
    const target = new EventTarget();
    let listeners = 0;
    const origAdd = target.addEventListener.bind(target);
    const origRemove = target.removeEventListener.bind(target);
    // Count registrations by identity so a removal that does not match its add cannot hide.
    const live = new Set<string>();
    const key = (t: string, f: unknown, o: unknown) =>
      `${t}|${(f as any).__id ?? ((f as any).__id = ++listeners)}|${typeof o === "boolean" ? o : !!(o as any)?.capture}`;
    const stub: any = target;
    stub.addEventListener = (t: string, f: any, o: any) => {
      live.add(key(t, f, o));
      origAdd(t, f, o);
    };
    stub.removeEventListener = (t: string, f: any, o: any) => {
      live.delete(key(t, f, o));
      origRemove(t, f, o);
    };
    Object.defineProperties(stub, {
      // The stub of the visual viewport, defined against the real layout viewport (an allowlisted in-page read: check-spec-viewport-bound).
      width: { get: () => window.innerWidth / state.scale },
      // As above; the keyboard is the part of the layout viewport the visual one lost.
      height: { get: () => (window.innerHeight - state.kb) / state.scale },
      offsetTop: { get: () => state.offsetTop },
      offsetLeft: { get: () => 0 },
      pageTop: { get: () => window.scrollY + state.offsetTop },
      pageLeft: { get: () => window.scrollX },
      scale: { get: () => state.scale },
    });
    Object.defineProperty(window, "visualViewport", { value: stub, configurable: true });
    (window as any).__dxKb = {
      set(next: { height?: number; offsetTop?: number; scale?: number }) {
        if (next.height !== undefined) state.kb = next.height;
        if (next.offsetTop !== undefined) state.offsetTop = next.offsetTop;
        if (next.scale !== undefined) state.scale = next.scale;
        stub.dispatchEvent(new Event("resize"));
        stub.dispatchEvent(new Event("scroll"));
      },
      liveListeners: () => live.size,
    };
  });
}

async function keyboard(page: Page, kb: Kb) {
  await page.evaluate((k) => (window as any).__dxKb.set(k), kb);
}

/** The largest inline `padding-bottom` among the focused field's ancestors: the helper's scroll slack
 * (a demo's own inline padding is 0 to 16px; the slack is at least 48px). */
const slackPx = (page: Page) =>
  page.evaluate(() => {
    let max = 0;
    for (let e: HTMLElement | null = document.activeElement as HTMLElement; e; e = e.parentElement) {
      max = Math.max(max, parseFloat(e.style.paddingBottom || "0") || 0);
    }
    return max;
  });

const liveListeners = (page: Page) => page.evaluate(() => (window as any).__dxKb.liveListeners() as number);

/** The panel's state: the contract attributes plus the geometry a stylesheet decides. */
async function panelState(panel: Locator) {
  return panel.evaluate((el) => {
    const cs = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    return {
      attr: el.getAttribute("data-dx-keyboard"),
      inset: el.style.getPropertyValue("--dx-keyboard-inset"),
      top: el.style.getPropertyValue("--dx-keyboard-top"),
      paddingBottomInline: el.style.paddingBottom,
      maxHeight: cs.maxHeight,
      bottomCss: cs.bottom,
      topCss: cs.top,
      overflowY: cs.overflowY,
      rect: { top: r.top, bottom: r.bottom, height: r.height },
    };
  });
}

/** The visual viewport band the stub reports, in layout-viewport coordinates. */
async function band(page: Page) {
  return page.evaluate(() => {
    const vv = window.visualViewport!;
    return { top: vv.offsetTop, bottom: vv.offsetTop + vv.height };
  });
}

const rectOf = (l: Locator) =>
  l.evaluate((el) => {
    const r = el.getBoundingClientRect();
    return { top: r.top, bottom: r.bottom };
  });

/** A field is "fully visible" when it lies inside the visual viewport AND inside its panel. */
async function expectFieldVisible(page: Page, panel: Locator, field: Locator) {
  await expect
    .poll(
      async () => {
        const b = await band(page);
        const f = await rectOf(field);
        const p = await rectOf(panel);
        return f.top >= Math.max(b.top, p.top) - 0.5 && f.bottom <= Math.min(b.bottom, p.bottom) + 0.5;
      },
      { timeout: 5000, message: "the focused field is inside the visual viewport and its panel" },
    )
    .toBe(true);
}

async function expectPanelInsideViewport(page: Page, panel: Locator) {
  const b = await band(page);
  const p = await rectOf(panel);
  expect(p.top).toBeGreaterThanOrEqual(b.top - 1);
  expect(p.bottom).toBeLessThanOrEqual(b.bottom + 1);
}

/** No keyboard: nothing of the contract exists and the geometry is the recorded baseline. */
async function expectAtRest(panel: Locator, rest: Awaited<ReturnType<typeof panelState>>) {
  await expect
    .poll(async () => {
      const s = await panelState(panel);
      return JSON.stringify({ ...s, rect: undefined });
    })
    .toBe(JSON.stringify({ ...rest, rect: undefined }));
  const s = await panelState(panel);
  expect(s.attr).toBeNull();
  expect(s.inset).toBe("");
  expect(s.top).toBe("");
  expect(s.paddingBottomInline).toBe(rest.paddingBottomInline);
  // Geometry equals the baseline too (at rest, no animation in flight).
  expect(Math.abs(s.rect.top - rest.rect.top)).toBeLessThan(1);
  expect(Math.abs(s.rect.bottom - rest.rect.bottom)).toBeLessThan(1);
}

type Case = {
  name: string;
  url: string;
  open: (page: Page) => Promise<void>;
  /** A text field inside the panel, to focus. */
  field: (panel: Locator) => Locator;
  /** Prepares the page before `open` (the Drawer demos have no field of their own). */
  prepare?: (panel: Locator) => Promise<void>;
  /** The panel's ARIA role (`alertdialog` for AlertDialog). */
  role?: "dialog" | "alertdialog";
};

const CASES: Case[] = [
  {
    name: "Dialog (centred)",
    url: `${BASE_URL}/component/dialog/`,
    open: async (page) => {
      await page.locator("#component-preview-frame-edit_profile").getByRole("button", { name: "Open Dialog" }).click();
    },
    field: (panel) => panel.locator("#dialog-edit-username"),
  },
  {
    name: "Sheet (right)",
    url: `${BASE_URL}/component/sheet/`,
    open: async (page) => {
      await page.getByRole("button", { name: "Right", exact: true }).first().click();
    },
    field: (panel) => panel.locator("#sheet-demo-username"),
  },
  {
    name: "Sheet (bottom)",
    url: `${BASE_URL}/component/sheet/`,
    open: async (page) => {
      await page.getByRole("button", { name: "Bottom", exact: true }).first().click();
    },
    field: (panel) => panel.locator("#sheet-demo-username"),
  },
  {
    name: "Drawer (bottom)",
    url: `${BASE_URL}/component/drawer/`,
    open: async (page) => {
      await page.getByRole("button", { name: "Move Goal" }).click();
    },
    // The Drawer demos carry no text field; this one is added as the panel's LAST child, i.e. in the
    // footer region, the part a bottom-anchored panel loses first behind a keyboard.
    prepare: async (panel) => {
      await panel.evaluate((el) => {
        const input = document.createElement("input");
        input.id = "kb-probe-field";
        input.type = "text";
        input.setAttribute("aria-label", "Keyboard probe");
        input.style.cssText = "margin: 0 1rem 1rem; height: 2.25rem; font-size: 16px;";
        el.appendChild(input);
      });
    },
    field: (panel) => panel.locator("#kb-probe-field"),
  },
  {
    name: "AlertDialog (centred)",
    url: `${BASE_URL}/component/alert_dialog/`,
    role: "alertdialog",
    open: async (page) => {
      await page.getByRole("button", { name: "Show Alert Dialog" }).first().click();
    },
    // The demo has no text field; this one is added as the panel's last child.
    prepare: async (panel) => {
      await panel.evaluate((el) => {
        const input = document.createElement("input");
        input.id = "kb-probe-field";
        input.type = "text";
        input.setAttribute("aria-label", "Keyboard probe");
        input.style.cssText = "height: 2.25rem; font-size: 16px;";
        el.appendChild(input);
      });
    },
    field: (panel) => panel.locator("#kb-probe-field"),
  },
];

for (const c of CASES) {
  test.describe(c.name, () => {
    async function openIt(page: Page) {
      await stubVisualViewport(page);
      await gotoHydrated(page, c.url, GOTO_OPTS);
      await c.open(page);
      const panel = page.getByRole(c.role ?? "dialog");
      await expect(panel).toBeVisible();
      await awaitAnimationsSettled(panel);
      if (c.prepare) await c.prepare(panel);
      return panel;
    }

    test("with a field focused, a keyboard keeps the field and the panel inside the visual viewport; closing it restores everything", async ({
      page,
    }) => {
      const panel = await openIt(page);
      const field = c.field(panel);
      const rest = await panelState(panel);
      expect(rest.attr).toBeNull();

      await field.focus();
      // Focus alone, with no keyboard, changes nothing.
      await expectAtRest(panel, rest);

      await keyboard(page, { height: 336 });
      await expect(panel).toHaveAttribute("data-dx-keyboard", "open");
      const open = await panelState(panel);
      expect(open.inset).toBe("336px");
      expect(open.top).toBe("0px");
      await expectFieldVisible(page, panel, field);
      await expectPanelInsideViewport(page, panel);

      // A taller keyboard (landscape-ish / suggestions bar): still inside, inset follows.
      await keyboard(page, { height: 420 });
      await expect.poll(async () => (await panelState(panel)).inset).toBe("420px");
      await expectFieldVisible(page, panel, field);
      await expectPanelInsideViewport(page, panel);

      await keyboard(page, { height: 0 });
      await expectAtRest(panel, rest);
    });

    test("the panel's max-height follows the inset", async ({ page }) => {
      const panel = await openIt(page);
      const rest = await panelState(panel);
      await c.field(panel).focus();
      // A 3000px spacer makes the panel as tall as its cap allows, so the box height IS the cap
      // (a computed `max-height` is a `calc()` string, not a number).
      await panel.evaluate((el) => {
        const spacer = document.createElement("div");
        spacer.id = "kb-spacer";
        spacer.style.cssText = "flex: none; height: 3000px;";
        el.appendChild(spacer);
      });
      const heightAt = async (kb: number) => {
        await keyboard(page, { height: kb });
        await expect.poll(async () => (await panelState(panel)).inset).toBe(`${kb}px`);
        await awaitAnimationsSettled(panel);
        return (await panelState(panel)).rect.height;
      };
      const small = await heightAt(300);
      const large = await heightAt(450);
      expect(small).toBeLessThanOrEqual(PHONE.height - 300);
      expect(large).toBeLessThanOrEqual(PHONE.height - 450);
      // One for one: 150px more keyboard is 150px less panel.
      expect(Math.round(small - large)).toBe(150);
      await keyboard(page, { height: 0 });
      await panel.locator("#kb-spacer").evaluate((el) => el.remove());
      await expectAtRest(panel, rest);
    });

    test("a field far below the fold is scrolled into the visible band (the helper's own scroll, not the browser's)", async ({
      page,
    }) => {
      const panel = await openIt(page);
      const field = c.field(panel);
      const rest = await panelState(panel);
      // 700px of content above the field pushes it far out of the capped panel.
      await field.evaluate((el) => {
        const spacer = document.createElement("div");
        spacer.id = "kb-above";
        spacer.style.cssText = "flex: none; height: 700px;";
        el.parentElement!.insertBefore(spacer, el);
      });
      await keyboard(page, { height: 336 });
      await expect(panel).toHaveAttribute("data-dx-keyboard", "open");
      // `preventScroll`: the browser's own focus scroll would make this pass without the helper.
      await field.evaluate((el) => (el as HTMLElement).focus({ preventScroll: true }));
      await expectFieldVisible(page, panel, field);
      // The slack that let it reach the middle is inline padding on a scroller, gone with the keyboard.
      const padded = await slackPx(page);
      expect(padded).toBeGreaterThanOrEqual(48);
      await keyboard(page, { height: 0 });
      await page.locator("#kb-above").evaluate((el) => el.remove());
      await expect.poll(() => slackPx(page)).toBeLessThan(48);
      await expectAtRest(panel, rest);
    });

    test("nothing changes for a browser-chrome resize (< 60px) or a pinch-zoomed page", async ({ page }) => {
      const panel = await openIt(page);
      const rest = await panelState(panel);
      await c.field(panel).focus();

      await keyboard(page, { height: 44 }); // collapsing URL bar
      await expectAtRest(panel, rest);

      await keyboard(page, { height: 0 });
      // Pinch-zoomed: the visual viewport is smaller, but it is not a keyboard.
      await keyboard(page, { height: 336, scale: 1.5 });
      await expectAtRest(panel, rest);
      await keyboard(page, { height: 0, scale: 1 });
      await expectAtRest(panel, rest);
    });

    test("a keyboard that is already open when the field is focused, and a panned visual viewport", async ({
      page,
    }) => {
      const panel = await openIt(page);
      const field = c.field(panel);
      const rest = await panelState(panel);
      await keyboard(page, { height: 336 });
      await expect(panel).toHaveAttribute("data-dx-keyboard", "open");
      await field.focus();
      await expectFieldVisible(page, panel, field);
      // iOS pans the visual viewport inside the layout viewport while the keyboard is up: the panel
      // stays in the visible band.
      // Same keyboard (336px under the layout viewport), the visual viewport panned 40px down.
      await keyboard(page, { height: 336 + 40, offsetTop: 40 });
      await expect.poll(async () => (await panelState(panel)).top).toBe("40px");
      await expect.poll(async () => (await panelState(panel)).inset).toBe("336px");
      await expectPanelInsideViewport(page, panel);
      await keyboard(page, { height: 0, offsetTop: 0 });
      await expectAtRest(panel, rest);
    });

    test("closing the panel with the keyboard open removes every listener and leaves no residue", async ({
      page,
    }) => {
      // Whatever else the page registers on the visual viewport (anchored popovers, other
      // primitives) is the baseline; the panel's own registrations come and go on top of it.
      await stubVisualViewport(page);
      await gotoHydrated(page, c.url, GOTO_OPTS);
      const before = await liveListeners(page);
      await c.open(page);
      const panel = page.getByRole(c.role ?? "dialog");
      await expect(panel).toBeVisible();
      await awaitAnimationsSettled(panel);
      if (c.prepare) await c.prepare(panel);
      await expect.poll(() => liveListeners(page)).toBeGreaterThan(before);
      await c.field(panel).focus();
      await keyboard(page, { height: 336 });
      await expect(panel).toHaveAttribute("data-dx-keyboard", "open");
      await page.keyboard.press("Escape");
      await expect(page.getByRole(c.role ?? "dialog")).toHaveCount(0);
      await expect.poll(() => liveListeners(page)).toBe(before);
    });
  });
}
