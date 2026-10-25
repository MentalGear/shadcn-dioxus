import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

/**
 * The control height scale (`--dx-control-height-xs/sm/default/lg`) follows the touch switch
 * (`data-dx-coarse`, `dx-components-theme.css` "Coarse sizing"), so every sized control does. Heights stay
 * fixed tokens, which is what keeps an input, a select, a text button and an icon button in one row exactly
 * aligned; this spec holds that in both modes.
 *
 *   (a) fine (`off`): the desktop heights are Nova's, token by token (xs 24, sm 28,
 *       default 32, lg 36);
 *   (b) coarse (`on`): every consumer is at its coarse height (36 / 40 / 44 / 48), icon-only controls square;
 *   (c) rows of mixed controls have one height and one top edge, and an input group's addon is centred;
 *   (d) no text is clipped (`scrollHeight <= clientHeight`; tabs are checked by their trigger sitting inside the list, because the trigger's bar pseudo-element extends its scroll box), and text entry is 14px fine, 17px coarse.
 *
 * Two kinds of probe. A set of real `dx-` controls, written as markup into a flex row on a component page,
 * covers every consumer of the scale independent of any demo's copy; the real demos of the components that
 * compose mixed controls (input group, toolbar, button group, pagination, tabs) are measured as shipped.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8106 npx playwright test --config=ssg.local.config.ts coarse-alignment.spec.ts
 */

type Mode = "off" | "on";
type Step = "xs" | "sm" | "default" | "lg";

/** The scale in px. Fine is Nova's h-6 / h-7 / h-8 / h-9. */
const HEIGHTS: Record<Mode, Record<Step, number>> = {
  off: { xs: 24, sm: 28, default: 32, lg: 36 },
  on: { xs: 36, sm: 40, default: 44, lg: 48 },
};
const TEXT_ENTRY_FONT: Record<Mode, string> = { off: "14px", on: "17px" };
const EPS = 0.5;

async function setMode(page: Page, mode: Mode): Promise<void> {
  await page.evaluate((m) => document.documentElement.setAttribute("data-dx-coarse", m), mode);
}

type Box = { name: string; top: number; bottom: number; left: number; width: number; height: number; scrollH: number; clientH: number; font: string };

/** Every `[data-probe]` inside the container, read at rest. */
async function readBoxes(root: import("@playwright/test").Locator): Promise<Record<string, Box[]>> {
  return readAtRest(root, (el) => {
    const out: Record<string, Box[]> = {};
    for (const node of Array.from(el.querySelectorAll<HTMLElement>("[data-probe]"))) {
      const r = node.getBoundingClientRect();
      const name = node.getAttribute("data-probe")!;
      (out[name] ??= []).push({
        name,
        top: r.top,
        bottom: r.bottom,
        left: r.left,
        width: r.width,
        height: r.height,
        scrollH: node.scrollHeight,
        clientH: node.clientHeight,
        font: getComputedStyle(node).fontSize,
      });
    }
    return out;
  });
}

function near(actual: number, expected: number, what: string): void {
  expect(Math.abs(actual - expected), `${what}: ${actual} vs ${expected}`).toBeLessThanOrEqual(EPS);
}

const ICON = '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14"/></svg>';
const btn = (probe: string, size: string, body: string, extra = "") =>
  `<button type="button" class="dx-button" data-probe="${probe}" data-clip data-size="${size}" data-style="outline" data-radius="default" ${extra}>${body}</button>`;

