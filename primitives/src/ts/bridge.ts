// The generic JS bridge (row 434 step 3): the one small typed glue layer between Rust and the browser
// APIs Dioxus does not wrap. Generic verbs only, no component logic:
//
//   scope(channel)   owns every listener, observer and rule it creates; `dispose()` undoes all of them
//   listen / on      a listener on window, document or an element; `listen` streams named event fields to
//                    Rust, `on` runs a JS handler (the shape of `js_listener.rs`'s `listen`)
//   call             showPopover, hidePopover, showModal, close, setPointerCapture, focus({preventScroll}),
//                    scrollIntoView, setSelectionRange ...: a closed list, never `el[name](...)`
//   get              computed style, offset*, client*, scroll*, rect, activeElement, visualViewport
//   observe          Resize, Intersection (rootMargin, threshold) and Mutation observers
//   matchMedia       a subscription to a media query
//   transaction      several ops in one JS turn
//   commit           resolve after a frame plus a forced style flush
//   sample           per-frame sampling while a condition holds; each field is coalesced (the last value of
//                    the frame) or batched (every event of the frame with its own timestamp)
//   set              writes a CSS custom property on an element whose style Rust does not own
//   rules            install/remove/sync of the closed `Rule` set (see `rules/index.ts`)
//   serve            the loop a Rust `bridge::Session` runs: setup commands in, events out
//
// The file is a script, not a module: `primitives/src/bridge/mod.rs` evaluates the bundled
// `js/bridge.js` once per page and every verb is then reached as `window.__dxBridge`. All browser
// globals are read through `window`, so a test can run the shipped file against a fake one.
//
// SLOW-CHANNEL TEST MODE: `window.__dxBridgeDelayFrames = N` defers every message through a bridge
// `channel` by N animation frames, in both directions, to model a desktop webview (an IPC hop per
// message) on a browser where the eval channel costs a microtask. Unset it costs one property read.
// `"never"` drops every message to Rust (the answer that never arrives, backlog row 434: a pending
// gate must still end visible); commands from Rust still arrive. A read that does not go through
// `get` (Dioxus's `MountedData` on web is synchronous) is not delayed.
//
// LIFETIMES: a registration on window or document lives until its scope ends (Rust sends the teardown
// over the scope's own channel, never a new eval). A registration on an ELEMENT also ends when the
// element leaves the document, without waiting for Rust (Dioxus has no merged `onunmounted`).
//
// A bridge message reaches Rust after the event's default action is committed and after the frame is
// painted, so this layer carries only work that may be late. A decision inside the listener call
// belongs to a rule (`rules/index.ts`), which Rust publishes beforehand.

/// <reference path="./types/dioxus.d.ts" />
import { applyRule, readRuleAttributes, RULE_ATTR_PREFIX } from "./rules/index";
import type { Rule, RuleScope } from "./rules/types";

type ListenOptions = { capture?: boolean; passive?: boolean; once?: boolean };
type TargetSpec =
  | { kind: "window" }
  | { kind: "document" }
  | { kind: "id"; id: string }
  | { kind: "selector"; selector: string }
  | { kind: "active" };
type Targetish = TargetSpec | EventTarget | string | null | undefined;

type ListenSpec = {
  target: Targetish;
  event: string;
  options?: ListenOptions;
  fields?: string[];
  /** Also send once at registration, as if an event `{ type: "initial", target, currentTarget }` fired. */
  initial?: boolean;
};
type ObserveSpec =
  | { target: Targetish; kind: "resize"; options?: { box?: ResizeObserverBoxOptions } }
  | {
      target: Targetish;
      kind: "intersection";
      options?: { root?: Targetish; rootMargin?: string; threshold?: number | number[] };
    }
  | {
      target: Targetish;
      kind: "mutation";
      options?: {
        attributes?: boolean;
        attributeFilter?: string[];
        childList?: boolean;
        subtree?: boolean;
        characterData?: boolean;
      };
    };
type SampleUntil =
  | { kind: "forever" }
  | { kind: "frames"; n: number }
  | { kind: "ms"; ms: number }
  | { kind: "attr"; name: string; value?: string };
/**
 * A sampled field. A string, or `{ name, mode: "coalesced" }` with `name` the property read by `get`, is
 * coalesced: the value at the frame's end, sent only when it changed. `{ name, mode: "batched", event, pick }`
 * is an event stream: every `event` the target gets between two frames, as `{ timeStamp, ...pick }`, in order,
 * delivered as the array `name` of that frame's one message.
 */
