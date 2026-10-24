/**
 * Release-only timing spec for the Message Scroller's streaming path (backlog row 213 item 8; the
 * scroll-frame cost of the same 2,000-row transcript is in `message_scroller.spec.ts`).
 *
 * The promise is a relation, not a number of milliseconds: rows appended the way `MessageScrollerRows`
 * renders them (into chunks of 20, the unit `content-visibility` skips) cost the main thread clearly less
 * per frame than the same rows appended as direct children of the content (one skippable element per row).
 * The test measures both on the same page in the same run, interleaved, and asserts the ratio, so a busy
 * box scales both sides and the relation holds (`frame-cost.ts`, row 402). The per-frame time is only
 * meaningful on a release build, so the test is skipped unless the build under test is a known release build
 * (CLAUDE.md: timing specs only on release; `PW_BUILD_PROFILE=release` or `SSG_SITE_DIR` names it).
 */
import { test, expect } from "./fixtures";
import { BASE_URL, buildProfile } from "./base-url";
import { gotoHydrated } from "./hydration";
import { calibrationMs, frameCostMeter } from "./frame-cost";

/** Chunked append must cost at most this fraction of the per-row append (measured ~0.55-0.65; no chunking is ~1.0). */
const MAX_CHUNKED_TO_PER_ROW_RATIO = 0.85;
const ROUNDS = 3;

test("appending 1,000 rows to a 2,000-row transcript while following costs less chunked than per row (release only)", async ({
  page,
}) => {
  // Wall time is not what is measured (main-thread task time per frame is), but a loaded box stretches the
  // interleaved rounds past the default timeout.
  test.setTimeout(5 * 60_000);
  test.skip(
    buildProfile() !== "release",
    "per-frame main-thread time is only measured on a release build (PW_BUILD_PROFILE=release)",
  );
  await gotoHydrated(page, `${BASE_URL}/component/message_scroller/`, { timeout: 20 * 60 * 1000 });
  const frame = page.locator("#component-preview-frame-long").first();
  await frame.scrollIntoViewIfNeeded();
  const vp = frame.locator("[data-message-scroller-viewport]");
  await expect(vp).not.toHaveAttribute("data-pending-scroll");
  const distance = () => vp.evaluate((el) => Math.round(el.scrollHeight - el.scrollTop - el.clientHeight));
  await expect.poll(distance).toBeLessThanOrEqual(1);

  const meter = await frameCostMeter(page);
  // A stream: ten rows per frame for a hundred frames, cloned from the last real row. "chunked" adds them the
  // way `MessageScrollerRows` renders them (into the last chunk of 20, opening a new chunk when it is full);
  // "per-row" adds them as direct children of the content. Every node added is remembered, so `restore` puts
  // the transcript back to its 2,000 rows before the next measurement.
  const stream = (mode: "chunked" | "per-row") => () =>
    vp.evaluate(async (el, mode) => {
      const w = window as unknown as { __streamAdded: HTMLElement[] };
      w.__streamAdded = [];
      const content = el.querySelector("[data-message-scroller-content]")!;
      const spacer = content.querySelector("[data-message-scroller-spacer]")!;
      const chunks = content.querySelectorAll("[data-message-scroller-chunk]");
      let chunk = chunks[chunks.length - 1] as HTMLElement;
      const template = chunk.lastElementChild as HTMLElement;
      let frames = 0;
      for (let batch = 0; batch < 100; batch++) {
        for (let i = 0; i < 10; i++) {
          const row = template.cloneNode(true) as HTMLElement;
          row.setAttribute("data-message-id", `stream-${batch}-${i}`);
          if (mode === "chunked") {
            if (chunk.childElementCount >= 20) {
              chunk = chunk.cloneNode(false) as HTMLElement;
              content.insertBefore(chunk, spacer);
              w.__streamAdded.push(chunk);
            }
            chunk.appendChild(row);
          } else {
            content.insertBefore(row, spacer);
          }
          w.__streamAdded.push(row);
        }
        frames++;
        await new Promise((resolve) => requestAnimationFrame(resolve));
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
      return frames;
    }, mode);
  const restore = async () => {
    await vp.evaluate((el) => {
      const w = window as unknown as { __streamAdded: HTMLElement[] };
      for (const node of w.__streamAdded) node.remove();
      w.__streamAdded = [];
      el.scrollTop = el.scrollHeight;
    });
    await expect.poll(distance, { timeout: 30_000 }).toBeLessThanOrEqual(1);
  };
  const measure = async (mode: "chunked" | "per-row") => {
    const cost = await meter.perFrameCost(stream(mode));
    // Following lands exactly at the end after the burst.
    await expect.poll(distance, { timeout: 30_000 }).toBeLessThanOrEqual(1);
    await restore();
    return cost;
  };

  // Interleaved rounds, so a change in the box's load between rounds lands on both paths; the least disturbed
  // round of each path is the one compared.
  const chunked: number[] = [];
  const perRow: number[] = [];
  for (let round = 0; round < ROUNDS; round++) {
    chunked.push(await measure("chunked"));
    perRow.push(await measure("per-row"));
  }
  const best = { chunked: Math.min(...chunked), perRow: Math.min(...perRow) };
  const ratio = best.chunked / best.perRow;
  const calibration = await calibrationMs(page);
  console.log(
    `message_scroller long, ten appended rows per frame: chunked ${best.chunked.toFixed(1)} ms, per row ${best.perRow.toFixed(1)} ms ` +
      `(ratio ${ratio.toFixed(2)}, best of ${ROUNDS} interleaved rounds; box calibration ${calibration.toFixed(0)} ms)`,
  );
  // 3,000 rows in the DOM. Both paths pay the page's own cost for ten rows in a plain box; the per-row path adds
  // the browser's per-frame bookkeeping for a skippable element on every row.
  expect(ratio).toBeLessThan(MAX_CHUNKED_TO_PER_ROW_RATIO);
});
