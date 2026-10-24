/**
 * Same-run main-thread cost measurement for the release-only timing specs (backlog row 402).
 *
 * A millisecond threshold written as a constant ("under 12 ms per frame") goes red when the shared
 * 4-core box is busy with other builds, because the machine's load scales the measurement and not the
 * constant. What a component promises is a relation: "a chunked transcript costs a fraction of what the
 * same rows cost with one skippable element per row". A spec therefore measures the component's path AND a
 * reference path on the same page in the same run, interleaved, and asserts the relation between the two:
 * load multiplies both sides, so the ratio holds. `perFrameCost` is the one measurement (a spec runs several
 * interleaved rounds per path and compares the minimum of each: load only ever adds time, so the minimum is the
 * least disturbed round), and `calibrationMs` reports how fast the box ran a fixed workload at that moment, so
 * a result can be read against the load it was measured under.
 *
 * Where the promise IS an absolute budget (the 50 ms Long Tasks definition in `main-thread.ts`), that
 * module says so and keeps the constant.
 */
import type { Page } from "@playwright/test";

/** Chromium main-thread task time (`Performance.getMetrics` `TaskDuration`) per frame of `run`. */
export async function frameCostMeter(page: Page) {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Performance.enable");
  const taskSeconds = async () => {
    const { metrics } = await cdp.send("Performance.getMetrics");
    return metrics.find((m) => m.name === "TaskDuration")!.value;
  };
  return {
    /**
     * Idles 500 ms, runs `run` (which drives the page and resolves with the number of frames it drove,
     * including its own settle time), and returns milliseconds of main-thread task time per frame.
     */
    async perFrameCost(run: () => Promise<number>): Promise<number> {
      await page.waitForTimeout(500);
      const before = await taskSeconds();
      const frames = await run();
      return (((await taskSeconds()) - before) * 1000) / frames;
    },
  };
}

/**
 * Milliseconds the page needs for a fixed arithmetic workload (the median of seven runs, so a box that
 * preempts the page shows it, where a minimum would hide it). The same workload takes about 20 ms on a quiet
 * box of this class, so a larger number says the result next to it was measured under load. A report, never
 * an assertion.
 */
export async function calibrationMs(page: Page): Promise<number> {
  return page.evaluate(() => {
    const runs: number[] = [];
    for (let run = 0; run < 7; run++) {
      const start = performance.now();
      let x = 0;
      for (let i = 0; i < 20_000_000; i++) x = (x * 31 + i) | 0;
      if (x === 42) console.log(x);
      runs.push(performance.now() - start);
    }
    return runs.sort((a, b) => a - b)[3];
  });
}
