// `bun test primitives/ts-test`: one case group per bridge verb, run against the SHIPPED
// `primitives/src/js/bridge.js` (so a stale bundle fails here too) with the fake browser of `fake-dom.ts`.

import { describe, expect, test } from "bun:test";
import { FakeElement, loadBridge, makeChannel, makeWindow, ticks } from "./fake-dom";

const setup = () => {
  const { win, bridge } = loadBridge();
  return { win, bridge, doc: win.document };
};

describe("install", () => {
  test("is idempotent: a second run keeps the first bridge", () => {
    const { win, bridge } = setup();
    expect(bridge.version).toBe(1);
    const again = loadBridge(win);
    expect(again.bridge).toBe(bridge);
  });
});

describe("on / scope (the js_listener contract)", () => {
  test("registers on window, document and an element, and records the live registry", () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("a");
    const s = bridge.scope();
    s.on(win, "resize", () => {});
    s.on(doc, "keydown", () => {}, true);
    s.on(el, "click", () => {});
    expect(win.count("resize")).toBe(1);
    expect(doc.listeners[0].options).toEqual({ capture: true });
    expect([...win.__dxJsListeners.live.values()]).toEqual(["resize", "keydown", "click"]);
    s.dispose();
    expect(win.count()).toBe(0);
    expect(doc.count()).toBe(0);
    expect(el.count()).toBe(0);
    expect(win.__dxJsListeners.live.size).toBe(0);
  });

  test("the scroll-blocking events default to passive unless the caller says otherwise", () => {
    const { win, bridge } = setup();
    const s = bridge.scope();
    for (const type of ["wheel", "mousewheel", "touchstart", "touchmove", "DOMMouseScroll"]) {
      s.on(win, type, () => {});
    }
    s.on(win, "keydown", () => {});
    s.on(win, "wheel", () => {}, { passive: false });
    const passive = (type: string) => win.listeners.filter((l: any) => l.type === type).map((l: any) => l.options.passive);
    expect(passive("wheel")).toEqual([true, false]);
    for (const type of ["mousewheel", "touchstart", "touchmove", "DOMMouseScroll"]) expect(passive(type)).toEqual([true]);
    expect(passive("keydown")).toEqual([undefined]);
  });

  test("once forgets the registration when it fires; off is idempotent", () => {
    const { win, bridge } = setup();
    const s = bridge.scope();
    let calls = 0;
    s.on(win, "scrollend", () => calls++, { once: true });
    expect(win.__dxJsListeners.live.size).toBe(1);
    win.dispatch("scrollend");
    expect(calls).toBe(1);
    expect(win.count()).toBe(0);
    expect(win.__dxJsListeners.live.size).toBe(0);
    const off = s.on(win, "blur", () => {});
    off();
    off();
    expect(win.count()).toBe(0);
  });

  test("a missing target is ignored, and nothing registers after dispose", () => {
    const { win, bridge } = setup();
    const s = bridge.scope();
    expect(s.on(null, "click", () => {})).toBeInstanceOf(Function);
    s.dispose();
    s.on(win, "click", () => {});
    expect(win.count()).toBe(0);
  });

  test("dispose undoes in reverse order, survives a throwing teardown, and cleanup after the end runs at once", () => {
    const { bridge } = setup();
    const s = bridge.scope();
    const order: number[] = [];
    s.cleanup(() => order.push(1));
    s.cleanup(() => {
      throw new Error("boom");
    });
    s.cleanup(() => order.push(3));
    s.dispose();
    s.dispose();
    expect(order).toEqual([3, 1]);
    s.cleanup(() => order.push(9));
    expect(order).toEqual([3, 1, 9]);
  });
});

