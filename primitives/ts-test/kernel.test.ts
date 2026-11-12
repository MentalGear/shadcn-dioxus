// The decision kernel's page half (`ts/lib/kernel.ts`) against the SHIPPED bridge and the SHIPPED module: the
// marshalling, the load paths, and every way the kernel can be absent. The shared vectors run in behaviours.test.ts.

import { describe, expect, test } from "bun:test";
import { KERNEL_WASM, loadBridge, loadModule, makeChannel, makeEvent, makeWindow, ticks } from "./fake-dom";

const fixture = (over: Record<string, unknown> = {}) => ({
  kind: "fixture",
  event: "keydown",
  attr: "data-dx-fixture",
  value: "on",
  capture: true,
  cancel: true,
  leaseMs: null,
  ...over,
});
const FIELDS = { type: "keydown" };
const OPEN = { "data-dx-fixture-block": null };

/** A module with the kernel's exports whose `dx_decide_fixture` traps. */
function trappingModule(): Uint8Array {
  const str = (s: string) => [s.length, ...[...s].map((c) => c.charCodeAt(0))];
  const section = (id: number, body: number[]) => [id, body.length, ...body];
  const exports = [5, ...str("memory"), 2, 0, ...str("dx_abi"), 0, 0, ...str("dx_input"), 0, 1, ...str("dx_output"), 0, 2, ...str("dx_decide_fixture"), 0, 3];
  return Uint8Array.from([
    0, 0x61, 0x73, 0x6d, 1, 0, 0, 0,
    ...section(1, [2, 0x60, 0, 1, 0x7f, 0x60, 1, 0x7f, 1, 0x7f]),
    ...section(3, [4, 0, 1, 0, 1]),
    ...section(5, [1, 0, 1]),
    ...section(7, exports),
    ...section(10, [4, 4, 0, 0x41, 1, 0x0b, 4, 0, 0x41, 0, 0x0b, 4, 0, 0x41, 0, 0x0b, 3, 0, 0, 0x0b]),
  ]);
}

const quiet = <T>(fn: () => T): { out: T; warnings: string[] } => {
  const warn = console.warn;
  const warnings: string[] = [];
  console.warn = (...a: unknown[]) => void warnings.push(a.map(String).join(" "));
  try {
    return { out: fn(), warnings };
  } finally {
    console.warn = warn;
  }
};

describe("the kernel is loaded", () => {
  test("a module compiles synchronously: it is ready when load returns, and says how and when", async () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    expect(bridge.kernel.state).toBe("absent");
    const ready = bridge.kernel.load(KERNEL_WASM);
    expect(bridge.kernel.state).toBe("ready");
    expect(await ready).toBe(true);
    expect(bridge.kernel.stats.mode).toBe("sync");
    expect(bridge.kernel.stats.readyMs).toBeGreaterThanOrEqual(bridge.kernel.stats.startMs);
  });

  test("the module's base64 text loads the same module", () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    void bridge.kernel.load(Buffer.from(KERNEL_WASM).toString("base64"));
    expect(bridge.kernel.state).toBe("ready");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})).toEqual({
      prevent: true,
      attrs: { "data-dx-fixture": "on" },
      send: { behaviour: "fixture", event: "keydown" },
    });
  });

  test("the asynchronous path stands down until the compile finishes, then decides", async () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    const ready = bridge.kernel.load(KERNEL_WASM, "async");
    expect(bridge.kernel.state).toBe("loading");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})).toEqual({});
    expect(await ready).toBe(true);
    expect(bridge.kernel.stats.mode).toBe("async");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}).prevent).toBe(true);
  });

  test("a second load while loading or ready changes nothing", async () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    const first = bridge.kernel.load(KERNEL_WASM, "async");
    expect(await bridge.kernel.load(KERNEL_WASM)).toBe(false);
    expect(await first).toBe(true);
    expect(await bridge.kernel.load(KERNEL_WASM)).toBe(true);
  });
});

