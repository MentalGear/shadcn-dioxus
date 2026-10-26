import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";

// The generic JS bridge (row 434 step 3, primitives/src/ts/bridge.ts) against REAL Chromium: the shipped bundle
// `primitives/src/js/bridge.js`, injected into a blank page, driven through `window.__dxBridge`. The `bun test` suite
// (primitives/ts-test) proves the logic against a fake browser; this spec proves that the verbs do what they claim
// on real popovers, observers, media queries, style flushes and events. No site is loaded: the page is `setContent`.

const BRIDGE = readFileSync(join(__dirname, "../primitives/src/js/bridge.js"), "utf8");
const FIXTURE = readFileSync(join(__dirname, "../primitives/src/js/behaviours/fixture.js"), "utf8");

const HTML = `<!doctype html><meta name="viewport" content="width=device-width">
<style>#t{width:100px;height:40px;transition:opacity 1s linear;opacity:1;color:rgb(1,2,3)}
#pop{width:50px;height:20px}</style>
<div id="t" data-x="1"><button id="b">b</button></div>
<div id="pop" popover>pop</div>
<input id="inp" value="hello world">
<div id="far" style="margin-top:4000px">far</div>
<dialog id="dlg">d</dialog>`;

async function load(page: Page) {
  await page.setContent(HTML);
  await page.addScriptTag({ content: BRIDGE });
  await page.addScriptTag({ content: FIXTURE });
  await page.evaluate(() => {
    // a stand-in for the `dioxus` object of one eval
    (window as any).makeCh = () => {
      const ch: any = { sent: [] as unknown[], send(m: unknown) { ch.sent.push(m); }, recv: () => new Promise(() => {}) };
      return ch;
    };
    (window as any).frames = (n: number) => new Promise<void>((r) => { const t = () => (n-- > 0 ? requestAnimationFrame(t) : r()); t(); });
  });
}

