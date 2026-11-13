import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL, buildProfile } from "./base-url";
import { gotoHydrated } from "./hydration";
import { gotoReference } from "./page-kinds";
import { TIMING_TAG } from "./timing";

// The decision kernel (row 509): the pure Rust `decide` of the fixture behaviour, compiled to wasm, called
// synchronously by the bridge's capture-phase, non-passive listener inside a REAL event's dispatch in Chromium. The
// shipped files are injected the way the transport delivers them: `bridge.js`, the behaviour module, and the kernel as
// base64.

const BRIDGE = readFileSync(join(__dirname, "../primitives/src/js/bridge.js"), "utf8");
const FIXTURE = readFileSync(join(__dirname, "../primitives/src/js/behaviours/fixture.js"), "utf8");
const KERNEL_B64 = readFileSync(join(__dirname, "../primitives/src/kernel/decision-kernel.b64"), "utf8");

const HTML = `<!doctype html><meta charset="utf-8"><input id="inp" value=""><div id="out"></div>`;

/** The fixture behaviour on `#inp`: keydown, capture phase, non-passive, cancelling. */
const INSTALL = () => {
  const b = (window as any).__dxBridge;
  const x = { kind: "fixture", event: "keydown", attr: "data-dx-fixture", value: "on", capture: true, cancel: true, leaseMs: null };
  const ch = { sent: [] as unknown[], send(m: unknown) { ch.sent.push(m); }, recv: () => new Promise(() => {}) };
  (window as any).ch = ch;
  b.scope(ch).run({
    cmd: "behaviour",
    target: "inp",
    behaviour: x,
    declared: { target: "element", event: "keydown", capture: true, passive: false, leaseMs: 60000 },
    epoch: { key: "k", n: 1 },
  });
};

async function blank(page: Page, csp?: string) {
  await page.setContent(csp ? HTML.replace("<meta charset", `<meta http-equiv="Content-Security-Policy" content="${csp}"><meta charset`) : HTML);
  await page.addScriptTag({ content: BRIDGE });
  await page.addScriptTag({ content: FIXTURE });
}

test.describe("the kernel decides inside a real event's dispatch", () => {
  test("a keystroke on a focused input is cancelled by the kernel's answer, and the bridge sends and sets from it", async ({ page }) => {
    await blank(page);
    await page.evaluate((b64) => void (window as any).__dxBridge.kernel.load(b64), KERNEL_B64);
    await page.evaluate(INSTALL);
    await page.focus("#inp");
    await page.keyboard.type("abc");
    // the default action (inserting the character) never ran, so the value is empty
    expect(await page.inputValue("#inp")).toBe("");
    const r = await page.evaluate(() => {
      const e = new KeyboardEvent("keydown", { key: "x", cancelable: true, bubbles: true });
      document.getElementById("inp")!.dispatchEvent(e);
      return { prevented: e.defaultPrevented, attr: document.getElementById("inp")!.getAttribute("data-dx-fixture"), sent: (window as any).ch.sent.length, state: (window as any).__dxBridge.kernel.state, mode: (window as any).__dxBridge.kernel.stats.mode };
    });
    expect(r).toEqual({ prevented: true, attr: "on", sent: 4, state: "ready", mode: "sync" });
  });

  test("the host's block attribute makes the kernel's answer a stand-down in the same dispatch", async ({ page }) => {
    await blank(page);
    await page.evaluate((b64) => void (window as any).__dxBridge.kernel.load(b64), KERNEL_B64);
    await page.evaluate(INSTALL);
    await page.evaluate(() => document.getElementById("inp")!.setAttribute("data-dx-fixture-block", ""));
    await page.focus("#inp");
    await page.keyboard.type("abc");
    expect(await page.inputValue("#inp")).toBe("abc");
  });
});