/** The probe rows. A row is one flex line, items top-aligned so equal heights are what equal tops mean. */
const PROBES = `
<style>[data-row] > * { flex: none; }</style>
<div data-row="default" style="display:flex;align-items:flex-start;gap:8px;margin-block:12px">
  ${btn("button-default", "default", "Agjy")}
  ${btn("button-icon", "icon", ICON)}
  ${btn("button-icon-default-addon", "icon-sm", ICON, "data-addon")}
  <input class="dx-input" data-probe="input" data-clip data-text-entry value="Agjy" aria-label="probe input" style="width:120px">
  <div class="dx-select"><button type="button" class="dx-select-trigger" data-probe="select" data-clip style="min-width:0;width:120px"><span>Agjy</span></button></div>
  <div class="dx-native-select-wrapper" style="width:120px"><select class="dx-native-select" data-probe="native-select" data-clip data-text-entry aria-label="probe native"><option>Agjy</option></select></div>
  <button type="button" class="dx-toggle" data-probe="toggle-default" data-clip data-size="default" aria-label="t"></button>
  <div class="dx-tabs" data-size="default" data-variant="default" data-orientation="horizontal"><div class="dx-tabs-list" data-probe="tabs-list-default" role="tablist"><button type="button" class="dx-tabs-trigger" data-probe="tabs-trigger-default" role="tab">Agjy</button><button type="button" class="dx-tabs-trigger" role="tab">Two</button></div></div>
  <button type="button" class="dx-toolbar-button" data-probe="toolbar-button" data-clip aria-label="b"></button>
  <a href="#" class="dx-pagination-link" data-probe="pagination-link" data-clip data-size="default" data-active="false">Agjy</a>
  <a href="#" class="dx-pagination-link" data-probe="pagination-icon" data-clip data-size="icon" data-active="false">1</a>
  <span class="dx-pagination-ellipsis" data-probe="pagination-ellipsis">${ICON}</span>
  <div class="dx-input-otp-group" data-probe="otp-group"><div class="dx-input-otp-slot" data-probe="otp-slot" data-clip>1</div><div class="dx-input-otp-slot">2</div></div>
  <div class="dx-date-picker-group" data-probe="date-picker-group" data-clip><span class="dx-date-picker-segment" data-probe="date-picker-segment" data-text-entry>10</span><span>/</span><span class="dx-date-picker-segment">25</span></div>
  <input class="dx-combobox-input" data-probe="combobox-input" data-clip data-text-entry value="Agjy" aria-label="probe combobox" style="width:120px">
  <div class="dx-input-group" data-probe="input-group" style="width:240px"><input class="dx-input dx-input-group-control" data-probe="input-group-control" data-clip data-text-entry value="Agjy" aria-label="probe group"><div class="dx-input-group-addon" data-align="end">${btn("input-group-addon-button", "xs", "Go", "data-addon")}</div></div>
</div>
<div data-row="sm" style="display:flex;align-items:flex-start;gap:8px;margin-block:12px">
  ${btn("button-sm", "sm", "Agjy")}
  ${btn("button-icon-sm", "icon-sm", ICON)}
  <div class="dx-select"><button type="button" class="dx-select-trigger" data-probe="select-sm" data-clip data-size="sm" style="min-width:0;width:120px"><span>Agjy</span></button></div>
  <div class="dx-native-select-wrapper" style="width:120px"><select class="dx-native-select" data-probe="native-select-sm" data-clip data-text-entry data-size="sm" aria-label="probe native sm"><option>Agjy</option></select></div>
  <button type="button" class="dx-toggle" data-probe="toggle-sm" data-clip data-size="sm" aria-label="t"></button>
  <div class="dx-tabs" data-size="sm" data-variant="default" data-orientation="horizontal"><div class="dx-tabs-list" data-probe="tabs-list-sm" role="tablist"><button type="button" class="dx-tabs-trigger" data-probe="tabs-trigger-sm" role="tab">Agjy</button></div></div>
</div>
<div data-row="lg" style="display:flex;align-items:flex-start;gap:8px;margin-block:12px">
  ${btn("button-lg", "lg", "Agjy")}
  ${btn("button-icon-lg", "icon-lg", ICON)}
  <button type="button" class="dx-toggle" data-probe="toggle-lg" data-clip data-size="lg" aria-label="t"></button>
  <div class="dx-tabs" data-size="lg" data-variant="default" data-orientation="horizontal"><div class="dx-tabs-list" data-probe="tabs-list-lg" role="tablist"><button type="button" class="dx-tabs-trigger" data-probe="tabs-trigger-lg" role="tab">Agjy</button></div></div>
</div>
<div data-row="xs" style="display:flex;align-items:flex-start;gap:8px;margin-block:12px">
  ${btn("button-xs", "xs", "Agjy")}
  ${btn("button-icon-xs", "icon-xs", ICON)}
</div>
<div style="width:260px"><div class="dx-command-input-wrapper" data-probe="command-input-wrapper"><input class="dx-command-input" data-probe="command-input" data-clip data-text-entry value="Agjy" aria-label="probe command"></div></div>
`;

