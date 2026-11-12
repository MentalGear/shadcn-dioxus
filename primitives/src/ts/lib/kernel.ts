// The decision kernel's page half: loads the wasm module `decision-kernel` builds and calls its `decide` for a
// named behaviour synchronously, inside the event's dispatch. Bundled into `bridge.js`; a behaviour module reaches it
// as `window.__dxBridge.kernel` through `kernelDecide` (lib/register.ts).
//
// BYTES. A request is one array `[config, memory, snapshot, event]`, an answer one object, both in the encoding
// `decision-kernel/src/value.rs` documents (a tag byte per value: null, false, true, f64, string, array, object;
// lengths and counts are u32 little-endian). The page writes the request into the kernel's input buffer
// (`dx_input`), calls `dx_decide_<kind>`, and decodes the answer from `dx_output`. The memory may grow in a call, so
// every view of it is taken fresh.
//
// FAIL OPEN. Until the kernel is ready, after a load that failed, and after a trap, `decide` answers `{}`: the
// behaviour stands down and the browser keeps its native action. A kernel that is absent can never trap an input.

type Wire = null | boolean | number | string | Wire[] | { [key: string]: Wire };

/** The layout version `decision-kernel/src/ffi.rs` exports as `dx_abi`. */
const ABI = 1;
/** The most levels of nesting either side accepts (`decision-kernel/src/value.rs` has the same limit). */
const MAX_DEPTH = 16;