test.describe("the bridge's verbs on a real page", () => {
  test.beforeEach(async ({ page }) => load(page));

  test("call: popover and dialog methods are idempotent and really open and close", async ({ page }) => {
    const r = await page.evaluate(() => {
      const b = (window as any).__dxBridge;
      const open = () => document.getElementById("pop")!.matches(":popover-open");
      b.call("pop", "showPopover");
      b.call("pop", "showPopover");
      const a = open();
      b.call({ kind: "id", id: "pop" }, "hidePopover");
      b.call("pop", "hidePopover");
      const c = open();
      b.call("dlg", "showModal");
      b.call("dlg", "showModal");
      const d = (document.getElementById("dlg") as HTMLDialogElement).open;
      b.call("dlg", "close");
      return [a, c, d, (document.getElementById("dlg") as HTMLDialogElement).open];
    });
    expect(r).toEqual([true, false, true, false]);
    expect(await page.evaluate(() => { try { (window as any).__dxBridge.call("pop", "remove"); } catch (e) { return String(e); } })).toContain("no such method");
  });

  test("call: focus preventScroll does not scroll, a plain focus does; setSelectionRange; scrollIntoView", async ({ page }) => {
    await page.evaluate(() => { const f = document.createElement("button"); f.id = "farb"; f.textContent = "x"; document.getElementById("far")!.appendChild(f); });
    const r = await page.evaluate(() => {
      const b = (window as any).__dxBridge;
      b.call("farb", "focus", [{ preventScroll: true }]);
      const kept = window.scrollY;
      const active = b.get({ kind: "document" }, ["activeElement"]).activeElement.id;
      b.call("inp", "setSelectionRange", [2, 5]);
      const inp = document.getElementById("inp") as HTMLInputElement;
      b.call("farb", "scrollIntoView", [{ block: "start" }]);
      return { kept, active, sel: [inp.selectionStart, inp.selectionEnd], scrolled: window.scrollY > 1000 };
    });
    expect(r).toEqual({ kept: 0, active: "farb", sel: [2, 5], scrolled: true });
  });

  test("get: computed style, geometry, attributes and the active element in one call", async ({ page }) => {
    const r = await page.evaluate(() => {
      const b = (window as any).__dxBridge;
      document.getElementById("b")!.focus();
      return b.get("t", ["style:color", "offsetWidth", "offsetHeight", "clientWidth", "attr:data-x", "attr:nope", "rect", "activeElement", "bogus"]);
    });
    expect(r["style:color"]).toBe("rgb(1, 2, 3)");
    expect([r.offsetWidth, r.offsetHeight, r.clientWidth]).toEqual([100, 40, 100]);
    expect(r["attr:data-x"]).toBe("1");
    expect(r["attr:nope"]).toBeNull();
    expect(r.rect.width).toBe(100);
    expect(r.activeElement).toEqual({ id: "b", tagName: "BUTTON", within: true });
    expect(r.bogus).toBeNull();
    const vv = await page.evaluate(() => (window as any).__dxBridge.get({ kind: "window" }, ["visualViewport"]));
    expect(vv.visualViewport.width).toBeGreaterThan(0);
    expect(vv.visualViewport.scale).toBe(1);
  });

  test("set writes a custom property and refuses a real one", async ({ page }) => {
    const r = await page.evaluate(() => {
      const b = (window as any).__dxBridge;
      b.set("t", "--band", "12px");
      const set = document.getElementById("t")!.style.getPropertyValue("--band");
      b.set("t", "--band", null);
      let err = "";
      try { b.set("t", "width", "1px"); } catch (e) { err = String(e); }
      return [set, document.getElementById("t")!.style.getPropertyValue("--band"), err];
    });
    expect(r[0]).toBe("12px");
    expect(r[1]).toBe("");
    expect(r[2]).toContain("custom properties");
  });

  test("commit: a write before it is its own style change (a transition starts); without it there is nothing to transition from", async ({ page }) => {
    const r = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      const t = document.getElementById("t")!;
      const settle = async () => {
        t.style.transition = "none";
        t.style.opacity = "1";
        await (window as any).frames(2);
      };
      // write 0, then arm a transition and write 1, with nothing between: the browser never styled the 0,
      // so there is no change to transition and the value is 1 at once
      await settle();
      t.style.opacity = "0";
      t.style.transition = "opacity 10s linear";
      t.style.opacity = "1";
      const without = getComputedStyle(t).opacity;
      // the same writes with a commit after the 0: the 0 was styled, and the 1 transitions away from it
      await settle();
      t.style.opacity = "0";
      await b.commit();
      t.style.transition = "opacity 10s linear";
      t.style.opacity = "1";
      const withCommit = getComputedStyle(t).opacity;
      return { without, withCommit };
    });
    expect(r.without).toBe("1");
    expect(Number(r.withCommit)).toBeLessThan(0.5);
  });

  test("transaction: the ops before a commit run in one turn, in order, errors per op", async ({ page }) => {
    const r = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      let turn = "same";
      queueMicrotask(() => { turn = "later"; });
      const p = b.transaction([
        { op: "call", target: "pop", method: "showPopover" },
        { op: "get", target: "pop", props: ["popoverOpen"] },
        { op: "call", target: "pop", method: "nope" },
        { op: "set", target: "t", name: "--a", value: "1" },
        { op: "commit" },
      ]);
      const during = document.getElementById("pop")!.matches(":popover-open") && turn === "same";
      const out = await p;
      return { during, out };
    });
    expect(r.during).toBe(true);
    expect(r.out[1]).toEqual({ ok: { popoverOpen: true } });
    expect(r.out[2].err).toContain("no such method");
    expect(r.out[4]).toEqual({ ok: null });
  });
});

