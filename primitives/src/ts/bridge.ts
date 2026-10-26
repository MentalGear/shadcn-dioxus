// The generic JS bridge (row 434 step 3): the one small typed glue layer between Rust and the browser
// APIs Dioxus does not wrap. Generic verbs and a registry of named behaviours, no component logic.
//
// THE SURFACE IS CLOSED. Rust sends closed enums (`EventKind`, `EventField`, `Method`, `Prop`,
// `ObserverKind`, `Behaviour`), and this file maps each through a closed OWN-property table with a
// receiver kind per row (`EVENTS`, `FIELD_GETTERS`, `METHODS`, `PROPS`). There is no `el[method](...)`,
// no `event[field]` and no `target[prop]` keyed by a string from outside: a name that is not in a table
// is an error, never a property read (otherwise `call(window, "setTimeout", ["code"])` would be an eval).
//
//   scope(channel)   owns every listener, observer, behaviour and loop it creates; `dispose()` undoes all
//   listen           a STREAMING listener (window, document, element): named event fields to Rust. It can
//                    never cancel an event, so it is always passive; a frame-rate event (`hot`) is
//                    coalesced to one message per animation frame
//   on               a listener running a JS handler, for `js_listener.rs`'s bespoke scripts
//   call             a closed list of element methods (popover, dialog, scroll, selection...). Pointer capture and
//                    focus are NOT verbs: a call lands after the event's dispatch, too late for either; they are
//                    effects of a named behaviour or rsx handlers. There is no `commit` either (it was a frame wait
//                    that does not order anything on desktop)
//   get              a closed list of reads (geometry, computed style, window metrics, active element)
//   set              a `--dx-*` CSS custom property on an element whose style Rust does not own
//   observe          Resize, Intersection and Mutation observers
//   matchMedia       a subscription to a media query
//   sample           per-frame sampling while a condition holds; a field is coalesced (the last value of
//                    the frame) or batched (every event with its own timestamp), flushed per frame or per
//                    event task
//   transaction      several ops in one JS turn (a show and the read of its rects see one DOM state)
//   behaviour        a NAMED behaviour from the registry (`ts/behaviours/<kind>.ts`), see below
//   serve / pump     the loop a Rust `bridge::Session` runs, and the ordered write channel of a webview
//
// WHERE WORK GOES. An rsx handler decides synchronously on every renderer, so a target inside a Dioxus
// element and an event Dioxus supports is an rsx handler. A window, document or outside target, an
// event Dioxus lacks or the capture phase is a named behaviour. A continuous stream (scroll, resize,
// pointermove, wheel) is `listen` / `observe` / `sample`: never a vdom handler, because on desktop every
// vdom event is a blocking synchronous request and a frame-rate stream would queue behind itself.
//
// BEHAVIOURS. A behaviour is a PURE `decide(behaviour, event, state)` plus this file's thin adapter. Rust
// declares target, event, phase and passivity (`DxDeclared`); the module declares only the closed event
// fields and attribute reads it needs. The adapter owns the listener: the hub below registers ONE DOM
// listener per (target, type, capture, passive) with a reference count, so a non-passive listener exists
// only while some behaviour needs it. A decision's `prevent` / `stop` are applied only when Rust declared
// the behaviour non-passive; `attrs` are limited to `data-*` / `aria-*`. Every install carries an epoch
// (a stale install is dropped, an equal one renews) and may carry a lease (the listener removes itself
// when Rust stops renewing: the fail-open path, WCAG 2.1.2). An op that carries an epoch is dropped the
// same way, and its reply echoes it.
//
// The file is a script, not a module: `primitives/src/bridge/transport.rs` evaluates the bundled
// `js/bridge.js` once per page and every verb is then reached as `window.__dxBridge`. All browser
// globals are read through `window`, so a test can run the shipped file against a fake one.
//
// BRIDGE-LATENCY HARNESS (not a desktop model): `window.__dxBridgeDelayFrames = N` defers every message
// through a bridge channel by N animation frames, in both directions, so a spec sees what a slow channel
// does to Rust-side logic on a browser where the eval channel costs a microtask. `"never"` drops every
// message to Rust (the answer that never arrives: a pending state must still end visible); commands from
// Rust still arrive. It does not model desktop's cadence (edits applied in a frame, one render per
// frame): `channelDelay` is the one seam where a "desktop cadence" mode would plug in.
//
// LIFETIMES: a registration on window or document lives until its scope ends (Rust sends the teardown
// over the scope's own channel, never a new eval). A registration on an ELEMENT also ends when the
// element leaves the document, without waiting for Rust (Dioxus has no merged `onunmounted`).

/// <reference path="./types/dioxus.d.ts" />
/// <reference path="./types/bridge.d.ts" />

type Recv = "w" | "d" | "e" | "we" | "wd" | "de" | "wde";
type Fields = Record<string, unknown>;

type ListenSpec = Extract<DxCommand, { cmd: "listen" }>;
type ObserveSpec = Extract<DxCommand, { cmd: "observe" }>;
type SampleSpec = Extract<DxCommand, { cmd: "sample" }>;
type BehaviourSpec = Extract<DxCommand, { cmd: "behaviour" }>;

