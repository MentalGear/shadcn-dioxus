import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoReference } from "./page-kinds";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

// The bridge's channel counters (`window.__dxBridgeStats`) and the busy-case spin hook in REAL Chromium: the shipped
// bundle `primitives/src/js/bridge.js` on a blank page, one wrapped channel standing in for the `dioxus` object of an
// eval. The same object is what Rust reads into its read-out file on a desktop webview, so a web run and a desktop
// run report the same numbers.

const BRIDGE = readFileSync(join(__dirname, "../primitives/src/js/bridge.js"), "utf8");

async function load(page: Page, url = "http://dx.test/") {
  await page.route("http://dx.test/**", (route) => route.fulfill({ contentType: "text/html", body: "<!doctype html><title>t</title>" }));
  await gotoReference(page, url);
  await page.addScriptTag({ content: BRIDGE });
  await page.evaluate(() => {
    const queue: unknown[] = [];
    let waiting: ((message: unknown) => void) | null = null;
    const raw: any = {
      sent: [] as unknown[],
      send(message: unknown) {
        raw.sent.push(message);
      },
      recv: () => (queue.length ? Promise.resolve(queue.shift()) : new Promise((resolve) => (waiting = resolve))),
      push(message: unknown) {
        if (waiting) {
          const w = waiting;
          waiting = null;
          w(message);
        } else queue.push(message);
      },
    };
    (window as any).raw = raw;
    (window as any).ch = (window as any).__dxBridge.channel(raw);
  });
}

test.describe("the bridge's channel counters", () => {
  test("count a round trip and sample the page's share of it", async ({ page }) => {
    await load(page);
    const stats = await page.evaluate(async () => {
      const w = window as any;
      w.raw.push({ id: 1, ops: [] });
      await w.ch.recv();
      await new Promise((resolve) => setTimeout(resolve, 20));
      w.ch.send({ id: 1, results: [] });
      return JSON.parse(JSON.stringify(w.__dxBridgeStats));
    });
    expect(stats.channels).toHaveLength(1);
    expect(stats.channels[0].sent).toBe(1);
    expect(stats.channels[0].recv).toBe(1);
    expect(stats.channels[0].turn).toHaveLength(1);
    expect(stats.channels[0].turn[0]).toBeGreaterThanOrEqual(19);
  });

  test("a message that asks for stats is answered with them", async ({ page }) => {
    await load(page);
    const answer = await page.evaluate(async () => {
      const w = window as any;
      w.raw.push({ id: 5, ops: [], stats: true });
      await w.ch.recv();
      w.ch.send({ id: 5, results: [] });
      return JSON.parse(JSON.stringify(w.raw.sent[0]));
    });
    expect(answer.id).toBe(5);
    expect(answer.stats.channels[0].recv).toBe(1);
  });

  test("?dx-bridge-spin burns that long per message each way", async ({ page }) => {
    await load(page, "http://dx.test/?dx-bridge-spin=5");
    const ms = await page.evaluate(async () => {
      const w = window as any;
      const spun = w.__dxBridge.channel({ send() {}, recv: () => Promise.resolve("m") });
      const t0 = performance.now();
      spun.send("a");
      await spun.recv();
      return performance.now() - t0;
    });
    expect(ms).toBeGreaterThanOrEqual(10);
  });
});

test("on the served site the pump and the sessions report counters", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/dialog/`);
  await expect
    .poll(() => page.evaluate(() => (window as any).__dxBridgeStats?.channels?.[0]?.recv ?? 0))
    .toBeGreaterThan(0);
  const [pump, ...sessions] = await page.evaluate(() => JSON.parse(JSON.stringify((window as any).__dxBridgeStats.channels)));
  expect(pump.sent).toBeGreaterThan(0);
  expect(pump.turn.length).toBeGreaterThan(0);
  expect(pump.turn.length).toBeLessThanOrEqual(256);
  expect(sessions.length).toBeGreaterThan(0);
});