type SampleField =
  | string
  | { name: string; mode?: "coalesced" }
  | { name: string; mode: "batched"; event: string; pick?: string[]; capture?: boolean };
type SampleSpec = { target: Targetish; fields: SampleField[]; until: SampleUntil; onlyChanged?: boolean };

/** A command a Rust session sends to `serve`. */
type Command =
  | ({ cmd: "listen" } & ListenSpec)
  | ({ cmd: "observe" } & ObserveSpec)
  | { cmd: "media"; query: string }
  | ({ cmd: "sample" } & SampleSpec)
  | { cmd: "rule"; target: Targetish; rule: Rule; key?: string }
  | { cmd: "sync-rules"; target: Targetish };

/** An op of a transaction. */
type Op =
  | { op: "call"; target: Targetish; method: string; args?: unknown[] }
  | { op: "get"; target: Targetish; props: string[] }
  | { op: "set"; target: Targetish; name: string; value: string | null }
  | { op: "commit" };
type OpResult = { ok: unknown } | { err: string };

interface Scope extends RuleScope {
  /** The channel the scope's messages go through (the slow-channel mode applies to it). */
  readonly channel: DioxusChannel;
  /** Ends the scope: every registration it made is undone, in reverse order. Idempotent. */
  dispose(): void;
  listen(spec: ListenSpec): () => void;
  observe(spec: ObserveSpec): () => void;
  media(query: string): () => void;
  sample(spec: SampleSpec): () => void;
  /** Runs one session command. */
  run(command: Command): void;
}

interface Bridge {
  version: number;
  channel(raw: DioxusChannel): DioxusChannel;
  scope(raw?: DioxusChannel): Scope;
  serve(raw: DioxusChannel): Promise<void>;
  pump(raw: DioxusChannel): Promise<void>;
  call(target: Targetish, method: string, args?: unknown[]): unknown;
  get(target: Targetish, props: string[]): Record<string, unknown>;
  set(target: Targetish, name: string, value: string | null): void;
  commit(): Promise<void>;
  transaction(ops: Op[]): Promise<OpResult[]>;
  rules: {
    install(target: Targetish, rule: Rule, scope?: RuleScope, key?: string): () => void;
    remove(target: Targetish, key?: string): void;
    read(target: Targetish): [string, Rule][];
    sync(target: Targetish, scope?: RuleScope): void;
  };
}

declare global {
  interface Window {
    __dxBridge?: Bridge;
    /** Test only: defer every bridge message by this many animation frames, in both directions. */
    __dxBridgeDelayFrames?: number | "never";
    /** `window.__dxJsListeners.live` maps a registration id to its event type while it is live. */
    __dxJsListeners?: { n: number; live: Map<number, string> };
  }
}

const W = window as any;

/** The message that ends a session or script (`js_listener.rs`'s `TEARDOWN`). */
const TEARDOWN = "__dx_listeners_off__";
/** The events whose non-passive listeners block scrolling: `passive: true` unless the caller says so. */
const BLOCKING: Record<string, 1> = { wheel: 1, mousewheel: 1, touchstart: 1, touchmove: 1, DOMMouseScroll: 1 };

// ---------------------------------------------------------------- the slow-channel mode

const delayFrames = (): number => {
  const mode = W.__dxBridgeDelayFrames;
  return typeof mode === "number" ? mode | 0 : 0;
};

function afterFrames(n: number, fn: () => void): void {
  if (n <= 0) fn();
  else W.requestAnimationFrame(() => afterFrames(n - 1, fn));
}

/** `raw` with the slow-channel mode applied to `send` and `recv`. Wrapping a wrapped channel returns it. */
function channel(raw: DioxusChannel): DioxusChannel {
  if ((raw as any).__dxWrapped) return raw;
  return {
    __dxWrapped: true,
    send(message) {
      if (W.__dxBridgeDelayFrames === "never") return;
      const n = delayFrames();
      if (n) afterFrames(n, () => raw.send(message));
      else raw.send(message);
    },
    recv() {
      return raw.recv().then((message) => {
        const n = delayFrames();
        return n ? new Promise((resolve) => afterFrames(n, () => resolve(message))) : message;
      });
    },
  } as DioxusChannel;
}

// ---------------------------------------------------------------- targets

