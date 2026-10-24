import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

/**
 * A toast that is dismissed leaves the layout at once (Sonner's model): the toasts still present
 * close the gap WHILE it fades, so no remaining toast jumps when the leaving one unmounts, and a
 * toast that becomes visible slides and fades in from its start values. Every number here is read
 * once per animation frame (`requestAnimationFrame`) from the toast's own rect and computed style,
 * so a jump shows up as a step between two frames, not as a wrong value at rest.
 */
const STACK = "Stacked toast notifications";
const TOP = "Top toast notifications";
const PAUSE = "Pausing toast notifications";

const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => gotoHydrated(page, `${BASE_URL}/component/toast/`);

type Frame = { t: number; items: Record<string, { y: number; o: number; ty: number; removed: boolean }> };

/**
 * Starts a per-frame sampler of every toast of the region (by element id), stopped by `stopSampler`.
 * `keepCollapsed` blurs the region every frame: closing with the close button hands focus to the
 * region (so the pile would expand under the click), and a collapsed pile is what a countdown or
 * `Toasts::close` from code leaves, so the spec holds it collapsed to measure that case.
 */
async function startSampler(page: Page, name: string, keepCollapsed = false) {
  await region(page, name).evaluate((reg, blur) => {
    const w = window as unknown as { __dxF: Frame[]; __dxStop: boolean };
    w.__dxF = [];
    w.__dxStop = false;
    const tick = () => {
      const active = document.activeElement as HTMLElement | null;
      if (blur && active && reg.contains(active)) active.blur();
      const items: Frame["items"] = {};
      reg.querySelectorAll<HTMLElement>('[role="alertdialog"]').forEach((el) => {
        const r = el.getBoundingClientRect(); // rest-ok: the sampler measures motion itself, frame by frame
        const cs = getComputedStyle(el);
        const ty = cs.translate === "none" ? 0 : parseFloat(cs.translate.split(" ")[1] ?? "0");
        items[el.id] = { y: r.top + r.height / 2, o: parseFloat(cs.opacity), ty, removed: el.dataset.removed === "true" };
      });
      w.__dxF.push({ t: performance.now(), items });
      if (!w.__dxStop && w.__dxF.length < 900) requestAnimationFrame(tick);
    };
    tick();
  }, keepCollapsed);
}

async function stopSampler(page: Page): Promise<Frame[]> {
  return page.evaluate(() => {
    const w = window as unknown as { __dxF: Frame[]; __dxStop: boolean };
    w.__dxStop = true;
    return w.__dxF;
  });
}

type Motion = { id: string; total: number; maxStep: number; atLeave: number; jumpAfter: number };

/**
 * For every toast that is present in every frame (the remaining ones): how far it travelled in
 * total, its largest single-frame step, how much of the travel was done when the leaving toast's
 * last frame was drawn, and the step in the frame right after it unmounted.
 */
function analyse(frames: Frame[], leaving: string): { motions: Motion[]; lastLeaving: number } {
  const last = frames.reduce((n, f, i) => (leaving in f.items ? i : n), -1);
  expect(last, "the leaving toast was sampled").toBeGreaterThan(0);
  expect(last, "and it did unmount while sampling").toBeLessThan(frames.length - 1);
  const ids = Object.keys(frames[0].items).filter((id) => id !== leaving && frames.every((f) => id in f.items));
  const motions = ids.map((id) => {
    const ys = frames.map((f) => f.items[id].y);
    const total = Math.abs(ys[ys.length - 1] - ys[0]);
    let maxStep = 0;
    for (let i = 1; i < ys.length; i++) maxStep = Math.max(maxStep, Math.abs(ys[i] - ys[i - 1]));
    return {
      id,
      total,
      maxStep,
      atLeave: Math.abs(ys[last] - ys[0]),
      jumpAfter: Math.abs(ys[last + 1] - ys[last]),
    };
  });
  return { motions, lastLeaving: last };
}