describe("listen (streams named fields)", () => {
  test("sends the named, dotted fields and nulls anything that is not JSON-shaped", () => {
    const { win, bridge } = setup();
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.listen({
      target: { kind: "window" },
      event: "keydown",
      options: { capture: true },
      fields: ["key", "target.id", "clientX", "target", "missing.deep"],
    });
    win.dispatch("keydown", { key: "Escape", target: new FakeElement("t"), clientX: 12 });
    expect(ch.sent).toEqual([{ key: "Escape", "target.id": "t", clientX: 12, target: null, "missing.deep": null }]);
    expect(win.listeners[0].options).toEqual({ capture: true });
  });

  test("targets resolve by id, selector and the document", () => {
    const { bridge, doc } = setup();
    const a = doc.add("a");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.listen({ target: { kind: "id", id: "a" }, event: "click", fields: ["detail"] });
    s.listen({ target: { kind: "selector", selector: "#a" }, event: "focus", fields: [] });
    s.listen({ target: { kind: "document" }, event: "visibilitychange", fields: [] });
    expect(a.count()).toBe(2);
    expect(doc.count("visibilitychange")).toBe(1);
    a.dispatch("click", { detail: 2 });
    expect(ch.sent).toEqual([{ detail: 2 }]);
    s.dispose();
    expect(a.count()).toBe(0);
  });
});

describe("call", () => {
  test("is a closed list of methods with their arguments", () => {
    const { bridge, doc } = setup();
    const el = doc.add("p");
    bridge.call({ kind: "id", id: "p" }, "focus", [{ preventScroll: true }]);
    bridge.call("p", "setPointerCapture", [7]);
    bridge.call("p", "scrollIntoView", [{ block: "nearest" }]);
    bridge.call("p", "setSelectionRange", [1, 3, "forward"]);
    expect(el.calls).toEqual([
      ["focus", [{ preventScroll: true }]],
      ["setPointerCapture", [7]],
      ["scrollIntoView", [{ block: "nearest" }]],
      ["setSelectionRange", [1, 3, "forward"]],
    ]);
    expect(() => bridge.call("p", "remove", [])).toThrow("no such method");
    expect(() => bridge.call("p", "constructor", [])).toThrow("no such method");
  });

  test("popover and dialog methods tolerate the state they are called in", () => {
    const { bridge, doc } = setup();
    const el = doc.add("p");
    bridge.call("p", "showPopover");
    bridge.call("p", "showPopover");
    expect(el.calls.map((c) => c[0])).toEqual(["showPopover"]);
    bridge.call("p", "hidePopover");
    bridge.call("p", "hidePopover");
    expect(el.calls.map((c) => c[0])).toEqual(["showPopover", "hidePopover"]);
    bridge.call("p", "showModal");
    bridge.call("p", "showModal");
    bridge.call("p", "close");
    bridge.call("p", "close");
    expect(el.calls.map((c) => c[0])).toEqual(["showPopover", "hidePopover", "showModal", "close"]);
    expect(bridge.call("p", "togglePopover")).toBe(true);
  });

  test("a missing element is a null result, not an error; window is a target", () => {
    const { win, bridge } = setup();
    expect(bridge.call("nope", "focus", [])).toBeNull();
    let scrolled: unknown[] = [];
    win.scrollTo = (...a: unknown[]) => (scrolled = a);
    bridge.call({ kind: "window" }, "scrollTo", [0, 10]);
    expect(scrolled).toEqual([0, 10]);
  });
});

describe("get", () => {
  test("reads computed style, geometry, attributes and state in one call", () => {
    const { bridge, doc } = setup();
    const el = doc.add("g");
    (el as any).styles = { "padding-top": "4px" };
    el.setAttribute("data-state", "open");
    el.popoverOpen = true;
    expect(
      bridge.get("g", ["style:padding-top", "offsetWidth", "offsetHeight", "scrollTop", "attr:data-state", "attr:nope", "popoverOpen", "rect", "bogus"]),
    ).toEqual({
      "style:padding-top": "4px",
      offsetWidth: 100,
      offsetHeight: 50,
      scrollTop: 7,
      "attr:data-state": "open",
      "attr:nope": null,
      popoverOpen: true,
      rect: el.rect,
      bogus: null,
    });
  });

  test("activeElement says whether focus is inside the target; visualViewport and window metrics", () => {
    const { win, bridge, doc } = setup();
    const host = doc.add("host");
    const inner = doc.add("inner");
    host.children.push(inner);
    inner.focus();
    expect(bridge.get({ kind: "document" }, ["activeElement"]).activeElement).toEqual({ id: "inner", tagName: "DIV", within: false });
    expect(bridge.get("host", ["activeElement"]).activeElement.within).toBe(true);
    expect(bridge.get({ kind: "window" }, ["visualViewport", "innerWidth", "innerHeight"])).toEqual({
      visualViewport: win.visualViewport,
      innerWidth: 1024,
      innerHeight: 768,
    });
    expect(bridge.get("missing", ["offsetWidth", "rect"])).toEqual({ offsetWidth: null, rect: null });
  });
});

