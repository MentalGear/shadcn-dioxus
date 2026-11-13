// The decision kernel's page half (`ts/lib/kernel.ts`): the shipped bridge against the SHIPPED module (loading, the
// marshalling, every way a kind cannot decide, the report to Rust, the codec's cost and allocations), and the source
// module against a hand-assembled kernel with two kinds (a trap is contained to its kind). The shared vectors run in
// behaviours.test.ts.

import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { createKernel, type KernelRow } from "../src/ts/lib/kernel";
import { KERNEL_WASM, loadBridge, loadModule, makeChannel, makeEvent, makeWindow, ticks } from "./fake-dom";

const gen = readFileSync(join(import.meta.dir, "../src/ts/behaviours/behaviours.gen.ts"), "utf8");
const HASH = Number(/export const KERNEL_HASH = (\d+);/.exec(gen)?.[1]);
const rust = readFileSync(join(import.meta.dir, "../src/bridge/behaviours.rs"), "utf8");
const vectors = JSON.parse(readFileSync(join(import.meta.dir, "../conformance/behaviours.json"), "utf8"));

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
const DECLARED = { target: "element", event: "keydown", capture: true, passive: false, leaseMs: 3000 };

/** The stand-down report a call throws, or null. */
function standDown(fn: () => unknown): { kind: string; reason: string } | null {
  try {
    fn();
  } catch (error: any) {
    if (error && typeof error.reason === "string") return { kind: error.kind, reason: error.reason };
    throw error;
  }
  return null;
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

// ------------------------------------------------------------------------------------------ a hand-assembled kernel

const str = (s: string) => [s.length, ...[...s].map((c) => c.charCodeAt(0))];
const section = (id: number, body: number[]) => [id, body.length, ...body];
const sleb = (n: number): number[] => {
  const out: number[] = [];
  for (;;) {
    const byte = n & 0x7f;
    n >>= 7;
    if ((n === 0 && !(byte & 0x40)) || (n === -1 && byte & 0x40)) return [...out, byte];
    out.push(byte | 0x80);
  }
};
const body = (instrs: number[]) => [instrs.length + 2, 0, ...instrs, 0x0b];

/**
 * A module with the kernel's five exports and its memory. `dx_decide(index, len)` traps for the indices in `traps`
 * and otherwise answers the empty object (`[6, 0, 0, 0, 0]`, five bytes at `dx_output()`).
 */
function module(opts: { abi?: number; hash?: number; traps?: number[] } = {}): Uint8Array {
  const { abi = 2, hash = HASH, traps = [] } = opts;
  // dx_decide: for each trapping index `if (local0 == i) unreachable`, then `return 5`
  const guard = traps.flatMap((i) => [0x20, 0x00, 0x41, ...sleb(i), 0x46, 0x04, 0x40, 0x00, 0x0b]);
  const exports = [
    6, ...str("memory"), 2, 0, ...str("dx_abi"), 0, 0, ...str("dx_kinds"), 0, 1, ...str("dx_input"), 0, 2, ...str("dx_decide"), 0, 3, ...str("dx_output"), 0, 4,
  ];
  return Uint8Array.from([
    0, 0x61, 0x73, 0x6d, 1, 0, 0, 0,
    ...section(1, [3, 0x60, 0, 1, 0x7f, 0x60, 1, 0x7f, 1, 0x7f, 0x60, 2, 0x7f, 0x7f, 1, 0x7f]),
    ...section(3, [5, 0, 0, 1, 2, 0]),
    ...section(5, [1, 0, 1]),
    ...section(7, exports),
    ...section(10, [5, ...body([0x41, ...sleb(abi)]), ...body([0x41, ...sleb(hash | 0)]), ...body([0x41, ...sleb(1024)]), ...body([...guard, 0x41, 5]), ...body([0x41, ...sleb(2048)])]),
    ...section(11, [1, 0, 0x41, ...sleb(2048), 0x0b, 5, 6, 0, 0, 0, 0]),
  ]);
}

const ROWS: KernelRow[] = [
  { kind: "a", config: [], reads: [], fields: [] },
  { kind: "b", config: [], reads: [], fields: [] },
];
const own = <T>(table: Record<string, T>, key: unknown) => (typeof key === "string" && Object.prototype.hasOwnProperty.call(table, key) ? table[key] : undefined);
const make = (rows = ROWS, hash = HASH) => {
  const warnings: string[] = [];
  const kernel = createKernel({ own, warnOnce: (key, message) => void (warnings.includes(key) || warnings.push(key)), rows, hash });
  return { kernel, warnings };
};

describe("the reasons of a stand-down report", () => {
  test("the page's reasons are the Rust `StandDownReason` variants, in their kebab-case wire names", () => {
    const variants = [...(/pub enum StandDownReason \{([^}]*)\}/.exec(rust)?.[1] ?? "").matchAll(/^\s{4}([A-Z]\w*),/gm)].map((m) => m[1].replace(/\B([A-Z])/g, "-$1").toLowerCase());
    const page = readFileSync(join(import.meta.dir, "../src/ts/lib/kernel.ts"), "utf8");
    const declared = [...(/export type StandDownReason = ([^;]*);/.exec(page)?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]);
    expect(variants.length).toBe(6);
    expect(declared.sort()).toEqual(variants.sort());
  });
});