interface Scope extends DxScope {
  listen(spec: Omit<ListenSpec, "cmd">): () => void;
  observe(spec: Omit<ObserveSpec, "cmd">): () => void;
  media(query: string): () => void;
  sample(spec: Omit<SampleSpec, "cmd">): () => void;
  behaviour(spec: Omit<BehaviourSpec, "cmd">): () => void;
}

const W = window as any;

/** The message that ends a session or script (`js_listener.rs`'s `TEARDOWN`). */
const TEARDOWN = "__dx_listeners_off__";
/** The events whose non-passive listeners block scrolling: `passive: true` unless the caller says so. */
const BLOCKING: Record<string, 1> = { wheel: 1, mousewheel: 1, touchstart: 1, touchmove: 1, DOMMouseScroll: 1 };

const hasOwn = (table: object, key: string): boolean => Object.prototype.hasOwnProperty.call(table, key);
/** The row of a closed table, own properties only (`"constructor"` and `"__proto__"` are not rows). */
function own<T>(table: Record<string, T>, key: unknown): T | undefined {
  return typeof key === "string" && hasOwn(table, key) ? table[key] : undefined;
}
const norm = (value: unknown): unknown => (value === undefined ? null : value);
const prim = (value: unknown): unknown => {
  const type = typeof value;
  return type === "string" || type === "number" || type === "boolean" ? value : null;
};

// ---------------------------------------------------------------- the closed tables

/**
 * The events the bridge may listen to. `recv` says what it may be listened on: w window, d document,
 * e element. `hot` marks a frame-rate event: `listen` coalesces it to one message per animation frame.
 * Mirrors `bridge/events.rs`'s `EventKind` (a test compares the rows).
 */
const EVENTS: Record<string, { recv: Recv; hot?: 1 }> = {
  "keydown": { recv: "wde" },
  "keyup": { recv: "wde" },
  "beforeinput": { recv: "e" },
  "input": { recv: "e" },
  "change": { recv: "e" },
  "compositionstart": { recv: "e" },
  "compositionend": { recv: "e" },
  "click": { recv: "wde" },
  "dblclick": { recv: "wde" },
  "contextmenu": { recv: "wde" },
  "pointerdown": { recv: "wde" },
  "pointerup": { recv: "wde" },
  "pointermove": { recv: "wde", hot: 1 },
  "pointercancel": { recv: "wde" },
  "mousedown": { recv: "wde" },
  "mouseup": { recv: "wde" },
  "mousemove": { recv: "wde", hot: 1 },
  "touchstart": { recv: "wde" },
  "touchmove": { recv: "wde", hot: 1 },
  "touchend": { recv: "wde" },
  "touchcancel": { recv: "wde" },
  "wheel": { recv: "wde", hot: 1 },
  "scroll": { recv: "wde", hot: 1 },
  "resize": { recv: "w", hot: 1 },
  "focusin": { recv: "wde" },
  "focusout": { recv: "wde" },
  "focus": { recv: "e" },
  "blur": { recv: "e" },
  "dragenter": { recv: "wde" },
  "dragover": { recv: "wde", hot: 1 },
  "dragleave": { recv: "wde" },
  "drop": { recv: "wde" },
  "copy": { recv: "wde" },
  "cut": { recv: "wde" },
  "paste": { recv: "wde" },
  "visibilitychange": { recv: "d" },
  "selectionchange": { recv: "d" },
  "pagehide": { recv: "w" },
  "pageshow": { recv: "w" },
  "online": { recv: "w" },
  "offline": { recv: "w" },
  "hashchange": { recv: "w" },
  "popstate": { recv: "w" },
  "beforetoggle": { recv: "e" },
  "toggle": { recv: "e" },
  "close": { recv: "e" },
  "cancel": { recv: "e" },
  "transitionend": { recv: "e" },
  "animationend": { recv: "e" },
};

/** The event fields the bridge may read, each one getter. Mirrors `bridge/events.rs`'s `EventField`. */
const FIELD_GETTERS: Record<string, (event: any) => unknown> = {
  "type": (e) => prim(e.type),
  "key": (e) => prim(e.key),
  "code": (e) => prim(e.code),
  "shiftKey": (e) => prim(e.shiftKey),
  "ctrlKey": (e) => prim(e.ctrlKey),
  "altKey": (e) => prim(e.altKey),
  "metaKey": (e) => prim(e.metaKey),
  "repeat": (e) => prim(e.repeat),
  "isComposing": (e) => prim(e.isComposing),
  "defaultPrevented": (e) => prim(e.defaultPrevented),
  "clientX": (e) => prim(e.clientX),
  "clientY": (e) => prim(e.clientY),
  "pointerId": (e) => prim(e.pointerId),
  "pointerType": (e) => prim(e.pointerType),
  "button": (e) => prim(e.button),
  "buttons": (e) => prim(e.buttons),
  "deltaX": (e) => prim(e.deltaX),
  "deltaY": (e) => prim(e.deltaY),
  "deltaMode": (e) => prim(e.deltaMode),
  "timeStamp": (e) => prim(e.timeStamp),
  "inputType": (e) => prim(e.inputType),
  "data": (e) => prim(e.data),
  "newState": (e) => prim(e.newState),
  "oldState": (e) => prim(e.oldState),
  "touchCount": (e) => prim(e.touches?.length),
  "target.id": (e) => prim(e.target?.id),
  "target.tagName": (e) => prim(e.target?.tagName),
  "target.value": (e) => prim(e.target?.value),
  "target.scrollTop": (e) => prim(e.target?.scrollTop),
  "target.scrollLeft": (e) => prim(e.target?.scrollLeft),
  "target.innerWidth": (e) => prim(e.target?.innerWidth),
  "target.innerHeight": (e) => prim(e.target?.innerHeight),
  "currentTarget.id": (e) => prim(e.currentTarget?.id),
};