describe("observe", () => {
  test("resize, intersection (rootMargin, threshold) and mutation send their summaries and disconnect with the scope", () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("o");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.observe({ target: "o", kind: "resize", options: { box: "border-box" } });
    s.observe({ target: "o", kind: "intersection", options: { rootMargin: "10px", threshold: [0, 1] } });
    s.observe({ target: "o", kind: "mutation", options: { attributes: true } });
    const [ro] = win.ResizeObserver.all;
    const [io] = win.IntersectionObserver.all;
    const [mo] = win.MutationObserver.all;
    expect(ro.observed).toEqual([[el, { box: "border-box" }]]);
    expect(io.init).toEqual({ root: null, rootMargin: "10px", threshold: [0, 1] });
    ro.callback([{ contentRect: { width: 10, height: 20 }, borderBoxSize: [{ inlineSize: 12, blockSize: 22 }] }]);
    io.callback([{ isIntersecting: true, intersectionRatio: 0.5, boundingClientRect: { top: 1, left: 2, width: 3, height: 4 } }]);
    mo.callback([
      { attributeName: "class", addedNodes: [1, 2], removedNodes: [] },
      { attributeName: "class", addedNodes: [], removedNodes: [1] },
    ]);
    expect(ch.sent).toEqual([
      { width: 10, height: 20, borderWidth: 12, borderHeight: 22 },
      { isIntersecting: true, ratio: 0.5, top: 1, left: 2, width: 3, height: 4 },
      { count: 2, attributes: ["class"], added: 2, removed: 1 },
    ]);
    s.dispose();
    expect([ro, io, mo].map((o) => o.disconnected)).toEqual([true, true, true]);
  });
});

describe("matchMedia", () => {
  test("sends the current answer, then every change, and unsubscribes with the scope", () => {
    const { win, bridge } = setup();
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.media("(min-width: 600px)");
    const mql = win.media.get("(min-width: 600px)");
    expect(ch.sent).toEqual([{ matches: false }]);
    mql.dispatch("change", { matches: true });
    expect(ch.sent).toEqual([{ matches: false }, { matches: true }]);
    s.dispose();
    expect(mql.count()).toBe(0);
  });
});

describe("commit and transaction", () => {
  test("commit resolves after a frame and a forced style flush", async () => {
    const { win, bridge, doc } = setup();
    let done = false;
    bridge.commit().then(() => (done = true));
    await ticks();
    expect(done).toBe(false);
    expect(doc.reads).toBe(0);
    win.step();
    await ticks();
    expect(doc.reads).toBe(1);
    expect(done).toBe(true);
  });

  test("a transaction runs its ops in one JS turn up to the first commit, in order, and reports errors per op", async () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("t");
    const run = bridge.transaction([
      { op: "call", target: "t", method: "showPopover" },
      { op: "get", target: "t", props: ["popoverOpen"] },
      { op: "call", target: "t", method: "nope" },
      { op: "commit" },
      { op: "call", target: "t", method: "hidePopover" },
    ]);
    // nothing has awaited yet: the ops before the commit already ran, the one after has not
    expect(el.calls.map((c) => c[0])).toEqual(["showPopover"]);
    win.step();
    const results = await run;
    expect(el.calls.map((c) => c[0])).toEqual(["showPopover", "hidePopover"]);
    expect(results[0]).toEqual({ ok: null });
    expect(results[1]).toEqual({ ok: { popoverOpen: true } });
    expect((results[2] as any).err).toContain("no such method");
    expect(results[3]).toEqual({ ok: null });
  });
});

