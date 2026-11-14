// The page side of the counterfactual-witness harness (`playwright/witness/`). It lives beside the tests, outside
// `primitives/src/`, so no gate that reads the shipped TypeScript reads it and it can never become a bundle entry:
// it is test code, type-checked by `tsc -p primitives/ts-test`. Playwright serialises `witnessPage` into `addInitScript`, so it runs before any page script and must
// capture nothing from this module (every helper is a closure inside it).
//
// What it does, and nothing the release bundle knows about:
//   * it patches the platform members an effect lands on (`Event.prototype.preventDefault`, `setAttribute`, `focus`, ...)
//     and drops, delays (in COMPLETED PAINTS) or rehomes a call only while the adapter's marker `window.__dxApplying`
//     (`src/ts/bridge.ts`, "the apply table") names the site and the effect under test: an unmarked call, a Rust-side call
//     and every other effect run untouched;
//   * it holds a bridge message back until the next trusted event has finished dispatch or k paints have completed
//     (the order counterfactual, through `window.__dxBridgeDelayFrames` taken as a scheduler);
//   * it records the library projection of every element of the host subtree at the end of each completed paint,
//     snapshots the default-action observables, and logs the sentinels that decide `outcomeOrder`.
// The probes that judge these records live in `playwright/witness/probes/`; nothing here judges anything.

export type WitnessEffect =
  | "prevent" | "stop" | "dataTransfer" | "attrs" | "setVar" | "scrollBy" | "scrollTo" | "scrollIntoView"
  | "setValue" | "focus" | "capture" | "show" | "hide" | "setProp";

export interface WitnessArm {
  site: string;
  effect: WitnessEffect;
  /** `drop`: the call never happens. `delay`: it happens `frames` completed paints later. `rehome` (`prevent` only):
   * it happens in the bubble phase at `host`, and is dropped when the event does not pass through it. `order`: the
   * bridge's messages are held (see the header); the patched members are not touched. */
  mode: "drop" | "delay" | "rehome" | "order";
  frames: number;
  host?: string;
}

export interface WitnessSentinel {
  name: string;
  type: string;
  /** Where the later listener sits on the path: the window in the capture or bubble phase, or an element. */
  where: "window-capture" | "window-bubble" | "selector";
  selector?: string;
}

