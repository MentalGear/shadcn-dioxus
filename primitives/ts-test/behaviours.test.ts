// The behaviour layer (row 434 step 3): the registry, the shared vectors, install / epoch / lease /
// ref-counting / passive derivation / teardown, all against the SHIPPED bundles and the fake browser.

import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { loadBridge, loadModule, makeChannel, makeEvent } from "./fake-dom";

const root = join(import.meta.dir, "..");
const gen = readFileSync(join(root, "src/ts/behaviours/behaviours.gen.ts"), "utf8");
const KINDS: string[] = [...(/export const KINDS = \[([^\]]*)\]/.exec(gen)?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]);
const vectors: Record<string, any[]> = JSON.parse(readFileSync(join(root, "conformance/behaviours.json"), "utf8"));

const setup = () => {
  const { win, bridge } = loadBridge();
  for (const kind of KINDS) loadModule(win, `behaviours/${kind}.js`);
  const el = win.document.add("host");
  return { win, bridge, el, doc: win.document };
};

const fixture = (over: Record<string, unknown> = {}) => ({
  kind: "fixture",
  event: "click",
  attr: "data-dx-fixture",
  value: "on",
  capture: false,
  cancel: false,
  leaseMs: null,
  ...over,
});
const declare = (b: any, target: "element" | "window" | "document" = "element") => ({
  target,
  event: b.event,
  capture: b.capture,
  passive: !b.cancel,
  leaseMs: b.leaseMs,
});
const install = (s: any, b: any, n: number, key = "behaviour:fixture", target: any = "host") =>
  s.run({ cmd: "behaviour", target, behaviour: b, declared: declare(b), epoch: { key, n } });

describe("the registry and the generated kinds", () => {
  test("the loaded modules are exactly the generated kinds", () => {
    const { bridge } = setup();
    expect(KINDS.length).toBeGreaterThan(0);
    expect(Object.keys(bridge.behaviours).sort()).toEqual([...KINDS].sort());
  });

  test("every kind has vectors and `decide` returns exactly the expected decision", () => {
    const { bridge } = setup();
    for (const kind of KINDS) {
      expect(vectors[kind]?.length).toBeGreaterThan(0);
      for (const v of vectors[kind]) {
        expect(bridge.behaviours[kind].decide(v.behaviour, v.event, v.state)).toEqual(v.expect);
      }
    }
  });

  test("a module that names a field outside the closed table does not register", () => {
    const { bridge } = setup();
    expect(() => bridge.register("bad", { fields: ["constructor"], decide: () => ({}) })).toThrow("no such event field");
    expect(bridge.behaviours.bad).toBeUndefined();
  });
});