describe("sample", () => {
  test("samples once per frame while the condition holds, sends changes only, and ends with $done", () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("s");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    el.setAttribute("data-moving", "");
    s.sample({ target: "s", fields: ["scrollTop"], until: { kind: "attr", name: "data-moving" } });
    win.step();
    win.step();
    el.scrollTop = 9;
    win.step();
    expect(ch.sent).toEqual([{ scrollTop: 7 }, { scrollTop: 9 }]);
    el.attrs.delete("data-moving");
    win.step();
    expect(ch.sent[2]).toEqual({ scrollTop: 9, $done: true });
    win.step();
    expect(win.frames.length).toBe(0);
    expect(ch.sent.length).toBe(3);
  });

  test("frames and ms conditions, onlyChanged off, and stop on dispose", () => {
    const { win, bridge, doc } = setup();
    doc.add("s");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.sample({ target: "s", fields: ["offsetWidth"], until: { kind: "frames", n: 2 }, onlyChanged: false });
    win.settle();
    expect(ch.sent).toEqual([{ offsetWidth: 100 }, { offsetWidth: 100 }, { offsetWidth: 100, $done: true }]);
    const ch2 = makeChannel();
    const s2 = bridge.scope(ch2);
    s2.sample({ target: "s", fields: ["offsetWidth"], until: { kind: "ms", ms: 40 } });
    win.step(16);
    win.step(16);
    win.step(16);
    expect((ch2.sent.at(-1) as any).$done).toBe(true);
    const ch3 = makeChannel();
    const s3 = bridge.scope(ch3);
    s3.sample({ target: "s", fields: ["offsetWidth"], until: { kind: "forever" } });
    win.step();
    s3.dispose();
    win.step();
    win.step();
    expect(ch3.sent.length).toBe(1);
  });
});

describe("sample, batched fields", () => {
  test("delivers every event of the frame with its own timestamp as one message, and coalesced fields beside them", () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("w");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.sample({
      target: "w",
      fields: ["scrollTop", { name: "wheel", mode: "batched", event: "wheel", pick: ["deltaX", "deltaY"] }],
      until: { kind: "frames", n: 3 },
    });
    // the batched listener is registered once, passive by the wheel default
    expect(el.listeners.map((l) => [l.type, l.options.passive])).toEqual([["wheel", true]]);
    el.dispatch("wheel", { timeStamp: 1, deltaX: 0, deltaY: 4 });
    el.dispatch("wheel", { timeStamp: 9, deltaX: 1, deltaY: 8 });
    win.step();
    expect(ch.sent).toEqual([
      {
        scrollTop: 7,
        wheel: [
          { timeStamp: 1, deltaX: 0, deltaY: 4 },
          { timeStamp: 9, deltaX: 1, deltaY: 8 },
        ],
      },
    ]);
    win.step(); // nothing new: no message
    expect(ch.sent.length).toBe(1);
    el.dispatch("wheel", { timeStamp: 30, deltaX: 0, deltaY: 2 });
    win.step();
    expect(ch.sent[1]).toEqual({ scrollTop: 7, wheel: [{ timeStamp: 30, deltaX: 0, deltaY: 2 }] });
    win.settle();
    expect((ch.sent.at(-1) as any).$done).toBe(true);
    expect((ch.sent.at(-1) as any).wheel).toEqual([]);
    expect(el.count()).toBe(0);
  });

  test("a batched-only sample stays silent in a frame without events", () => {
    const { win, bridge, doc } = setup();
    doc.add("w");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.sample({ target: "w", fields: [{ name: "e", mode: "batched", event: "wheel" }], until: { kind: "forever" } });
    win.step();
    win.step();
    expect(ch.sent).toEqual([]);
    s.dispose();
    expect(win.document.byId.get("w").count()).toBe(0);
  });
});

describe("set", () => {
  test("writes and removes a custom property, refuses anything else, and works in a transaction", async () => {
    const { bridge, doc } = setup();
    const el = doc.add("v");
    bridge.set("v", "--band", "12px");
    expect(el.customProps.get("--band")).toBe("12px");
    bridge.set({ kind: "id", id: "v" }, "--band", null);
    expect(el.customProps.has("--band")).toBe(false);
    expect(() => bridge.set("v", "width", "1px")).toThrow("custom properties");
    bridge.set("missing", "--x", "1");
    const out = await bridge.transaction([
      { op: "set", target: "v", name: "--a", value: "1" },
      { op: "set", target: "v", name: "color", value: "red" },
    ]);
    expect(el.customProps.get("--a")).toBe("1");
    expect(out[0]).toEqual({ ok: null });
    expect((out[1] as any).err).toContain("custom properties");
  });
});