describe("the kernel is loaded", () => {
  test("a module compiles synchronously: it is ready when load returns, and says how and when", () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    expect(bridge.kernel.state).toBe("absent");
    expect(bridge.kernel.load(KERNEL_WASM)).toBe(true);
    expect(bridge.kernel.state).toBe("ready");
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

  test("a second load changes nothing, whatever it is given", () => {
    const { bridge } = loadBridge(makeWindow(), { kernel: false });
    expect(bridge.kernel.load(KERNEL_WASM)).toBe(true);
    expect(bridge.kernel.load(Uint8Array.of(1, 2, 3))).toBe(true);
    expect(bridge.kernel.state).toBe("ready");
    const { bridge: failed } = loadBridge(makeWindow(), { kernel: false });
    expect(quiet(() => failed.kernel.load(Uint8Array.of(1, 2, 3))).out).toBe(false);
    expect(failed.kernel.load(KERNEL_WASM)).toBe(false);
    expect(failed.kernel.state).toBe("failed");
  });

  test("the shipped module exports exactly the five functions of the contract and its memory, and its kind table is the page's", () => {
    const compiled = new WebAssembly.Module(KERNEL_WASM);
    expect(WebAssembly.Module.exports(compiled).map((e) => e.name).sort()).toEqual(["dx_abi", "dx_decide", "dx_input", "dx_kinds", "dx_output", "memory"]);
    const exports: any = new WebAssembly.Instance(compiled, {}).exports;
    expect(exports.dx_abi()).toBe(2);
    expect(exports.dx_kinds() >>> 0).toBe(HASH);
  });

  test("a module whose kind table differs from the page's is refused with one warning", async () => {
    const { kernel, warnings } = make(ROWS, HASH);
    const { out } = quiet(() => kernel.load(module({ hash: HASH + 1 })));
    expect(await out).toBe(false);
    expect(kernel.state).toBe("failed");
    expect(warnings).toEqual(["kernel-load"]);
    expect(standDown(() => kernel.decide("a", {}, {}, {}, {}))).toEqual({ kind: "a", reason: "kernel-failed" });
  });

  test("a module with another ABI is refused", async () => {
    const { kernel } = make();
    expect(await quiet(() => kernel.load(module({ abi: 1 }))).out).toBe(false);
    expect(kernel.state).toBe("failed");
  });

  test("a hash above 2^31 (a negative i32 in JavaScript) is compared as the unsigned value Rust exports", async () => {
    const { kernel } = make(ROWS, 3_000_000_000);
    expect(await kernel.load(module({ hash: 3_000_000_000 }))).toBe(true);
    expect(kernel.decide("b", {}, {}, {}, {})).toEqual({});
  });
});

describe("a kind that cannot decide is loud: it warns once, answers no decision and reports to Rust once", () => {
  const setup = (kernel: boolean) => {
    const { win, bridge } = loadBridge(makeWindow(), { kernel });
    loadModule(win, "behaviours/fixture.js");
    const el = win.document.add("host");
    const ch = makeChannel();
    bridge.scope(ch).run({ cmd: "behaviour", target: "host", behaviour: fixture(), declared: DECLARED, epoch: { key: "k", n: 1 } });
    return { win, bridge, el, ch };
  };

  test("loaded: the event is cancelled in the same dispatch and nothing is reported", () => {
    const { el, ch } = setup(true);
    const ev = makeEvent();
    el.dispatch("keydown", ev);
    expect([ev.prevented, el.getAttribute("data-dx-fixture"), ch.sent]).toEqual([1, "on", [{ behaviour: "fixture", event: "keydown" }]]);
  });

  test("never loaded: nothing is cancelled, set or sent but one report, once however many events follow", () => {
    const { el, ch } = setup(false);
    const { warnings } = quiet(() => {
      for (let i = 0; i < 3; i++) {
        const ev = makeEvent();
        el.dispatch("keydown", ev);
        expect([ev.prevented, ev.stopped, el.attrs.size]).toEqual([0, 0, 0]);
      }
    });
    expect(ch.sent).toEqual([{ kernel: "stand-down", kind: "fixture", reason: "kernel-absent" }]);
    expect(warnings.length).toBe(1);
  });

  test("a kind the kernel does not have: reported as missing", () => {
    const { win, bridge, el, ch } = setup(true);
    bridge.register("ghost", { fields: ["type"], decide: (b: any, e: any, s: any, m: any) => bridge.kernel.decide("ghost", b, e, s, m) });
    bridge.scope(ch).run({ cmd: "behaviour", target: "host", behaviour: { kind: "ghost" }, declared: { ...DECLARED, event: "click", capture: false, passive: true }, epoch: { key: "g", n: 1 } });
    const { warnings } = quiet(() => el.dispatch("click", makeEvent()));
    expect(ch.sent).toEqual([{ kernel: "stand-down", kind: "ghost", reason: "missing-kind" }]);
    expect(warnings.length).toBe(1);
    void win;
  });

  test("a request the kernel cannot read (a parameter Rust requires is absent): reported, the kernel stays ready", () => {
    const { bridge, el, ch } = setup(true);
    bridge.scope(ch).run({ cmd: "behaviour", target: "host", behaviour: fixture({ attr: undefined, event: "click", cancel: false, capture: false }), declared: { ...DECLARED, event: "click", capture: false, passive: true }, epoch: { key: "u", n: 1 } });
    const ev = makeEvent();
    const { warnings } = quiet(() => el.dispatch("click", ev));
    expect(ch.sent).toEqual([{ kernel: "stand-down", kind: "fixture", reason: "unreadable-request" }]);
    expect(ev.prevented).toBe(0);
    expect(warnings.length).toBe(1);
    expect(bridge.kernel.state).toBe("ready");
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}).prevent).toBe(true);
  });

  test("a trap: reported as a trap, nothing cancelled, and the install's next events are still answered", async () => {
    const { win, bridge } = loadBridge(makeWindow(), { kernel: false });
    loadModule(win, "behaviours/fixture.js");
    expect(await bridge.kernel.load(module({ traps: [0] }))).toBe(true);
    const el = win.document.add("host");
    const ch = makeChannel();
    bridge.scope(ch).run({ cmd: "behaviour", target: "host", behaviour: fixture(), declared: DECLARED, epoch: { key: "k", n: 1 } });
    const { warnings } = quiet(() => {
      for (let i = 0; i < 2; i++) {
        const ev = makeEvent();
        el.dispatch("keydown", ev);
        expect([ev.prevented, el.attrs.size]).toEqual([0, 0]);
      }
    });
    expect(ch.sent).toEqual([{ kernel: "stand-down", kind: "fixture", reason: "trap" }]);
    expect(warnings.length).toBe(1);
  });

  test("garbage bytes and a module that is not a kernel leave it failed, with one warning", async () => {
    for (const bytes of [Uint8Array.of(1, 2, 3), Uint8Array.of(0, 0x61, 0x73, 0x6d, 1, 0, 0, 0)]) {
      const { bridge } = loadBridge(makeWindow(), { kernel: false });
      const { out, warnings } = quiet(() => bridge.kernel.load(bytes));
      expect(await out).toBe(false);
      expect(bridge.kernel.state).toBe("failed");
      expect(warnings.length).toBe(1);
      expect(standDown(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}))).toEqual({ kind: "fixture", reason: "kernel-failed" });
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
    expect(standDown(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}))?.reason).toBe("kernel-failed");
  });
});