test.describe("the kernel absent: the behaviour stands down, no input is trapped, and Rust is told once", () => {
  test("never loaded", async ({ page }) => {
    await blank(page);
    await page.evaluate(INSTALL);
    await page.focus("#inp");
    await page.keyboard.type("abc");
    expect(await page.inputValue("#inp")).toBe("abc");
    expect(await page.evaluate(() => (window as any).__dxBridge.kernel.state)).toBe("absent");
    expect(await page.evaluate(() => (window as any).ch.sent)).toEqual([{ kernel: "stand-down", kind: "fixture", reason: "kernel-absent" }]);
  });

  test("blocked by a Content-Security-Policy without 'wasm-unsafe-eval': one warning, one report, nothing trapped", async ({ page }) => {
    await blank(page, "script-src 'unsafe-inline'");
    const warnings: string[] = [];
    page.on("console", (m) => m.type() === "warning" && warnings.push(m.text()));
    const loaded = await page.evaluate((b64) => (window as any).__dxBridge.kernel.load(b64), KERNEL_B64);
    expect(loaded).toBe(false);
    expect(await page.evaluate(() => (window as any).__dxBridge.kernel.state)).toBe("failed");
    await page.evaluate(INSTALL);
    await page.focus("#inp");
    await page.keyboard.type("abc");
    expect(await page.inputValue("#inp")).toBe("abc");
    expect(warnings.some((w) => w.includes("kernel did not load"))).toBe(true);
    expect(await page.evaluate(() => (window as any).ch.sent)).toEqual([{ kernel: "stand-down", kind: "fixture", reason: "kernel-failed" }]);
    console.log("CSP script-src 'unsafe-inline' only:", warnings[0]);
  });

  test("allowed by a Content-Security-Policy with only 'wasm-unsafe-eval' added", async ({ page }) => {
    await blank(page, "script-src 'unsafe-inline' 'wasm-unsafe-eval'");
    expect(await page.evaluate((b64) => (window as any).__dxBridge.kernel.load(b64), KERNEL_B64)).toBe(true);
    await page.evaluate(INSTALL);
    await page.focus("#inp");
    await page.keyboard.type("abc");
    expect(await page.inputValue("#inp")).toBe("");
  });
});