function resolve(target: Targetish): any {
  if (target == null) return null;
  if (typeof target === "string") return W.document.getElementById(target);
  if (typeof (target as EventTarget).addEventListener === "function") return target;
  switch ((target as TargetSpec).kind) {
    case "window":
      return W;
    case "document":
      return W.document;
    case "id":
      return W.document.getElementById((target as { id: string }).id);
    case "selector":
      return W.document.querySelector((target as { selector: string }).selector);
    case "active":
      return W.document.activeElement;
  }
  return null;
}

/** The value at a dotted `path` of `source`, if it is JSON-shaped; an element or a function is `null`. */
function pick(source: any, path: string): unknown {
  let value = source;
  for (const part of path.split(".")) {
    if (value == null) return null;
    value = value[part];
  }
  if (value === undefined) return null;
  if (value === null) return null;
  const type = typeof value;
  return type === "string" || type === "number" || type === "boolean" ? value : null;
}

function pickAll(source: any, fields: string[]): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const field of fields) out[field] = pick(source, field);
  return out;
}

// ---------------------------------------------------------------- call

const norm = (value: unknown): unknown => (value === undefined ? null : value);

/** The methods Rust may call, each tolerant of the state it is called in (a popover that is already open is not an error). */
const METHODS: Record<string, (el: any, args: any[]) => unknown> = {
  showPopover(el) {
    if (typeof el.showPopover === "function" && !el.matches(":popover-open")) el.showPopover();
  },
  hidePopover(el) {
    if (typeof el.hidePopover === "function" && el.matches(":popover-open")) el.hidePopover();
  },
  togglePopover: (el) => el.togglePopover(),
  showModal(el) {
    if (!el.open) el.showModal();
  },
  close(el, args) {
    if (el.open) el.close(...args);
  },
  setPointerCapture: (el, args) => el.setPointerCapture(args[0]),
  releasePointerCapture(el, args) {
    if (el.hasPointerCapture?.(args[0])) el.releasePointerCapture(args[0]);
  },
  focus: (el, args) => el.focus(...args),
  blur: (el) => el.blur(),
  scrollIntoView: (el, args) => el.scrollIntoView(...args),
  scrollTo: (el, args) => el.scrollTo(...args),
  scrollBy: (el, args) => el.scrollBy(...args),
  setSelectionRange: (el, args) => el.setSelectionRange(...args),
  select: (el) => el.select(),
};

function call(target: Targetish, method: string, args: unknown[] = []): unknown {
  const fn = Object.prototype.hasOwnProperty.call(METHODS, method) ? METHODS[method] : undefined;
  if (!fn) throw new Error("bridge.call: no such method: " + method);
  const el = resolve(target);
  return el ? norm(fn(el, args)) : null;
}

// ---------------------------------------------------------------- get

const NUMERIC = new Set([
  "offsetWidth",
  "offsetHeight",
  "offsetTop",
  "offsetLeft",
  "clientWidth",
  "clientHeight",
  "clientTop",
  "clientLeft",
  "scrollWidth",
  "scrollHeight",
  "scrollTop",
  "scrollLeft",
  "innerWidth",
  "innerHeight",
  "scrollX",
  "scrollY",
  "devicePixelRatio",
  "selectionStart",
  "selectionEnd",
]);

const describe = (el: any): { id: string; tagName: string } | null =>
  el ? { id: el.id ?? "", tagName: el.tagName ?? "" } : null;

function read(el: any, prop: string): unknown {
  if (prop.startsWith("style:")) {
    return el && el.nodeType === 1 ? W.getComputedStyle(el).getPropertyValue(prop.slice(6)) : null;
  }
  if (prop.startsWith("attr:")) return el?.getAttribute ? el.getAttribute(prop.slice(5)) : null;
  if (prop.startsWith("matches:")) return !!el?.matches && el.matches(prop.slice(8));
  if (NUMERIC.has(prop)) return el ? norm(el[prop]) : null;
  switch (prop) {
    case "rect": {
      if (!el?.getBoundingClientRect) return null;
      const r = el.getBoundingClientRect();
      return { x: r.x, y: r.y, width: r.width, height: r.height, top: r.top, left: r.left, right: r.right, bottom: r.bottom };
    }
    case "activeElement": {
      const doc = el && el.nodeType === 9 ? el : (el?.ownerDocument ?? W.document);
      const active = doc.activeElement;
      const out = describe(active) as Record<string, unknown> | null;
      if (out) out.within = !!el?.contains && el.nodeType === 1 && el.contains(active);
      return out;
    }
    case "visualViewport": {
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
    }
    case "value":
    case "checked":
      return el ? norm(el[prop]) : null;
    case "open":
      return el ? !!el.open : null;
    case "popoverOpen":
      return !!el?.matches && el.matches(":popover-open");
  }
  return null;
}