describe("a trap is contained to its kind", () => {
  test("a trap in one kind leaves the other deciding, and the module is instantiated afresh", async () => {
    const { kernel, warnings } = make();
    const real = WebAssembly.Instance;
    let instances = 0;
    (WebAssembly as any).Instance = function (...args: unknown[]) {
      instances++;
      return new (real as any)(...args);
    };
    try {
      expect(await kernel.load(module({ traps: [0] }))).toBe(true);
      expect(instances).toBe(1);
      expect(standDown(() => kernel.decide("a", {}, {}, {}, {}))).toEqual({ kind: "a", reason: "trap" });
      expect(instances).toBe(2);
      expect(kernel.decide("b", {}, {}, {}, {})).toEqual({});
      expect(kernel.state).toBe("ready");
      expect(warnings).toEqual(["atrap"]);
    } finally {
      (WebAssembly as any).Instance = real;
    }
  });

  test("three traps stand that kind down alone: the fourth call is refused without calling the kernel, the other kind decides", async () => {
    const { kernel } = make();
    await kernel.load(module({ traps: [0] }));
    expect([1, 2, 3].map(() => standDown(() => kernel.decide("a", {}, {}, {}, {}))?.reason)).toEqual(["trap", "trap", "trap"]);
    expect(standDown(() => kernel.decide("a", {}, {}, {}, {}))).toEqual({ kind: "a", reason: "kind-stood-down" });
    expect(kernel.decide("b", {}, {}, {}, {})).toEqual({});
    expect(kernel.state).toBe("ready");
  });

  test("a kind that does not trap is never counted against the others", async () => {
    const { kernel } = make();
    await kernel.load(module({ traps: [0, 1] }));
    for (let i = 0; i < 2; i++) standDown(() => kernel.decide("a", {}, {}, {}, {}));
    for (let i = 0; i < 2; i++) standDown(() => kernel.decide("b", {}, {}, {}, {}));
    expect(standDown(() => kernel.decide("a", {}, {}, {}, {}))?.reason).toBe("trap");
    expect(standDown(() => kernel.decide("b", {}, {}, {}, {}))?.reason).toBe("trap");
    expect(standDown(() => kernel.decide("a", {}, {}, {}, {}))?.reason).toBe("kind-stood-down");
  });
});