/** probe name -> [scale step, square?]: the height each consumer reads from the scale. */
const CONSUMERS: Record<string, [Step, boolean]> = {
  "button-xs": ["xs", false],
  "button-sm": ["sm", false],
  "button-default": ["default", false],
  "button-lg": ["lg", false],
  "button-icon-xs": ["xs", true],
  "button-icon-sm": ["sm", true],
  "button-icon": ["default", true],
  "button-icon-lg": ["lg", true],
  "button-icon-default-addon": ["default", true],
  input: ["default", false],
  select: ["default", false],
  "select-sm": ["sm", false],
  "native-select": ["default", false],
  "native-select-sm": ["sm", false],
  "toggle-sm": ["sm", true],
  "toggle-default": ["default", true],
  "toggle-lg": ["lg", true],
  "tabs-list-sm": ["sm", false],
  "tabs-list-default": ["default", false],
  "tabs-list-lg": ["lg", false],
  "toolbar-button": ["default", true],
  "pagination-link": ["default", false],
  "pagination-icon": ["default", true],
  "pagination-ellipsis": ["default", true],
  "otp-slot": ["default", true],
  "date-picker-group": ["default", false],
  "combobox-input": ["default", false],
  "command-input-wrapper": ["default", false],
  "input-group": ["default", false],
};