const isWin = (el: any): boolean => el === W;
const isDoc = (el: any): boolean => !!el && el === W.document;
const isEl = (el: any): boolean => !!el && el.nodeType === 1;

/** Whether `el` is a receiver the letters `recv` allow. */
function receives(recv: Recv | "any", el: any): boolean {
  if (recv === "any") return !!el;
  return (recv.includes("w") && isWin(el)) || (recv.includes("d") && isDoc(el)) || (recv.includes("e") && isEl(el));
}

const clampNum = (value: unknown): number | undefined => (typeof value === "number" && isFinite(value) ? value : undefined);
const oneOf = <T extends string>(value: unknown, allowed: readonly T[]): T | undefined =>
  allowed.includes(value as T) ? (value as T) : undefined;

/** A scroll options object rebuilt from its closed members. */
function scrollOptions(raw: any): any {
  const o = raw && typeof raw === "object" ? raw : {};
  const out: any = {};
  const left = clampNum(o.left);
  const top = clampNum(o.top);
  if (left !== undefined) out.left = left;
  if (top !== undefined) out.top = top;
  const behavior = oneOf(o.behavior, ["auto", "instant", "smooth"] as const);
  if (behavior) out.behavior = behavior;
  const block = oneOf(o.block, ["start", "center", "end", "nearest"] as const);
  if (block) out.block = block;
  const inline = oneOf(o.inline, ["start", "center", "end", "nearest"] as const);
  if (inline) out.inline = inline;
  return out;
}

/** The methods Rust may call, each tolerant of the state it is called in. Mirrors `Method` in `bridge/mod.rs`. */
const METHODS: Record<string, { recv: Recv; run: (el: any, args: any[]) => unknown }> = {
  showPopover: {
    recv: "e",
    run(el) {
      if (typeof el.showPopover === "function" && !el.matches(":popover-open")) el.showPopover();
    },
  },
  hidePopover: {
    recv: "e",
    run(el) {
      if (typeof el.hidePopover === "function" && el.matches(":popover-open")) el.hidePopover();
    },
  },
  togglePopover: { recv: "e", run: (el) => el.togglePopover() },
  showModal: {
    recv: "e",
    run(el) {
      if (!el.open) el.showModal();
    },
  },
  close: {
    recv: "e",
    run(el, args) {
      if (el.open) typeof args[0] === "string" ? el.close(args[0]) : el.close();
    },
  },
  blur: { recv: "e", run: (el) => el.blur() },
  scrollIntoView: { recv: "e", run: (el, args) => el.scrollIntoView(scrollOptions(args[0])) },
  scrollTo: { recv: "we", run: (el, args) => el.scrollTo(scrollOptions(args[0])) },
  scrollBy: { recv: "we", run: (el, args) => el.scrollBy(scrollOptions(args[0])) },
  setSelectionRange: {
    recv: "e",
    run: (el, args) => el.setSelectionRange(Number(args[0]) | 0, Number(args[1]) | 0),
  },
  select: { recv: "e", run: (el) => el.select() },
};

const describe = (el: any): { id: string; tagName: string } | null =>
  el ? { id: el.id ?? "", tagName: el.tagName ?? "" } : null;

/** The reads Rust may ask for. Mirrors `Prop` in `bridge/mod.rs`; `style:`, `attr:` and `matches:` take an argument. */
const PROPS: Record<string, { recv: Recv | "any"; get: (el: any) => unknown }> = {
  "offsetWidth": { recv: "e", get: (el) => el.offsetWidth },
  "offsetHeight": { recv: "e", get: (el) => el.offsetHeight },
  "offsetTop": { recv: "e", get: (el) => el.offsetTop },
  "offsetLeft": { recv: "e", get: (el) => el.offsetLeft },
  "clientWidth": { recv: "e", get: (el) => el.clientWidth },
  "clientHeight": { recv: "e", get: (el) => el.clientHeight },
  "clientTop": { recv: "e", get: (el) => el.clientTop },
  "clientLeft": { recv: "e", get: (el) => el.clientLeft },
  "scrollWidth": { recv: "e", get: (el) => el.scrollWidth },
  "scrollHeight": { recv: "e", get: (el) => el.scrollHeight },
  "scrollTop": { recv: "e", get: (el) => el.scrollTop },
  "scrollLeft": { recv: "e", get: (el) => el.scrollLeft },
  "innerWidth": { recv: "w", get: (el) => el.innerWidth },
  "innerHeight": { recv: "w", get: (el) => el.innerHeight },
  "scrollX": { recv: "w", get: (el) => el.scrollX },
  "scrollY": { recv: "w", get: (el) => el.scrollY },
  "devicePixelRatio": { recv: "w", get: (el) => el.devicePixelRatio },
  "selectionStart": { recv: "e", get: (el) => el.selectionStart },
  "selectionEnd": { recv: "e", get: (el) => el.selectionEnd },
  "value": { recv: "e", get: (el) => el.value },
  "checked": { recv: "e", get: (el) => el.checked },
  "open": { recv: "e", get: (el) => !!el.open },
  "popoverOpen": { recv: "e", get: (el) => !!el.matches && el.matches(":popover-open") },
  "rect": {
    recv: "e",
    get(el) {
      if (!el.getBoundingClientRect) return null;
      const r = el.getBoundingClientRect();
      return { x: r.x, y: r.y, width: r.width, height: r.height, top: r.top, left: r.left, right: r.right, bottom: r.bottom };
    },
  },
  "activeElement": {
    recv: "any",
    get(el) {
      const doc = isDoc(el) ? el : (el.ownerDocument ?? W.document);
      const active = doc.activeElement;
      const out = describe(active) as Fields | null;
      if (out) out.within = isEl(el) && !!el.contains && el.contains(active);
      return out;
    },
  },
  "visualViewport": {
    recv: "any",
    get() {
      const v = W.visualViewport;
      return v
        ? {
            width: v.width,
            height: v.height,
            offsetTop: v.offsetTop,
            offsetLeft: v.offsetLeft,
            pageTop: v.pageTop,
            pageLeft: v.pageLeft,
            scale: v.scale,
          }
        : null;
    },
  },
};