interface KernelExports {
  memory: WebAssembly.Memory;
  dx_abi(): number;
  dx_input(len: number): number;
  dx_output(): number;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

// ---------------------------------------------------------------- encoding

let buf: Uint8Array<ArrayBuffer> = new Uint8Array(512);
let view = new DataView(buf.buffer);
let used = 0;

function room(n: number): void {
  if (used + n <= buf.length) return;
  let size = buf.length * 2;
  while (size < used + n) size *= 2;
  const next = new Uint8Array(size);
  next.set(buf.subarray(0, used));
  buf = next;
  view = new DataView(next.buffer);
}

function putTag(tag: number): void {
  view.setUint8(used++, tag);
}

function putLen(n: number): void {
  room(4);
  view.setUint32(used, n, true);
  used += 4;
}

function putStr(s: string): void {
  room(4 + s.length * 3);
  const { written } = encoder.encodeInto(s, buf.subarray(used + 4));
  view.setUint32(used, written, true);
  used += 4 + written;
}

function put(value: unknown, depth: number): void {
  if (depth > MAX_DEPTH) throw new Error("kernel: value nested too deep");
  room(1);
  if (value === null || value === undefined) putTag(0);
  else if (value === false) putTag(1);
  else if (value === true) putTag(2);
  else if (typeof value === "number") {
    room(9);
    putTag(3);
    view.setFloat64(used, value, true);
    used += 8;
  } else if (typeof value === "string") {
    putTag(4);
    putStr(value);
  } else if (Array.isArray(value)) {
    putTag(5);
    putLen(value.length);
    for (const item of value) put(item, depth + 1);
  } else if (typeof value === "object") {
    const entries = Object.entries(value);
    putTag(6);
    putLen(entries.length);
    for (const [key, item] of entries) {
      putStr(key);
      put(item, depth + 1);
    }
  } else throw new Error("kernel: cannot encode a " + typeof value);
}

// ---------------------------------------------------------------- decoding

// The answer is read straight out of the module's memory: a cursor and two views that are rebuilt only when the
// memory has grown (a call then allocates no closure and no view of its own).
let rbuf: ArrayBufferLike | null = null;
let rdv = new DataView(new ArrayBuffer(0));
let rbytes = new Uint8Array(0);
let rat = 0;

/** Views of the module's memory as it is now (the memory object changes whenever it grows). */
function memoryViews(memory: WebAssembly.Memory): void {
  const now = memory.buffer;
  if (now === rbuf) return;
  rbuf = now;
  rdv = new DataView(now);
  rbytes = new Uint8Array(now);
}

function rLen(): number {
  const n = rdv.getUint32(rat, true);
  rat += 4;
  return n;
}

function rStr(): string {
  const n = rLen();
  const s = decoder.decode(rbytes.subarray(rat, rat + n));
  rat += n;
  return s;
}

function read(depth: number): Wire {
  if (depth > MAX_DEPTH) throw new Error("kernel: answer nested too deep");
  const tag = rdv.getUint8(rat++);
  switch (tag) {
    case 0:
      return null;
    case 1:
      return false;
    case 2:
      return true;
    case 3: {
      const n = rdv.getFloat64(rat, true);
      rat += 8;
      return n;
    }
    case 4:
      return rStr();
    case 5: {
      const items: Wire[] = [];
      for (let i = rLen(); i > 0; i--) items.push(read(depth + 1));
      return items;
    }
    case 6: {
      const pairs: [string, Wire][] = [];
      for (let i = rLen(); i > 0; i--) {
        const key = rStr();
        pairs.push([key, read(depth + 1)]);
      }
      // fromEntries defines each key as data: a key named `__proto__` is never the prototype setter
      return Object.fromEntries(pairs);
    }
    default:
      throw new Error("kernel: bad tag " + tag);
  }
}

// ---------------------------------------------------------------- the kernel

type State = "absent" | "loading" | "ready" | "failed";

let state: State = "absent";
let exp: KernelExports | null = null;
const decides = new Map<string, (len: number) => number>();
const stats: { mode: "sync" | "async" | null; startMs: number | null; readyMs: number | null } = { mode: null, startMs: null, readyMs: null };

const warned = new Set<string>();
const warnOnce = (key: string, message: string): void => {
  if (warned.has(key)) return;
  warned.add(key);
  console.warn("dx bridge:", message);
};

const bytesOf = (source: BufferSource | string): BufferSource => {
  if (typeof source !== "string") return source;
  const raw = atob(source);
  const out = new Uint8Array(raw.length);
  const dv = new DataView(out.buffer);
  for (let i = 0; i < raw.length; i++) dv.setUint8(i, raw.charCodeAt(i));
  return out;
};

function adopt(module: WebAssembly.Module): void {
  const instance = new WebAssembly.Instance(module, {});
  const found = instance.exports as unknown as KernelExports;
  if (typeof found.dx_abi !== "function" || found.dx_abi() !== ABI) throw new Error("kernel: wrong ABI");
  decides.clear();
  for (const [name, fn] of Object.entries(instance.exports)) {
    if (name.startsWith("dx_decide_") && typeof fn === "function") decides.set(name.slice(10).replace(/_/g, "-"), fn as (len: number) => number);
  }
  exp = found;
  state = "ready";
  stats.readyMs = performance.now();
}

function fail(error: unknown): void {
  state = "failed";
  exp = null;
  decides.clear();
  warnOnce("kernel-load", "the decision kernel did not load, behaviours stand down: " + String(error));
}

/**
 * Compiles and instantiates the kernel. A module is compiled synchronously, so that it is ready before the
 * eval that loads it returns (every engine this was tried on accepts a synchronous compile at this size); where
 * the engine refuses, the asynchronous compile takes over and the behaviours stand down until it finishes. A
 * module that does not compile, is not a kernel or is blocked (a Content-Security-Policy without
 * `'wasm-unsafe-eval'`) leaves it `failed`. Resolves true when ready.
 */
function load(source: BufferSource | string, mode: "auto" | "async" = "auto"): Promise<boolean> {
  if (state === "ready" || state === "loading") return Promise.resolve(state === "ready");
  stats.startMs = performance.now();
  state = "loading";
  let bytes: BufferSource;
  try {
    bytes = bytesOf(source);
  } catch (error) {
    fail(error);
    return Promise.resolve(false);
  }
  if (mode === "auto") {
    try {
      adopt(new WebAssembly.Module(bytes));
      stats.mode = "sync";
      return Promise.resolve(true);
    } catch (error) {
      if (!(error instanceof RangeError)) {
        fail(error);
        return Promise.resolve(false);
      }
    }
  }
  stats.mode = "async";
  return WebAssembly.compile(bytes).then(
    (module) => {
      try {
        adopt(module);
        return true;
      } catch (error) {
        fail(error);
        return false;
      }
    },
    (error) => {
      fail(error);
      return false;
    },
  );
}

/**
 * The decision of the behaviour `kind`, or `{}` (stand down) while there is no kernel, for a kind the kernel does
 * not export, for a malformed request, and for ever after a trap.
 */
function decide(kind: string, behaviour: unknown, event: unknown, snapshot: unknown, memory: unknown): DxDecision {
  const call = state === "ready" ? decides.get(kind) : undefined;
  const k = exp;
  if (!call || !k) return {};
  used = 0;
  put([behaviour, memory, snapshot, event], 0);
  try {
    // the memory is viewed after each call that can grow it, never before
    const at = k.dx_input(used);
    memoryViews(k.memory);
    rbytes.set(buf.subarray(0, used), at);
    const n = call(used);
    if (n < 0) return {};
    const out = k.dx_output();
    memoryViews(k.memory);
    rat = out;
    return read(0) as DxDecision;
  } catch (error) {
    // a trap leaves the module's state unknown: it takes no more decisions
    state = "failed";
    exp = null;
    decides.clear();
    warnOnce("kernel-trap", "the decision kernel trapped, behaviours stand down: " + String(error));
    return {};
  }
}

export const kernel: DxKernel = {
  load,
  decide,
  get state() {
    return state;
  },
  stats,
};