function get(target: Targetish, props: string[]): Record<string, unknown> {
  const el = resolve(target);
  const out: Record<string, unknown> = {};
  for (const prop of props) out[prop] = read(el, prop);
  return out;
}

// ---------------------------------------------------------------- set

/** Writes (or, with `null`, removes) the CSS custom property `name` on an element whose style Rust does not own. */
function set(target: Targetish, name: string, value: string | null): void {
  if (!name.startsWith("--")) throw new Error("bridge.set: only custom properties (--name): " + name);
  const el = resolve(target);
  if (!el?.style) return;
  if (value === null) el.style.removeProperty(name);
  else el.style.setProperty(name, value);
}

// ---------------------------------------------------------------- commit and transaction

/** A forced style and layout flush: whatever was written before is a style change of its own. */
function flush(): void {
  void W.document.documentElement.offsetHeight;
}

function commit(): Promise<void> {
  return new Promise((done) => {
    W.requestAnimationFrame(() => {
      flush();
      done();
    });
  });
}

/** Runs `ops` in order. Everything up to the first `commit` runs in the caller's JS turn. */
async function transaction(ops: Op[]): Promise<OpResult[]> {
  const out: OpResult[] = [];
  for (const op of ops) {
    try {
      switch (op.op) {
        case "call":
          out.push({ ok: call(op.target, op.method, op.args) });
          break;
        case "get":
          out.push({ ok: get(op.target, op.props) });
          break;
        case "set":
          set(op.target, op.name, op.value);
          out.push({ ok: null });
          break;
        case "commit":
          await commit();
          out.push({ ok: null });
          break;
      }
    } catch (error) {
      out.push({ err: String(error) });
    }
  }
  return out;
}

// ---------------------------------------------------------------- rules

type Installed = { text: string; off: () => void };
const installedRules = new WeakMap<object, Map<string, Installed>>();

/** A scope that sends nothing, for a rule installed without one (it lives until `remove`). */
const detachedScope = (): RuleScope => ({
  on: (target, type, handler, options) => bareOn(target, type, handler, options),
  cleanup() {},
  send() {},
});

const rules: Bridge["rules"] = {
  install(target, rule, scope, key) {
    const el = resolve(target);
    if (!el || el.nodeType !== 1) return () => {};
    const slot = key ?? rule.kind;
    return installRule(el, slot, rule, JSON.stringify(rule), scope ?? detachedScope(), !!scope);
  },
  remove(target, key) {
    const el = resolve(target);
    const slots = el && installedRules.get(el);
    if (!slots) return;
    for (const slot of key === undefined ? [...slots.keys()] : [key]) {
      slots.get(slot)?.off();
      slots.delete(slot);
    }
  },
  read(target) {
    const el = resolve(target);
    return el && el.nodeType === 1 ? readRuleAttributes(el) : [];
  },
  sync(target, scope) {
    const el = resolve(target);
    if (!el || el.nodeType !== 1) return;
    const wanted = new Map(readRuleAttributes(el));
    const slots = installedRules.get(el);
    for (const slot of slots ? [...slots.keys()] : []) {
      if (slot.startsWith(RULE_ATTR_PREFIX) && !wanted.has(slot)) rules.remove(el, slot);
    }
    for (const [name, rule] of wanted) {
      const text = el.getAttribute(name) ?? "";
      if (installedRules.get(el)?.get(name)?.text === text) continue;
      installRule(el, name, rule, text, scope ?? detachedScope(), !!scope);
    }
  },
};

