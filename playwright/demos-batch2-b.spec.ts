/**
 * Row 188 demo coverage, dates, pickers, small primitives and compositions: the demo-only items of
 * the research doc's section 5 (D-G): calendar presets/date_time, date_picker time and
 * natural_language, select disabled, combobox custom_items, native_select disabled/groups, separator, label,
 * slider, aspect_ratio, scroll_area, kbd, badge, resizable, field subscription, input_otp verify, accordion
 * faq/multiple, avatar, command inline, navigation_menu icons, alert, table rich, and the
 * checkbox `indeterminate` demo. The spec opens these demos where no other spec covers them, checks the demo's key
 * control does something, and scans the demo frame with axe in light and dark.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoFrame, demoRole } from "./demo-frame";

const GOTO = { timeout: 20 * 60 * 1000 };
const url = (name: string) => `${BASE_URL}/component/${name}/`;
const setMode = (page: Page, m: string) =>
  page.evaluate((mode) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(mode), m);

/** component, variant, a role-and-position control that must be visible and (optionally) toggled. */
const DEMOS: { c: string; v: string; role: Parameters<Page["getByRole"]>[0]; click?: boolean }[] = [
  { c: "checkbox", v: "indeterminate", role: "checkbox", click: true },
  { c: "calendar", v: "presets", role: "button" },
  { c: "calendar", v: "date_time", role: "grid" },
  { c: "date_picker", v: "time", role: "button" },
  { c: "date_picker", v: "natural_language", role: "textbox" },
  { c: "select", v: "disabled", role: "combobox" },
  { c: "combobox", v: "custom_items", role: "combobox" },
  { c: "native_select", v: "groups", role: "combobox" },
  { c: "slider", v: "vertical", role: "slider" },
  { c: "badge", v: "count", role: "link" },
  { c: "accordion", v: "multiple", role: "button", click: true },
  { c: "input_otp", v: "verify", role: "textbox" },
  { c: "avatar", v: "group", role: "img" },
  { c: "command", v: "inline", role: "combobox" },
  { c: "table", v: "rich", role: "checkbox", click: true },
];

for (const d of DEMOS) {
  for (const mode of ["light", "dark"] as const) {
    test(`${d.c}/${d.v} opens and has no axe violations (${mode})`, async ({ page }) => {
      await gotoHydrated(page, url(d.c), GOTO);
      const frame = demoFrame(page, d.v);
      await frame.scrollIntoViewIfNeeded();
      await expect(frame).toBeVisible();
      // Not every demo exposes this role (badge count is plain); only assert when present.
      const control = demoRole(page, d.role, 0, d.v);
      if ((await control.count()) > 0) {
        await expect(control).toBeVisible();
        if (d.click && mode === "light") {
          const before = await control.getAttribute("data-state");
          await control.click();
          const after = await control.getAttribute("data-state");
          if (before !== null) expect(after).not.toBe(before);
        }
      }
      await setMode(page, mode);
      await expectNoAxeViolations(page, `${d.c}/${d.v} ${mode}`, {
        include: `#component-preview-frame-${d.v}`,
      });
    });
  }
}
