// The decision kernel's page half: loads the wasm module `decision-kernel` builds and calls its `decide` for a named
// behaviour synchronously, inside the event's dispatch. `createKernel` is called once by `ts/bridge.ts`, which exposes
// the result as `window.__dxBridge.kernel`; a behaviour module reaches it through `kernelBehaviour` (lib/register.ts).
//
// THE TABLE. A kind is an index into the kernel's kind table, which Rust generates (`behaviours.gen.ts`:
// `KERNEL_KINDS` in wire-index order, `KERNEL_HASH`). The module exports exactly five functions, `dx_abi`, `dx_kinds`,
// `dx_input`, `dx_decide(kind_index, len)` and `dx_output`; a module whose `dx_kinds` differs from the page's hash is
// refused (state `failed`, one warning). The keys of each kind's three inputs are the Rust input types' wire keys, so
// the page collects exactly what the kernel reads.
//
// BYTES, IN PLACE. A request is one array `[config, memory, snapshot, event]`, an answer one object, both in the
// encoding `decision-kernel/src/value.rs` documents (a tag byte per value: null, false, true, f64, string, array,
// object; lengths and counts u32 little-endian). The request is written straight into the buffer `dx_input` returns
// and the answer is read straight out of `dx_output`, through one `DataView` of the module's memory that is rebuilt
// only when the memory has grown. A call allocates no staging buffer, no `subarray`, no entries array and no pairs
// array: a string is coded through its UTF-8 byte string (`unescape(encodeURIComponent(s))` and back) and moved byte by
// byte, the three keyed inputs are written from their generated key lists, and an answer object is built by
// `Object.fromEntries` over a `Pairs` cursor that reuses one entry.
//
// DEPTH. The kernel refuses a request nested past 16 levels (`decision-kernel/src/value.rs`): that is an unreadable
// request, and a value nested so deep that the page's own recursion fails is the same.
//
// FAILING LOUDLY. A kind that cannot decide throws a `KernelStandDown` report (the bridge answers no decision, so the browser
// keeps its native action, and reports once to Rust): the kernel has no such kind, it cannot read the request, it
// trapped, it stood down after three traps, or it never loaded. Each also `warnOnce`s. No case answers `{}` silently.
//
// A TRAP is contained to its kind. The kernel keeps no state between calls but its two buffers, so after a trap the
// module is instantiated afresh from the cached `WebAssembly.Module`; a kind that traps three times in one page stands
// down alone and the other kinds keep deciding.

type Wire = null | boolean | number | string | Wire[] | { [key: string]: Wire };

/** The layout version `decision-kernel/src/ffi.rs` exports as `dx_abi`. */
const ABI = 2;
/** Traps of one kind in one page after which that kind decides no more. */
const MAX_TRAPS = 3;
/** The size the input buffer is first asked for; it doubles when a request does not fit. */
const FIRST_CAPACITY = 4096;

/** Why a kind could not decide: each is the `reason` of the report Rust receives (`StandDownReason`). */
export type StandDownReason = "missing-kind" | "unreadable-request" | "trap" | "kind-stood-down" | "kernel-failed" | "kernel-absent";

/**
 * What `decide` throws when a kind cannot decide, and what the bridge sends to Rust as it is: a plain object, so that
 * nothing but this shape (`kernel` is the marker no decision's `send` carries) tells a report from a message.
 */
export interface KernelStandDown {
  readonly kernel: "stand-down";
  readonly kind: string;
  readonly reason: StandDownReason;
}

/** One row of the generated kind table. */
export interface KernelRow {
  readonly kind: string;
  /** The keys of the behaviour's parameters the decision reads. */
  readonly config: readonly string[];
  /** The attribute reads (the snapshot's keys). */
  readonly reads: readonly string[];
  /** The event fields (the event's keys). */
  readonly fields: readonly string[];
}

export interface KernelDeps {
  /** The bridge's closed-table reader: the one sanctioned computed read. */
  own<T>(table: Record<string, T>, key: unknown): T | undefined;
  warnOnce(key: string, message: string): void;
  /** The generated kind table, in wire-index order. */
  rows: readonly KernelRow[];
  /** The generated hash of that table. */
  hash: number;
}

interface KernelExports {
  memory: WebAssembly.Memory;
  dx_abi(): number;
  dx_kinds(): number;
  dx_input(len: number): number;
  dx_decide(kind: number, len: number): number;
  dx_output(): number;
}

type State = "absent" | "ready" | "failed";

