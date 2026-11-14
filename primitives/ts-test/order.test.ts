// The order contract's page side: the page-minted sequence number every inbound frame carries, and the
// fence of a slot (`based_on >= last_page_transition`). Run against the SHIPPED bundle and the fake browser.

import { describe, expect, test } from "bun:test";
import { loadBridge, loadModule, makeChannel } from "./fake-dom";

const setup = () => {
  const { win, bridge } = loadBridge();
  loadModule(win, "behaviours/fixture.js");
  return { win, bridge, doc: win.document, host: win.document.add("host") };
};

const fixture = (attr: string, value = "on") => ({
  kind: "fixture",
  event: "click",
  attr,
  value,
  capture: false,
  cancel: false,
  focus: false,
  leaseMs: null,
});
const declared = { target: "element", event: "click", capture: false, passive: true, leaseMs: null };
const install = (s: any, attr: string, n = 1) =>
  s.run({ cmd: "behaviour", target: "host", behaviour: fixture(attr), declared, epoch: { key: "behaviour:fixture", n } });
const write = (bridge: any, slot: string, value: string, basedOn?: number) =>
  bridge.transaction([{ op: "write", target: "host", slot, value, ...(basedOn === undefined ? {} : { basedOn }) }])[0];

describe("the inbound sequence number", () => {
  test("every message a scope sends is a frame {seq, m}, and seq rises across scopes", () => {
    const { bridge, host } = setup();
    const a = makeChannel();
    const b = makeChannel();
    const sa = bridge.scope(a);
    const sb = bridge.scope(b);
    install(sa, "data-dx-fixture");
    sa.send({ one: 1 });
    sb.send({ two: 2 });
    sa.send({ three: 3 });
    host.dispatch("click");
    const frames = [...a.frames, ...b.frames].sort((x: any, y: any) => x.seq - y.seq);
    expect(frames.map((f: any) => f.m)).toEqual([{ one: 1 }, { two: 2 }, { three: 3 }, { behaviour: "fixture", event: "click" }]);
    expect(frames.map((f: any) => f.seq)).toEqual([1, 2, 3, 4]);
  });

  test("a disposed scope mints nothing", () => {
    const { bridge } = setup();
    const ch = makeChannel();
    const s = bridge.scope(ch);
    s.dispose();
    s.send({ late: true });
    expect(ch.frames).toEqual([]);
  });
});

describe("a slot's fence", () => {
  test("a page-caused write of a slot is reported as a transition frame {seq, t}", () => {
    const { bridge, host } = setup();
    const ch = makeChannel();
    install(bridge.scope(ch), "data-dx-keyboard");
    host.dispatch("click");
    expect(ch.frames).toEqual([
      { seq: 1, t: "data-dx-keyboard" },
      { seq: 2, m: { behaviour: "fixture", event: "click" } },
    ]);
  });

  test("a write based on a sequence number older than the slot's last page transition is dropped", () => {
    const { bridge, host } = setup();
    const ch = makeChannel();
    install(bridge.scope(ch), "data-dx-keyboard", 1);
    host.dispatch("click");
    const transition = ch.frames.find((f: any) => f.t === "data-dx-keyboard").seq;
    expect(write(bridge, "data-dx-keyboard", "rust", transition - 1)).toEqual({ ok: null, dropped: true });
    expect(host.getAttribute("data-dx-keyboard")).toBe("on");
    expect(write(bridge, "data-dx-keyboard", "rust", transition)).toEqual({ ok: null });
    expect(host.getAttribute("data-dx-keyboard")).toBe("rust");
  });

  test("the fence is per slot: a transition of one slot does not hold back another", () => {
    const { bridge, host } = setup();
    const ch = makeChannel();
    install(bridge.scope(ch), "data-dx-keyboard");
    host.dispatch("click");
    expect(write(bridge, "data-dx-keyboard-slack", "1", 0)).toEqual({ ok: null });
    expect(host.getAttribute("data-dx-keyboard-slack")).toBe("1");
  });

  test("a Rust-caused write does not advance the fence", () => {
    const { bridge, host } = setup();
    const ch = makeChannel();
    install(bridge.scope(ch), "data-dx-keyboard");
    host.dispatch("click");
    const transition = ch.frames.find((f: any) => f.t === "data-dx-keyboard").seq;
    write(bridge, "data-dx-keyboard", "a", transition);
    expect(write(bridge, "data-dx-keyboard", "b", transition)).toEqual({ ok: null });
    expect(host.getAttribute("data-dx-keyboard")).toBe("b");
  });

  test("a write with no basedOn is not fenced", () => {
    const { bridge, host } = setup();
    install(bridge.scope(makeChannel()), "data-dx-keyboard");
    host.dispatch("click");
    expect(write(bridge, "data-dx-keyboard", "x")).toEqual({ ok: null });
  });
});