// ---------------------------------------------------------------- the bridge-latency harness

/** The frames a message is deferred by: the one seam where another test mode (desktop cadence) plugs in. */
function channelDelay(): number | "never" {
  const mode = W.__dxBridgeDelayFrames;
  if (mode === "never") return "never";
  return typeof mode === "number" ? mode | 0 : 0;
}

function afterFrames(n: number, fn: () => void): void {
  if (n <= 0) fn();
  else W.requestAnimationFrame(() => afterFrames(n - 1, fn));
}

/** `raw` with the harness applied to `send` and `recv`. Wrapping a wrapped channel returns it. */
function channel(raw: DioxusChannel): DioxusChannel {
  if ((raw as any).__dxWrapped) return raw;
  return {
    __dxWrapped: true,
    send(message) {
      const n = channelDelay();
      if (n === "never") return;
      if (n) afterFrames(n, () => raw.send(message));
      else raw.send(message);
    },
    recv() {
      return raw.recv().then((message) => {
        const n = channelDelay();
        return typeof n === "number" && n ? new Promise((resolve) => afterFrames(n, () => resolve(message))) : message;
      });
    },
  } as DioxusChannel;
}

// ---------------------------------------------------------------- targets

function resolve(target: DxTargetish): any {
  if (target == null) return null;
  if (typeof target === "string") return W.document.getElementById(target);
  if (typeof (target as EventTarget).addEventListener === "function") return target;
  const spec = target as DxTarget;
  switch (spec.kind) {
    case "window":
      return W;
    case "document":
      return W.document;
    case "id":
      return W.document.getElementById(spec.id);
    case "selector":
      return W.document.querySelector(spec.selector);
    case "active":
      return W.document.activeElement;
  }
  return null;
}

function pickAll(source: any, fields: readonly string[]): Fields {
  const out: Fields = {};
  for (const field of fields) {
    const getter = own(FIELD_GETTERS, field);
    out[field] = getter ? getter(source) : null;
  }
  return out;
}

function checkFields(fields: readonly string[] | undefined): readonly string[] {
  for (const field of fields ?? []) if (!own(FIELD_GETTERS, field)) throw new Error("bridge: no such event field: " + field);
  return fields ?? [];
}

function checkEvent(event: unknown, el: any): { recv: Recv; hot?: 1 } {
  const row = own(EVENTS, event);
  if (!row) throw new Error("bridge: no such event: " + String(event));
  if (!receives(row.recv, el)) throw new Error("bridge: " + String(event) + " cannot be listened on this target");
  return row;
}

// ---------------------------------------------------------------- epochs

const seenEpochs = new WeakMap<object, Map<string, number>>();

/** True when an epoch newer than `epoch` has already been seen on `el` for its key; otherwise records it. */
function stale(el: object | null, epoch: DxEpoch | null | undefined): boolean {
  if (!epoch || !el) return false;
  let seen = seenEpochs.get(el);
  if (!seen) seenEpochs.set(el, (seen = new Map()));
  const last = seen.get(epoch.key);
  if (last !== undefined && epoch.n < last) return true;
  seen.set(epoch.key, epoch.n);
  return false;
}

// ---------------------------------------------------------------- call, get, set

function call(target: DxTargetish, method: string, args: unknown[] = [], epoch?: DxEpoch | null): unknown {
  const row = own(METHODS, method);
  if (!row) throw new Error("bridge.call: no such method: " + method);
  const el = resolve(target);
  if (!el) return null;
  if (!receives(row.recv, el)) throw new Error("bridge.call: " + method + " needs an element");
  if (stale(el, epoch)) return STALE;
  return norm(row.run(el, args));
}

/** The sentinel a dropped (stale) op returns. */
const STALE = Symbol("stale");