export function createKernel(deps: KernelDeps): DxKernel {
  const { own, warnOnce, rows, hash } = deps;

  const kinds = new Map<string, { index: number; row: KernelRow }>();
  rows.forEach((row, index) => kinds.set(row.kind, { index, row }));

  let state: State = "absent";
  let compiled: WebAssembly.Module | null = null;
  let exp: KernelExports | null = null;
  const traps = new Map<string, number>();
  const stats: { mode: "sync" | null; startMs: number | null; readyMs: number | null } = { mode: null, startMs: null, readyMs: null };

  // ------------------------------------------------------------ the module's memory

  let dv = new DataView(new ArrayBuffer(0));
  /** The view of the module's memory as it is now (the memory object changes whenever it grows). */
  const views = (memory: WebAssembly.Memory): void => {
    if (dv.buffer !== memory.buffer) dv = new DataView(memory.buffer);
  };
  /** The cursor: the next byte to write or read, an absolute address in the memory. */
  let pos = 0;

  // ------------------------------------------------------------ encoding

  let end = 0;
  let base = 0;
  let cap = FIRST_CAPACITY;
  /** Thrown inside the encoder when the request does not fit the input buffer; the call retries with a larger one. */
  const FULL = {};

  const need = (n: number): void => {
    if (pos + n > end) throw FULL;
  };
  const wr = (byte: number): void => void dv.setUint8(pos++, byte);
  const putLen = (n: number): void => {
    dv.setUint32(pos, n, true);
    pos += 4;
  };

  const putStr = (text: string): void => {
    let raw: string;
    try {
      raw = unescape(encodeURIComponent(text));
    } catch {
      // a lone surrogate is U+FFFD on the wire, the way TextEncoder writes it
      raw = unescape(encodeURIComponent(text.toWellFormed()));
    }
    need(4 + raw.length);
    putLen(raw.length);
    for (let i = 0; i < raw.length; i++) wr(raw.charCodeAt(i));
  };

  /** Writes `value`; an object of `keys` (the generated lists) is written from them: absent members are null. */
  const put = (value: unknown, keys?: readonly string[]): void => {
    need(9);
    if (value === null || value === undefined) wr(0);
    else if (typeof value === "boolean") wr(value ? 2 : 1);
    else if (typeof value === "number") {
      wr(3);
      dv.setFloat64(pos, value, true);
      pos += 8;
    } else if (typeof value === "string") {
      wr(4);
      putStr(value);
    } else if (Array.isArray(value)) {
      wr(5);
      putLen(value.length);
      for (const item of value) put(item);
    } else if (typeof value === "object") {
      // the members of `keys`, else (a behaviour's memory) every own enumerable member, in order
      wr(6);
      const countAt = pos;
      pos += 4;
      let count = 0;
      const record = value as Record<string, unknown>;
      if (keys) {
        for (const key of keys) {
          putStr(key);
          put(own(record, key));
        }
        count = keys.length;
      } else {
        for (const key in record) {
          putStr(key);
          put(own(record, key));
          count++;
        }
      }
      dv.setUint32(countAt, count, true);
    } else throw new Error("not encodable");
  };

  // ------------------------------------------------------------ decoding

  const rd = (): number => dv.getUint8(pos++);
  const string = (): string => {
    const n = dv.getUint32(pos, true);
    pos += 4;
    let raw = "";
    for (const stop = pos + n; pos < stop; ) raw += String.fromCharCode(rd());
    return decodeURIComponent(escape(raw));
  };

  const entry: [string, Wire] = ["", null];
  const result: { done: boolean; value: [string, Wire] } = { done: false, value: entry };

  /** An answer object's members as `[key, value]` entries, decoded one at a time into the one reused entry. */
  class Pairs {
    constructor(private left: number) {}
    [Symbol.iterator](): Pairs {
      return this;
    }
    next(): IteratorResult<[string, Wire]> {
      if (this.left <= 0) {
        result.done = true;
        return result as IteratorResult<[string, Wire]>;
      }
      this.left--;
      // a nested object is decoded (and reuses the entry) before this one's entry is filled
      const key = string();
      const value = read();
      entry[0] = key;
      entry[1] = value;
      result.done = false;
      return result as IteratorResult<[string, Wire]>;
    }
  }

  const read = (): Wire => {
    const tag = rd();
    switch (tag) {
      case 0:
        return null;
      case 1:
        return false;
      case 2:
        return true;
      case 3: {
        const n = dv.getFloat64(pos, true);
        pos += 8;
        return n;
      }
      case 4:
        return string();
      case 5: {
        const count = dv.getUint32(pos, true);
        pos += 4;
        const items: Wire[] = [];
        for (let i = count; i > 0; i--) items.push(read());
        return items;
      }
      case 6: {
        const count = dv.getUint32(pos, true);
        pos += 4;
        // fromEntries defines each key as data: a key named `__proto__` is never the prototype setter
        return Object.fromEntries(new Pairs(count));
      }
      default:
        throw new Error("bad tag " + tag);
    }
  };

  // ------------------------------------------------------------ the module

  const fail = (error: unknown): false => {
    state = "failed";
    exp = null;
    warnOnce("kernel-load", "kernel did not load: " + String(error));
    return false;
  };

  const instantiate = (module: WebAssembly.Module): KernelExports => {
    const found = new WebAssembly.Instance(module, {}).exports as unknown as KernelExports;
    if (found.dx_abi() !== ABI) throw new Error("wrong ABI");
    if (found.dx_kinds() >>> 0 !== hash) throw new Error("kind table differs");
    return found;
  };

  /**
   * Compiles and instantiates the kernel synchronously, so that it is ready before the eval that loads it returns: no
   * event can precede it, and there is no window in which a cancelling behaviour is silently off. A module that does
   * not compile (an engine that refuses a synchronous compile of this size, a Content-Security-Policy without
   * `'wasm-unsafe-eval'`), is not a kernel, or has another ABI or kind table leaves it `failed` (one warning; every
   * decision then stands down and reports). True when ready.
   */
  const load = (source: BufferSource | string): boolean => {
    if (state !== "absent") return state === "ready";
    stats.startMs = performance.now();
    try {
      compiled = new WebAssembly.Module(typeof source === "string" ? Uint8Array.from(atob(source), (c) => c.charCodeAt(0)) : source);
      exp = instantiate(compiled);
      state = "ready";
      stats.mode = "sync";
      stats.readyMs = performance.now();
      return true;
    } catch (error) {
      return fail(error);
    }
  };

  const standDown = (kind: string, reason: StandDownReason, detail: string): KernelStandDown => {
    warnOnce(kind + reason, "kernel cannot decide " + kind + " (" + reason + "): " + detail);
    return { kernel: "stand-down", kind, reason };
  };

  /** A trap (or an answer that cannot be read): a fresh instance of the cached module; a kind's third trap ends it. */
  const trapped = (kind: string, error: unknown): KernelStandDown => {
    const count = (traps.get(kind) ?? 0) + 1;
    traps.set(kind, count);
    try {
      exp = instantiate(compiled!);
    } catch (again) {
      fail(again);
    }
    return standDown(kind, "trap", String(error) + " (#" + count + ")");
  };

  /**
   * The decision of the behaviour `kind`. Throws a `KernelStandDown` report for a kind that cannot decide.
   */
  const decide = (kind: string, behaviour: unknown, event: unknown, snapshot: unknown, memory: unknown): DxDecision => {
    const found = kinds.get(kind);
    if (!found) throw standDown(kind, "missing-kind", "absent");
    const k = exp;
    if (state !== "ready" || !k) throw standDown(kind, state === "failed" ? "kernel-failed" : "kernel-absent", state);
    if ((traps.get(kind) ?? 0) >= MAX_TRAPS) throw standDown(kind, "kind-stood-down", "trapped " + MAX_TRAPS + " times");
    const { index, row } = found;
    // where a failure came from decides its reason: the page's own encoding, or the kernel
    let reason: StandDownReason = "unreadable-request";
    try {
      let len = 0;
      for (;;) {
        const input = k.dx_input(cap);
        views(k.memory);
        pos = base = input;
        end = input + cap;
        try {
          need(5);
          wr(5);
          putLen(4);
          put(behaviour, row.config);
          put(memory);
          put(snapshot, row.reads);
          put(event, row.fields);
          len = pos - base;
          break;
        } catch (error) {
          if (error !== FULL) throw error;
          cap *= 2;
        }
      }
      reason = "trap";
      const n = k.dx_decide(index, len);
      views(k.memory);
      if (n < 0) {
        reason = "unreadable-request";
        throw "the kernel answered no length";
      }
      pos = k.dx_output();
      return read() as DxDecision;
    } catch (error) {
      throw reason === "trap" ? trapped(kind, error) : standDown(kind, reason, String(error));
    }
  };

  return {
    load,
    decide,
    get state() {
      return state;
    },
    stats,
  };
}