test.describe("what the kernel costs per event", () => {
  // A cross-origin isolated page has a 5 microsecond timer instead of 100: served by a route so the page is a
  // secure, isolated context.
  async function isolated(page: Page) {
    await page.route("https://probe.test/**", (route) =>
      route.fulfill({ body: HTML, contentType: "text/html", headers: { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" } }),
    );
    await gotoReference(page, "https://probe.test/");
    await page.addScriptTag({ content: BRIDGE });
    await page.addScriptTag({ content: FIXTURE });
    expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);
  }

  test("over 10,000 synthetic events from a cold start the kernel call's p99 is under 0.5 ms", async ({ page }) => {
    await isolated(page);
    const r = await page.evaluate(async (b64) => {
      const b = (window as any).__dxBridge;
      const t0 = performance.now();
      await b.kernel.load(b64);
      const loadMs = performance.now() - t0;
      const behaviour = { kind: "fixture", event: "keydown", attr: "data-dx-fixture", value: "on", capture: true, cancel: true, leaseMs: null };
      const snapshot = { "data-dx-fixture-block": null };
      const stats = (a: number[]) => {
        const s = [...a].sort((x, y) => x - y);
        const at = (q: number) => s[Math.min(s.length - 1, Math.floor(s.length * q))];
        return { n: s.length, p50: at(0.5), p99: at(0.99), p999: at(0.999), max: s[s.length - 1], mean: s.reduce((x, y) => x + y, 0) / s.length };
      };
      // the kernel call alone: marshal the request, call, read the decision
      const call: number[] = [];
      let sink = 0;
      for (let i = 0; i < 10000; i++) {
        const t = performance.now();
        const d = b.kernel.decide("fixture", behaviour, { type: "keydown" }, snapshot, {});
        call.push(performance.now() - t);
        sink += d.prevent ? 1 : 0;
      }
      // the whole listener: a real dispatchEvent through the hub, the decision, the cancel, the attribute, the send
      const input = document.getElementById("inp")!;
      b.scope({ send() {}, recv: () => new Promise(() => {}) }).run({
        cmd: "behaviour", target: "inp", behaviour, declared: { target: "element", event: "keydown", capture: true, passive: false, leaseMs: 60000 }, epoch: { key: "k", n: 1 },
      });
      const dispatch: number[] = [];
      for (let i = 0; i < 10000; i++) {
        const e = new KeyboardEvent("keydown", { key: "a", cancelable: true, bubbles: true });
        const t = performance.now();
        input.dispatchEvent(e);
        dispatch.push(performance.now() - t);
        sink += e.defaultPrevented ? 1 : 0;
      }
      // the noise floor of this harness: a plain JavaScript function building the same decision
      const js = (x: any, st: any) => (st["data-dx-fixture-block"] !== null ? {} : { prevent: x.cancel, attrs: { [x.attr]: x.value }, send: { behaviour: "fixture", event: x.event } });
      const floor: number[] = [];
      for (let i = 0; i < 10000; i++) {
        const t = performance.now();
        const d: any = js(behaviour, snapshot);
        floor.push(performance.now() - t);
        sink += d.prevent ? 1 : 0;
      }
      // the machine's own jitter over a run as long as the kernel loop: the same count of iterations, each a
      // fixed amount of arithmetic of about the kernel call's duration
      const spin: number[] = [];
      let acc = 0;
      for (let i = 0; i < 10000; i++) {
        const t = performance.now();
        for (let j = 0; j < 4000; j++) acc += j * 1.0001;
        spin.push(performance.now() - t);
      }
      sink += acc > 0 ? 0 : 1;
      return { spin: stats(spin), jsFloor: stats(floor), loadMs, mode: b.kernel.stats.mode, timerResolutionMs: 0.005, call: stats(call), callWarm: stats(call.slice(500)), dispatch: stats(dispatch), sink, bytes: b.kernel.stats };
    }, KERNEL_B64);
    console.log("KERNEL-COST " + JSON.stringify(r));
    expect(r.sink).toBe(30000);
    expect(r.call.p99).toBeLessThan(0.5);
  });
});

test.describe("the first call and the compile on a throttled CPU (release only)", () => {
  // The review measured the cold first call at 1.3 to 11.6 ms under JavaScriptCore on a loaded box, and the 10,000-event p99
  // hides it: the first keystroke is the one a user meets. A low-end phone is stood in for by the CDP CPU throttle.
  test("on a fresh page the cold first kernel call is under 2 ms and the synchronous compile plus instantiate under 8 ms with the CPU throttled 4x", { tag: TIMING_TAG }, async ({ browser, browserName }) => {
    test.setTimeout(5 * 60_000);
    test.skip(browserName !== "chromium", "CDP Emulation.setCPUThrottlingRate");
    test.skip(buildProfile() !== "release", "first-call timing is only meaningful on a release build (PW_BUILD_PROFILE=release)");
    const runs: { compileMs: number; firstCallMs: number }[] = [];
    for (let run = 0; run < 3; run++) {
      const context = await browser.newContext();
      const page = await context.newPage();
      await (await context.newCDPSession(page)).send("Emulation.setCPUThrottlingRate", { rate: 4 });
      await gotoHydrated(page, `${BASE_URL}/component/popover/`);
      await page.getByRole("button", { name: "Show Popover" }).focus();
      await page.keyboard.press("Enter");
      await expect(page.getByRole("dialog")).toBeVisible();
      await expect.poll(() => page.evaluate(() => (window as any).__dxBridge?.kernel.state)).toBe("ready");
      runs.push(
        await page.evaluate(() => {
          const k = (window as any).__dxBridge.kernel;
          const behaviour = { kind: "fixture", event: "keydown", attr: "data-dx-fixture", value: "on", capture: true, cancel: true, leaseMs: null };
          const t = performance.now();
          k.decide("fixture", behaviour, { type: "keydown" }, { "data-dx-fixture-block": null }, {});
          return { compileMs: k.stats.readyMs - k.stats.startMs, firstCallMs: performance.now() - t };
        }),
      );
      await context.close();
    }
    console.log("KERNEL-FIRST-CALL " + JSON.stringify(runs));
    const median = (key: "compileMs" | "firstCallMs") => runs.map((r) => r[key]).sort((a, b) => a - b)[1];
    expect(median("firstCallMs")).toBeLessThan(2);
    expect(median("compileMs")).toBeLessThan(8);
  });
});

test.describe("on the served site (run against a release SSG build)", () => {
  // The page's own transport loads the bridge and, in the same eval, the kernel; the fixture module is the one
  // shipped file no component evaluates, so the spec adds it and installs the behaviour on an input it adds.
  test("the kernel the transport loaded decides inside a real keydown, capture phase, non-passive", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/popover/`);
    await page.getByRole("button", { name: "Show Popover" }).focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect.poll(() => page.evaluate(() => (window as any).__dxBridge?.kernel.state)).toBe("ready");
    const info = await page.evaluate(() => {
      const k = (window as any).__dxBridge.kernel;
      return { mode: k.stats.mode, readyAfterStartMs: k.stats.readyMs - k.stats.startMs };
    });
    console.log("SITE-KERNEL " + JSON.stringify(info));
    expect(info.mode).toBe("sync");
    await page.keyboard.press("Escape");
    await page.evaluate(() => {
      const input = document.createElement("input");
      input.id = "inp";
      document.body.prepend(input);
    });
    await page.addScriptTag({ content: FIXTURE });
    await page.evaluate(INSTALL);
    await page.focus("#inp");
    await page.keyboard.type("abc");
    expect(await page.inputValue("#inp")).toBe("");
    expect(await page.evaluate(() => (window as any).ch.sent.length)).toBe(3);
  });
});
