/**
 * Release-only timing spec for the Message Scroller's streaming path (backlog row 213 item 8; the
 * scroll-frame cost of the same 2,000-row transcript is in `message_scroller.spec.ts`).
 *
 * The one threshold here is a main-thread time per frame, meaningful only on a release build (debug
 * builds measure 5-10x slower), so it is skipped unless the build under test is a known release build
 * (CLAUDE.md: timing specs only on release; `PW_BUILD_PROFILE=release` or `SSG_SITE_DIR` names it) --
 * the same marker `message_scroller.spec.ts` and `oracle/tier2-html/main-thread.spec.ts` use.
 */
import { test, expect } from "./fixtures";
import { BASE_URL, buildProfile } from "./base-url";
import { gotoHydrated } from "./hydration";

test("appending 1,000 rows to a 2,000-row transcript while following costs the main thread little per frame (release only)", async ({
  page,
}) => {
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

  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Performance.enable");
  const taskSeconds = async () => {
    const { metrics } = await cdp.send("Performance.getMetrics");
    return metrics.find((m) => m.name === "TaskDuration")!.value;
  };
  await page.waitForTimeout(500);
  const before = await taskSeconds();
  // A stream: ten rows per frame for a hundred frames, cloned from the last real row and added the way
  // `MessageScrollerRows` renders them: into the last chunk of 20, opening a new chunk when it is full
  // (a chunk is the unit `content-visibility` skips).
  const frames = await vp.evaluate(async (el) => {
    const content = el.querySelector("[data-message-scroller-content]")!;
    const spacer = content.querySelector("[data-message-scroller-spacer]")!;
    const chunks = content.querySelectorAll("[data-message-scroller-chunk]");
    let chunk = chunks[chunks.length - 1] as HTMLElement;
    const template = chunk.lastElementChild as HTMLElement;
    let frames = 0;
    for (let batch = 0; batch < 100; batch++) {
      for (let i = 0; i < 10; i++) {
        if (chunk.childElementCount >= 20) {
          chunk = chunk.cloneNode(false) as HTMLElement;
          content.insertBefore(chunk, spacer);
        }
        const row = template.cloneNode(true) as HTMLElement;
        row.setAttribute("data-message-id", `stream-${batch}-${i}`);
        chunk.appendChild(row);
      }
      frames++;
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
    return frames;
  });
  const msPerFrame = (((await taskSeconds()) - before) * 1000) / frames;
  console.log(`message_scroller long: ${msPerFrame.toFixed(1)} ms of main-thread time per appended-frame (10 rows)`);
  // Following still lands exactly at the end after the burst.
  await expect.poll(distance, { timeout: 10_000 }).toBeLessThanOrEqual(1);
  // 3,000 rows in the DOM. On a loaded 4-core VM against a debug SSG build (this path is JS and
  // browser layout, which a release wasm build barely changes) a frame of ten appended rows costs
  // ~8.0 ms, of which about 4 ms is the page's own cost for ten rows in a plain box; appending the
  // same rows as unchunked direct children (per-row `content-visibility`) costs ~14.9 ms. 12 ms
  // separates the two and stays under the 16.7 ms frame budget.
  expect(msPerFrame).toBeLessThan(12);
});