function installRule(el: any, slot: string, rule: Rule, text: string, scope: RuleScope, owned: boolean): () => void {
  let slots = installedRules.get(el);
  if (!slots) installedRules.set(el, (slots = new Map()));
  slots.get(slot)?.off();
  const off = applyRule(rule, el, scope);
  const entry: Installed = { text, off };
  slots.set(slot, entry);
  const remove = () => {
    if (slots.get(slot) !== entry) return;
    off();
    slots.delete(slot);
  };
  if (owned) scope.cleanup(remove);
  return remove;
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
  if (!el || el.nodeType !== 1 || !el.isConnected) return () => {};
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

/**
 * Registers `handler` on `target` and returns its `off`. The one place a listener is added: the
 * registration is recorded in `window.__dxJsListeners.live` while it is live, the scroll-blocking
 * events default to passive, and a `{ once: true }` registration forgets itself when it fires.
 */
function bareOn(
  target: any,
  type: string,
  handler: (event: any) => void,
  options?: boolean | ListenOptions,
  own?: (off: () => void) => void,
): () => void {
  const el = resolve(target);
  if (!el) return () => {};
  const o: ListenOptions = typeof options === "boolean" ? { capture: options } : Object.assign({}, options);
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
  own?.(off);
  return off;
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
      const fields = spec.fields ?? [];
      const off = on(spec.target as any, spec.event, (event) => send(pickAll(event, fields)), spec.options);
      const el = resolve(spec.target);
      if (spec.initial && el) send(pickAll({ type: "initial", target: el, currentTarget: el }, fields));
      return off;
    },
    observe(spec) {
      const el = resolve(spec.target);
      if (!el || done) return () => {};
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
          ro.observe(el, spec.options);
          observer = ro;
          break;
        }
        case "intersection": {
          const o = spec.options ?? {};
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
          mo.observe(el, spec.options ?? { attributes: true, childList: true, subtree: true });
          observer = mo;
          break;
        }
      }
      const off = () => observer.disconnect();
      offs.push(off);
      return off;
    },
    media(query) {
      if (done) return () => {};
      const mql = W.matchMedia(query);
      send({ matches: mql.matches });
      return on(mql, "change", (event) => send({ matches: event.matches }));
    },
    sample(spec) {
      const el = resolve(spec.target);
      if (!el || done) return () => {};
      const started = W.performance.now();
      const onlyChanged = spec.onlyChanged !== false;
      const reads: string[] = [];
      const batches: { name: string; entries: Record<string, unknown>[] }[] = [];
      const stops: (() => void)[] = [];
      for (const f of spec.fields) {
        if (typeof f === "string") reads.push(f);
        else if (f.mode === "batched") {
          const batch = { name: f.name, entries: [] as Record<string, unknown>[] };
          batches.push(batch);
          const pickFields = f.pick ?? [];
          stops.push(
            on(el, f.event, (event) => batch.entries.push({ timeStamp: event.timeStamp, ...pickAll(event, pickFields) }), {
              capture: !!f.capture,
            }),
          );
        } else reads.push(f.name);
      }
      let frames = 0;
      // a sample with no coalesced field has nothing to report until an event arrives
      let last = reads.length ? "" : "{}";
      let frame = 0;
      let live = true;
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
        const sample: Record<string, unknown> = {};
        for (const field of reads) sample[field] = read(el, field);
        const text = JSON.stringify(sample);
        const events = batches.some((b) => b.entries.length > 0);
        // every batched field is always present in a message (an empty array when it saw nothing)
        for (const b of batches) sample[b.name] = b.entries.splice(0, b.entries.length);
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
    run(command) {
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
        case "rule":
          rules.install(command.target, command.rule, s, command.key);
          break;
        case "sync-rules":
          rules.sync(command.target, s);
          break;
      }
    },
  };
  return s;
}

/**
 * The loop of a Rust `bridge::Session`: the first message is the array of setup commands (later
 * arrays add more), the `TEARDOWN` string ends it, and everything registered is undone either way.
 */
async function serve(raw: DioxusChannel): Promise<void> {
  const s = scope(raw);
  try {
    for (;;) {
      const message = await s.channel.recv();
      if (message === TEARDOWN) break;
      if (Array.isArray(message)) for (const command of message) s.run(command as Command);
    }
  } finally {
    s.dispose();
  }
}

/**
 * The ordered fire-and-forget channel of a webview: every message is an array of ops, run in the order
 * received (a `commit` holds the next message back). One per webview, so two writes from different
 * components keep their order even where separate evals would not (a desktop webview).
 */
async function pump(raw: DioxusChannel): Promise<void> {
  const ch = channel(raw);
  for (;;) {
    const ops = await ch.recv();
    if (!Array.isArray(ops)) continue;
    for (const result of await transaction(ops as Op[])) if ("err" in result) console.warn("dx bridge:", result.err);
  }
}

if (!W.__dxBridge) {
  const bridge: Bridge = { version: 1, channel, scope, serve, pump, call, get, set, commit, transaction, rules };
  W.__dxBridge = bridge;
}

export {};
