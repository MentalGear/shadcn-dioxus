/**
 * A mouse gesture on a timeline the test controls.
 *
 * A gesture whose outcome reads the release velocity (a carousel fling, a toast flick) is driven by Playwright's
 * `page.mouse.move` calls, and the browser stamps each trusted event with the moment it RECEIVED it. The spacing of the
 * stamps is then the spacing of the CDP round trips, which stretches on a loaded machine and turns a fast fling into
 * a slow drag: the verdict depends on the machine, not on the code.
 *
 * `dragOnTimeline` sends the same trusted mouse events through `Input.dispatchMouseEvent` with an explicit `timestamp`
 * per event, so the page sees `event.timeStamp` values exactly `atMs` apart however long each call takes. The
 * handlers, hit-testing, pointer capture and default actions are the browser's own; only the clock is the test's.
 *
 *     await dragOnTimeline(page, { x, y }, [{ x: x - 20, y, atMs: 8 }, { x: x - 40, y, atMs: 16 }]);
 *
 * `atMs` counts from the press. The release happens `releaseAfterMs` after the last move (default 0: released while
 * still moving, which is what a flick is).
 */
import type { Page } from "@playwright/test";

export interface TimelineStep {
  x: number;
  y: number;
  /** Milliseconds after the press at which the pointer is at (x, y). */
  atMs: number;
}

export async function dragOnTimeline(
  page: Page,
  from: { x: number; y: number },
  steps: readonly TimelineStep[],
  releaseAfterMs = 0,
): Promise<void> {
  const cdp = await page.context().newCDPSession(page);
  try {
    // One epoch for every event of the gesture; the browser converts it to its own clock.
    const t0 = Date.now() / 1000;
    const stamp = (ms: number) => t0 + ms / 1000;
    await page.mouse.move(from.x, from.y);
    const base = { button: "left" as const, clickCount: 1, pointerType: "mouse" as const };
    await cdp.send("Input.dispatchMouseEvent", {
      type: "mousePressed", x: from.x, y: from.y, buttons: 1, timestamp: stamp(0), ...base,
    });
    let last = { x: from.x, y: from.y, atMs: 0 };
    for (const s of steps) {
      await cdp.send("Input.dispatchMouseEvent", {
        type: "mouseMoved", x: s.x, y: s.y, buttons: 1, timestamp: stamp(s.atMs), ...base,
      });
      last = s;
    }
    await cdp.send("Input.dispatchMouseEvent", {
      type: "mouseReleased", x: last.x, y: last.y, buttons: 0, timestamp: stamp(last.atMs + releaseAfterMs), ...base,
    });
  } finally {
    await cdp.detach().catch(() => {});
  }
}
