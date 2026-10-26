// A just-enough fake of the browser for `bun test`: the bridge reads every global through `window`, so the
// SHIPPED `primitives/src/js/bridge.js` runs unchanged against `makeWindow()`. Nothing here is a DOM
// implementation; each class records what the bridge did to it.

import { readFileSync } from "node:fs";
import { join } from "node:path";

type Listener = { type: string; fn: (event: any) => void; options: any };

export class FakeTarget {
  listeners: Listener[] = [];
  addEventListener(type: string, fn: (event: any) => void, options?: any) {
    this.listeners.push({ type, fn, options });
  }
  removeEventListener(type: string, fn: (event: any) => void, options?: any) {
    const capture = typeof options === "boolean" ? options : !!options?.capture;
    const at = this.listeners.findIndex(
      (l) => l.type === type && l.fn === fn && !!(typeof l.options === "boolean" ? l.options : l.options?.capture) === capture,
    );
    if (at >= 0) this.listeners.splice(at, 1);
  }
  /** Calls every live listener of `type` (a copy, as the browser does). */
  dispatch(type: string, event: any = {}) {
    for (const l of [...this.listeners]) if (l.type === type) l.fn({ type, ...event });
  }
  count(type?: string) {
    return this.listeners.filter((l) => type === undefined || l.type === type).length;
  }
}

export class FakeElement extends FakeTarget {
  nodeType = 1;
  tagName = "DIV";
  attrs = new Map<string, string>();
  calls: [string, unknown[]][] = [];
  open = false;
  isConnected = true;
  popoverOpen = false;
  offsetWidth = 100;
  offsetHeight = 50;
  scrollTop = 7;
  value = "";
  rect = { x: 1, y: 2, width: 100, height: 50, top: 2, left: 1, right: 101, bottom: 52 };
  ownerDocument: FakeDocument | null = null;
  children: FakeElement[] = [];
  customProps = new Map<string, string>();
  style = {
    setProperty: (name: string, value: string) => void this.customProps.set(name, value),
    removeProperty: (name: string) => void this.customProps.delete(name),
  };
  constructor(public id = "") {
    super();
  }
  getAttribute(name: string) {
    return this.attrs.has(name) ? (this.attrs.get(name) as string) : null;
  }
  setAttribute(name: string, value: string) {
    this.attrs.set(name, value);
  }
  hasAttribute(name: string) {
    return this.attrs.has(name);
  }
  getAttributeNames() {
    return [...this.attrs.keys()];
  }
  matches(selector: string) {
    return selector === ":popover-open" ? this.popoverOpen : selector === `#${this.id}`;
  }
  contains(other: unknown) {
    return other === this || this.children.includes(other as FakeElement);
  }
  getBoundingClientRect() {
    return this.rect;
  }
  private record(name: string, args: unknown[]) {
    this.calls.push([name, args]);
  }
  showPopover() {
    this.record("showPopover", []);
    this.popoverOpen = true;
  }
  hidePopover() {
    this.record("hidePopover", []);
    this.popoverOpen = false;
  }
  togglePopover() {
    this.record("togglePopover", []);
    this.popoverOpen = !this.popoverOpen;
    return this.popoverOpen;
  }
  showModal() {
    this.record("showModal", []);
    this.open = true;
  }
  close(...args: unknown[]) {
    this.record("close", args);
    this.open = false;
  }
  setPointerCapture(...args: unknown[]) {
    this.record("setPointerCapture", args);
  }
  hasPointerCapture() {
    return true;
  }
  releasePointerCapture(...args: unknown[]) {
    this.record("releasePointerCapture", args);
  }
  focus(...args: unknown[]) {
    this.record("focus", args);
    if (this.ownerDocument) this.ownerDocument.activeElement = this;
  }
  blur() {
    this.record("blur", []);
  }
  scrollIntoView(...args: unknown[]) {
    this.record("scrollIntoView", args);
  }
  scrollTo(...args: unknown[]) {
    this.record("scrollTo", args);
  }
  scrollBy(...args: unknown[]) {
    this.record("scrollBy", args);
  }
  setSelectionRange(...args: unknown[]) {
    this.record("setSelectionRange", args);
  }
  select() {
    this.record("select", []);
  }
}

