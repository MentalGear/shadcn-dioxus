/**
 * Row 212 (next-round-plan item 10): every form-control demo of checkbox,
 * radio_group and switch is named by a VISIBLE label, the way shadcn's demos
 * pair a control with its text, not by an `aria-label` that no sighted user
 * can read.
 *
 * "Named by a visible label" means one of:
 *   - a `<label for="<control id>">` whose text is rendered,
 *   - a wrapping `<label>` whose text is rendered,
 *   - `aria-labelledby` naming rendered text,
 *   - the control's own rendered text (a RadioItem's content).
 * `aria-label` alone does not count. A reasoned allowlist covers the
 * exceptions; a stale allowlist entry (one that no longer matches a bare
 * control) fails the test, so the list cannot quietly rot.
 *
 * Scope: checkbox, radio_group and switch. slider and progress are not in
 * this list: their demos are bare in shadcn's own demo files too, and most of
 * their non-main variants are aria-label-only, so they need their own
 * decision (row 212, follow-up), not a silent exemption here.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app

const CONTROL_PAGES = ["checkbox", "radio_group", "switch"] as const;

/** `<component>/<variant>` -> why its bare controls are acceptable. */
const ALLOWLIST: Record<string, string> = {
  "checkbox/table":
    "bulk-select column of a data table: each row checkbox is named by its row " +
    "(aria-label 'Select member <name>'), and the name is in the adjacent cell, " +
    "which is the table's own labelling pattern, not a form field's",
};

interface Control {
  variant: string;
  role: string;
  labelled: boolean;
  ariaLabel: string | null;
}

/** Runs in the page: every checkbox/radio/switch inside a demo frame, and
 * whether its name comes from rendered text. */
function collectControls(): Control[] {
  const visible = (e: Element | null): boolean => {
    if (!e) return false;
    return e.getClientRects().length > 0 && getComputedStyle(e).visibility !== "hidden";
  };
  const textOf = (e: Element) => (e.textContent ?? "").replace(/\s+/g, " ").trim();
  const frames = Array.from(
    document.querySelectorAll('[id="component-preview-frame"], [id^="component-preview-frame-"]'),
  );
  const out: Control[] = [];
  for (const frame of frames) {
    const variant =
      frame.id === "component-preview-frame" ? "main" : frame.id.slice("component-preview-frame-".length);
    for (const el of Array.from(frame.querySelectorAll('[role="checkbox"], [role="radio"], [role="switch"]'))) {
      const visibleText: string[] = [];
      if (el.id) {
        for (const l of Array.from(document.querySelectorAll(`label[for="${CSS.escape(el.id)}"]`))) {
          if (visible(l) && textOf(l)) visibleText.push(textOf(l));
        }
      }
      const wrapper = el.closest("label");
      if (wrapper && visible(wrapper) && textOf(wrapper)) visibleText.push(textOf(wrapper));
      const labelledby = (el.getAttribute("aria-labelledby") ?? "").split(/\s+/).filter(Boolean);
      if (labelledby.length) {
        const parts = labelledby.map((id) => document.getElementById(id));
        if (parts.every((p) => visible(p) && textOf(p!))) {
          visibleText.push(parts.map((p) => textOf(p!)).join(" "));
        }
      }
      if (visible(el) && textOf(el)) visibleText.push(textOf(el));
      out.push({
        variant,
        role: el.getAttribute("role") ?? "",
        labelled: visibleText.length > 0,
        ariaLabel: el.getAttribute("aria-label"),
      });
    }
  }
  return out;
}

async function openComponent(page: Page, name: string) {
  await gotoHydrated(page, `${BASE_URL}/component/${name}/`, { timeout: NAV_TIMEOUT });
  await expect(page.locator("#component-preview-frame")).toBeAttached();
}

test.describe("row 212: form-control demos are named by a visible label", () => {
  for (const component of CONTROL_PAGES) {
    test(`${component}: every control demo has a visible label, bar reasoned exceptions`, async ({ page }) => {
      await openComponent(page, component);
      const controls = await page.evaluate(collectControls);
      expect(controls.length, `${component} renders controls to check`).toBeGreaterThan(0);

      const bare = controls.filter((c) => !c.labelled);
      const bareKeys = new Set(bare.map((c) => `${component}/${c.variant}`));
      const failing = [...bareKeys].filter((key) => !(key in ALLOWLIST));
      expect(
        failing,
        `variants whose controls are named only by aria-label (or nothing): ${JSON.stringify(
          bare.map((c) => ({ variant: c.variant, role: c.role, ariaLabel: c.ariaLabel })),
        )}`,
      ).toEqual([]);

      for (const key of Object.keys(ALLOWLIST)) {
        if (!key.startsWith(`${component}/`)) continue;
        expect(bareKeys.has(key), `stale allowlist entry ${key}: no bare control is left there`).toBe(true);
      }
    });
  }

  test("the detector itself: aria-label alone is bare, a visible label is not", async ({ page }) => {
    await openComponent(page, "switch");
    // A synthetic demo frame, appended and removed again, so the detector is
    // proven against both shapes of control it has to tell apart.
    await page.evaluate(() => {
      const frame = document.createElement("div");
      frame.id = "component-preview-frame-calibration";
      frame.innerHTML =
        '<button role="switch" aria-label="Only aria"></button>' +
        '<button role="switch" id="cal-visible"></button><label for="cal-visible">Visible label</label>';
      document.body.appendChild(frame);
    });
    const cal = (await page.evaluate(collectControls)).filter((c) => c.variant === "calibration");
    await page.evaluate(() => document.getElementById("component-preview-frame-calibration")?.remove());
    expect(cal.map((c) => c.labelled)).toEqual([false, true]);
  });
});