describe("the inbound eval", () => {
  test("a scope that took its sub id sends every frame on the inbound channel, tagged with it", () => {
    const { bridge, host } = setup();
    const inbound = makeChannel();
    void bridge.inbound(inbound);
    const own = makeChannel();
    const s = bridge.scope(own);
    s.run({ cmd: "sub", id: 7 });
    install(s, "data-dx-keyboard");
    host.dispatch("click");
    s.send({ x: 1 });
    expect(own.frames).toEqual([]);
    expect(inbound.frames).toEqual([
      { seq: 1, sub: 7, t: "data-dx-keyboard" },
      { seq: 2, sub: 7, m: { behaviour: "fixture", event: "click" } },
      { seq: 3, sub: 7, m: { x: 1 } },
    ]);
  });

  test("frames minted before the inbound eval exists are held and flushed in order, even after the scope ended", () => {
    const { bridge } = setup();
    const s = bridge.scope(makeChannel());
    s.run({ cmd: "sub", id: 3 });
    s.send("a");
    s.send("b");
    s.dispose();
    const inbound = makeChannel();
    void bridge.inbound(inbound);
    expect(inbound.frames).toEqual([
      { seq: 1, sub: 3, m: "a" },
      { seq: 2, sub: 3, m: "b" },
    ]);
  });

  test("a scope with no sub id keeps sending on its own channel", () => {
    const { bridge } = setup();
    const own = makeChannel();
    bridge.scope(own).send("z");
    expect(own.frames).toEqual([{ seq: 1, m: "z" }]);
  });
});

describe("a call's fence covers every slot the verb can touch", () => {
  const call = (bridge: any, method: string, basedOn?: number) =>
    bridge.transaction([{ op: "call", target: "host", method, args: [], ...(basedOn === undefined ? {} : { basedOn }) }])[0];

  test("showModal touches the open and focus slots: a transition of either drops a call decided before it", () => {
    for (const slot of ["open", "focus"]) {
      const { bridge, host } = setup();
      const ch = makeChannel();
      const s = bridge.scope(ch);
      s.transitioned(host, slot);
      const seq = ch.frames[0].seq;
      expect(call(bridge, "showModal", seq - 1)).toEqual({ ok: null, dropped: true });
      expect(host.calls).toEqual([]);
      expect(call(bridge, "showModal", seq)).toEqual({ ok: null });
      expect(host.calls.map((c: any) => c[0])).toEqual(["showModal"]);
    }
  });

  test("a verb that does not touch the slot is not held back by its transition", () => {
    const { bridge, host } = setup();
    const ch = makeChannel();
    bridge.scope(ch).transitioned(host, "open");
    expect(call(bridge, "scrollTo", 0)).toEqual({ ok: null });
  });

  test("a call with no basedOn is not fenced", () => {
    const { bridge, host } = setup();
    bridge.scope(makeChannel()).transitioned(host, "open");
    expect(call(bridge, "showModal")).toEqual({ ok: null });
  });
});