function read(el: any, prop: string): unknown {
  if (!el) return null;
  if (prop.startsWith("style:")) return isEl(el) ? W.getComputedStyle(el).getPropertyValue(prop.slice(6)) : null;
  if (prop.startsWith("attr:")) return isEl(el) ? el.getAttribute(prop.slice(5)) : null;
  if (prop.startsWith("matches:")) return isEl(el) && !!el.matches && el.matches(prop.slice(8));
  const row = own(PROPS, prop);
  if (!row || !receives(row.recv, el)) return null;
  return norm(row.get(el));
}

function get(target: DxTargetish, props: string[]): Fields {
  const el = resolve(target);
  const out: Fields = {};
  for (const prop of props) out[prop] = read(el, prop);
  return out;
}

/** Writes (or, with `null`, removes) the CSS custom property `name` on an element whose style Rust does not own. */
function set(target: DxTargetish, name: string, value: string | null, epoch?: DxEpoch | null): unknown {
  if (!name.startsWith("--dx-")) throw new Error("bridge.set: only the theme's custom properties (--dx-name): " + name);
  const el = resolve(target);
  if (!isEl(el) || !el.style) return null;
  if (stale(el, epoch)) return STALE;
  if (value === null) el.style.removeProperty(name);
  else el.style.setProperty(name, value);
  return null;
}

// ---------------------------------------------------------------- transaction

/** Runs `ops` in order, all in the caller's JS turn (so a show and the read of its rects see one DOM state). */
function transaction(ops: DxOp[]): DxOpResult[] {
  const out: DxOpResult[] = [];
  for (const op of ops) {
    try {
      let result: DxOpResult;
      switch (op.op) {
        case "call": {
          const value = call(op.target, op.method, op.args, op.epoch);
          result = value === STALE ? { ok: null, dropped: true } : { ok: value };
          break;
        }
        case "get": {
          const el = resolve(op.target);
          result = stale(el, op.epoch) ? { ok: null, dropped: true } : { ok: get(el, op.props) };
          break;
        }
        case "set": {
          const value = set(op.target, op.name, op.value, op.epoch);
          result = value === STALE ? { ok: null, dropped: true } : { ok: null };
          break;
        }
        default:
          throw new Error("no such op: " + String((op as { op?: unknown }).op));
      }
      const epoch = "epoch" in op ? op.epoch : null;
      if (epoch && "ok" in result) result.epoch = epoch;
      out.push(result);
    } catch (error) {
      out.push({ err: String(error) });
    }
  }
  return out;
}

// ---------------------------------------------------------------- the listener registry

/** Registrations on an element, ended by `sweep` when the element leaves the document. */
const elementRegs = new Set<{ el: any; off: () => void }>();
let pageObserver: { disconnect(): void } | null = null;

function sweep(): void {
  for (const reg of [...elementRegs]) if (!reg.el.isConnected) reg.off();
}

/**
 * Ties `off` to the element's life: one page-wide `MutationObserver`, created with the first such
 * registration and disconnected with the last, ends every registration whose element left the
 * document. An element that is not in the document yet is not tracked. Returns the untracking function.
 */
function watchElement(el: any, off: () => void): () => void {
  if (!isEl(el) || !el.isConnected) return () => {};
  const reg = { el, off };
  elementRegs.add(reg);
  if (!pageObserver) {
    const mo = new W.MutationObserver(sweep);
    mo.observe(W.document.documentElement, { childList: true, subtree: true });
    pageObserver = mo;
  }
  return () => {
    elementRegs.delete(reg);
    if (!elementRegs.size && pageObserver) {
      pageObserver.disconnect();
      pageObserver = null;
    }
  };
}

type OnOptions = { capture?: boolean; passive?: boolean; once?: boolean };

/**
 * Registers `handler` on `target` and returns its `off`. The one place a listener is added: the
 * registration is recorded in `window.__dxJsListeners.live` while it is live, the scroll-blocking
 * events default to passive, and a `{ once: true }` registration forgets itself when it fires.
 * `ended` runs once when the registration ends, however it ends.
 */
function bareOn(
  target: any,
  type: string,
  handler: (event: any) => void,
  options?: boolean | OnOptions,
  track?: (off: () => void) => void,
  ended?: () => void,
): () => void {
  const el = resolve(target);
  if (!el) return () => {};
  const o: OnOptions = typeof options === "boolean" ? { capture: options } : Object.assign({}, options);
  if (o.passive === undefined && BLOCKING[type]) o.passive = true;
  const reg = (W.__dxJsListeners = W.__dxJsListeners || { n: 0, live: new Map() });
  const id = ++reg.n;
  let live = true;
  let unwatch = () => {};
  const off = () => {
    if (!live) return;
    live = false;
    el.removeEventListener(type, h, o);
    reg.live.delete(id);
    unwatch();
    ended?.();
  };
  const h = o.once
    ? (event: any) => {
        off();
        return handler(event);
      }
    : handler;
  // listener-ok: the bridge's own registration -- the removal is the returned `off`, owned by the scope
  el.addEventListener(type, h, o);
  reg.live.set(id, type);
  unwatch = watchElement(el, off);
  track?.(off);
  return off;
}

// ---------------------------------------------------------------- the listener hub

type HubEntry = { handlers: Set<(event: any) => void>; off: () => void };
const hubs = new WeakMap<object, Map<string, HubEntry>>();

/**
 * Adds `handler` to the ONE DOM listener for (`el`, `type`, `capture`, `passive`), creating it with the
 * first handler and removing it with the last: a non-passive listener lives only while a behaviour
 * needs it. Returns the removal of this handler.
 */