/** The expectations of a dismissal: the toasts that move do so smoothly and have settled before the unmount. */
function expectSmooth(motions: Motion[], movers: number, travel: number) {
  const moving = motions.filter((m) => m.total > 1.5);
  if (process.env.PW_MOTION_LOG) console.log(`motion ${test.info().title.slice(0, 40)}: ${JSON.stringify(motions)}`);
  expect(moving.length, `${movers} toast(s) close the gap`).toBe(movers);
  for (const m of moving) {
    expect(m.total, "they travel the leaving toast's footprint").toBeGreaterThan(travel);
    expect(m.maxStep, `no single frame carries the move (${JSON.stringify(m)})`).toBeLessThan(m.total * 0.5);
    expect(m.atLeave, `the gap is closed before the leaving toast unmounts (${JSON.stringify(m)})`).toBeGreaterThan(m.total * 0.9);
    expect(m.jumpAfter, `no jump when the leaving toast unmounts (${JSON.stringify(m)})`).toBeLessThan(1);
  }
  for (const m of motions.filter((x) => x.total <= 1.5)) {
    expect(m.maxStep, "a toast that does not move does not twitch").toBeLessThan(1);
  }
}

async function add(page: Page, button: string, n: number, name: string) {
  const b = page.getByRole("button", { name: button });
  for (let i = 0; i < n; i++) await b.click();
  await expect(toasts(page, name)).toHaveCount(n);
}

async function holdOverPile(page: Page, name: string) {
  const box = await measureAtRest(toasts(page, name).first());
  await page.mouse.move(box.x + 20, box.y + box.height / 2);
  await expect.poll(() => region(page, name).evaluate((el) => el.matches(":hover"))).toBe(true);
  await page.waitForTimeout(350);
}

