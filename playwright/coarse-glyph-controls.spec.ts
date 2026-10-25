import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

/**
 * Backlog row 420: the glyph-sized controls (checkbox, radio, switch, tag remove button, slider thumb, accordion
 * trigger) follow the one touch-sizing switch, `data-dx-coarse="auto|on|off"` on `<html>`, through the global
 * hit size `--dx-control-hit-size` (24px fine, 44px coarse). Fine (`off`) keeps today's layout (the layout box,
 * the border box plus its negative margins, equals the desktop sizes below) and the target is 24px; coarse (`on`)
 * gives every control a target of at least 44px, with the visible glyph unchanged, and no two targets of a demo
 * overlap. The TagGroup remove button is a real target on desktop too.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8106 npx playwright test --config=ssg.local.config.ts coarse-glyph-controls.spec.ts
 */

type Size = { w: number; h: number };
type Item = {
  /** The preview frame the control lives in. */
  frame: string;
  /** The control's border box. */
  box: Size;
  /** The border box plus its margins: what the control takes in its parent's layout. */
  layout: Size;
  /** The hit target: the border box, or the thumb's `::after` for a slider. */
  target: { x: number; y: number; w: number; h: number };
  /** The visible glyph. */
  glyph: Size;
};

type Control = {
  name: string;
  route: string;
  selector: string;
  /** In-page reader of one element's glyph size. */
  glyph: string;
  fine: { layout?: (s: Size) => boolean; glyph: Size };
  coarse: { target: Size; glyph?: Size };
};

const near = (a: number, b: number) => Math.abs(a - b) < 0.6;

const CONTROLS: Control[] = [
  {
    name: "checkbox",
    route: "/component/checkbox/",
    selector: ".dx-checkbox",
    glyph: `(el) => { const r = el.firstElementChild.getBoundingClientRect(); return { w: r.width, h: r.height }; }`,
    fine: { layout: (s) => near(s.w, 16) && (near(s.h, 16) || near(s.h, 17)), glyph: { w: 16, h: 16 } },
    coarse: { target: { w: 44, h: 44 }, glyph: { w: 16, h: 16 } },
  },
  {
    name: "radio",
    route: "/component/radio_group/",
    selector: ".dx-radio-group-item",
    glyph: `(el) => { const c = getComputedStyle(el, "::before"); return { w: parseFloat(c.width), h: parseFloat(c.height) }; }`,
    fine: { layout: (s) => near(s.h, 16) || near(s.h, 17), glyph: { w: 16, h: 16 } },
    coarse: { target: { w: 44, h: 44 }, glyph: { w: 16, h: 16 } },
  },
  {
    name: "switch",
    route: "/component/switch/",
    selector: ".dx-switch",
    glyph: `(el) => { const c = getComputedStyle(el, "::before"); return { w: parseFloat(c.width), h: parseFloat(c.height) }; }`,
    fine: { layout: (s) => near(s.h, 18.39) || near(s.h, 14), glyph: { w: 0, h: 0 } },
    coarse: { target: { w: 44, h: 44 } },
  },
  {
    name: "tag remove button",
    route: "/component/tag_group/",
    selector: ".dx-tag-group-remove-button",
    glyph: `(el) => { const r = el.querySelector("svg").getBoundingClientRect(); return { w: r.width, h: r.height }; }`,
    fine: { glyph: { w: 12, h: 12 } },
    coarse: { target: { w: 44, h: 44 }, glyph: { w: 12, h: 12 } },
  },
  {
    name: "slider thumb",
    route: "/component/slider/",
    selector: ".dx-slider-thumb",
    glyph: `(el) => { const r = el.getBoundingClientRect(); return { w: r.width, h: r.height }; }`,
    fine: { glyph: { w: 12, h: 12 } },
    coarse: { target: { w: 44, h: 44 }, glyph: { w: 20, h: 20 } },
  },
  {
    name: "accordion trigger",
    route: "/component/accordion/",
    selector: "#component-preview-frame .dx-accordion-trigger",
    glyph: `(el) => { const r = el.getBoundingClientRect(); return { w: r.width, h: r.height }; }`,
    fine: { layout: (s) => near(s.h, 38) || near(s.h, 54), glyph: { w: 240, h: 38 } },
    coarse: { target: { w: 0, h: 44 } },
  },
];

/** Every visible control of the page, read at rest with `data-dx-coarse` set on `<html>`. */
async function read(page: Page, c: Control, mode: "on" | "off"): Promise<Item[]> {
  await page.evaluate((m) => document.documentElement.setAttribute("data-dx-coarse", m), mode);
  return readAtRest(
    page.locator(c.selector).first(),
    (_el, arg: { selector: string; glyph: string }) => {
      const glyph = new Function(`return ${arg.glyph}`)() as (el: Element) => Size;
      return [...document.querySelectorAll(arg.selector)]
        .filter((e) => e.getClientRects().length)
        .map((e): Item => {
          const r = e.getBoundingClientRect();
          const after = getComputedStyle(e, "::after");
          const thumb = e.classList.contains("dx-slider-thumb") && after.content !== "none";
          const w = thumb ? parseFloat(after.width) : r.width;
          const h = thumb ? parseFloat(after.height) : r.height;
          const m = getComputedStyle(e);
          return {
            layout: { w: r.width + parseFloat(m.marginLeft) + parseFloat(m.marginRight), h: r.height + parseFloat(m.marginTop) + parseFloat(m.marginBottom) },
            frame: e.closest("[id^=component-preview-frame]")?.id ?? "-",
            box: { w: r.width, h: r.height },
            target: { x: r.x + r.width / 2 - w / 2, y: r.y + r.height / 2 - h / 2, w, h },
            glyph: glyph(e),
          };
        });
    },
    { selector: c.selector, glyph: c.glyph },
  );
}