function hubAdd(el: any, type: string, capture: boolean, passive: boolean, handler: (event: any) => void): () => void {
  let map = hubs.get(el);
  if (!map) hubs.set(el, (map = new Map()));
  const key = type + "|" + (capture ? 1 : 0) + "|" + (passive ? 1 : 0);
  let entry = map.get(key);
  if (!entry) {
    const handlers = new Set<(event: any) => void>();
    const made: HubEntry = { handlers, off: () => {} };
    const slots = map;
    made.off = bareOn(
      el,
      type,
      (event) => {
        for (const h of [...handlers]) h(event);
      },
      { capture, passive },
      undefined,
      () => {
        if (slots.get(key) === made) slots.delete(key);
        handlers.clear();
      },
    );
    map.set(key, made);
    entry = made;
  }
  const mine = entry;
  mine.handlers.add(handler);
  return () => {
    mine.handlers.delete(handler);
    if (!mine.handlers.size) mine.off();
  };
}

// ---------------------------------------------------------------- behaviours

/** The behaviour modules loaded in this page, by kind (a null-prototype table: own keys only). */
const behaviours: Record<string, DxBehaviourImpl<any>> = Object.create(null);

function register(kind: string, impl: DxBehaviourImpl<any>): void {
  if (typeof kind !== "string" || !impl || typeof impl.decide !== "function") throw new Error("bridge.register: bad behaviour");
  checkFields(impl.fields);
  behaviours[kind] = impl;
}

type Installed = { n: number; off: () => void; renew: () => void };
const installed = new WeakMap<object, Map<string, Installed>>();
const warned = new Set<string>();
const warnOnce = (key: string, message: string) => {
  if (warned.has(key)) return;
  warned.add(key);
  console.warn("dx bridge:", message);
};

/** What a decision may set: `data-*` and `aria-*` only. */
const ATTR_OK = /^(data|aria)-[a-z0-9-]+$/;

function installBehaviour(sc: Scope, spec: Omit<BehaviourSpec, "cmd">): () => void {
  const el = resolve(spec.target);
  const b = spec.behaviour;
  const d = spec.declared;
  if (!el) return () => {};
  const impl = own(behaviours, b.kind);
  if (!impl) throw new Error("bridge.behaviour: not loaded: " + String(b.kind));
  const want = d.target === "window" ? "w" : d.target === "document" ? "d" : "e";
  if (!receives(want, el)) throw new Error("bridge.behaviour: " + b.kind + " declared " + d.target + " but the target is not");
  checkEvent(d.event, el);
  // a capture listener runs before anything has decided, so `defaultPrevented` there is always false
  if (d.capture && impl.fields.includes("defaultPrevented")) {
    throw new Error("bridge.behaviour: " + b.kind + " reads defaultPrevented, which only a bubble-phase behaviour may");
  }
  const key = spec.epoch ? spec.epoch.key : "behaviour:" + b.kind;
  let slots = installed.get(el);
  if (!slots) installed.set(el, (slots = new Map()));
  const current = slots.get(key);
  const n = spec.epoch ? spec.epoch.n : current ? current.n + 1 : 0;
  if (spec.epoch && stale(el, spec.epoch)) return () => {};
  if (current) {
    if (current.n === n && spec.epoch) {
      current.renew();
      return current.off;
    }
    current.off();
  }
  const host = isEl(el) ? el : null;
  // per-install state a behaviour carries between events (a gesture's origin, a counter): it lives and
  // dies with the install, so a re-install, a lease expiry and an element disconnect all start clean
  let memory: Record<string, unknown> = {};
  const handler = (event: any) => {
    const fields = pickAll(event, impl.fields);
    const state: Record<string, string | null> = {};
    for (const name of impl.reads ?? []) state[name] = host ? host.getAttribute(name) : null;
    let decision: DxDecision;
    try {
      decision = impl.decide(b, fields, state, memory) ?? {};
    } catch (error) {
      // a behaviour that throws does nothing: the event goes on as if it were not installed
      warnOnce("throw:" + b.kind, b.kind + " threw: " + String(error));
      return;
    }
    if (decision.prevent || decision.stop) {
      if (d.passive) warnOnce("passive:" + b.kind, b.kind + " asked to cancel an event Rust declared passive");
      else {
        if (decision.prevent) event.preventDefault();
        // desktop never forwards stopPropagation and a window-level stop would starve Rust's own handlers:
        // only an element-level capture behaviour may stop
        if (decision.stop) {
          if (d.capture && d.target === "element") event.stopPropagation();
          else warnOnce("stop:" + b.kind, b.kind + " may stop propagation only at an element in the capture phase");
        }
      }
    }
    if (decision.remember) memory = { ...memory, ...decision.remember };
    if (decision.attrs && host) {
      for (const name of Object.keys(decision.attrs)) {
        if (ATTR_OK.test(name)) host.setAttribute(name, decision.attrs[name]);
        else warnOnce("attr:" + b.kind + name, b.kind + " may not set " + name);
      }
    }
    if (decision.send !== undefined) sc.send(decision.send);
  };
  const removeHandler = hubAdd(el, d.event, !!d.capture, !!d.passive, handler);
  let timer = 0;
  let unwatch = () => {};
  const entry: Installed = {
    n,
    off() {
      if (timer) W.clearTimeout(timer);
      timer = 0;
      unwatch();
      memory = {};
      removeHandler();
      if (slots.get(key) === entry) slots.delete(key);
    },
    renew() {
      if (!d.leaseMs) return;
      if (timer) W.clearTimeout(timer);
      timer = W.setTimeout(entry.off, d.leaseMs);
    },
  };
  slots.set(key, entry);
  // a behaviour tears down when its element leaves the document (window and document targets never do)
  if (host) unwatch = watchElement(host, entry.off);
  entry.renew();
  sc.cleanup(entry.off);
  return entry.off;
}

