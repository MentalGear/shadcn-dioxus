import type { Page } from "@playwright/test";

/**
 * The drop indicator (the line or highlight that marks where a dragged row lands) is the theme's accent
 * colour: one token, `--dx-drop-indicator`, defaulting to `var(--dx-primary)` and read by Drag and Drop
 * List, Data Table / Table reorder and the Virtual List reorder demo. It is NOT the focus ring (grey by
 * default). Shared by the three specs that drag a row (drag_and_drop_list, data_table_reorder, item-key).
 */

export type Mode = "light" | "dark";

/** The accent every drop-indicator spec uses: a coloured preset, so primary and the grey ring differ. */
export const ACCENT = "blue";

/** Apply a mode (through the one writer `__dxApplyMode`) and the accent preset, as the theme picker does. */
export async function applyModeAndAccent(page: Page, mode: Mode, accent: string = ACCENT) {
  await page.evaluate(
    ([m, a]) => {
      (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m);
      document.documentElement.setAttribute("data-theme-accent", a);
    },
    [mode, accent],
  );
}

/**
 * A theme token resolved to the colour the browser paints (a probe element's computed `background-color`,
 * so every spelling of a colour serialises the same way).
 */
export async function resolvedToken(page: Page, token: string): Promise<string> {
  return page.evaluate((t) => {
    const probe = document.createElement("div");
    probe.style.cssText = `position:fixed;width:1px;height:1px;background-color:var(${t})`;
    document.body.appendChild(probe);
    const colour = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return colour;
  }, token);
}

/**
 * The colour a drop indicator paints: the first colour of `prop` (`boxShadow` for a line drawn as
 * `inset 0 2px 0 <colour>`, `backgroundColor` for the Drag and Drop List's pseudo-element line) on the
 * first element matching `selector` (or its `pseudo`), normalised through the same probe as `resolvedToken`.
 */
export async function paintedColour(
  page: Page,
  selector: string,
  prop: "boxShadow" | "backgroundColor" | "borderTopColor",
  pseudo = "",
): Promise<string> {
  return page.evaluate(
    ([sel, property, ps]) => {
      const el = document.querySelector(sel)!;
      const value = getComputedStyle(el, ps || null)[property as "boxShadow"];
      const match = /(?:rgba?|oklch|oklab|lab|lch|color)\([^)]*\)/.exec(value);
      if (!match) return `no colour in ${property}: ${value}`;
      const probe = document.createElement("div");
      probe.style.cssText = `position:fixed;width:1px;height:1px;background-color:${match[0]}`;
      document.body.appendChild(probe);
      const colour = getComputedStyle(probe).backgroundColor;
      probe.remove();
      return colour;
    },
    [selector, prop, pseudo] as [string, string, string],
  );
}