describe("install", () => {
  test("a behaviour acts on its event: sets the attribute, sends, and reads state beforehand", () => {
    const { bridge, el } = setup();
    const ch = makeChannel();
    const s = bridge.scope(ch);
    install(s, fixture(), 1);
    el.dispatch("click");
    expect(el.getAttribute("data-dx-fixture")).toBe("on");
    expect(ch.sent).toEqual([{ behaviour: "fixture", event: "click" }]);
    el.attrs.delete("data-dx-fixture");
    el.setAttribute("data-dx-fixture-block", "");
    el.dispatch("click");
    expect(el.getAttribute("data-dx-fixture")).toBeNull();
    expect(ch.sent.length).toBe(1);
  });

  test("passive is declared, never chosen by the page: only a non-passive behaviour can cancel", () => {
    const { bridge, el } = setup();
    const s = bridge.scope(makeChannel());
    install(s, fixture({ cancel: true }), 1, "a");
    // declared passive although it asks to cancel: refused (with one console warning)
    const warn = console.warn;
    const seen: unknown[][] = [];
    console.warn = (...a: unknown[]) => seen.push(a);
    try {
      s.run({
        cmd: "behaviour",
        target: "host",
        behaviour: fixture({ cancel: true, event: "keydown" }),
        declared: { ...declare(fixture({ event: "keydown" })), passive: true },
        epoch: { key: "b", n: 1 },
      });
      const ev = makeEvent();
      el.dispatch("keydown", ev);
      expect(ev.prevented).toBe(0);
    } finally {
      console.warn = warn;
    }
    expect(seen.length).toBe(1);
    const click = makeEvent();
    el.dispatch("click", click);
    expect(click.prevented).toBe(1);
    const options = el.listeners.map((l: any) => [l.type, l.options.passive]);
    expect(options).toContainEqual(["click", false]);
    expect(options).toContainEqual(["keydown", true]);
  });

  test("a decision's attributes are limited to data-* and aria-*", () => {
    const { win, bridge, el } = setup();
    bridge.register("sneaky", { fields: ["type"], decide: () => ({ attrs: { onclick: "x", "data-ok": "1", "aria-busy": "true" } }) });
    const s = bridge.scope(makeChannel());
    s.run({
      cmd: "behaviour",
      target: "host",
      behaviour: { kind: "sneaky" },
      declared: { target: "element", event: "click", capture: false, passive: true },
    });
    const warn = console.warn;
    console.warn = () => {};
    try {
      el.dispatch("click");
    } finally {
      console.warn = warn;
    }
    expect([...el.attrs.keys()].sort()).toEqual(["aria-busy", "data-ok"]);
    void win;
  });

  test("a behaviour that throws does nothing: the event goes on", () => {
    const { bridge, el } = setup();
    bridge.register("boom", {
      fields: ["type"],
      decide() {
        throw new Error("boom");
      },
    });
    const s = bridge.scope(makeChannel());
    s.run({
      cmd: "behaviour",
      target: "host",
      behaviour: { kind: "boom" },
      declared: { target: "element", event: "click", capture: false, passive: false },
    });
    const warn = console.warn;
    console.warn = () => {};
    try {
      const ev = makeEvent();
      el.dispatch("click", ev);
      expect(ev.prevented + ev.stopped).toBe(0);
    } finally {
      console.warn = warn;
    }
  });

  test("a kind that is not loaded, a declaration the target contradicts, an unknown event: warn, install nothing", () => {
    const { win, bridge, el } = setup();
    const s = bridge.scope(makeChannel());
    const warn = console.warn;
    const seen: string[] = [];
    console.warn = (...a: unknown[]) => seen.push(String(a[1]));
    try {
      s.run({ cmd: "behaviour", target: "host", behaviour: { kind: "nope" }, declared: declare(fixture()), epoch: { key: "x", n: 1 } });
      s.run({ cmd: "behaviour", target: "host", behaviour: fixture(), declared: declare(fixture(), "window"), epoch: { key: "y", n: 1 } });
      s.run({ cmd: "behaviour", target: "host", behaviour: fixture({ event: "bogus" }), declared: declare(fixture({ event: "bogus" })), epoch: { key: "z", n: 1 } });
      // `resize` is window-only: not an element event
      s.run({ cmd: "behaviour", target: "host", behaviour: fixture({ event: "resize" }), declared: declare(fixture({ event: "resize" })), epoch: { key: "w", n: 1 } });
    } finally {
      console.warn = warn;
    }
    expect(seen.length).toBe(4);
    expect(el.count()).toBe(0);
    expect(win.count()).toBe(0);
  });
});

describe("ref-counted registration", () => {
  test("one DOM listener per (target, type, capture, passive); it goes with the last behaviour that needs it", () => {
    const { bridge, el, win } = setup();
    const a = bridge.scope(makeChannel());
    const b = bridge.scope(makeChannel());
    install(a, fixture(), 1, "slot-a");
    install(b, fixture(), 1, "slot-b");
    expect(el.count("click")).toBe(1);
    install(a, fixture({ capture: true }), 2, "slot-c");
    install(a, fixture({ cancel: true }), 3, "slot-d");
    expect(el.count("click")).toBe(3);
    expect(win.__dxJsListeners.live.size).toBe(3);
    a.dispose();
    expect(el.count("click")).toBe(1);
    b.dispose();
    expect(el.count()).toBe(0);
    expect(win.__dxJsListeners.live.size).toBe(0);
  });

  test("a registration on window or document is allowed when declared so, and ends with the scope", () => {
    const { bridge, win, doc } = setup();
    const s = bridge.scope(makeChannel());
    s.run({ cmd: "behaviour", target: { kind: "window" }, behaviour: fixture({ event: "online" }), declared: declare(fixture({ event: "online" }), "window"), epoch: { key: "w", n: 1 } });
    s.run({ cmd: "behaviour", target: { kind: "document" }, behaviour: fixture({ event: "selectionchange" }), declared: declare(fixture({ event: "selectionchange" }), "document"), epoch: { key: "d", n: 1 } });
    expect(win.count("online")).toBe(1);
    expect(doc.count("selectionchange")).toBe(1);
    s.dispose();
    expect(win.count()).toBe(0);
    expect(doc.count()).toBe(0);
  });

  test("the element leaving the document ends its behaviours without Rust", () => {
    const { bridge, el, win } = setup();
    install(bridge.scope(makeChannel()), fixture(), 1);
    expect(el.count("click")).toBe(1);
    el.isConnected = false;
    for (const mo of win.MutationObserver.all) if (!mo.disconnected) mo.callback([]);
    expect(el.count()).toBe(0);
  });
});