// ---------------------------------------------------------------- scope

function scope(raw?: DioxusChannel): Scope {
  const offs: (() => void)[] = [];
  let done = false;
  const ch = raw ? channel(raw) : null;
  const send = (message: unknown) => {
    if (ch && !done) ch.send(message);
  };
  const cleanup = (fn: () => void) => {
    if (done) fn();
    else offs.push(fn);
  };
  const on: Scope["on"] = (target, type, handler, options) => {
    if (done) return () => {};
    return bareOn(target, type, handler, options, (off) => offs.push(off));
  };
  const s: Scope = {
    channel: ch as DioxusChannel,
    on,
    cleanup,
    send,
    dispose() {
      done = true;
      for (let i = offs.length - 1; i >= 0; i--) {
        try {
          offs[i]();
        } catch {
          // one failing teardown must not keep the others from running
        }
      }
      offs.length = 0;
    },
    listen(spec) {
      const el = resolve(spec.target);
      if (!el || done) return () => {};
      const row = checkEvent(spec.event, el);
      const fields = checkFields(spec.fields);
      let pending: Fields | null = null;
      let frame = 0;
      const flushPending = () => {
        frame = 0;
        const message = pending;
        pending = null;
        if (message) send(message);
      };
      // a streaming listener cannot cancel an event: always passive
      const off = on(
        el,
        spec.event,
        (event) => {
          const message = pickAll(event, fields);
          if (!row.hot) return send(message);
          pending = message;
          if (!frame) frame = W.requestAnimationFrame(flushPending);
        },
        { capture: !!spec.options?.capture, passive: true, once: !!spec.options?.once },
      );
      if (row.hot) offs.push(() => frame && W.cancelAnimationFrame(frame));
      if (spec.initial) send(pickAll({ type: "initial", target: el, currentTarget: el }, fields));
      return off;
    },
    observe(spec) {
      const el = resolve(spec.target);
      if (!isEl(el) || done) return () => {};
      let observer: { disconnect(): void };
      switch (spec.kind) {
        case "resize": {
          const ro = new W.ResizeObserver((entries: ResizeObserverEntry[]) => {
            const e = entries[entries.length - 1];
            const box = e.borderBoxSize?.[0];
            send({
              width: e.contentRect.width,
              height: e.contentRect.height,
              borderWidth: box ? box.inlineSize : e.contentRect.width,
              borderHeight: box ? box.blockSize : e.contentRect.height,
            });
          });
          const boxOption = oneOf(spec.options?.box, ["content-box", "border-box", "device-pixel-content-box"] as const);
          ro.observe(el, boxOption ? { box: boxOption } : undefined);
          observer = ro;
          break;
        }
        case "intersection": {
          const o = (spec.options ?? {}) as { root?: DxTargetish; rootMargin?: string; threshold?: number | number[] };
          const io = new W.IntersectionObserver(
            (entries: IntersectionObserverEntry[]) => {
              const e = entries[entries.length - 1];
              const r = e.boundingClientRect;
              send({
                isIntersecting: e.isIntersecting,
                ratio: e.intersectionRatio,
                top: r.top,
                left: r.left,
                width: r.width,
                height: r.height,
              });
            },
            { root: o.root == null ? null : resolve(o.root), rootMargin: o.rootMargin, threshold: o.threshold },
          );
          io.observe(el);
          observer = io;
          break;
        }
        case "mutation": {
          const mo = new W.MutationObserver((records: MutationRecord[]) => {
            const attributes: string[] = [];
            let added = 0;
            let removed = 0;
            for (const r of records) {
              if (r.attributeName && !attributes.includes(r.attributeName)) attributes.push(r.attributeName);
              added += r.addedNodes.length;
              removed += r.removedNodes.length;
            }
            send({ count: records.length, attributes, added, removed });
          });
          const o = (spec.options ?? {}) as MutationObserverInit;
          mo.observe(el, {
            attributes: o.attributes,
            attributeFilter: o.attributeFilter,
            childList: o.childList,
            subtree: o.subtree,
            characterData: o.characterData,
            ...(Object.keys(o).length ? {} : { attributes: true, childList: true, subtree: true }),
          });
          observer = mo;
          break;
        }
        default:
          throw new Error("bridge.observe: no such observer: " + String((spec as { kind: unknown }).kind));
      }
      const off = () => observer.disconnect();
      offs.push(off);
      return off;
    },
    media(query) {
      if (done) return () => {};
      const mql = W.matchMedia(query);
      send({ matches: mql.matches });
      return on(mql, "change", (event) => send({ matches: event.matches }), { passive: true });
    },
    sample(spec) {
      const el = resolve(spec.target);
      if (!el || done) return () => {};
      const started = W.performance.now();
      const onlyChanged = spec.onlyChanged !== false;
      const reads: string[] = [];
      type Batch = { name: string; entries: Fields[]; task: boolean; queued: boolean };
      const batches: Batch[] = [];
      const stops: (() => void)[] = [];
      let live = true;
      for (const f of spec.fields) {
        if (typeof f === "string") reads.push(f);
        else if (f.mode === "batched") {
          checkEvent(f.event, el);
          const pickFields = checkFields(f.pick);
          const batch: Batch = { name: f.name, entries: [], task: f.flush === "task", queued: false };
          batches.push(batch);
          stops.push(
            on(
              el,
              f.event,
              (event) => {
                batch.entries.push({ timeStamp: event.timeStamp, ...pickAll(event, pickFields) });
                if (!batch.task || batch.queued) return;
                // flush per event task: the entries of this task go out in a microtask, not at the frame's end
                batch.queued = true;
                W.queueMicrotask(() => {
                  batch.queued = false;
                  if (!live || !batch.entries.length) return;
                  const message: Fields = {};
                  message[batch.name] = batch.entries.splice(0, batch.entries.length);
                  send(message);
                });
              },
              { capture: !!f.capture, passive: true },
            ),
          );
        } else reads.push(f.name);
      }
      const framed = batches.filter((b) => !b.task);
      let frames = 0;
      // a sample with no coalesced field has nothing to report until an event arrives
      let last = reads.length ? "" : "{}";
      let frame = 0;
      const holds = (): boolean => {
        const until = spec.until;
        switch (until.kind) {
          case "forever":
            return true;
          case "frames":
            return frames < until.n;
          case "ms":
            return W.performance.now() - started < until.ms;
          case "attr":
            return until.value === undefined
              ? el.hasAttribute(until.name)
              : el.getAttribute(until.name) === until.value;
        }
        return false;
      };
      const end = () => {
        live = false;
        if (frame) W.cancelAnimationFrame(frame);
        for (const stop of stops) stop();
      };
      const tick = () => {
        frame = 0;
        if (!live) return;
        const going = holds();
        const sample: Fields = {};
        for (const field of reads) sample[field] = read(el, field);
        const text = JSON.stringify(sample);
        const events = framed.some((b) => b.entries.length > 0);
        // every frame-flushed batched field is always present in a message (an empty array when it saw nothing)
        for (const b of framed) sample[b.name] = b.entries.splice(0, b.entries.length);
        if (!onlyChanged || text !== last || events || !going) {
          last = text;
          send(going ? sample : Object.assign(sample, { $done: true }));
        }
        frames += 1;
        if (going) frame = W.requestAnimationFrame(tick);
        else end();
      };
      frame = W.requestAnimationFrame(tick);
      offs.push(end);
      return end;
    },
    behaviour(spec) {
      return done ? () => {} : installBehaviour(s, spec);
    },
    run(command) {
      try {
        switch (command.cmd) {
          case "listen":
            s.listen(command);
            break;
          case "observe":
            s.observe(command);
            break;
          case "media":
            s.media(command.query);
            break;
          case "sample":
            s.sample(command);
            break;
          case "behaviour":
            s.behaviour(command);
            break;
        }
      } catch (error) {
        // a bad command is Rust's bug: say so in the console, keep serving the rest
        console.warn("dx bridge:", String(error));
      }
    },
  };
  return s;
}