describe("the page-lifetime transition listeners", () => {
  const call = (bridge: any, method: string, basedOn?: number) =>
    bridge.transaction([{ op: "call", target: "host", method, args: [], ...(basedOn === undefined ? {} : { basedOn }) }])[0];
  const started = () => {
    const ctx = setup();
    const inbound = makeChannel();
    void ctx.bridge.inbound(inbound);
    return { ...ctx, inbound };
  };

  test("a user focus change is a transition of the one document-wide focus slot", () => {
    const { bridge, win, host, inbound } = started();
    win.document.dispatch("focusin", { target: host });
    expect(inbound.frames).toEqual([{ seq: 1, sub: 0, t: "focus" }]);
    // the fence covers every element: a call that touches focus and was decided before it is dropped
    expect(call(bridge, "blur", 0)).toEqual({ ok: null, dropped: true });
    expect(call(bridge, "blur", 1)).toEqual({ ok: null });
  });

  test("a popover or dialog transition fences the open slot of its own element only", () => {
    const { bridge, win, host, doc, inbound } = started();
    const other = doc.add("other");
    win.document.dispatch("beforetoggle", { target: other });
    win.document.dispatch("cancel", { target: host });
    expect(inbound.frames.map((f: any) => f.t)).toEqual(["open", "open"]);
    expect(call(bridge, "showPopover", 1)).toEqual({ ok: null, dropped: true });
    expect(call(bridge, "showPopover", 2)).toEqual({ ok: null });
  });

  test("a transition a Rust call causes is reported with its cause and does not advance the fence", () => {
    const { bridge, win, host, inbound } = started();
    host.showModal = () => win.document.dispatch("beforetoggle", { target: host });
    expect(call(bridge, "showModal", 0)).toEqual({ ok: null });
    expect(inbound.frames).toEqual([{ seq: 1, sub: 0, t: "open", c: 1 }]);
    expect(call(bridge, "showPopover", 0)).toEqual({ ok: null });
  });
});

describe("the ready attribute", () => {
  test("html[data-dx-bridge] is `ready` once the inbound eval runs, and not before", () => {
    const { win, bridge } = setup();
    const html = win.document.documentElement;
    expect(html.getAttribute("data-dx-bridge")).toBeNull();
    void bridge.inbound(makeChannel());
    expect(html.getAttribute("data-dx-bridge")).toBe("ready");
  });

  test("held frames are flushed before the attribute is set", () => {
    const { win, bridge } = setup();
    const s = bridge.scope(makeChannel());
    s.run({ cmd: "sub", id: 1 });
    s.send("early");
    const inbound = makeChannel();
    const html = win.document.documentElement;
    const at: unknown[] = [];
    const send = inbound.send.bind(inbound);
    inbound.send = (m: unknown) => (at.push(html.getAttribute("data-dx-bridge")), send(m));
    void bridge.inbound(inbound);
    expect(at).toEqual([null]);
  });
});

describe("a focusout that leaves the document", () => {
  test("a focusout with no relatedTarget mints a focus transition, one with a relatedTarget does not", () => {
    const { win, host, bridge } = setup();
    const inbound = makeChannel();
    void bridge.inbound(inbound);
    win.document.dispatch("focusout", { target: host, relatedTarget: host });
    expect(inbound.frames).toEqual([]);
    win.document.dispatch("focusout", { target: host, relatedTarget: null });
    expect(inbound.frames).toEqual([{ seq: 1, sub: 0, t: "focus" }]);
  });
});

describe("a decision's focus slot write", () => {
  test("focuses the host without scrolling, and the focus it causes is Rust's own", () => {
    const { bridge, win, host } = setup();
    const inbound = makeChannel();
    void bridge.inbound(inbound);
    const seen: unknown[][] = [];
    host.focus = (...args: unknown[]) => {
      seen.push(args);
      win.document.dispatch("focusin", { target: host });
    };
    const s = bridge.scope(makeChannel());
    s.run({ cmd: "sub", id: 2 });
    s.run({ cmd: "behaviour", target: "host", behaviour: { ...fixture("data-dx-fixture"), focus: true }, declared, epoch: { key: "behaviour:fixture", n: 1 } });
    host.dispatch("click");
    expect(seen).toEqual([[{ preventScroll: true }]]);
    expect(inbound.frames.filter((f: any) => f.t)).toEqual([{ seq: 1, sub: 0, t: "focus", c: 1 }]);
  });

  test("a decision with no focus write does not focus", () => {
    const { bridge, host } = setup();
    let focused = 0;
    host.focus = () => focused++;
    const s = bridge.scope(makeChannel());
    install(s, "data-dx-fixture");
    host.dispatch("click");
    expect(focused).toBe(0);
  });
});