const overlaps = (items: Item[]) => {
  const out: string[] = [];
  const same = items.filter((i) => i.frame !== "-");
  for (let i = 0; i < same.length; i++)
    for (let j = i + 1; j < same.length; j++) {
      const a = same[i].target, b = same[j].target;
      if (a.x < b.x + b.w - 0.5 && b.x < a.x + a.w - 0.5 && a.y < b.y + b.h - 0.5 && b.y < a.y + a.h - 0.5)
        out.push(`${same[i].frame} #${i} x #${j}`);
    }
  return out;
};

for (const c of CONTROLS) {
  test.describe(c.name, () => {
    test.beforeEach(async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}${c.route}`);
    });

    test("fine (off): today's layout, a 24px target, targets do not overlap", async ({ page }) => {
      const items = await read(page, c, "off");
      expect(items.length).toBeGreaterThan(0);
      for (const it of items) {
        if (c.fine.layout) expect(c.fine.layout(it.layout), `${c.name} fine layout ${JSON.stringify(it.layout)}`).toBe(true);
        if (c.name !== "accordion trigger") {
          // A switch in a narrow flex row is squeezed to a narrower track on a fine pointer, so its width is not asserted.
          if (c.name !== "switch") expect(it.target.w, `${c.name} fine target width`).toBeGreaterThanOrEqual(24 - 0.5);
          expect(it.target.h, `${c.name} fine target height`).toBeGreaterThanOrEqual(24 - 0.5);
        }
        if (c.fine.glyph.w > 0 && c.name !== "accordion trigger") {
          expect(it.glyph.w).toBeCloseTo(c.fine.glyph.w, 0);
          expect(it.glyph.h).toBeCloseTo(c.fine.glyph.h, 0);
        }
      }
      expect(overlaps(items)).toEqual([]);
    });

    test("coarse (on): targets are at least 44px, the glyph keeps its size, targets do not overlap", async ({ page }) => {
      const fine = await read(page, c, "off");
      const items = await read(page, c, "on");
      expect(items.length).toBe(fine.length);
      items.forEach((it, i) => {
        expect(it.target.h, `${c.name} #${i} target height`).toBeGreaterThanOrEqual(44 - 0.5);
        expect(it.target.w, `${c.name} #${i} target width`).toBeGreaterThanOrEqual(c.coarse.target.w - 0.5);
        if (c.coarse.glyph) {
          expect(it.glyph.w).toBeCloseTo(c.coarse.glyph.w, 0);
          expect(it.glyph.h).toBeCloseTo(c.coarse.glyph.h, 0);
        }
      });
      expect(overlaps(items)).toEqual([]);
    });
  });
}

test("tag: a fine tag is no taller than before and a coarse tag holds its 44px remove target", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/tag_group/`);
  const heights = async (mode: "on" | "off") => {
    await page.evaluate((m) => document.documentElement.setAttribute("data-dx-coarse", m), mode);
    return readAtRest(page.locator(".dx-tag-group-tag").first(), () =>
      [...document.querySelectorAll(".dx-tag-group-tag")].map((t) => {
        const r = t.getBoundingClientRect();
        const b = t.querySelector(".dx-tag-group-remove-button")?.getBoundingClientRect();
        return { tag: r.height, inside: !b || (b.top >= r.top - 0.5 && b.bottom <= r.bottom + 0.5) };
      }),
    );
  };
  for (const t of await heights("off")) expect(t.tag).toBeCloseTo(28, 1);
  for (const t of await heights("on")) {
    expect(t.tag).toBeGreaterThanOrEqual(44 - 0.5);
    expect(t.inside).toBe(true);
  }
});

test("slider: the coarse thumb's hit area is 44px (a press 21px from its centre reaches the thumb)", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/slider/`);
  await page.evaluate(() => document.documentElement.setAttribute("data-dx-coarse", "on"));
  const thumb = page.locator("#component-preview-frame .dx-slider-thumb").first();
  await thumb.scrollIntoViewIfNeeded();
  const hit = await readAtRest(thumb, (el) => {
    const r = el.getBoundingClientRect();
    const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
    return [[21, 0], [-21, 0], [0, 21], [0, -21], [24, 0]].map(([dx, dy]) => document.elementFromPoint(cx + dx, cy + dy) === el);
  });
  expect(hit).toEqual([true, true, true, true, false]);
});

// A checkbox in a table or grid cell keeps the row's height (a windowed grid's rows are fixed), so its target is
// capped at the small control height and pulled back to the glyph: neighbouring rows' targets never overlap.
for (const route of ["/component/data_table/", "/component/virtual_grid/", "/component/table/"]) {
  test(`cell checkboxes on ${route}: capped targets that never overlap, in both modes`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}${route}`);
    const cell: Control = { ...CONTROLS[0], route, selector: ':is(td, th, [role="gridcell"], [role="rowheader"], [role="columnheader"]) .dx-checkbox' };
    for (const [mode, cap] of [["off", 24], ["on", 36]] as const) {
      const items = await read(page, cell, mode);
      expect(items.length, `${route} has cell checkboxes`).toBeGreaterThan(0);
      for (const i of items) {
        expect(i.target.h, `${mode}: target capped at ${cap}px`).toBeLessThanOrEqual(cap + 0.5);
        expect(i.glyph.h, `${mode}: the glyph stays 16px`).toBeCloseTo(16, 0);
      }
      expect(overlaps(items), `${mode}: no two cell targets overlap`).toEqual([]);
    }
  });
}