/**
 * The loop of a Rust `bridge::Session`: the first message is the array of setup commands (later
 * arrays add more or renew), the `TEARDOWN` string ends it, and everything registered is undone either way.
 */
async function serve(raw: DioxusChannel): Promise<void> {
  const s = scope(raw);
  try {
    for (;;) {
      const message = await s.channel.recv();
      if (message === TEARDOWN) break;
      if (Array.isArray(message)) for (const command of message) s.run(command as DxCommand);
    }
  } finally {
    s.dispose();
  }
}

/**
 * The ONE ordered channel of a webview, for every write and every read Rust makes of an element: a message
 * is `{ id, ops }`, its ops run in the order received (one message after the other, each in one JS turn),
 * and a message with an `id` is answered with `{ id, results }` on the same channel. A message without an
 * `id` is fire-and-forget: its failed ops are logged. One channel means a write is ordered before the
 * read that follows it, which separate evals do not guarantee on desktop.
 */
async function pump(raw: DioxusChannel): Promise<void> {
  const ch = channel(raw);
  for (;;) {
    const message = (await ch.recv()) as DxPumpMessage | null;
    if (!message || !Array.isArray(message.ops)) continue;
    const results = transaction(message.ops);
    if (typeof message.id === "number") ch.send({ id: message.id, results });
    else for (const result of results) if ("err" in result) console.warn("dx bridge:", result.err);
  }
}

/** `call` for a bespoke script: a dropped (stale) call answers null. */
const callPublic: DxBridge["call"] = (target, method, args) => {
  const value = call(target, method, args);
  return value === STALE ? null : value;
};

if (!W.__dxBridge) {
  const bridge: DxBridge = {
    version: 3,
    channel,
    scope,
    serve,
    pump,
    call: callPublic,
    get,
    set: (target, name, value) => void set(target, name, value),
    transaction,
    behaviours,
    register,
  };
  W.__dxBridge = bridge;
}

export {};