test.describe("toast: a dismissal does not make the stack jump", () => {
  test("expanded, close button: the toasts above close the gap while it fades, then no jump", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add to stack", 4, STACK);
    await holdOverPile(page, STACK);
    const t = toasts(page, STACK);
    // DOM order is newest first. Close the second: the two older toasts above it move down 76px
    // (the oldest is the queued one, which also becomes visible).
    const leaving = await t.nth(1).evaluate((el) => el.id);
    await startSampler(page, STACK);
    await t.nth(1).getByRole("button", { name: "close" }).click();
    await expect(t).toHaveCount(3);
    await page.waitForTimeout(500);
    const { motions } = analyse(await stopSampler(page), leaving);
    expectSmooth(motions, 2, 60);
  });

  test("expanded, close button on the newest: nothing below it exists, the older ones glide", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add to stack", 3, STACK);
    await holdOverPile(page, STACK);
    const t = toasts(page, STACK);
    const leaving = await t.first().evaluate((el) => el.id);
    await startSampler(page, STACK);
    await t.first().getByRole("button", { name: "close" }).click();
    await expect(t).toHaveCount(2);
    await page.waitForTimeout(500);
    const { motions } = analyse(await stopSampler(page), leaving);
    expectSmooth(motions, 2, 60);
  });

  test("collapsed, dismissed by a click: the pile closes up while it fades, then no jump", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add to stack", 3, STACK);
    await page.waitForTimeout(500);
    const t = toasts(page, STACK);
    const leaving = await t.first().evaluate((el) => el.id);
    await startSampler(page, STACK, true);
    // A dispatched click: no hover, and the region's focus is dropped every frame, so the pile stays collapsed.
    await t.first().getByRole("button", { name: "close" }).dispatchEvent("click");
    await expect(t).toHaveCount(2);
    await page.waitForTimeout(500);
    const { motions } = analyse(await stopSampler(page), leaving);
    expectSmooth(motions, 2, 8);
  });

  test("collapsed, dismissed by its timeout: the toast that stays does not move", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    // Three timed toasts, 400ms apart: they expire oldest first (3s each), the newest stays longest.
    const b = page.getByRole("button", { name: "Show timed toast" });
    for (let i = 0; i < 3; i++) {
      await b.click();
      await page.waitForTimeout(400);
    }
    const t = toasts(page, PAUSE);
    await expect(t).toHaveCount(3);
    const leaving = await t.last().evaluate((el) => el.id); // the oldest expires first
    await startSampler(page, PAUSE);
    await expect(t).toHaveCount(2, { timeout: 8000 });
    await page.waitForTimeout(150);
    const { motions } = analyse(await stopSampler(page), leaving);
    expect(motions.length, "a newer toast was watched throughout").toBeGreaterThan(0);
    for (const m of motions) expect(m.jumpAfter, `no jump at an unmount (${JSON.stringify(m)})`).toBeLessThan(1);
  });

  test("expanded, swiped away: the others close the gap, no jump when the swiped toast unmounts", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add to stack", 3, STACK);
    await holdOverPile(page, STACK);
    const t = toasts(page, STACK);
    const leaving = await t.nth(1).evaluate((el) => el.id);
    const box = await measureAtRest(t.nth(1));
    const x = box.x + 40;
    const y = box.y + box.height / 2;
    await startSampler(page, STACK);
    await page.mouse.move(x, y);
    await page.mouse.down();
    for (let i = 1; i <= 10; i++) await page.mouse.move(x + i * 14, y);
    await page.mouse.up();
    await expect(t).toHaveCount(2);
    await page.waitForTimeout(500);
    const { motions } = analyse(await stopSampler(page), leaving);
    expectSmooth(motions, 1, 60);
  });

  test("top toaster, expanded: the toasts below close the gap upward without a jump", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Show top toast", 3, TOP);
    await holdOverPile(page, TOP);
    const t = toasts(page, TOP);
    const leaving = await t.first().evaluate((el) => el.id);
    await startSampler(page, TOP);
    await t.first().getByRole("button", { name: "close" }).click();
    await expect(t).toHaveCount(2);
    await page.waitForTimeout(500);
    const { motions } = analyse(await stopSampler(page), leaving);
    expectSmooth(motions, 2, 60);
  });

  test("a toast that becomes visible animates opacity and translate from their start values", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add to stack", 4, STACK);
    await holdOverPile(page, STACK);
    const t = toasts(page, STACK);
    const queued = await t.nth(3).evaluate((el) => el.id);
    await expect(t.nth(3)).toHaveAttribute("data-visible", "false");
    await startSampler(page, STACK);
    await t.first().getByRole("button", { name: "close" }).click();
    await expect(t).toHaveCount(3);
    await page.waitForTimeout(500);
    const frames = await stopSampler(page);
    const o = frames.filter((f) => queued in f.items).map((f) => f.items[queued].o);
    const ty = frames.filter((f) => queued in f.items).map((f) => f.items[queued].ty);
    expect(o[0], "starts transparent").toBeLessThan(0.05);
    expect(Math.abs(ty[0]), "starts displaced from its slot").toBeGreaterThan(8);
    expect(o.filter((v) => v > 0.05 && v < 0.95).length, "fades through intermediate values").toBeGreaterThan(2);
    expect(Math.max(...ty.map(Math.abs)), "translates through intermediate values").toBeGreaterThan(8);
    expect(o[o.length - 1]).toBeGreaterThan(0.99);
    expect(Math.abs(ty[ty.length - 1])).toBeLessThan(0.5);
  });

  test("a newly added toast: the pile makes room for it while it enters, in both states", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add to stack", 2, STACK);
    await holdOverPile(page, STACK);
    const t = toasts(page, STACK);
    const older = await t.nth(1).evaluate((el) => el.id);
    await startSampler(page, STACK);
    await page.getByRole("button", { name: "Add to stack" }).dispatchEvent("click");
    await expect(t).toHaveCount(3);
    await page.waitForTimeout(500);
    const frames = await stopSampler(page);
    const ys = frames.map((f) => f.items[older].y);
    const total = Math.abs(ys[ys.length - 1] - ys[0]);
    let maxStep = 0;
    for (let i = 1; i < ys.length; i++) maxStep = Math.max(maxStep, Math.abs(ys[i] - ys[i - 1]));
    expect(total, "the older toast is pushed up by one slot").toBeGreaterThan(60);
    expect(maxStep, "in more than one frame").toBeLessThan(total * 0.5);
  });
});