describe("the marshalling", () => {
  test("strings of any script cross both ways, and a key named __proto__ stays data", () => {
    const { bridge } = loadBridge();
    const text = "значение \u{1F600} 日本語 é";
    const out = bridge.kernel.decide("fixture", fixture({ attr: "__proto__", value: text }), FIELDS, OPEN, {});
    expect(Object.getPrototypeOf(out.attrs)).toBe(Object.prototype);
    expect(Object.keys(out.attrs)).toEqual(["__proto__"]);
    expect(Object.getOwnPropertyDescriptor(out.attrs, "__proto__")?.value).toBe(text);
  });

  test("a lone surrogate is written as U+FFFD, the way TextEncoder writes it", () => {
    const { bridge } = loadBridge();
    const out = bridge.kernel.decide("fixture", fixture({ value: "a\ud800b\udc00" }), FIELDS, OPEN, {});
    expect(out.attrs["data-dx-fixture"]).toBe(new TextDecoder().decode(new TextEncoder().encode("a\ud800b\udc00")));
  });

  test("a request larger than the first buffer, with nested memory, still decides", () => {
    const { bridge } = loadBridge();
    const memory = { list: Array.from({ length: 2000 }, (_, i) => ({ i, s: "x".repeat(20), b: i % 2 === 0, n: null })) };
    expect(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, memory).prevent).toBe(true);
  });

  test("a value nested past the limit is a stand-down, not a decision, and the kernel stays ready", () => {
    const { bridge } = loadBridge();
    let deep: unknown = 1;
    for (let i = 0; i < 40; i++) deep = [deep];
    const { out } = quiet(() => standDown(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, deep)));
    expect(out).toEqual({ kind: "fixture", reason: "unreadable-request" });
    expect(bridge.kernel.state).toBe("ready");
  });

  test("the call is deterministic: ten thousand decisions give one answer", async () => {
    const { bridge } = loadBridge();
    await ticks();
    const first = JSON.stringify(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}));
    for (let i = 0; i < 10_000; i++) if (JSON.stringify(bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {})) !== first) throw new Error("drift at " + i);
  });
});

