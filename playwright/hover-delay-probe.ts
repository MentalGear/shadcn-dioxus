/**
 * An in-page event log for the Tooltip / HoverCard `delay` / `close_delay` specs
 * (`tooltip.spec.ts`, `hover-card.spec.ts`; backlog row 166).
 *
 * A delay is a lower bound the product promises ("nothing opens before `delay`"), so it is measured
 * where it happens: the page stamps `performance.now()` on the pointer/focus event that starts a
 * delay and on the moment the `role="tooltip"` surface mounts or flips its `data-state`, and the
 * spec compares the two stamps. Measuring from the spec process instead (Date.now() around a
 * `hover()` call) adds the protocol round trip to every number and, on a slow debug build, can
 * only make an early open look late -- never the reverse -- so only the lower bound is asserted;
 * an upper bound would be a timing assertion and belongs on release builds.
 */
import type { Page } from "@playwright/test";

export type ProbeEntry = { t: number; what: string };

/** (Re)start logging. Idempotent: a second call clears the log and keeps one set of listeners. */
export async function startProbe(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as { __hoverProbe?: { log: ProbeEntry[] } };
    if (w.__hoverProbe) {
      w.__hoverProbe.log.length = 0;
      return;
    }
    const log: ProbeEntry[] = [];
    w.__hoverProbe = { log };
    const state = () => document.querySelector('[role="tooltip"]')?.getAttribute("data-state") ?? "none";
    let last = state();
    new MutationObserver(() => {
      const s = state();
      if (s !== last) {
        log.push({ t: performance.now(), what: `surface:${s}` });
        last = s;
      }
    }).observe(document.body, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ["data-state"],
    });
    for (const type of ["mouseover", "mouseout", "focusin", "focusout"]) {
      document.addEventListener(type, () => log.push({ t: performance.now(), what: type }), true);
    }
  });
}

export async function readProbe(page: Page): Promise<ProbeEntry[]> {
  return page.evaluate(
    () => (window as unknown as { __hoverProbe: { log: ProbeEntry[] } }).__hoverProbe.log.slice(),
  );
}

export async function clearProbe(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as unknown as { __hoverProbe: { log: ProbeEntry[] } }).__hoverProbe.log.length = 0;
  });
}

/**
 * Milliseconds from the first `from` entry to the first `to` entry after it, or `null` when either
 * is missing. Entries are `mouseover` / `mouseout` / `focusin` / `focusout` / `surface:open` /
 * `surface:closed`.
 */
export function elapsed(log: ProbeEntry[], from: string, to: string): number | null {
  const start = log.findIndex((e) => e.what === from);
  if (start < 0) return null;
  const end = log.findIndex((e, i) => i > start && e.what === to);
  return end < 0 ? null : log[end].t - log[start].t;
}