describe("epochs", () => {
  test("a stale install is dropped, an equal one renews, a newer one replaces", () => {
    const { bridge, el } = setup();
    const ch = makeChannel();
    const s = bridge.scope(ch);
    install(s, fixture({ value: "five" }), 5);
    install(s, fixture({ value: "three" }), 3);
    el.dispatch("click");
    expect(el.getAttribute("data-dx-fixture")).toBe("five");
    expect(el.count("click")).toBe(1);
    install(s, fixture({ value: "five-again" }), 5);
    el.dispatch("click");
    expect(el.getAttribute("data-dx-fixture")).toBe("five");
    install(s, fixture({ value: "seven" }), 7);
    el.dispatch("click");
    expect(el.getAttribute("data-dx-fixture")).toBe("seven");
    expect(el.count("click")).toBe(1);
  });

  test("a late install of a torn-down session does not come back (the epoch outlives the install)", () => {
    const { bridge, el } = setup();
    const old = bridge.scope(makeChannel());
    const next = bridge.scope(makeChannel());
    install(next, fixture(), 9);
    next.dispose();
    install(old, fixture(), 4);
    expect(el.count()).toBe(0);
  });

  test("an op carrying an older epoch is dropped and replies say so; replies echo the epoch", async () => {
    const { bridge, doc } = setup();
    const target = doc.add("t");
    const out = await bridge.transaction([
      { op: "set", target: "t", name: "--a", value: "2", epoch: { key: "k", n: 2 } },
      { op: "set", target: "t", name: "--a", value: "1", epoch: { key: "k", n: 1 } },
      { op: "call", target: "t", method: "focus", args: [], epoch: { key: "k", n: 1 } },
      { op: "get", target: "t", props: ["offsetWidth"], epoch: { key: "k", n: 3 } },
      { op: "get", target: "t", props: ["offsetWidth"], epoch: { key: "other", n: 1 } },
    ]);
    expect(target.customProps.get("--a")).toBe("2");
    expect(target.calls).toEqual([]);
    expect(out).toEqual([
      { ok: null, epoch: { key: "k", n: 2 } },
      { ok: null, dropped: true, epoch: { key: "k", n: 1 } },
      { ok: null, dropped: true, epoch: { key: "k", n: 1 } },
      { ok: { offsetWidth: 100 }, epoch: { key: "k", n: 3 } },
      { ok: { offsetWidth: 100 }, epoch: { key: "other", n: 1 } },
    ]);
  });
});

describe("the lease (the fail-open path)", () => {
  test("the listener removes itself when Rust stops renewing, and a renewal keeps it", () => {
    const { bridge, el, win } = setup();
    const s = bridge.scope(makeChannel());
    const b = fixture({ leaseMs: 100 });
    install(s, b, 1);
    win.advance(99);
    expect(el.count("click")).toBe(1);
    install(s, b, 1); // the renewal: same epoch
    win.advance(99);
    expect(el.count("click")).toBe(1);
    win.advance(2);
    expect(el.count("click")).toBe(0);
    expect(win.timers.length).toBe(0);
    // an expired behaviour that Rust renews comes back (the session still lives)
    install(s, b, 1);
    expect(el.count("click")).toBe(1);
    s.dispose();
    expect(el.count()).toBe(0);
    expect(win.timers.length).toBe(0);
  });

  test('"never": a channel that drops every message to Rust still lets the page decide and lease out', () => {
    const { bridge, el, win } = setup();
    win.__dxBridgeDelayFrames = "never";
    const ch = makeChannel();
    const s = bridge.scope(ch);
    install(s, fixture({ leaseMs: 50, cancel: true }), 1);
    const ev = makeEvent();
    el.dispatch("click", ev);
    expect(ev.prevented).toBe(1);
    expect(el.getAttribute("data-dx-fixture")).toBe("on");
    expect(ch.sent).toEqual([]);
    win.advance(60);
    expect(el.count()).toBe(0);
  });
});