test.describe("subscriptions on a real page", () => {
  test.beforeEach(async ({ page }) => load(page));

  test("listen streams named fields, owns removal, and defaults wheel to passive", async ({ page }) => {
    const r = await page.evaluate(() => {
      const b = (window as any).__dxBridge;
      const ch = (window as any).makeCh();
      const s = b.scope(ch);
      s.listen({ target: { kind: "id", id: "b" }, event: "click", fields: ["type", "target.id", "button"] });
      s.listen({ target: { kind: "window" }, event: "keydown", options: { capture: true }, fields: ["key"] });
      document.getElementById("b")!.click();
      document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
      const live = (window as any).__dxJsListeners.live.size;
      // passive by default for a wheel handler that tries to cancel
      let defaultPrevented = "";
      const w = b.scope();
      w.on(window, "wheel", (e: WheelEvent) => e.preventDefault());
      w.on(document.getElementById("t"), "wheel", (e: WheelEvent) => e.preventDefault(), { passive: false });
      const e1 = new WheelEvent("wheel", { cancelable: true, bubbles: true });
      document.getElementById("far")!.dispatchEvent(e1);
      const e2 = new WheelEvent("wheel", { cancelable: true, bubbles: true });
      document.getElementById("t")!.dispatchEvent(e2);
      defaultPrevented = `${e1.defaultPrevented},${e2.defaultPrevented}`;
      s.dispose();
      w.dispose();
      document.getElementById("b")!.click();
      return { sent: ch.sent, live, defaultPrevented, after: (window as any).__dxJsListeners.live.size, afterSent: ch.sent.length };
    });
    expect(r.sent).toEqual([{ type: "click", "target.id": "b", button: 0 }, { key: "Escape" }]);
    expect(r.live).toBe(2);
    expect(r.defaultPrevented).toBe("false,true");
    expect(r.after).toBe(0);
    expect(r.afterSent).toBe(2);
  });

  test("a registration on an element ends when the element leaves the document, without Rust", async ({ page }) => {
    const r = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      const s = b.scope((window as any).makeCh());
      const el = document.createElement("i");
      document.body.appendChild(el);
      s.on(el, "click", () => {});
      s.on(window, "resize", () => {});
      const before = (window as any).__dxJsListeners.live.size;
      el.remove();
      await (window as any).frames(1);
      const after = (window as any).__dxJsListeners.live.size;
      s.dispose();
      return [before, after, (window as any).__dxJsListeners.live.size];
    });
    expect(r).toEqual([2, 1, 0]);
  });

  test("observers: resize, intersection and mutation report, and stop with the scope", async ({ page }) => {
    const r = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      const ch = (window as any).makeCh();
      const s = b.scope(ch);
      s.observe({ target: "t", kind: "resize" });
      s.observe({ target: "t", kind: "intersection", options: { rootMargin: "10px", threshold: [0, 1] } });
      s.observe({ target: "t", kind: "mutation", options: { attributes: true } });
      await (window as any).frames(2);
      document.getElementById("t")!.style.width = "150px";
      document.getElementById("t")!.setAttribute("data-y", "2");
      await (window as any).frames(2);
      const n = ch.sent.length;
      s.dispose();
      document.getElementById("t")!.style.width = "170px";
      document.getElementById("t")!.setAttribute("data-y", "3");
      await (window as any).frames(2);
      return { sent: ch.sent, stopped: ch.sent.length === n };
    });
    const sizes = r.sent.filter((m: any) => "borderWidth" in m);
    expect(sizes[sizes.length - 1].borderWidth).toBe(150);
    expect(r.sent.some((m: any) => m.isIntersecting === true && m.ratio === 1)).toBe(true);
    expect(r.sent.some((m: any) => m.attributes?.includes("data-y"))).toBe(true);
    expect(r.stopped).toBe(true);
  });

  test("matchMedia sends the answer now and on every change", async ({ page }) => {
    await page.setViewportSize({ width: 800, height: 600 });
    await page.evaluate(() => {
      (window as any).ch = (window as any).makeCh();
      (window as any).s = (window as any).__dxBridge.scope((window as any).ch);
      (window as any).s.media("(min-width: 600px)");
    });
    await page.setViewportSize({ width: 400, height: 600 });
    await expect.poll(() => page.evaluate(() => (window as any).ch.sent)).toEqual([{ matches: true }, { matches: false }]);
    await page.evaluate(() => (window as any).s.dispose());
    await page.setViewportSize({ width: 800, height: 600 });
    await page.evaluate(() => (window as any).frames(2));
    expect(await page.evaluate(() => (window as any).ch.sent.length)).toBe(2);
  });

  test("sample: a batched field delivers every wheel event of a frame, with its own timestamp, in one message", async ({ page }) => {
    const sent = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      const ch = (window as any).makeCh();
      const s = b.scope(ch);
      s.sample({
        target: "t",
        fields: ["scrollTop", { name: "wheel", mode: "batched", event: "wheel", pick: ["deltaY"] }],
        until: { kind: "frames", n: 4 },
      });
      const t = document.getElementById("t")!;
      for (const dy of [3, 5, 8]) t.dispatchEvent(new WheelEvent("wheel", { deltaY: dy, bubbles: true }));
      await (window as any).frames(6);
      return ch.sent;
    });
    expect(sent[0].wheel.map((e: any) => e.deltaY)).toEqual([3, 5, 8]);
    expect(sent[0].wheel.every((e: any) => typeof e.timeStamp === "number")).toBe(true);
    expect(sent[sent.length - 1].$done).toBe(true);
  });

  test("a named behaviour cancels only when declared non-passive, shares one listener per kind, and goes with its scope", async ({ page }) => {
    const r = await page.evaluate(() => {
      const b = (window as any).__dxBridge;
      const ch = (window as any).makeCh();
      const s = b.scope(ch);
      const t = document.getElementById("t")!;
      const fixture = (over: object) => ({ kind: "fixture", event: "wheel", attr: "data-hit", value: "a", capture: false, cancel: false, leaseMs: null, ...over });
      const declare = (x: any) => ({ target: "element", event: x.event, capture: x.capture, passive: !x.cancel, leaseMs: x.leaseMs });
      const install = (x: any, key: string, n: number) =>
        s.run({ cmd: "behaviour", target: "t", behaviour: x, declared: declare(x), epoch: { key, n } });
      const wheel = () => {
        const e = new WheelEvent("wheel", { cancelable: true, bubbles: true });
        t.dispatchEvent(e);
        return e.defaultPrevented;
      };
      install(fixture({}), "p", 1);
      const passiveOnly = wheel();
      install(fixture({ cancel: true }), "c", 1);
      const withCancelling = wheel();
      const live = (window as any).__dxJsListeners.live.size;
      const msgs = ch.sent.length;
      s.dispose();
      const afterDispose = wheel();
      return { passiveOnly, withCancelling, live, hit: t.getAttribute("data-hit"), msgs, afterDispose, left: (window as any).__dxJsListeners.live.size };
    });
    // a passive behaviour cannot cancel a real wheel event; the non-passive one can; they are two listeners
    expect(r).toEqual({ passiveOnly: false, withCancelling: true, live: 2, hit: "a", msgs: 3, afterDispose: false, left: 0 });
  });

  test("a hot event reaches Rust once per frame with the latest fields", async ({ page }) => {
    const sent = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      const ch = (window as any).makeCh();
      const s = b.scope(ch);
      s.listen({ target: { kind: "id", id: "t" }, event: "wheel", fields: ["deltaY"] });
      const t = document.getElementById("t")!;
      for (const dy of [1, 2, 3, 4]) t.dispatchEvent(new WheelEvent("wheel", { deltaY: dy, bubbles: true }));
      const during = ch.sent.length;
      await (window as any).frames(2);
      return { during, sent: ch.sent };
    });
    expect(sent).toEqual({ during: 0, sent: [{ deltaY: 4 }] });
  });
});

test.describe("the slow-channel test mode on a real page", () => {
  test.beforeEach(async ({ page }) => load(page));

  test("N frames defers a message by N animation frames, never drops it, unset is immediate", async ({ page }) => {
    const r = await page.evaluate(async () => {
      const b = (window as any).__dxBridge;
      const raw = (window as any).makeCh();
      const ch = b.channel(raw);
      ch.send("now");
      const immediate = raw.sent.length;
      (window as any).__dxBridgeDelayFrames = 2;
      ch.send("late");
      const queued = raw.sent.length;
      await new Promise((r) => requestAnimationFrame(r));
      const afterOne = raw.sent.length;
      await (window as any).frames(2);
      const afterThree = raw.sent.length;
      (window as any).__dxBridgeDelayFrames = "never";
      ch.send("lost");
      await (window as any).frames(4);
      return { immediate, queued, afterOne, afterThree, dropped: raw.sent.length === afterThree };
    });
    expect(r).toEqual({ immediate: 1, queued: 1, afterOne: 1, afterThree: 2, dropped: true });
  });
});