describe("rules", () => {
  test("install owns the listener, remove undoes it, a scope removes what it installed", () => {
    const { bridge, doc } = setup();
    const el = doc.add("r");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    const rule = { kind: "fixture", event: "click", attr: "data-hit", value: "yes" };
    bridge.rules.install("r", rule, s);
    expect(el.count("click")).toBe(1);
    el.dispatch("click");
    el.dispatch("click");
    expect(el.getAttribute("data-hit")).toBe("yes");
    expect(ch.sent).toEqual([
      { rule: "fixture", fired: 1 },
      { rule: "fixture", fired: 2 },
    ]);
    bridge.rules.install("r", rule, s);
    expect(el.count("click")).toBe(1);
    s.dispose();
    expect(el.count("click")).toBe(0);
    bridge.rules.install("r", rule);
    expect(el.count("click")).toBe(1);
    bridge.rules.remove("r");
    expect(el.count("click")).toBe(0);
  });

  test("data-dx-rule-* attributes are read and synced: added, changed and removed", () => {
    const { bridge, doc } = setup();
    const el = doc.add("r");
    const a = JSON.stringify({ kind: "fixture", event: "click", attr: "data-a", value: "1" });
    el.setAttribute("data-dx-rule-fixture", a);
    el.setAttribute("data-dx-rule-broken", "{nope");
    el.setAttribute("data-other", "x");
    expect(bridge.rules.read("r")).toEqual([["data-dx-rule-fixture", JSON.parse(a)]]);
    const s = bridge.scope(makeChannel());
    bridge.rules.sync("r", s);
    bridge.rules.sync("r", s);
    expect(el.count("click")).toBe(1);
    el.dispatch("click");
    expect(el.getAttribute("data-a")).toBe("1");
    el.setAttribute("data-dx-rule-fixture", JSON.stringify({ kind: "fixture", event: "keydown", attr: "data-a", value: "2" }));
    bridge.rules.sync("r", s);
    expect(el.count("click")).toBe(0);
    expect(el.count("keydown")).toBe(1);
    el.attrs.delete("data-dx-rule-fixture");
    bridge.rules.sync("r", s);
    expect(el.count()).toBe(0);
  });

  test("a kind this build has no case for installs nothing", () => {
    const { bridge, doc } = setup();
    const el = doc.add("r");
    const warn = console.warn;
    const seen: unknown[][] = [];
    console.warn = (...a: unknown[]) => seen.push(a);
    try {
      bridge.rules.install("r", { kind: "from-the-future" });
    } finally {
      console.warn = warn;
    }
    expect(el.count()).toBe(0);
    expect(seen[0]).toEqual(["dx bridge: unknown rule kind", "from-the-future"]);
  });

  test("session commands install a rule and sync attributes", () => {
    const { bridge, doc } = setup();
    const el = doc.add("r");
    const s = bridge.scope(makeChannel());
    s.run({ cmd: "rule", target: { kind: "id", id: "r" }, rule: { kind: "fixture", event: "click", attr: "x", value: "y" } });
    expect(el.count("click")).toBe(1);
    s.dispose();
    expect(el.count()).toBe(0);
  });
});