describe("the kernel is absent: the behaviour stands down and no input is trapped", () => {
  const setup = (kernel: boolean) => {
    const { win, bridge } = loadBridge(makeWindow(), { kernel });
    loadModule(win, "behaviours/fixture.js");
    const el = win.document.add("host");
    const ch = makeChannel();
    bridge.scope(ch).run({
      cmd: "behaviour",
      target: "host",
      behaviour: fixture(),
      declared: { target: "element", event: "keydown", capture: true, passive: false, leaseMs: 3000 },
      epoch: { key: "k", n: 1 },
    });
    return { win, bridge, el, ch };
  };

  test("never loaded: nothing is cancelled, set or sent", () => {
    const { el, ch } = setup(false);
    const ev = makeEvent();
    el.dispatch("keydown", ev);
    expect([ev.prevented, ev.stopped, el.attrs.size, ch.sent.length]).toEqual([0, 0, 0, 0]);
  });

  test("loaded: the same event is cancelled in the same dispatch", () => {
    const { el, ch } = setup(true);
    const ev = makeEvent();
    el.dispatch("keydown", ev);
    expect([ev.prevented, el.getAttribute("data-dx-fixture"), ch.sent.length]).toEqual([1, "on", 1]);
  });

  test("garbage bytes, a module that is not a kernel and a wrong ABI leave it failed, with one warning", async () => {
    for (const bytes of [Uint8Array.of(1, 2, 3), Uint8Array.of(0, 0x61, 0x73, 0x6d, 1, 0, 0, 0)]) {
      const { win, bridge } = loadBridge(makeWindow(), { kernel: false });
      const { out, warnings } = quiet(() => bridge.kernel.load(bytes));
      expect(await out).toBe(false);
      expect(bridge.kernel.state).toBe("failed");
      expect(warnings.length).toBe(1);
      expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})).toEqual({});
      void win;
    }
  });

  test("a blocked compile (a Content-Security-Policy without wasm-unsafe-eval throws a CompileError) leaves it failed", async () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    const real = WebAssembly.Module;
    (WebAssembly as any).Module = function () {
      throw new WebAssembly.CompileError("Refused to compile or instantiate WebAssembly module because 'unsafe-eval' is not an allowed source");
    };
    try {
      const { out } = quiet(() => bridge.kernel.load(KERNEL_WASM));
      expect(await out).toBe(false);
    } finally {
      (WebAssembly as any).Module = real;
    }
    expect(bridge.kernel.state).toBe("failed");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})).toEqual({});
  });

  test("a trap in the kernel stands the kernel down for good", async () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    expect(await bridge.kernel.load(trappingModule())).toBe(true);
    const { out, warnings } = quiet(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}));
    expect(out).toEqual({});
    expect(warnings.length).toBe(1);
    expect(bridge.kernel.state).toBe("failed");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})).toEqual({});
  });

  test("a kind the module does not export has no decision", () => {
    const { bridge } = loadBridge();
    expect(bridge.kernel.decide("nope", fixture(), FIELDS, OPEN, {})).toEqual({});
    expect(bridge.kernel.decide("constructor", fixture(), FIELDS, OPEN, {})).toEqual({});
  });
});

describe("the marshalling", () => {
  test("strings of any script cross both ways, and a key named __proto__ stays data", () => {
    const { bridge } = loadBridge();
    const out = bridge.kernel.decide("fixture", fixture({ attr: "__proto__", value: "значение \u{1F600} 日本語" }), FIELDS, OPEN, {});
    expect(Object.getPrototypeOf(out.attrs)).toBe(Object.prototype);
    expect(Object.keys(out.attrs)).toEqual(["__proto__"]);
    expect(Object.getOwnPropertyDescriptor(out.attrs, "__proto__")?.value).toBe("значение \u{1F600} 日本語");
  });

  test("a request larger than the first buffer, with nested memory, still decides", () => {
    const { bridge } = loadBridge();
    const memory = { list: Array.from({ length: 2000 }, (_, i) => ({ i, s: "x".repeat(20), b: i % 2 === 0, n: null })) };
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, memory).prevent).toBe(true);
  });

  test("a value nested past the limit is an error for the caller, not a decision", () => {
    const { bridge } = loadBridge();
    let deep: unknown = 1;
    for (let i = 0; i < 40; i++) deep = [deep];
    expect(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, deep)).toThrow("nested too deep");
    expect(bridge.kernel.state).toBe("ready");
  });

  test("a request the kernel cannot read (a missing parameter) is a stand-down, and the kernel stays ready", () => {
    const { bridge } = loadBridge();
    expect(bridge.kernel.decide("fixture", { kind: "fixture" }, FIELDS, OPEN, {})).toEqual({});
    expect(bridge.kernel.state).toBe("ready");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}).prevent).toBe(true);
  });

  test("the call is deterministic: ten thousand decisions give one answer", async () => {
    const { bridge } = loadBridge();
    await ticks();
    const first = JSON.stringify(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}));
    for (let i = 0; i < 10_000; i++) if (JSON.stringify(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})) !== first) throw new Error("drift at " + i);
  });
});
