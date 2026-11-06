/**
 * The two waits of a "nothing moves by itself" spec (`layout-stability.spec.ts`, `charts-hydration-stability.spec.ts`),
 * each as the signal it stands for instead of a wall-clock window.
 *
 * `holdWasmUntilPainted`: the wasm request waits until the prerendered page is on screen (the browser reports a paint
 * entry, buffered, so one that already happened counts) and three animation frames have shown it, so the prerendered
 * frame is painted at every width on every machine. Nothing polls and nothing sleeps.
 *
 * `settleUntilQuiet`: after hydration, resolves when everything that can move a box by itself has settled: the web
 * fonts are loaded, every image has loaded, no timer that can fire inside the window is pending (`trackPendingTimers`), no
 * finite CSS animation or transition is running, and `QUIET_FRAMES`
 * consecutive animation frames have passed in which neither the recorder saw a box event nor the DOM mutated. Any box
 * event or mutation restarts the count, so a page that keeps changing is observed for as long as it changes. The
 * ceiling `CEILING_MS` is the observation window the specs used to wait in full: a page that never goes quiet (a live
 * region that mutates every frame) is judged on that window, so the check never watches less than before.
 */
import type { Page } from "@playwright/test";

/** Consecutive frames without a box event or a DOM mutation that count as "settled" (~0.5 s at 60 Hz). */
export const QUIET_FRAMES = 30;
/** The longest observation window: what the specs waited in full before they waited on signals. */
export const CEILING_MS = 1500;

/**
 * Count the page's pending timers that can fire inside the observation window (`window.__pendingTimers`): a box that
 * moves "by itself" is moved by a timer, a network response, an animation or an input, and the page is not quiet while
 * a timer that could still move one is pending. Install before the navigation, like the recorder.
 */
export async function trackPendingTimers(page: Page): Promise<void> {
  await page.addInitScript((ceilingMs) => {
    const w = window as unknown as { __pendingTimers: number };
    w.__pendingTimers = 0;
    const set = window.setTimeout.bind(window);
    const clear = window.clearTimeout.bind(window);
    const live = new Map<number, true>();
    window.setTimeout = ((fn: TimerHandler, delay?: number, ...args: unknown[]) => {
      if ((delay ?? 0) > ceilingMs) return set(fn, delay, ...args);
      const id: number = set(() => {
        if (live.delete(id)) w.__pendingTimers = live.size;
        if (typeof fn === "function") fn(...args);
      }, delay);
      live.set(id, true);
      w.__pendingTimers = live.size;
      return id;
    }) as typeof window.setTimeout;
    window.clearTimeout = ((id?: number) => {
      if (id !== undefined && live.delete(id)) w.__pendingTimers = live.size;
      clear(id);
    }) as typeof window.clearTimeout;
  }, CEILING_MS);
}

export async function holdWasmUntilPainted(page: Page): Promise<void> {
  await page.route("**/*.wasm", async (route) => {
    await page
      .evaluate(
        () =>
          new Promise<void>((resolve) => {
            const frames = (n: number) => (n === 0 ? resolve() : requestAnimationFrame(() => frames(n - 1)));
            new PerformanceObserver((_list, observer) => {
              observer.disconnect();
              frames(3);
            }).observe({ type: "paint", buffered: true });
          }),
      )
      .catch(() => {});
    await route.continue().catch(() => {});
  });
}

/**
 * Wait until the page is quiet (see the header). `progress` is a page-side expression whose value changes whenever the
 * recorder has seen something move (a box event count, the number of recorded frames with a new state).
 */
export async function settleUntilQuiet(page: Page, progress: string): Promise<void> {
  await page.evaluate(
    ({ progress, quietFrames, ceilingMs }) =>
      new Promise<void>((resolve) => {
        const read = new Function(`return (${progress});`) as () => unknown;
        const start = performance.now();
        let mutated = true;
        new MutationObserver(() => (mutated = true)).observe(document, { subtree: true, childList: true, attributes: true, characterData: true });
        let last = read();
        let quiet = 0;
        const settled = () => {
          if (document.fonts.status !== "loaded") return false;
          if ([...document.images].some((img) => !img.complete && img.loading !== "lazy")) return false;
          if ((window as unknown as { __pendingTimers?: number }).__pendingTimers) return false;
          return !document.getAnimations().some((a) => {
            const end = a.effect?.getComputedTiming().endTime;
            return a.playState === "running" && typeof end === "number" && Number.isFinite(end);
          });
        };
        const tick = () => {
          const now = read();
          const idle = settled();
          if (now !== last || mutated || !idle) quiet = 0;
          else quiet++;
          last = now;
          mutated = false;
          if (quiet >= quietFrames || performance.now() - start >= ceilingMs) return resolve();
          requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      }),
    { progress, quietFrames: QUIET_FRAMES, ceilingMs: CEILING_MS },
  );
}