describe("the slow-channel mode", () => {
  test("is off by default: send is immediate", () => {
    const { bridge } = setup();
    const raw = makeChannel();
    bridge.channel(raw).send("a");
    expect(raw.sent).toEqual(["a"]);
  });

  test("defers send by N frames and keeps the order", () => {
    const { win, bridge } = setup();
    win.__dxBridgeDelayFrames = 2;
    const raw = makeChannel();
    const ch = bridge.channel(raw);
    ch.send("a");
    win.step();
    ch.send("b");
    expect(raw.sent).toEqual([]);
    win.step();
    expect(raw.sent).toEqual(["a"]);
    win.step();
    expect(raw.sent).toEqual(["a", "b"]);
  });

  test("defers recv by N frames", async () => {
    const { win, bridge } = setup();
    win.__dxBridgeDelayFrames = 3;
    const raw = makeChannel();
    const ch = bridge.channel(raw);
    let got: unknown = "none";
    ch.recv().then((m: unknown) => (got = m));
    raw.push("hello");
    await ticks();
    expect(got).toBe("none");
    win.step();
    win.step();
    await ticks();
    expect(got).toBe("none");
    win.step();
    await ticks();
    expect(got).toBe("hello");
  });

  test('"never" drops every message to Rust but still delivers commands from Rust', async () => {
    const { win, bridge } = setup();
    win.__dxBridgeDelayFrames = "never";
    const raw = makeChannel();
    const ch = bridge.channel(raw);
    ch.send("lost");
    win.settle();
    expect(raw.sent).toEqual([]);
    raw.push("command");
    expect(await ch.recv()).toBe("command");
    win.__dxBridgeDelayFrames = 0;
    ch.send("kept");
    expect(raw.sent).toEqual(["kept"]);
  });

  test("a wrapped channel is not wrapped (and so not delayed) twice", () => {
    const { win, bridge } = setup();
    win.__dxBridgeDelayFrames = 1;
    const raw = makeChannel();
    const once = bridge.channel(raw);
    expect(bridge.channel(once)).toBe(once);
    bridge.scope(once).listen({ target: { kind: "window" }, event: "blur", fields: ["type"] });
    win.dispatch("blur");
    win.step();
    expect(raw.sent).toEqual([{ type: "blur" }]);
  });

  test("a scope's listen messages go through it, and the mode can be switched off again", () => {
    const { win, bridge } = setup();
    const raw = makeChannel();
    const s = bridge.scope(raw);
    s.listen({ target: { kind: "window" }, event: "blur", fields: ["type"] });
    win.__dxBridgeDelayFrames = 1;
    win.dispatch("blur");
    expect(raw.sent).toEqual([]);
    win.step();
    expect(raw.sent).toEqual([{ type: "blur" }]);
    win.__dxBridgeDelayFrames = 0;
    win.dispatch("blur");
    expect(raw.sent.length).toBe(2);
  });

  test("a delayed answer to get arrives N frames late; the value is the one read at request time", async () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("d");
    win.__dxBridgeDelayFrames = 2;
    const raw = makeChannel();
    const ch = bridge.channel(raw);
    ch.send({ ok: bridge.get("d", ["offsetWidth"]) });
    el.offsetWidth = 999;
    win.step();
    win.step();
    expect(raw.sent).toEqual([{ ok: { offsetWidth: 100 } }]);
  });
});

describe("element lifetime", () => {
  /** The page-wide observer that ends registrations of removed elements. */
  const pageObserver = (win: any) => win.MutationObserver.all.find((o: any) => o.observed.some(([el]: any) => el === win.document.documentElement));

  test("a registration on an element ends when the element leaves the document, without Rust", () => {
    const { win, bridge, doc } = setup();
    const a = doc.add("a");
    const b = doc.add("b");
    const s = bridge.scope();
    s.on(a, "click", () => {});
    s.on(b, "click", () => {});
    s.on(win, "resize", () => {});
    const mo = pageObserver(win);
    expect(mo).toBeDefined();
    a.isConnected = false;
    mo.callback([]);
    expect(a.count()).toBe(0);
    expect(b.count()).toBe(1);
    expect(win.count()).toBe(1);
    expect(win.__dxJsListeners.live.size).toBe(2);
    expect(mo.disconnected).toBe(false);
    b.isConnected = false;
    mo.callback([]);
    expect(mo.disconnected).toBe(true);
    expect(win.count()).toBe(1);
  });

  test("an element that is re-inserted in the same batch keeps its listener, a detached one is not tracked", () => {
    const { win, bridge, doc } = setup();
    const a = doc.add("a");
    const detached = doc.add("d");
    detached.isConnected = false;
    const s = bridge.scope();
    s.on(a, "click", () => {});
    s.on(detached, "click", () => {});
    const mo = pageObserver(win);
    mo.callback([]);
    expect(a.count()).toBe(1);
    expect(detached.count()).toBe(1);
  });

  test("an off before the element leaves does not leave the observer behind", () => {
    const { win, bridge, doc } = setup();
    const a = doc.add("a");
    const s = bridge.scope();
    const off = s.on(a, "click", () => {});
    const mo = pageObserver(win);
    off();
    expect(mo.disconnected).toBe(true);
    s.dispose();
  });

  test("rules die with their element too", () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("r");
    bridge.rules.install("r", { kind: "fixture", event: "click", attr: "x", value: "y" });
    expect(el.count("click")).toBe(1);
    el.isConnected = false;
    pageObserver(win).callback([]);
    expect(el.count()).toBe(0);
  });
});