for (const mode of ["off", "on"] as const) {
  test.describe(`data-dx-coarse="${mode}": the shipped controls`, () => {
    test.beforeEach(async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/button/`, { timeout: 20 * 60 * 1000 });
      await setMode(page, mode);
      await page.evaluate((html) => {
        document.getElementById("alignment-probe")?.remove();
        const host = document.createElement("div");
        host.id = "alignment-probe";
        host.style.cssText = "width:1500px;padding:16px;";
        host.innerHTML = html;
        document.body.append(host);
      }, PROBES);
    });

    test("every consumer is at its scale height, icon-only controls square", async ({ page }) => {
      const boxes = await readBoxes(page.locator("#alignment-probe"));
      for (const [name, [step, square]] of Object.entries(CONSUMERS)) {
        const [box] = boxes[name] ?? [];
        expect(box, `probe ${name} rendered`).toBeTruthy();
        near(box.height, HEIGHTS[mode][step], `${name} (${step}) height in ${mode}`);
        if (square) near(box.width, box.height, `${name} is square in ${mode}`);
      }
    });

    test("rows of mixed controls share one height and one top edge", async ({ page }) => {
      const root = page.locator("#alignment-probe");
      const rows = await readAtRest(root, (el) =>
        Array.from(el.querySelectorAll<HTMLElement>("[data-row]")).map((row) => ({
          row: row.getAttribute("data-row")!,
          items: Array.from(row.querySelectorAll<HTMLElement>("[data-probe]"))
            .map((n) => {
              const r = n.getBoundingClientRect();
              return { name: n.getAttribute("data-probe")!, top: r.top, height: r.height, bottom: r.bottom, nested: !!n.parentElement?.closest("[data-probe]") };
            }),
        })),
      );
      expect(rows.length).toBe(4);
      for (const { row, items } of rows) {
        // Only the controls placed directly in the row: parts inside a probe (a tab trigger in its list, an
        // input in its group) are checked against their parent below.
        const top = items.filter((i) => !i.nested);
        expect(top.length, `row ${row} has controls`).toBeGreaterThan(1);
        for (const i of top) {
          near(i.height, top[0].height, `${row}: ${i.name} height vs ${top[0].name} in ${mode}`);
          near(i.top, top[0].top, `${row}: ${i.name} top vs ${top[0].name} in ${mode}`);
          near(i.bottom, top[0].bottom, `${row}: ${i.name} bottom vs ${top[0].name} in ${mode}`);
        }
      }
    });

    test("an input group's addon button is centred in the group and the control fills it", async ({ page }) => {
      const boxes = await readBoxes(page.locator("#alignment-probe"));
      const [group] = boxes["input-group"];
      const [control] = boxes["input-group-control"];
      const [addon] = boxes["input-group-addon-button"];
      near(control.height, group.height, `group control height in ${mode}`);
      near(control.top, group.top, `group control top in ${mode}`);
      near(addon.top - group.top, group.bottom - addon.bottom, `addon button is vertically centred in ${mode}`);
      near(addon.height, HEIGHTS[mode].xs, `addon button (xs) height in ${mode}`);
    });

    test("tab triggers sit inside their list and are not clipped", async ({ page }) => {
      const boxes = await readBoxes(page.locator("#alignment-probe"));
      for (const step of ["sm", "default", "lg"] as const) {
        const [list] = boxes[`tabs-list-${step}`];
        const [trigger] = boxes[`tabs-trigger-${step}`];
        expect(trigger.top, `${step} trigger top inside list`).toBeGreaterThanOrEqual(list.top - EPS);
        expect(trigger.bottom, `${step} trigger bottom inside list`).toBeLessThanOrEqual(list.bottom + EPS);
        expect(trigger.height, `${step} trigger is tall enough for its text`).toBeGreaterThanOrEqual(HEIGHTS[mode][step] - 8);
      }
    });

    test("no text is clipped, text entry is 14px fine / 17px coarse", async ({ page }) => {
      const boxes = await readBoxes(page.locator("#alignment-probe"));
      const clipped = await readAtRest(page.locator("#alignment-probe"), (el) =>
        Array.from(el.querySelectorAll<HTMLElement>("[data-clip]"))
          .filter((n) => n.scrollHeight > n.clientHeight + 0.5)
          .map((n) => `${n.getAttribute("data-probe")} ${n.scrollHeight}>${n.clientHeight}`),
      );
      expect(clipped, `elements whose content is taller than their box in ${mode}`).toEqual([]);
      const entries = await readAtRest(page.locator("#alignment-probe"), (el) =>
        Array.from(el.querySelectorAll<HTMLElement>("[data-text-entry]")).map((n) => [n.getAttribute("data-probe"), getComputedStyle(n).fontSize]),
      );
      expect(entries.length).toBeGreaterThan(4);
      for (const [name, font] of entries) expect(font, `${name} font size in ${mode}`).toBe(TEXT_ENTRY_FONT[mode]);
      expect(Object.keys(boxes).length).toBeGreaterThan(20);
    });
  });
}

/**
 * The shipped demos of the components that compose mixed controls, measured in both modes on one load: the
 * controls in a row share a height and a top, and what the demo draws agrees with the scale.
 */
test.describe("shipped demos stay aligned in both modes", () => {
  test("input group: the control, the group and the addon agree", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/input_group/`, { timeout: 20 * 60 * 1000 });
    const frame = page.locator("#component-preview-frame").first();
    for (const mode of ["off", "on"] as const) {
      await setMode(page, mode);
      const groups = await readAtRest(frame, (el) =>
        Array.from(el.querySelectorAll<HTMLElement>(".dx-input-group"))
          .filter((g) => !g.querySelector('.dx-input-group-addon[data-align^="block"]'))
          .map((g) => {
            const r = g.getBoundingClientRect();
            const control = g.querySelector<HTMLElement>(".dx-input-group-control")!.getBoundingClientRect();
            return {
              top: r.top,
              bottom: r.bottom,
              height: r.height,
              control: { top: control.top, height: control.height },
              addons: Array.from(g.querySelectorAll<HTMLElement>(".dx-input-group-addon")).map((a) => {
                const ar = a.getBoundingClientRect();
                const inner = a.firstElementChild?.getBoundingClientRect() ?? ar;
                return { top: inner.top, bottom: inner.bottom };
              }),
            };
          }),
      );
      expect(groups.length, "single-row input groups on the page").toBeGreaterThan(0);
      for (const g of groups) {
        near(g.height, HEIGHTS[mode].default, `input group height in ${mode}`);
        near(g.control.height, g.height, `input group control height in ${mode}`);
        near(g.control.top, g.top, `input group control top in ${mode}`);
        for (const a of g.addons) near(a.top - g.top, g.bottom - a.bottom, `addon centred in the group in ${mode}`);
      }
    }
  });

  test("toolbar: every button is one height and top", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toolbar/`, { timeout: 20 * 60 * 1000 });
    const frame = page.locator("#component-preview-frame").first();
    for (const mode of ["off", "on"] as const) {
      await setMode(page, mode);
      const buttons = await readAtRest(frame, (el) =>
        Array.from(el.querySelectorAll<HTMLElement>(".dx-toolbar-button")).map((b) => {
          const r = b.getBoundingClientRect();
          return { top: r.top, height: r.height, group: Array.from(el.querySelectorAll(".dx-toolbar-group")).indexOf(b.parentElement!), clipped: b.scrollHeight > b.clientHeight + 0.5 };
        }),
      );
      expect(buttons.length).toBeGreaterThan(3);
      for (const b of buttons) {
        near(b.height, HEIGHTS[mode].default, `toolbar button height in ${mode}`);
        near(b.top, buttons.find((o) => o.group === b.group)!.top, `toolbar button top within its group in ${mode}`);
        expect(b.clipped, `toolbar button text clipped in ${mode}`).toBe(false);
      }
    }
  });

  test("button group: the buttons of a group share a height and a top", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
    const frame = page.locator("#component-preview-frame").first();
    for (const mode of ["off", "on"] as const) {
      await setMode(page, mode);
      const groups = await readAtRest(frame, (el) =>
        Array.from(el.querySelectorAll<HTMLElement>('[role="group"]')).map((g) =>
          Array.from(g.querySelectorAll<HTMLElement>(":scope > .dx-button, :scope > * > .dx-button")).map((b) => {
            const r = b.getBoundingClientRect();
            return { top: r.top, height: r.height, size: b.getAttribute("data-size") };
          }),
        ),
      );
      expect(groups.flat().length).toBeGreaterThan(2);
      for (const g of groups) {
        for (const b of g) {
          near(b.height, g[0].height, `button group item height in ${mode}`);
          near(b.top, g[0].top, `button group item top in ${mode}`);
        }
      }
      for (const b of groups.flat()) {
        if (b.size === "default" || b.size === "icon") near(b.height, HEIGHTS[mode].default, `default button in a group in ${mode}`);
      }
    }
  });

  test("pagination: the links are one height and top, icon links square", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/pagination/`, { timeout: 20 * 60 * 1000 });
    const frame = page.locator("#component-preview-frame").first();
    for (const mode of ["off", "on"] as const) {
      await setMode(page, mode);
      const items = await readAtRest(frame, (el) =>
        Array.from(el.querySelectorAll<HTMLElement>(".dx-pagination-link, .dx-pagination-ellipsis")).map((n) => {
          const r = n.getBoundingClientRect();
          return { top: r.top, width: r.width, height: r.height, icon: n.getAttribute("data-size") === "icon" || n.classList.contains("dx-pagination-ellipsis") };
        }),
      );
      expect(items.length).toBeGreaterThan(4);
      for (const i of items) {
        near(i.height, HEIGHTS[mode].default, `pagination item height in ${mode}`);
        near(i.top, items[0].top, `pagination item top in ${mode}`);
        if (i.icon) near(i.width, i.height, `pagination icon item is square in ${mode}`);
      }
    }
  });

  test("tabs: the list is the scale's default height and its triggers fit it", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tabs/`, { timeout: 20 * 60 * 1000 });
    const frame = page.locator("#component-preview-frame").first();
    for (const mode of ["off", "on"] as const) {
      await setMode(page, mode);
      const list = await readAtRest(frame, (el) => {
        const l = el.querySelector<HTMLElement>(".dx-tabs[data-size='default'] > .dx-tabs-list")!;
        const r = l.getBoundingClientRect();
        return {
          top: r.top,
          bottom: r.bottom,
          height: r.height,
          triggers: Array.from(l.querySelectorAll<HTMLElement>(".dx-tabs-trigger")).map((t) => {
            const tr = t.getBoundingClientRect();
            return { top: tr.top, bottom: tr.bottom };
          }),
        };
      });
      near(list.height, HEIGHTS[mode].default, `tabs list height in ${mode}`);
      expect(list.triggers.length).toBeGreaterThan(1);
      for (const t of list.triggers) {
        near(t.top, list.triggers[0].top, `tab trigger top in ${mode}`);
        expect(t.top, `trigger inside the list in ${mode}`).toBeGreaterThanOrEqual(list.top - EPS);
        expect(t.bottom, `trigger inside the list in ${mode}`).toBeLessThanOrEqual(list.bottom + EPS);
      }
    }
  });

  test("calendar: the caption row holds its controls and the nav buttons stay centred", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    const frame = page.locator("#component-preview-frame").first();
    for (const mode of ["off", "on"] as const) {
      await setMode(page, mode);
      const nav = await readAtRest(frame, (el) => {
        const n = el.querySelector<HTMLElement>(".dx-calendar-navigation")!;
        const r = n.getBoundingClientRect();
        return {
          top: r.top,
          bottom: r.bottom,
          height: r.height,
          kids: Array.from(n.querySelectorAll<HTMLElement>(".dx-calendar-nav-prev, .dx-calendar-nav-next, .dx-calendar-month-select-value, .dx-calendar-year-select-value, .dx-calendar-caption-text")).map((k) => {
            const kr = k.getBoundingClientRect();
            return { top: kr.top, bottom: kr.bottom };
          }),
        };
      });
      expect(nav.height, `caption row is at least the scale's xs in ${mode}`).toBeGreaterThanOrEqual(HEIGHTS[mode].xs - EPS);
      expect(nav.kids.length).toBeGreaterThan(1);
      for (const k of nav.kids) near(k.top - nav.top, nav.bottom - k.bottom, `caption control centred in its row in ${mode}`);
    }
  });
});
