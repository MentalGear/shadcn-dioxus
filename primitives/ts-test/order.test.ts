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