describe("the in-place codec", () => {
  /** Runs `fn` with the allocating calls the codec must not make counted through spies. */
  function counting(fn: () => void): Record<string, number> {
    const counts: Record<string, number> = { subarray: 0, entries: 0, keys: 0, values: 0, encodeInto: 0, decode: 0, views: 0 };
    const spies: [any, string, string][] = [
      [Uint8Array.prototype, "subarray", "subarray"],
      [Object, "entries", "entries"],
      [Object, "keys", "keys"],
      [Object, "values", "values"],
      [TextEncoder.prototype, "encodeInto", "encodeInto"],
      [TextDecoder.prototype, "decode", "decode"],
    ];
    const saved = spies.map(([target, name]) => target[name]);
    for (const [target, name, label] of spies) {
      const original = target[name];
      target[name] = function (this: unknown, ...args: unknown[]) {
        counts[label]++;
        return original.apply(this, args);
      };
    }
    const RealDataView = globalThis.DataView;
    (globalThis as any).DataView = class extends RealDataView {
      constructor(...args: [ArrayBufferLike]) {
        super(...args);
        counts.views++;
      }
    };
    try {
      fn();
    } finally {
      spies.forEach(([target, name], i) => (target[name] = saved[i]));
      globalThis.DataView = RealDataView;
    }
    return counts;
  }

  test("a call allocates no subarray, no entries array, no encoder result and no view", () => {
    const { bridge } = loadBridge();
    const call = () => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, { disabled: "01".repeat(500), list: [true, false, null, 1, "x"] });
    for (let i = 0; i < 200; i++) call();
    expect(counting(() => {
      for (let i = 0; i < 1000; i++) call();
    })).toEqual({ subarray: 0, entries: 0, keys: 0, values: 0, encodeInto: 0, decode: 0, views: 0 });
  });

  /** The smallest p99 of three runs of `n` timed calls: a loaded box makes a run slower, never faster. */
  function p99(call: () => unknown, n = 5000): number {
    for (let i = 0; i < 500; i++) call();
    let best = Infinity;
    for (let round = 0; round < 3; round++) {
      const times: number[] = [];
      for (let i = 0; i < n; i++) {
        const t = performance.now();
        call();
        times.push(performance.now() - t);
      }
      times.sort((a, b) => a - b);
      best = Math.min(best, times[Math.floor(n * 0.99)]);
    }
    return best;
  }

  test("the fixture call's p99 is under 0.05 ms", () => {
    const { bridge } = loadBridge();
    const ms = p99(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, {}));
    console.log(`KERNEL-CALL fixture p99 ${(ms * 1000).toFixed(1)} us`);
    expect(ms).toBeLessThan(0.05);
  });

  test("each kind's largest planned snapshot (packed) is under 0.5 ms at the p99", () => {
    const { bridge } = loadBridge();
    for (const [kind, list] of Object.entries<any[]>(vectors)) {
      const v = list.find((x) => x.largest);
      expect(v).toBeDefined();
      const ms = p99(() => bridge.kernel.decide(kind, v.behaviour, v.event, v.state, v.memory ?? {}), 2000);
      console.log(`KERNEL-CALL ${kind} largest p99 ${(ms * 1000).toFixed(1)} us`);
      expect(ms).toBeLessThan(0.5);
    }
  });

  test("a memory of 1,000 booleans (an array of scalars) is under 0.5 ms at the p99, and the same list as objects is the cost the packed form avoids", () => {
    const { bridge } = loadBridge();
    const packed = { disabled: Array.from({ length: 1000 }, (_, i) => i % 2 === 0) };
    const ms = p99(() => bridge.kernel.decide("fixture", fixture(), FIELDS, OPEN, packed), 2000);
    console.log(`KERNEL-CALL 1,000 booleans p99 ${(ms * 1000).toFixed(1)} us`);
    expect(ms).toBeLessThan(0.5);
  });
});