export class FakeDocument extends FakeTarget {
  nodeType = 9;
  byId = new Map<string, FakeElement>();
  activeElement: FakeElement | null = null;
  documentElement = new FakeElement("html");
  reads = 0;
  constructor() {
    super();
    // a forced flush is a read of a layout property of the root: count it
    const doc = this;
    Object.defineProperty(this.documentElement, "offsetHeight", {
      get() {
        doc.reads += 1;
        return 0;
      },
    });
  }
  add(id: string): FakeElement {
    const el = new FakeElement(id);
    el.ownerDocument = this;
    this.byId.set(id, el);
    return el;
  }
  getElementById(id: string) {
    return this.byId.get(id) ?? null;
  }
  querySelector(selector: string) {
    return selector.startsWith("#") ? this.getElementById(selector.slice(1)) : null;
  }
}

export class FakeMedia extends FakeTarget {
  constructor(public query: string, public matches: boolean) {
    super();
  }
}

/** An observer that records its callback and what it observes, and is fired by hand. */
function observerClass(name: string) {
  return class {
    static all: any[] = [];
    observed: unknown[] = [];
    disconnected = false;
    constructor(public callback: (entries: any[]) => void, public init?: unknown) {
      (this.constructor as any).all.push(this);
      void name;
    }
    observe(el: unknown, options?: unknown) {
      this.observed.push([el, options]);
    }
    disconnect() {
      this.disconnected = true;
    }
  };
}

export function makeWindow() {
  const document = new FakeDocument();
  const frames: ((time: number) => void)[] = [];
  let now = 0;
  const media = new Map<string, FakeMedia>();
  const win: any = new FakeTarget();
  Object.assign(win, {
    document,
    frames,
    media,
    ResizeObserver: observerClass("ResizeObserver"),
    IntersectionObserver: observerClass("IntersectionObserver"),
    MutationObserver: observerClass("MutationObserver"),
    visualViewport: { width: 390, height: 600, offsetTop: 0, offsetLeft: 0, pageTop: 0, pageLeft: 0, scale: 1 },
    innerWidth: 1024,
    innerHeight: 768,
    requestAnimationFrame(cb: (time: number) => void) {
      frames.push(cb);
      return frames.length;
    },
    cancelAnimationFrame(id: number) {
      frames[id - 1] = () => {};
    },
    performance: { now: () => now },
    getComputedStyle: (el: any) => ({ getPropertyValue: (name: string) => el.styles?.[name] ?? "" }),
    matchMedia(query: string) {
      if (!media.has(query)) media.set(query, new FakeMedia(query, false));
      return media.get(query);
    },
    /** Runs the frame callbacks queued so far (callbacks they queue wait for the next step) and advances the clock. */
    step(ms = 16) {
      now += ms;
      const batch = frames.splice(0, frames.length);
      for (const cb of batch) cb(now);
    },
    /** Steps until no frame is queued (bounded). */
    settle(max = 50) {
      for (let i = 0; i < max && frames.length; i++) win.step();
    },
  });
  win.window = win;
  return win;
}

const BRIDGE_JS = readFileSync(join(import.meta.dir, "../src/js/bridge.js"), "utf8");

/** Runs the shipped bridge against `win` the way `bridge::ensure_installed` does in a page. */
export function loadBridge(win: any = makeWindow()) {
  new Function("window", BRIDGE_JS)(win);
  return { win, bridge: win.__dxBridge };
}

/** A stand-in for the `dioxus` object of one eval. */
export function makeChannel() {
  const sent: unknown[] = [];
  const inbox: unknown[] = [];
  let waiting: ((message: unknown) => void) | null = null;
  return {
    sent,
    send(message: unknown) {
      sent.push(message);
    },
    recv(): Promise<unknown> {
      if (inbox.length) return Promise.resolve(inbox.shift());
      return new Promise((resolve) => {
        waiting = resolve;
      });
    },
    /** Rust to JS. */
    push(message: unknown) {
      if (waiting) {
        const w = waiting;
        waiting = null;
        w(message);
      } else inbox.push(message);
    },
  };
}

/** Lets promise continuations run. */
export const ticks = async (n = 5) => {
  for (let i = 0; i < n; i++) await Promise.resolve();
};