describe("listen: initial", () => {
  test("sends once at registration with the element as the event target", () => {
    const { win, bridge } = setup();
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.listen({ target: { kind: "window" }, event: "resize", fields: ["target.innerWidth", "target.innerHeight", "type"], initial: true });
    expect(ch.sent).toEqual([{ "target.innerWidth": 1024, "target.innerHeight": 768, type: "initial" }]);
    win.dispatch("resize", { target: win });
    expect(ch.sent.length).toBe(2);
  });
});

describe("the bridge's exports", () => {
  test("every verb is on window.__dxBridge in the shipped bundle", () => {
    const { bridge } = setup();
    for (const verb of ["channel", "scope", "serve", "pump", "call", "get", "set", "commit", "transaction"]) {
      expect(typeof bridge[verb]).toBe("function");
    }
    for (const verb of ["install", "remove", "read", "sync"]) expect(typeof bridge.rules[verb]).toBe("function");
    expect(bridge.version).toBe(1);
  });
});

describe("pump (the ordered fire-and-forget channel)", () => {
  test("runs op arrays in the order received, a commit holds the next message back", async () => {
    const { win, bridge, doc } = setup();
    const el = doc.add("p");
    const ch = makeChannel();
    bridge.pump(ch);
    ch.push([{ op: "call", target: "p", method: "focus", args: [] }, { op: "commit" }]);
    ch.push([{ op: "call", target: "p", method: "blur", args: [] }]);
    await ticks();
    expect(el.calls.map((c) => c[0])).toEqual(["focus"]);
    win.step();
    await ticks(20);
    expect(el.calls.map((c) => c[0])).toEqual(["focus", "blur"]);
  });

  test("a failing op is logged and the pump keeps going", async () => {
    const { bridge, doc } = setup();
    const el = doc.add("p");
    const ch = makeChannel();
    const warn = console.warn;
    const seen: unknown[][] = [];
    console.warn = (...a: unknown[]) => seen.push(a);
    try {
      bridge.pump(ch);
      ch.push([{ op: "call", target: "p", method: "nope", args: [] }]);
      ch.push([{ op: "call", target: "p", method: "blur", args: [] }]);
      await ticks(20);
    } finally {
      console.warn = warn;
    }
    expect(seen.length).toBe(1);
    expect(el.calls.map((c) => c[0])).toEqual(["blur"]);
  });
});

describe("a long-lived channel survives many messages", () => {
  test("serve takes a thousand commands in a row and still tears down", async () => {
    const { win, bridge } = setup();
    const ch = makeChannel();
    const done = bridge.serve(ch);
    for (let i = 0; i < 1000; i++) {
      ch.push([{ cmd: "listen", target: { kind: "window" }, event: "e" + (i % 10), fields: [] }]);
      if (i % 50 === 0) await ticks();
    }
    for (let i = 0; i < 2000 && win.count() < 1000; i++) await ticks(5);
    expect(win.count()).toBe(1000);
    ch.push("__dx_listeners_off__");
    await done;
    expect(win.count()).toBe(0);
  });
});

describe("serve (the loop of a Rust session)", () => {
  test("runs the setup array, accepts later commands, and the teardown message undoes everything", async () => {
    const { win, bridge, doc } = setup();
    const a = doc.add("a");
    const ch = makeChannel();
    const done = bridge.serve(ch);
    ch.push([
      { cmd: "listen", target: { kind: "window" }, event: "resize", fields: ["type"] },
      { cmd: "listen", target: { kind: "id", id: "a" }, event: "click", fields: [] },
    ]);
    await ticks();
    expect(win.count("resize")).toBe(1);
    expect(a.count("click")).toBe(1);
    ch.push([{ cmd: "media", query: "(hover: hover)" }]);
    await ticks();
    expect(ch.sent).toEqual([{ matches: false }]);
    win.dispatch("resize");
    expect(ch.sent.at(-1)).toEqual({ type: "resize" });
    ch.push("__dx_listeners_off__");
    await done;
    expect(win.count()).toBe(0);
    expect(a.count()).toBe(0);
    expect(win.media.get("(hover: hover)").count()).toBe(0);
  });

  test("a fresh window per test: the registry is per page", () => {
    const one = loadBridge(makeWindow());
    const two = loadBridge(makeWindow());
    expect(one.bridge).not.toBe(two.bridge);
  });
});