export function witnessPage(): void {
  const W = window as any;
  if (W.__dxWitness) return;

  let arm: WitnessArm | null = null;
  let hits = 0;
  /** `site:effect` of every marked write that reached a patched member, intercepted or not: proof a decision was applied. */
  const applied = new Set<string>();
  let override: { site: string; effect: string } | null = null;

  /** `fn` after `n` completed paints: a frame callback runs BEFORE the paint it belongs to, so each paint is a frame
   * callback and then a hop that runs after it. */
  const paints = (n: number, fn: () => void): void => {
    if (n <= 0) return fn();
    requestAnimationFrame(() => {
      const channel = new MessageChannel();
      channel.port1.onmessage = () => paints(n - 1, fn);
      channel.port2.postMessage(0);
    });
  };

  /** The write clock: `paints` that stands still while the runner has an input in flight (`freeze`), so a delayed write
   * is due a number of paints after the trigger however long the harness takes to deliver the next input. */
  let frozen = false;
  const clocked = (n: number, fn: () => void): void => {
    if (n <= 0) return fn();
    requestAnimationFrame(() => {
      const channel = new MessageChannel();
      channel.port1.onmessage = () => clocked(frozen ? n : n - 1, fn);
      channel.port2.postMessage(0);
    });
  };

  // ---------------------------------------------------------------- the patched members
  type Member = { owner: any; name: string; kind: "method" | "set" };
  const method = (owner: any, name: string): Member => ({ owner, name, kind: "method" });
  const setter = (owner: any, name: string): Member => ({ owner, name, kind: "set" });
  const MEMBERS: Record<WitnessEffect, Member[]> = {
    prevent: [method(Event.prototype, "preventDefault")],
    stop: [method(Event.prototype, "stopPropagation")],
    dataTransfer: [method(DataTransfer.prototype, "setData"), setter(DataTransfer.prototype, "dropEffect")],
    attrs: [method(Element.prototype, "setAttribute")],
    setVar: [method(CSSStyleDeclaration.prototype, "setProperty")],
    scrollBy: [method(Element.prototype, "scrollBy"), method(W, "scrollBy")],
    scrollTo: [method(Element.prototype, "scrollTo"), method(W, "scrollTo")],
    scrollIntoView: [method(Element.prototype, "scrollIntoView")],
    setValue: [setter(HTMLInputElement.prototype, "value"), setter(HTMLTextAreaElement.prototype, "value")],
    focus: [method(HTMLElement.prototype, "focus")],
    capture: [method(Element.prototype, "setPointerCapture")],
    show: [method(HTMLElement.prototype, "showPopover"), method(HTMLDialogElement.prototype, "showModal"), method(HTMLDialogElement.prototype, "show")],
    hide: [method(HTMLElement.prototype, "hidePopover"), method(HTMLDialogElement.prototype, "close")],
    setProp: [setter(HTMLInputElement.prototype, "indeterminate"), setter(HTMLInputElement.prototype, "defaultChecked")],
  };

  /** What to do with this call: null runs it, otherwise the arm that intercepts it. */
  const intercept = (effect: WitnessEffect): WitnessArm | null => {
    const marker = W.__dxApplying ?? override;
    if (marker && marker.effect === effect) applied.add(marker.site + ":" + effect);
    const a = arm;
    if (!a || a.mode === "order" || a.effect !== effect) return null;
    return marker && marker.site === a.site && marker.effect === effect ? a : null;
  };

  const rehome = (a: WitnessArm, event: Event, run: () => void): void => {
    const host = a.host ? document.querySelector(a.host) : null;
    const target = event.target as Node | null;
    // the canceled flag is read after dispatch: a bubble listener on the host cancels exactly when the event passes it
    if (!host || !target || !host.contains(target)) return;
    const once = (later: Event) => {
      if (later !== event) return;
      host.removeEventListener(event.type, once);
      run();
    };
    host.addEventListener(event.type, once);
  };

  for (const [effect, members] of Object.entries(MEMBERS) as [WitnessEffect, Member[]][]) {
    for (const { owner, name, kind } of members) {
      const descriptor = Object.getOwnPropertyDescriptor(owner, name);
      if (!descriptor) continue;
      if (kind === "method" && typeof descriptor.value === "function") {
        const original: (...args: unknown[]) => unknown = descriptor.value;
        owner[name] = function (this: unknown, ...args: unknown[]) {
          const a = intercept(effect);
          if (!a) return original.apply(this, args);
          hits++;
          const run = () => original.apply(this, args);
          if (a.mode === "delay") clocked(a.frames, run);
          else if (a.mode === "rehome" && this instanceof Event) rehome(a, this, run);
          return undefined;
        };
      } else if (kind === "set" && descriptor.set) {
        const original = descriptor.set;
        Object.defineProperty(owner, name, {
          ...descriptor,
          set(this: unknown, value: unknown) {
            const a = intercept(effect);
            if (!a) return original.call(this, value);
            hits++;
            if (a.mode === "delay") clocked(a.frames, () => original.call(this, value));
          },
        });
      }
    }
  }

  // ---------------------------------------------------------------- the order hold
  const DELEGATED = ["click", "dblclick", "keydown", "keyup", "input", "change", "pointerdown", "pointerup", "mousedown", "mouseup", "focusin", "focusout", "wheel"];
  type Held = { since: number; release: () => void };
  const held: Held[] = [];
  const posts = new MessageChannel();
  let events = 0;
  posts.port1.onmessage = (message) => {
    const seq = message.data as number;
    const due = held.filter((h) => h.since < seq);
    for (const entry of due) held.splice(held.indexOf(entry), 1);
    for (const entry of due) entry.release();
  };
  for (const type of DELEGATED) {
    // a task posted from a window capture listener runs after the whole dispatch, which a microtask would not
    W.addEventListener(type, (event: Event) => event.isTrusted && posts.port2.postMessage(++events), true);
  }
  // the input the runner froze the write clock for is complete once its key has been released
  W.addEventListener("keyup", (event: Event) => void (event.isTrusted && (frozen = false)), true);
  const hold = (deliver: () => void): void => {
    hits++;
    let done = false;
    const release = () => done || ((done = true), deliver());
    held.push({ since: events, release });
    paints(arm?.frames ?? 0, () => {
      const at = held.findIndex((h) => h.release === release);
      if (at >= 0) held.splice(at, 1);
      release();
    });
  };

  // ---------------------------------------------------------------- the projection
  const PSEUDOS = [":checked", ":indeterminate", ":popover-open", ":modal", ":open", ":placeholder-shown"];
  const round = (n: number) => Math.round(n * 100) / 100;
  const pathOf = (el: Element, root: Element): string => {
    const steps: number[] = [];
    for (let at: Element | null = el; at && at !== root; at = at.parentElement) steps.push(Array.prototype.indexOf.call(at.parentElement?.children ?? [], at));
    return steps.reverse().join(".");
  };
  const matches = (el: Element, selector: string): boolean => {
    try {
      return el.matches(selector);
    } catch {
      return false;
    }
  };
  /** One element's state as the library sees it: geometry, scroll, the visible computed values, focus, the state
   * pseudo-classes, the native state, the `data-*` and `aria-*` attributes and the component sheet's own properties. */
  const project = (el: Element, props: string[]): string => {
    const style = getComputedStyle(el);
    const box = el.getBoundingClientRect();
    const native: Record<string, unknown> = {};
    for (const key of ["value", "checked", "indeterminate", "open"]) if (key in el) native[key] = (el as any)[key];
    const attrs = [...el.attributes].filter((a) => /^(data|aria)-/.test(a.name) && a.name !== "data-dx-witness-host").map((a) => a.name + "=" + a.value).sort();
    const computed: Record<string, string> = { opacity: style.opacity, visibility: style.visibility };
    for (const prop of props) computed[prop] = style.getPropertyValue(prop);
    return JSON.stringify({
      rect: [box.x, box.y, box.width, box.height].map(round),
      scroll: [el.scrollLeft, el.scrollTop],
      computed,
      focus: el === document.activeElement,
      pseudo: PSEUDOS.filter((p) => matches(el, p)),
      native,
      attrs,
    });
  };

  type Recording = { host: Element; props: string[]; frames: number; elements: Record<string, string[]>; on: boolean };
  let recording: Recording | null = null;
  const sample = (r: Recording): void => {
    r.frames++;
    for (const el of [r.host, ...r.host.querySelectorAll("*")]) (r.elements[pathOf(el, r.host)] ??= []).push(project(el, r.props));
  };
  const tick = (r: Recording): void => {
    if (!r.on) return;
    sample(r);
    paints(1, () => tick(r));
  };

  // ---------------------------------------------------------------- the sentinels
  const log: string[] = [];
  let order = 0;
  const note = (name: string): void => void log.push(name + "@" + order++);
  /** Which element each input event reached, relative to the watched host (the focus probe's observable). */
  const typed: string[] = [];
  let typedHost: Element | null = null;
  W.addEventListener(
    "input",
    (event: Event) => {
      const target = event.target;
      if (typedHost && target instanceof Element) typed.push(typedHost.contains(target) ? pathOf(target, typedHost) || "host" : "outside");
    },
    true,
  );

  // ---------------------------------------------------------------- the default-action observables
  const scrollChain = (from: Element | null): number[][] => {
    const out: number[][] = [];
    for (let el = from; el; el = el.parentElement) if (el.scrollTop || el.scrollLeft) out.push([el.scrollLeft, el.scrollTop]);
    out.push([window.scrollX, window.scrollY]);
    return out;
  };
  const snapshot = (hostSelector: string): Record<string, unknown> => {
    const host = document.querySelector(hostSelector) ?? document.documentElement;
    const active = document.activeElement;
    const field = active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement ? active : null;
    const selection = getSelection();
    return {
      activeElement: active ? (active === document.body ? "body" : pathOf(active, host) || "host") : null,
      scrollChain: scrollChain(active ?? host),
      scrollHost: scrollChain(host),
      fieldValue: field ? field.value : null,
      fieldSelection: field ? [field.selectionStart, field.selectionEnd] : null,
      selection: selection ? selection.toString() : "",
      open: [...host.querySelectorAll("dialog, details, [popover]")].map((el) => [pathOf(el, host), (el as any).open ?? matches(el, ":popover-open")]),
      activation: [...host.querySelectorAll("input, option")].map((el) => [pathOf(el, host), (el as any).checked ?? (el as any).selected ?? null]),
      location: location.href,
      insertion: host.textContent,
      outcome: log.slice(),
      typedInto: typed.slice(),
    };
  };

  // ---------------------------------------------------------------- pausing unrelated and own motion
  const style = document.createElement("style");
  style.textContent =
    '[data-dx-witness-host], [data-dx-witness-host] *, [data-dx-witness-host]::before, [data-dx-witness-host]::after,' +
    '[data-dx-witness-host] *::before, [data-dx-witness-host] *::after { animation-play-state: paused !important; }';
  const pause = (hostSelector: string, hostMotion: boolean): void => {
    const host = document.querySelector(hostSelector) ?? document.documentElement;
    if (hostMotion) {
      (document.head ?? document.documentElement).append(style);
      host.setAttribute("data-dx-witness-host", "");
    }
    // everything outside the host subtree: the siblings of the host and of each of its ancestors
    for (let at: Element | null = host; at && at.parentElement; at = at.parentElement) {
      for (const sibling of at.parentElement.children) if (sibling !== at && sibling !== style) sibling.setAttribute("data-dx-motion", "paused");
    }
  };

  W.__dxWitness = {
    arm(next: WitnessArm | null): void {
      arm = next;
      hits = 0;
      W.__dxBridgeDelayFrames = next && next.mode === "order" ? hold : 0;
    },
    hits: () => hits,
    applied: (): string[] => [...applied],
    /** Self-test seam: run `fn` as if the adapter were applying `effect` at `site`. */
    mark(site: string, effect: string, fn: () => void): void {
      override = { site, effect };
      try {
        fn();
      } finally {
        override = null;
      }
    },
    paints,
    /** Stops the write clock; the next trusted keyup restarts it. Called before the trigger is replayed. */
    freeze(): void {
      frozen = true;
    },
    /** Runs the write clock `n` paints, stops it in the same callback and calls `done`: the next input then lands
     * exactly `n` write-clock paints after the trigger, whatever the harness latency between this call and the input. */
    clockFor(n: number, done: () => void): void {
      frozen = false;
      clocked(n, () => {
        frozen = true;
        done();
      });
    },
    pause,
    record(hostSelector: string, props: string[]): void {
      const host = document.querySelector(hostSelector) ?? document.documentElement;
      recording = { host, props, frames: 0, elements: {}, on: true };
      tick(recording);
    },
    /** Start recording on the first frame the host exists (a client mount is the effect under test). */
    recordWhenPresent(hostSelector: string, props: string[]): void {
      const look = () => (document.querySelector(hostSelector) ? W.__dxWitness.record(hostSelector, props) : requestAnimationFrame(look));
      look();
    },
    stopRecording(): { frames: number; elements: Record<string, string[]> } {
      const r = recording;
      if (!r) return { frames: 0, elements: {} };
      r.on = false;
      recording = null;
      return { frames: r.frames, elements: r.elements };
    },
    snapshot,
    note,
    sentinel(spec: WitnessSentinel): void {
      const record = (event: Event) => note(spec.name + (event.defaultPrevented ? ":prevented" : ""));
      if (spec.where === "selector" && spec.selector) document.querySelector(spec.selector)?.addEventListener(spec.type, record);
      else W.addEventListener(spec.type, record, spec.where === "window-capture");
    },
    outcome: (): string[] => log.slice(),
    watchTyped(hostSelector: string): void {
      typedHost = document.querySelector(hostSelector) ?? document.documentElement;
    },
  };
}
