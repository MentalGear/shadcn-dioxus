import type { Page } from "@playwright/test";

/**
 * Helper for `computed-style-snapshot.spec.ts` (dev-docs/backlog.md row 135):
 * the HOVER pass and the OPEN-STATE pass.
 *
 * The rest-state pass of the snapshot reads whatever is in the DOM after load.
 * A closed overlay has no content in the DOM, and `:hover` is not a state the
 * page is ever in on its own, so neither is covered by it. These two passes put
 * the page IN those states with real (trusted) pointer/keyboard input and read
 * the computed style of exactly what the state changed.
 *
 * THE CONSTRUCTION -- a state is captured as a DELTA against a baseline, not as
 * a list of selectors:
 *
 *   1. `take()` records a signature (all captured computed properties) of every
 *      in-scope element, with the pointer parked on an inert spot and
 *      everything settled (finite animations/transitions finished, infinite
 *      ones parked on frame 0).
 *   2. Put the page in a state: hover/click/right-click/focus a target.
 *   3. `delta()` = every element that is NEW since the baseline or whose
 *      signature CHANGED. Those, and only those, are recorded (plus their
 *      `::before`/`::after`/`::placeholder`/`::marker`/`::backdrop`).
 *
 * So nothing here knows what a "dialog" or "menu" is: whatever opens (content
 * mounted into the page, a `<dialog>` in the top layer, a `[popover]`, a class
 * or attribute flip on existing nodes) is captured, and a restyle that changes
 * any of it shows up as a changed/added/removed record. What stays hand-written
 * is only how to POKE a target (click, right-click, hover, focus) and one small
 * table of pages whose triggers are plain buttons (`PLAIN_TRIGGER_PAGES`).
 *
 * Determinism (the A/A gate, two captures of one build are byte-identical):
 * every read is preceded by `settle()`; reads wait for DOM quiescence
 * (`quiet`) so positioning/hydration finished; the pointer is parked on a fixed
 * spot between targets; the overlay is verified closed (and the baseline
 * re-taken) before the next target, with a page reload as the last resort.
 */

export type Rec = Record<string, string | number>;

export type StateArgs = {
  /** Elements whose style is recorded (same selector as the rest-state pass). */
  sel: string;
  /** CSS selector of the roots that interaction targets live in (`body` for chrome routes). */
  scope: string;
  props: string[];
  colorProps: string[];
  mode: string;
};

export type Cand = {
  i: number;
  tag: string;
  role: string;
  hp: string;
  cls: string;
  text: string;
  desc: string;
  hoverFirst: boolean;
  ctxFirst: boolean;
};

export type StateCoverage = {
  /** Trigger candidates found (derived + plain). */
  candidates: number;
  /** Candidates that put the page in a changed state (probe differed). */
  opened: number;
  /** ... of which were opened by each input. */
  via: Record<string, number>;
  /** Candidates that did nothing, with the reason. */
  unopened: { desc: string; reason: string }[];
  /** Items hovered inside opened overlays. */
  itemHovers: number;
  itemHoverSkipped: number;
  /** Rest-state hover targets (after dedupe) / captured / skipped by reason. */
  hoverTargets: number;
  hoverCaptured: number;
  hoverSkipped: Record<string, number>;
  hoverCapped: number;
  /** Times an overlay could not be closed and the page was reloaded. */
  reloads: number;
  /** Records written. */
  openRecords: number;
  hoverRecords: number;
  /** Records of the keyboard-inset pass (included in `openRecords`). */
  kbdRecords: number;
};

export function emptyCoverage(): StateCoverage {
  return {
    candidates: 0, opened: 0, via: {}, unopened: [], itemHovers: 0, itemHoverSkipped: 0,
    hoverTargets: 0, hoverCaptured: 0, hoverSkipped: {}, hoverCapped: 0, reloads: 0,
    openRecords: 0, hoverRecords: 0, kbdRecords: 0,
  };
}

/**
 * Navigate and wait until the page is the page: the legacy `/component/?name=x&` route renders a
 * "Loading component..." stub and redirects client-side once wasm has hydrated (preview/src/main.rs
 * `ComponentDemo`), which on a loaded machine takes longer than any fixed wait (a capture of the stub is 66
 * records instead of ~400). So wait for the stub to be gone, for every `<img>` to have settled (an avatar shows
 * its fallback until `load`), then for the DOM to be quiet.
 */
export async function gotoSettled(page: Page, url: string) {
  // goto-ok: gotoSettled is its own settle (waits out the legacy redirect stub, images and DOM quiet); a hydration flag is not the signal
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => !document.querySelector(".dx-component-demo-redirect"), null, { timeout: 60000 });
  await page.waitForTimeout(1000);
  await page.waitForFunction(() => Array.from(document.images).every((i) => i.complete), null, { timeout: 30000 });
  await page.evaluate(() => new Promise<void>((r) => {
    let last = performance.now();
    const mo = new MutationObserver(() => (last = performance.now()));
    mo.observe(document.documentElement, { subtree: true, childList: true, attributes: true, characterData: true });
    const tick = () => (performance.now() - last >= 400 ? (mo.disconnect(), r()) : setTimeout(tick, 50));
    setTimeout(tick, 50);
  }));
}

/** Pages whose demo triggers are PLAIN buttons (`onclick` + a signal, no `aria-haspopup`/`aria-expanded`
 *  to derive them from). On these every `button`/`[role=button]` in the demo is tried as a trigger. On every
 *  other page only derived triggers are tried (clicking every button of a form or a calendar would just toggle
 *  state, not open anything). */
export const PLAIN_TRIGGER_PAGES = ["alert_dialog", "command", "dialog", "drawer", "sheet", "toast"];

/** Cap on derived/plain triggers per page, and on distinct hover targets per page / per opened overlay. */
export const MAX_TRIGGERS = 30;
export const MAX_HOVER_TARGETS = 60;
export const MAX_ITEM_HOVERS = 8;
/** `drain()` bounds: at least this long after a pointer move (a handler that schedules its timer a task later),
 *  at most this long (a timer chain that never ends must not hang the capture). */
const DRAIN_MIN_MS = 80;
const DRAIN_MAX_MS = 3000;

// ---------------------------------------------------------------------------------------------------------------
// In-page library. Serialised by Playwright: self-contained, no module-scope closure, promises not async/await.
// ---------------------------------------------------------------------------------------------------------------

export function installStateLib(a: StateArgs) {
  const w = window as any;
  const SKIP = ["SCRIPT", "STYLE", "LINK", "META", "TITLE", "HEAD", "NOSCRIPT"];
  const S: any = {};
  w.__snap = S;

  // TIMER-DRAIN RULE (row 245). A tooltip / hover card / submenu opens only after the pointer RESTED for the
  // component's configured `delay` (600 ms default, a demo may set any other), and no fixed wait can know it:
  // a tooltip chain rendered in 1 of 4 captures because the read raced the timer. The delay IS a `setTimeout`,
  // so instead of guessing it the harness watches it: between `arm()` (just before a pointer move) and the end
  // of `drain()`, every timeout the page schedules (delays up to DRAIN_MAX_DELAY_MS) is remembered, and
  // `drain()` resolves only once all of them have fired. Whatever delay a popup is configured with, the opener
  // waits it out, and a popup with no timer pays nothing. The harness's own waits use the raw `setTimeout`,
  // never the tracked one.
  const rawST: typeof window.setTimeout = w.__rawST ?? (w.__rawST = window.setTimeout.bind(window));
  const rawCT: typeof window.clearTimeout = w.__rawCT ?? (w.__rawCT = window.clearTimeout.bind(window));
  const setTimeout = (f: () => void, ms?: number) => rawST(f, ms);
  const DRAIN_MAX_DELAY_MS = 2500;
  if (!w.__timersTracked) {
    w.__timersTracked = true;
    w.__armed = false;
    w.__pending = new Set<number>();
    w.__maxPendingDelay = 0;
    (window as any).setTimeout = (handler: any, ms?: number, ...rest: any[]) => {
      const d = Number(ms) || 0;
      if (!w.__armed || typeof handler !== "function" || d > DRAIN_MAX_DELAY_MS) return rawST(handler, ms, ...rest);
      let id = 0;
      id = rawST(() => {
        w.__pending.delete(id);
        handler(...rest);
      }, ms) as unknown as number;
      w.__pending.add(id);
      if (d > w.__maxPendingDelay) w.__maxPendingDelay = d;
      return id;
    };
    (window as any).clearTimeout = (id?: number) => {
      if (id !== undefined) w.__pending.delete(id as number);
      return rawCT(id);
    };
  }
  S.arm = () => {
    w.__pending.clear();
    w.__armed = true;
  };
  /** Resolve once every timeout scheduled since `arm()` has fired and none was scheduled for `quietMs` more
   *  (a handler that schedules the render), but never before `minMs`, and at most after `maxMs`. */
  S.drain = (minMs: number, maxMs: number) =>
    new Promise<boolean>((resolve) => {
      const start = performance.now();
      let emptySince = -1;
      const done = (ok: boolean) => {
        w.__armed = false;
        w.__pending.clear();
        resolve(ok);
      };
      const tick = () => {
        S.settle();
        const now = performance.now();
        if (w.__pending.size > 0) emptySince = -1;
        else if (emptySince < 0) emptySince = now;
        if (now - start >= minMs && emptySince >= 0 && now - emptySince >= 60) return done(true);
        if (now - start >= maxMs) return done(false);
        setTimeout(tick, 20);
      };
      setTimeout(tick, 20);
    });

  S.settle = () => {
    for (const an of document.getAnimations()) {
      try {
        const isTransition = w.CSSTransition && an instanceof w.CSSTransition;
        const timing = an.effect ? an.effect.getComputedTiming() : null;
        if (!isTransition && timing && timing.iterations === Infinity) {
          an.pause();
          an.currentTime = 0;
        } else {
          an.finish();
        }
      } catch (_) {
        /* nothing to do for an animation we cannot settle */
      }
    }
  };

  S.forceTheme = () => {
    const root = document.documentElement;
    if (root.getAttribute("data-theme") !== a.mode) root.setAttribute("data-theme", a.mode);
    // `--dx-cv` is the ONE switch of every `content-visibility: auto` card (main.css): `visible` lays them all
    // out for real. With `auto`, scrolling a target into view renders a skipped card and the document height
    // (and every ancestor's) changes by an amount that depends on when the renderer got to it -- a delta that
    // differs between two runs of one build. The state passes run with the cards always laid out.
    root.style.setProperty("--dx-cv", "visible");
  };

  S.els = (): Element[] =>
    Array.from(document.querySelectorAll(a.sel)).filter((e) => SKIP.indexOf(e.tagName.toUpperCase()) < 0);
  S.roots = (): Element[] => (a.scope === "body" ? [document.body] : Array.from(document.querySelectorAll(a.scope)));
  S.visible = (el: Element) => el.getClientRects().length > 0;

  S.sig = (el: Element) => {
    const cs = getComputedStyle(el);
    let s = "";
    for (const p of a.props) s += cs.getPropertyValue(p) + "\u0001";
    return s;
  };
  /** Baseline `which` ("base" = closed rest state, "obase" = while an overlay is open). */
  S.scrollKey = () => `${window.scrollX},${window.scrollY},${window.innerWidth}x${window.innerHeight}`;
  S.take = (which: string) => {
    S.settle();
    if (which === "base") S.bscroll = S.scrollKey();
    const m = new Map<Element, string>();
    for (const el of S.els()) m.set(el, S.sig(el));
    S[which] = m;
    if (which === "base") {
      S.known = new Set(Array.from(document.querySelectorAll("*")));
      S.baseRendered = S.renderedSet();
      S.probe0 = S.probe();
      S.cprobe0 = S.cprobe();
      S.baseExp = new Map<Element, string | null>();
      document.querySelectorAll("[aria-expanded]").forEach((e) => S.baseExp.set(e, e.getAttribute("aria-expanded")));
    }
    return m.size;
  };

  /** Cheap "is something open?" fingerprint. Deliberately NOT the total element count: closed overlays may
   *  leave hidden nodes behind, so only RENDERED nodes and open-state markers count. */
  S.probe = () => {
    const qv = (sel: string) => {
      let n = 0;
      try {
        document.querySelectorAll(sel).forEach((e) => {
          if (e.getClientRects().length > 0) n++;
        });
      } catch (_) {
        n = -1;
      }
      return n;
    };
    const qa = (sel: string) => {
      try {
        return document.querySelectorAll(sel).length;
      } catch (_) {
        return -1;
      }
    };
    let vis = 0;
    for (const el of S.els()) if (el.getClientRects().length > 0) vis++;
    // WHICH elements carry an open-state marker (not just how many: a single-open accordion swaps one for another)
    // Only nodes that existed at the baseline: a closed overlay may leave a hidden node behind.
    const known: Set<Element> | undefined = S.known;
    let marks = "";
    let k = 0;
    document.querySelectorAll('[aria-expanded], [data-state], [open], [aria-hidden]').forEach((e) => {
      if (known && !known.has(e)) return;
      marks += `${k++}:${e.getAttribute("aria-expanded") ?? ""}${e.getAttribute("data-state") ?? ""}${e.hasAttribute("open") ? "o" : ""}${e.getAttribute("aria-hidden") ?? ""},`;
    });
    let h = 5381;
    for (let i = 0; i < marks.length; i++) h = ((h << 5) + h + marks.charCodeAt(i)) | 0;
    let unk = 0;
    if (known) {
      document.querySelectorAll("*").forEach((e) => {
        if (!known.has(e) && e.getClientRects().length > 0) unk++;
      });
    }
    return [
      vis,
      h,
      unk,
      qa("dialog[open]"),
      qa(":popover-open"),
      qv('[role="dialog"],[role="alertdialog"],[role="menu"],[role="listbox"],[role="tooltip"]'),
    ].join("|");
  };

  /** "Is it closed again?" fingerprint: how many open markers / top-layer nodes / visible overlay roles exist.
   *  Counts, not positions: Dioxus re-creates nodes, and a closed overlay may leave a hidden or collapsed
   *  (height 0, still has a box) node behind -- neither is "open". */
  S.cprobe = () => {
    let roles = 0;
    document.querySelectorAll('[role="dialog"],[role="alertdialog"],[role="menu"],[role="listbox"],[role="tooltip"]').forEach((e) => {
      if (e.getClientRects().length > 0) roles++;
    });
    const qa = (sel: string) => {
      try {
        return document.querySelectorAll(sel).length;
      } catch (_) {
        return -1;
      }
    };
    return [qa('[data-state="open"]'), qa('[aria-expanded="true"]'), qa("dialog[open]"), qa(":popover-open"), roles].join("|");
  };

  /** Toggles whose aria-expanded differs from the baseline (a single-open accordion that swapped its open item). */
  S.undoList = () => {
    const out: Element[] = [];
    (S.baseExp as Map<Element, string | null>).forEach((v, el) => {
      if (el.isConnected && el.getAttribute("aria-expanded") !== v && S.visible(el)) out.push(el);
    });
    S.fix = out;
    return out.length;
  };

  /** Visible dismiss-looking buttons inside whatever is open (a dialog that ignores Escape). */
  S.closerList = () => {
    const out: Element[] = [];
    document.querySelectorAll('[role="dialog"] button, [role="alertdialog"] button, dialog[open] button, [data-state="open"] button').forEach((e) => {
      const t = ((e.getAttribute("aria-label") || "") + " " + (e.textContent || "")).toLowerCase();
      if (/\b(close|cancel|dismiss|done|ok|got it|no thanks)\b/.test(t) && S.visible(e)) out.push(e);
    });
    S.fix = out;
    return out.length;
  };

  /** Resolve (true) once no DOM mutation happened for `ms` ms, capped at `max`. */
  S.quiet = (ms: number, max: number) =>
    new Promise<boolean>((resolve) => {
      let last = performance.now();
      const mo = new MutationObserver(() => {
        last = performance.now();
      });
      mo.observe(document.documentElement, { subtree: true, childList: true, attributes: true, characterData: true });
      const start = performance.now();
      const tick = () => {
        S.settle();
        const now = performance.now();
        if (now - last >= ms) {
          mo.disconnect();
          requestAnimationFrame(() => requestAnimationFrame(() => resolve(true)));
        } else if (now - start >= max) {
          mo.disconnect();
          resolve(false);
        } else {
          setTimeout(tick, 20);
        }
      };
      setTimeout(tick, 20);
    });

  /** Resolve true as soon as `probe()` equals (eq=true) / differs from (eq=false) `target`, false after `ms`. */
  S.waitProbe = (target: string, eq: boolean, ms: number, fn?: string) =>
    new Promise<boolean>((resolve) => {
      const start = performance.now();
      const tick = () => {
        S.settle();
        if ((S[fn || "probe"]() === target) === eq) return resolve(true);
        if (performance.now() - start >= ms) return resolve(false);
        setTimeout(tick, 25);
      };
      tick();
    });

  /** The in-scope elements that have a box right now. */
  S.renderedSet = (): Set<Element> => {
    const out = new Set<Element>();
    for (const el of S.els()) if (el.getClientRects().length > 0) out.add(el);
    return out;
  };

  /** Rendered elements that were NOT rendered at the last `take("base")` and are on their way out: inside a
   *  `data-state="closed"` node, a `[popover]` that is not `:popover-open`, or a `<dialog>` that is not `[open]`.
   *  A non-modal popover stays mounted (display and box intact) for its exit animation / unmount hold, ~300-600 ms,
   *  which no mutation and no probe change marks the end of except the unmount itself. */
  S.closingList = (): Element[] => {
    const base: Set<Element> | undefined = S.baseRendered;
    const out: Element[] = [];
    for (const el of S.els()) {
      if (el.getClientRects().length === 0 || (base && base.has(el))) continue;
      let closing = false;
      try {
        const po = el.closest("[popover]");
        closing = !!el.closest('[data-state="closed"]') || (!!po && !po.matches(":popover-open")) || !!el.closest("dialog:not([open])");
      } catch (_) {
        /* selector unsupported */
      }
      if (closing) out.push(el);
    }
    return out;
  };

  /** Resolve true once nothing that is closing remains rendered AND `probe()` has not changed for `stableMs`
   *  (so a late unmount cannot land inside the next trigger's strategy window); false after `max` ms. */
  S.quiesce = (stableMs: number, max: number) =>
    new Promise<boolean>((resolve) => {
      const start = performance.now();
      let last = "";
      let since = start;
      const tick = () => {
        S.settle();
        const now = performance.now();
        const p = S.probe();
        if (p !== last) {
          last = p;
          since = now;
        }
        if (S.closingList().length === 0 && now - since >= stableMs) return resolve(true);
        if (now - start >= max) return resolve(false);
        setTimeout(tick, 40);
      };
      tick();
    });

  /** What "open" looks like, for telling an OPENING from a late unmount: the elements carrying an open marker
   *  and the elements that are rendered. Opening ADDS to one of the two; a closing overlay only removes. */
  S.openMark = () => {
    const open = new Set<Element>();
    try {
      document.querySelectorAll('[aria-expanded="true"], [data-state="open"], [open], :popover-open, dialog:modal').forEach((e) => open.add(e));
    } catch (_) {
      /* selector unsupported */
    }
    return { open, rendered: S.renderedSet() };
  };
  S.beginTry = () => {
    S.pre = S.openMark();
    return S.probe();
  };
  S.gained = () => {
    const pre = S.pre;
    const cur = S.openMark();
    if (!pre) return true;
    for (const e of cur.open) if (!pre.open.has(e)) return true;
    return cur.rendered.size > pre.rendered.size;
  };
  /** Resolve true once `probe()` differs from `target` AND the difference is an addition (see `gained`); a pure
   *  removal (a previous overlay finishing its unmount) never counts. False after `ms`. */
  S.waitOpen = (target: string, ms: number) =>
    new Promise<boolean>((resolve) => {
      const start = performance.now();
      const tick = () => {
        S.settle();
        if (S.probe() !== target && S.gained()) return resolve(true);
        if (performance.now() - start >= ms) return resolve(false);
        setTimeout(tick, 25);
      };
      tick();
    });

  // ---- records ------------------------------------------------------------------------------------------------
  const rec = (cs: CSSStyleDeclaration, el: Element, props: string[], extra: Record<string, any>) => {
    const r: Record<string, any> = { ...extra, tag: el.tagName.toLowerCase(), cls: Array.from(el.classList).sort().join(" ") };
    for (const p of props) r[p] = cs.getPropertyValue(p);
    return r;
  };

  /** Records for `items`: the element itself and its pseudo-elements. */
  S.records = (items: { el: Element; n: number }[], extra: Record<string, any>) => {
    const out: Record<string, any>[] = [];
    for (const { el, n } of items) {
      const cs = getComputedStyle(el);
      out.push(rec(cs, el, a.props, { ...extra, n }));
      for (const ps of ["::before", "::after"]) {
        const pcs = getComputedStyle(el, ps);
        const content = pcs.content;
        if (content && content !== "none" && content !== "normal") out.push(rec(pcs, el, a.props, { ...extra, n, pseudo: ps }));
      }
      const tag = el.tagName.toLowerCase();
      if (tag === "input" || tag === "textarea") out.push(rec(getComputedStyle(el, "::placeholder"), el, a.colorProps, { ...extra, n, pseudo: "::placeholder" }));
      if (cs.display === "list-item") out.push(rec(getComputedStyle(el, "::marker"), el, a.colorProps, { ...extra, n, pseudo: "::marker" }));
      let top = false;
      try {
        top = el.matches("dialog:modal") || el.matches(":popover-open");
      } catch (_) {
        /* selector unsupported */
      }
      if (top) out.push(rec(getComputedStyle(el, "::backdrop"), el, a.props, { ...extra, n, pseudo: "::backdrop" }));
    }
    return out;
  };

  /** KEYBOARD-INSET pass (row 246). On a phone the on-screen keyboard makes `keyboard_inset.rs` publish
   *  `data-dx-keyboard="open"`, `--dx-keyboard-inset` and `--dx-keyboard-top` on the open modal panel, and the
   *  dialog / sheet / drawer sheets answer with `top`/`bottom`/`max-height` rules. A desktop capture never has a
   *  keyboard, so those rules were invisible to the harness. The CONTRACT is a stylesheet's input, so it can be
   *  supplied by hand: for each open `<dialog>` panel set the three exactly as the script does (a 280px keyboard,
   *  no pan), record every element whose style changed (with the two variables included), then undo. Run
   *  right after the open delta, while `lastDelta` is still the open delta. */
  S.kbdCapture = (extra: Record<string, any>) => {
    const panels: Element[] = (S.lastDelta as { el: Element; n: number }[])
      .map((d) => d.el)
      .filter((el) => el.matches('dialog[open][class*="dx-"]'));
    if (panels.length === 0) return [];
    S.settle();
    const before = new Map<Element, string>();
    for (const el of S.els()) before.set(el, S.sig(el));
    for (const p of panels as HTMLElement[]) {
      p.setAttribute("data-dx-keyboard", "open");
      p.style.setProperty("--dx-keyboard-inset", "280px");
      p.style.setProperty("--dx-keyboard-top", "0px");
    }
    S.settle();
    const props = a.props.concat(["--dx-keyboard-inset", "--dx-keyboard-top"]);
    const out: Record<string, any>[] = [];
    S.els().forEach((el: Element, n: number) => {
      if (before.get(el) !== undefined && before.get(el) === S.sig(el)) return;
      out.push(rec(getComputedStyle(el), el, props, { ...extra, n }));
    });
    for (const p of panels as HTMLElement[]) {
      p.removeAttribute("data-dx-keyboard");
      p.style.removeProperty("--dx-keyboard-inset");
      p.style.removeProperty("--dx-keyboard-top");
    }
    S.settle();
    return out;
  };

  /** Elements new since baseline `which` or whose signature changed. */
  S.delta = (which: string) => {
    S.settle();
    const base: Map<Element, string> = S[which];
    const out: { el: Element; n: number }[] = [];
    S.els().forEach((el: Element, n: number) => {
      const s = base.get(el);
      if (s === undefined || s !== S.sig(el)) out.push({ el, n });
    });
    S.lastDelta = out;
    return out;
  };
  /** Everything visibly different from baseline `which`? (ignores nodes that are new but not rendered) */
  S.restored = (which: string) => {
    S.settle();
    const base: Map<Element, string> = S[which];
    for (const el of S.els()) {
      const s = base.get(el);
      if (s === undefined) {
        if (S.visible(el)) return false;
      } else if (s !== S.sig(el)) {
        return false;
      }
    }
    return true;
  };

  // ---- target discovery ---------------------------------------------------------------------------------------
  const FOCUSABLE =
    'button, a[href], input, select, textarea, summary, [role="button"], [role="combobox"], [tabindex]:not([tabindex="-1"])';
  const HOVERABLE =
    'a[href], button, input, select, textarea, summary, label, tr, [role="button"], [role="tab"], [role="menuitem"], [role="menuitemcheckbox"], [role="menuitemradio"], [role="option"], [role="switch"], [role="checkbox"], [role="radio"], [role="slider"], [role="link"], [role="row"], [role="treeitem"], [role="combobox"], [tabindex], [draggable="true"]';
  /** `[tabindex]` alone also matches big focus containers (tab panels, scroll areas, the carousel): hovering
   *  those is arbitrary (where in a 1000 px panel?), so a tabindex-only element must be a leaf. */
  const isControl = (e: Element) =>
    e.matches(HOVERABLE.replace(", [tabindex]", "")) || (e.matches("[tabindex]") && e.children.length === 0);

  /** Page chrome that can show up in an overlay's delta but is never one of its items. */
  const ITEM_EXCLUDE = ".dx-skip-link";

  const short = (el: Element) => {
    const cls = Array.from(el.classList).sort().join(".");
    const text = (el.textContent || "").replace(/\s+/g, " ").trim().slice(0, 24);
    return el.tagName.toLowerCase() + (cls ? "." + cls : "") + (text ? ` "${text}"` : "");
  };
  const byDoc = (x: Element, y: Element) => (x.compareDocumentPosition(y) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1);
  const inScope = (el: Element) => S.roots().some((r: Element) => r.contains(el));

  S.scanCands = (plain: boolean): Element[] => {
    const set = new Set<Element>();
    const add = (e: Element | null) => {
      if (!e || !inScope(e) || !S.visible(e)) return;
      if ((e as HTMLButtonElement).disabled || e.getAttribute("aria-disabled") === "true") return;
      if (e.getAttribute("role") === "tab") return; // tabs: covered by the hover pass, not an overlay
      // a plain link would navigate away when clicked (hover/focus are still tried on it by the others)
      const href = e.getAttribute("href");
      if (e.tagName === "A" && href && href.charAt(0) !== "#" && !e.getAttribute("aria-haspopup")) return;
      set.add(e);
    };
    for (const r of S.roots() as Element[]) {
      // 1. elements that declare they open something
      r.querySelectorAll('[aria-haspopup]:not([aria-haspopup="false"]), [aria-expanded], [popovertarget], summary').forEach((e) => {
        const tag = e.tagName.toLowerCase();
        // already expanded at rest: that IS the rest state (captured by `base`), clicking would collapse it
        if (e.getAttribute("aria-expanded") === "true") return;
        if ((tag === "a" && e.getAttribute("href")) || (tag === "input" && !e.getAttribute("aria-haspopup") && e.getAttribute("aria-expanded") === null)) return;
        add(e);
      });
      // 2. tooltip-style triggers (aria-describedby is also used for field help text, hence the class filter)
      r.querySelectorAll("[aria-describedby]").forEach((e) => {
        if (/tooltip|hover-card|popover|trigger/.test(String((e as any).className?.baseVal ?? e.className))) add(e);
      });
      // 3. `-trigger` classed controls (accordion/collapsible/navigation/menubar/hover-card triggers)
      r.querySelectorAll('[class*="-trigger"]').forEach((e) => {
        if (e.matches(FOCUSABLE) || e.getAttribute("role") === "button") add(e);
      });
      // 4. a closed overlay root that is itself not focusable: its first focusable descendant is the trigger
      r.querySelectorAll('[data-state="closed"]').forEach((e) => {
        if (e.matches(FOCUSABLE)) return add(e);
        add(e.querySelector(FOCUSABLE));
      });
      // 5. pages whose triggers are plain buttons
      if (plain) r.querySelectorAll('button, [role="button"]').forEach((e) => add(e));
    }
    // a wrapper that contains a more specific trigger is not itself the trigger (`[data-state=closed]` roots)
    const all = Array.from(set);
    return all.filter((e) => !all.some((o) => o !== e && e.contains(o))).sort(byDoc);
  };

  S.listCands = (plain: boolean) => {
    S.plain = plain;
    const list: Element[] = S.scanCands(plain);
    S.cands = list.slice(0, 30);
    S.candDesc = S.cands.map(short);
    const capped = list.length - S.cands.length;
    return {
      capped,
      cands: S.cands.map((e: Element, i: number) => ({
        i,
        tag: e.tagName.toLowerCase(),
        role: e.getAttribute("role") || "",
        hp: e.getAttribute("aria-haspopup") || "",
        cls: String((e as any).className?.baseVal ?? e.className),
        text: (e.textContent || "").replace(/\s+/g, " ").trim().slice(0, 24),
        desc: short(e),
        hoverFirst: e.hasAttribute("aria-describedby") || /tooltip|hover-card/.test(String(e.className)),
        ctxFirst: e.getAttribute("aria-haspopup") === "menu" && e.tagName.toLowerCase() !== "button",
      })),
    };
  };

  /** Distinct-by-style-selectors signature of a hover target (digits masked so `2026-10-12` == `2026-10-13`). */
  const hoverSig = (el: Element): string => {
    // charts: every plot is its own target (its tooltip is configured per chart), never merged by class
    if (S.chartSvgs && S.chartSvgs.indexOf(el) >= 0) return `chart#${S.chartSvgs.indexOf(el)}`;
    const attrs = Array.from(el.attributes)
      .filter((at) => /^(data-|aria-(checked|selected|expanded|pressed|disabled|current|orientation|haspopup|invalid|sort|readonly|required))/.test(at.name) && at.name !== "data-dioxus-id")
      .map((at) => at.name + "=" + at.value.replace(/\d+/g, "#"))
      .sort()
      .join(",");
    const p = el.parentElement;
    return [
      el.tagName, String((el as any).className?.baseVal ?? el.className), el.getAttribute("role") || "", el.getAttribute("type") || "",
      (el as any).disabled ? "d" : "", attrs, p ? String(p.className?.baseVal ?? p.className) : "",
      el.matches(":first-child") ? "f" : "", el.matches(":last-child") ? "l" : "",
    ].join("|");
  };

  /** Hovering it may CHANGE the open state after a delay (a submenu, another menubar menu, a navigation-menu
   *  panel): it declares a popup, an expanded state, or is a `-trigger`. */
  const popup = (e: Element) => {
    const hp = e.getAttribute("aria-haspopup");
    return (!!hp && hp !== "false") || e.hasAttribute("aria-expanded") || /-trigger/.test(String((e as any).className?.baseVal ?? e.className));
  };

  /** Distinct hover targets among `pool` (document order), first MAX of each distinct signature. */
  const pickHover = (pool: Element[], max: number, exclude: Element | null) => {
    const seen = new Set<string>();
    const out: Element[] = [];
    let capped = 0;
    for (const el of pool.sort(byDoc)) {
      if (el === exclude || !S.visible(el)) continue;
      if (el.matches("html, body")) continue;
      const s = hoverSig(el);
      if (seen.has(s)) continue;
      seen.add(s);
      if (out.length >= max) capped++;
      else out.push(el);
    }
    return { out, capped };
  };

  S.listHover = (max: number) => {
    const pool: Element[] = [];
    S.chartSvgs = [];
    for (const r of S.roots() as Element[]) {
      r.querySelectorAll(HOVERABLE).forEach((e) => {
        if (isControl(e)) pool.push(e);
      });
      // chart plots: the tooltip appears when the pointer is over the plot, not over a control
      r.querySelectorAll('[class*="dx-chart"] svg[role="img"], svg[role="img"][class*="dx-chart"]').forEach((e) => {
        S.chartSvgs.push(e);
        pool.push(e);
      });
    }
    const { out, capped } = pickHover(pool, max, null);
    S.hov = out;
    return { n: out.length, capped, descs: out.map(short), pop: out.map(popup) };
  };

  /** Hover targets INSIDE the freshly opened overlay (elements of the last delta). */
  S.listItems = (max: number, trigger: number) => {
    const pool: Element[] = [];
    const trig = S.cands[trigger];
    for (const { el } of S.lastDelta as { el: Element; n: number }[]) {
      // `.dx-skip-link` is page chrome, not part of any overlay: it is in the delta because its style changes
      // when focus moves (row 245), sits off screen (`occluded`) and used to take hover slots.
      if (el !== trig && el.matches(HOVERABLE) && isControl(el) && !el.closest(ITEM_EXCLUDE)) pool.push(el);
    }
    const picked = pickHover(pool, max, trig);
    // Items that open something on hover (a submenu, after its hover-intent delay) go last: opening one
    // changes the overlay's state, which ends the item loop.
    const out = picked.out.filter((e) => !popup(e)).concat(picked.out.filter(popup));
    S.items = out;
    return { n: out.length, capped: picked.capped, descs: out.map(short), pop: out.map(popup) };
  };

  /** Scroll target `arr[j]` into view and report where to put the pointer. */
  S.prep = (arr: string, j: number) => {
    // the framework may have re-created a trigger while its overlay was open: find the same one again (same
    // description, same ordinal among equal descriptions) WITHOUT renumbering the list
    if (arr === "cands" && S.cands[j] && !S.cands[j].isConnected) {
      const fresh: Element[] = S.scanCands(S.plain).filter((e: Element) => short(e) === S.candDesc[j]);
      let k = 0;
      for (let q = 0; q < j; q++) if (S.candDesc[q] === S.candDesc[j]) k++;
      if (fresh[k]) S.cands[j] = fresh[k];
    }
    const el: Element | undefined = S[arr][j];
    if (!el || !el.isConnected) return { ok: false, reason: "detached" };
    el.scrollIntoView({ block: "center", inline: "center", behavior: "instant" } as ScrollIntoViewOptions);
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return { ok: false, reason: "no-box" };
    const x = Math.min(Math.max(r.left + r.width / 2, 1), window.innerWidth - 2);
    const y = Math.min(Math.max(r.top + r.height / 2, 1), window.innerHeight - 2);
    const hit = document.elementFromPoint(x, y);
    if (!hit || !(el === hit || el.contains(hit))) return { ok: false, reason: "occluded" };
    return { ok: true, x, y };
  };

  /** Scrolling a target into view can change styles (a navbar that gains a shadow once scrolled, in-view
   *  reveals), and the deltas must not contain that: when the scroll position moved since the baseline was
   *  taken, wait for the page to go quiet there and take the baseline again. */
  S.rebase = () => {
    if (S.bscroll === S.scrollKey()) return Promise.resolve(false);
    return S.quiet(100, 1500).then(() => {
      S.take("base");
      return true;
    });
  };

  S.blur = () => {
    const ae = document.activeElement as HTMLElement | null;
    if (ae && ae !== document.body && ae.blur) ae.blur();
  };

  /** Hover capture of `arr[j]` against baseline `which`; verifies the pointer really is over it. */
  S.hoverCapture = (arr: string, j: number, which: string, extra: Record<string, any>) => {
    const el: Element = S[arr][j];
    if (!el.isConnected) return { hovered: false, recs: [], changed: false };
    const hovered = el.matches(":hover");
    const items = S.delta(which);
    return { hovered, recs: hovered ? S.records(items, extra) : [], changed: S.cprobe() !== S.cprobe0 };
  };
}

// ---------------------------------------------------------------------------------------------------------------
// Node side.
// ---------------------------------------------------------------------------------------------------------------

type Ctx = {
  page: Page;
  args: StateArgs;
  plain: boolean;
  url: string;
  key: string;
  cov: StateCoverage;
  verbose: boolean;
};

/** Move the pointer onto a target and wait out every timer the move schedules (the TIMER-DRAIN RULE in
 *  `installStateLib`): a popup's configured open delay is then over before anything is read. */
async function movePointer(page: Page, x: number, y: number) {
  await page.evaluate(() => (window as any).__snap.arm());
  await page.mouse.move(x, y);
  await page.evaluate(([min, max]) => (window as any).__snap.drain(min, max), [DRAIN_MIN_MS, DRAIN_MAX_MS] as const);
}

async function park(page: Page) {
  const vs = page.viewportSize() ?? { width: 1280, height: 720 };
  await page.mouse.move(vs.width - 1, Math.floor(vs.height / 2));
}

async function initPage(c: Ctx, fresh: boolean) {
  const { page } = c;
  if (fresh) await gotoSettled(page, c.url);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.evaluate(installStateLib, c.args);
  await page.evaluate(() => {
    const S = (window as any).__snap;
    S.forceTheme();
    S.blur();
  });
  await park(c.page);
  await page.evaluate(() => (window as any).__snap.quiet(150, 3000));
  await page.evaluate(() => {
    const S = (window as any).__snap;
    S.take("base");
  });
}

/** Everything is closed again (the page's open-marker counts equal the baseline's), or the page is reloaded
 *  to make it so. Escalation, cheapest first: move the pointer away (closes hover-opened overlays), Escape
 *  (with focus still inside the overlay: its handler lives there), click back every toggle whose
 *  `aria-expanded` differs from the baseline (a single-open accordion), click a dismiss-looking button inside
 *  whatever is open (a dialog that ignores Escape), Escape again; reload. Returns false if it had to reload. */
async function ensureClosed(c: Ctx, via: string): Promise<boolean> {
  const { page } = c;
  const closed = (ms = 1200) =>
    page.evaluate((m) => (window as any).__snap.waitProbe((window as any).__snap.cprobe0, true, m, "cprobe"), ms);
  const clickAll = async (fn: "undoList" | "closerList") => {
    const n: number = await page.evaluate((f) => (window as any).__snap[f](), fn);
    for (let j = 0; j < Math.min(n, 3); j++) {
      const t = await page.evaluate((k) => (window as any).__snap.prep("fix", k), j);
      if (t.ok) await page.mouse.click(t.x!, t.y!);
      await page.evaluate(() => (window as any).__snap.quiet(80, 800));
    }
    return n;
  };
  await park(page);
  // Moving the pointer away is what closes a hover-opened overlay; for the others it is a short courtesy wait.
  let ok = await closed(via === "hover" ? 1200 : 100);
  if (!ok) {
    // NOT blurred first: the overlay's Escape handler lives on the focused element inside it.
    await page.keyboard.press("Escape");
    ok = await closed(500);
  }
  if (!ok && (await clickAll("undoList")) > 0) {
    await park(page);
    ok = await closed(500);
  }
  if (!ok && (await clickAll("closerList")) > 0) {
    await park(page);
    ok = await closed(800);
  }
  if (!ok) {
    await page.keyboard.press("Escape");
    ok = await closed(800);
  }
  await page.evaluate(() => (window as any).__snap.blur());
  if (ok) {
    // Closed does not mean gone: a non-modal popover stays mounted for its exit animation / unmount hold. Wait
    // for it to unmount (and the page to stop changing) BEFORE the baseline is re-taken, or its late unmount
    // lands in the next trigger's strategy window (and the baseline would contain it).
    await page.evaluate(() => (window as any).__snap.quiesce(150, 1500));
    await page.evaluate(() => (window as any).__snap.quiet(100, 2000));
    // Closed, but maybe not the page it was: a toggled control, nodes the overlay left behind. Re-take the
    // baseline so the next target is measured against what is really there now (the order of targets is fixed,
    // so this is deterministic).
    const restored = await page.evaluate(() => (window as any).__snap.restored("base"));
    await page.evaluate(() => (window as any).__snap.take("base"));
    if (!restored && c.verbose) console.log(`[snapshot]   ${c.key}: closed but not restored, baseline re-taken`);
    return true;
  }
  c.cov.reloads++;
  if (c.verbose) {
    const pr = await page.evaluate(() => [(window as any).__snap.cprobe0, (window as any).__snap.cprobe()]);
    console.log(`[snapshot]   ${c.key}: could not close (open markers ${pr[0]} -> ${pr[1]}), reloading`);
  }
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => Array.from(document.images).every((i) => i.complete), null, { timeout: 30000 });
  await page.waitForTimeout(1500);
  await initPage(c, false);
  await page.evaluate(([plain]) => (window as any).__snap.listCands(plain), [c.plain] as const);
  await page.evaluate((max) => (window as any).__snap.listHover(max), MAX_HOVER_TARGETS);
  return false;
}

/** HOVER pass: hover each distinct interactive control of the rest-state page and record what changed. */
export async function hoverPass(
  page: Page,
  url: string,
  key: string,
  args: StateArgs,
  cov: StateCoverage,
  plain: boolean,
  verbose: boolean,
): Promise<Rec[]> {
  const c: Ctx = { page, args, plain, url, key, cov, verbose };
  const out: Rec[] = [];
  await initPage(c, false);
  const list = await page.evaluate((max) => (window as any).__snap.listHover(max), MAX_HOVER_TARGETS);
  cov.hoverTargets += list.n;
  cov.hoverCapped += list.capped;
  const skip = (why: string) => (cov.hoverSkipped[why] = (cov.hoverSkipped[why] ?? 0) + 1);
  for (let j = 0; j < list.n; j++) {
    const t = await prepRetry(page, "hov", j);
    if (!t.ok) {
      skip(t.reason!);
      continue;
    }
    await page.evaluate(() => (window as any).__snap.rebase());
    await movePointer(page, t.x!, t.y!);
    await settleAfterHover(page, list.pop[j], 120, "probe0");
    const r = await page.evaluate(
      ([k, desc]) => (window as any).__snap.hoverCapture("hov", k, "base", { state: `hover:${k}`, target: desc }),
      [j, list.descs[j]] as const,
    );
    if (!r.hovered) {
      skip("not-hovered");
    } else {
      cov.hoverCaptured++;
      out.push(...(r.recs as Rec[]));
    }
    await park(page);
    if (r.changed) {
      // hovering opened something (tooltip, hover card, menubar menu): close it before the next target
      await ensureClosed(c, "hover");
    } else {
      await page.evaluate(() => (window as any).__snap.blur());
    }
  }
  cov.hoverRecords += out.length;
  return out;
}

/** After the pointer arrived on a target: wait for the page to go quiet. A target that opens something on hover
 *  (`aria-haspopup`: a submenu opens after a 200 ms hover-intent delay, menu_sub.rs) is first given time to open,
 *  otherwise the read would race the timer. */
async function settleAfterHover(page: Page, opensSomething: boolean, quietMs: number, ref: "probe0" | "probeOpen") {
  if (opensSomething) {
    await page.evaluate((r) => (window as any).__snap.waitProbe((window as any).__snap[r], false, 900), ref);
  }
  await page.evaluate((q) => (window as any).__snap.quiet(q, 1500), opensSomething ? 200 : quietMs);
}

/** `prep`, retried once after the page went quiet: a target that is momentarily under something that is still
 *  animating or laying out must not be skipped in one run and captured in the next. */
async function prepRetry(page: Page, arr: string, j: number) {
  let t = await page.evaluate(([a, k]) => (window as any).__snap.prep(a, k), [arr, j] as const);
  if (!t.ok && t.reason === "occluded") {
    await page.evaluate(() => (window as any).__snap.quiet(150, 1500));
    t = await page.evaluate(([a, k]) => (window as any).__snap.prep(a, k), [arr, j] as const);
  }
  return t;
}

/** HOVER-DELAY RULE (backlog: the hover_card opener race; row 245). A hover card or tooltip opens only after the
 *  pointer has RESTED on its trigger for the component's configured `delay` (600 ms default in
 *  `primitives/src/hover_card.rs` and `tooltip.rs`, any other value per demo). The harness used to guess that
 *  delay with a fixed wait (450, then 1050 ms), and a guess is a race: a tooltip chain rendered in 1 of 4
 *  captures, and earlier two captures recorded `via: click` vs `via: focus` for one trigger.
 *  Now the delay is not guessed at all: `movePointer` arms the TIMER-DRAIN RULE (`installStateLib`) before the
 *  pointer move and waits until every timeout the move scheduled has fired, so the opener waits out whatever
 *  delay the popup is configured with (the delays demos use 0, 10, 100, 600). The numbers below are only the
 *  short margin `waitOpen` then allows for the render after the timer, plus the click/contextmenu/focus
 *  strategies' own windows. A trigger that does not open on hover pays its configured delay (not more) and
 *  falls through to click/focus exactly as before. */
const WAIT_PROBE_MS: Record<string, number> = {
  click: 350,
  contextmenu: 350,
  hover: 450,
  focus: 300,
};

/** HOVER-BEFORE-CLICK RULE (backlog: navbar hover-intent). A trigger may open on pointer hover (navbar, hover
 *  cards, ...: `primitives/src/hover_intent.rs`, `HOVER_OPEN_INTENT_DELAY` = 150 ms). A bare `mouse.click` first
 *  moves the pointer onto the trigger, so that panel opens on the hover and the click that follows TOGGLES IT
 *  CLOSED: the open pass then saw "no change" for that trigger (S2: 2 of 6 navbar triggers produced open-state
 *  records). So before the `click` strategy the pointer is put on the trigger and given this long (> the
 *  hover-intent delay plus the render) to open something, judged by the same `waitOpen` (additions-only) as every
 *  other strategy. If it opened, the trigger is recorded as opened `via: "hover"` and NOT clicked; otherwise the
 *  click goes ahead exactly as before. The wait is skipped when the `hover` strategy already ran for this
 *  trigger (`hoverFirst`: tooltips), so those are unchanged. A trigger that does not open on hover is unaffected
 *  apart from this pause: the click, the park, the quiesce and the delta follow in the same order. */
const HOVER_INTENT_WAIT_MS = 450;

/** OPEN-STATE pass: put each trigger in its open state, record the delta, hover the items inside, close. */
export async function openPass(
  page: Page,
  url: string,
  key: string,
  args: StateArgs,
  cov: StateCoverage,
  plain: boolean,
  verbose: boolean,
): Promise<Rec[]> {
  const c: Ctx = { page, args, plain, url, key, cov, verbose };
  const out: Rec[] = [];
  await initPage(c, false);
  const found = await page.evaluate(([p]) => (window as any).__snap.listCands(p), [plain] as const);
  const cands = found.cands as Cand[];
  cov.candidates += cands.length;
  const one = async (i: number) => {
    const cd = cands[i];
    const order = ["click"];
    if (cd.ctxFirst) order.unshift("contextmenu");
    if (cd.hoverFirst) order.unshift("hover");
    for (const s of ["contextmenu", "hover", "focus"]) if (order.indexOf(s) < 0 && (s !== "contextmenu" || cd.ctxFirst || cd.hp === "menu")) order.push(s);

    let via = "";
    let lastReason = "";
    // set when the pre-click hover (HOVER_INTENT_WAIT_MS rule) opened the panel: the pointer is still on the
    // trigger and the panel is open, so the items inside are hovered like after a click-open
    let hoverIntentOpened = false;
    const tried = new Set<string>();
    for (const strat of order) {
      tried.add(strat);
      const t = await prepRetry(page, "cands", i);
      if (!t.ok) {
        lastReason = t.reason!;
        break;
      }
      await page.evaluate(() => (window as any).__snap.rebase());
      // Belt and braces for the first strategy: nothing from the previous overlay may still be unmounting.
      await page.evaluate(() => (window as any).__snap.quiesce(0, 1500));
      const before = await page.evaluate(() => (window as any).__snap.beginTry());
      if (strat === "click") {
        if (!tried.has("hover")) {
          // HOVER_INTENT_WAIT_MS rule: hover first; if that opened the panel, do not click it shut
          await movePointer(page, t.x!, t.y!);
          if (await page.evaluate(([b, ms]) => (window as any).__snap.waitOpen(b, ms), [before, HOVER_INTENT_WAIT_MS] as const)) {
            via = "hover";
            hoverIntentOpened = true;
            break;
          }
        }
        await page.mouse.click(t.x!, t.y!);
      } else if (strat === "contextmenu") await page.mouse.click(t.x!, t.y!, { button: "right" });
      else if (strat === "hover") await movePointer(page, t.x!, t.y!);
      else await page.evaluate((j) => (window as any).__snap.cands[j].focus({ focusVisible: true }), i);
      // The overlay opens UNDER the pointer, and which of its nodes is `:hover` then depends on when the
      // renderer next hit-tests (a calendar cell was hovered in one run and not in the next). Only a hover
      // strategy needs the pointer to stay; for the others put it on the inert spot straight away.
      if (strat !== "hover") await park(page);
      const changed = await page.evaluate(([b, ms]) => (window as any).__snap.waitOpen(b, ms), [before, WAIT_PROBE_MS[strat]] as const);
      if (changed) {
        via = strat;
        break;
      }
      lastReason = `${lastReason ? lastReason + "," : ""}${strat}:no-change`;
      await park(page);
      await page.evaluate(() => (window as any).__snap.blur());
      await page.evaluate(() => (window as any).__snap.quiet(60, 800));
    }
    if (!via) {
      cov.unopened.push({ desc: cd.desc, reason: lastReason });
      return;
    }
    cov.opened++;
    cov.via[via] = (cov.via[via] ?? 0) + 1;

    await page.evaluate(() => (window as any).__snap.quiet(300, 3000));
    const extra = { state: `open:${i}`, via, target: cd.desc };
    const recs = (await page.evaluate(
      ([e]) => (window as any).__snap.records((window as any).__snap.delta("base"), e),
      [extra] as const,
    )) as Rec[];
    out.push(...recs);
    cov.openRecords += recs.length;
    // the open modal panel as the on-screen keyboard shows it (row 246; `kbdCapture`)
    const krecs = (await page.evaluate(
      ([e]) => (window as any).__snap.kbdCapture(e),
      [{ state: `open:${i}/kbd`, via, target: cd.desc }] as const,
    )) as Rec[];
    if (krecs.length) {
      await page.evaluate(() => (window as any).__snap.quiet(100, 1500));
      out.push(...krecs);
      cov.openRecords += krecs.length;
      cov.kbdRecords += krecs.length;
    }

    // hover the items inside the opened overlay (menu items, options, links, buttons)
    if (via !== "hover" || hoverIntentOpened) {
      await page.evaluate(() => {
        const S = (window as any).__snap;
        S.take("obase");
        S.probeOpen = S.probe();
      });
      const items = await page.evaluate(([m, k]) => (window as any).__snap.listItems(m, k), [MAX_ITEM_HOVERS, i] as const);
      for (let j = 0; j < items.n; j++) {
        const t = await prepRetry(page, "items", j);
        if (!t.ok) {
          cov.itemHoverSkipped++;
          continue;
        }
        await movePointer(page, t.x!, t.y!);
        await settleAfterHover(page, items.pop[j], 150, "probeOpen");
        const r = await page.evaluate(
          ([k, i0, desc]) => (window as any).__snap.hoverCapture("items", k, "obase", { state: `open:${i0}/hover:${k}`, via: "hover", target: desc }),
          [j, i, items.descs[j]] as const,
        );
        if (!r.hovered) {
          cov.itemHoverSkipped++;
          continue;
        }
        cov.itemHovers++;
        cov.openRecords += r.recs.length;
        out.push(...(r.recs as Rec[]));
        const still = await page.evaluate(() => (window as any).__snap.probe() === (window as any).__snap.probeOpen);
        if (!still) {
          // hovering an item changed the open state (a submenu opened, or the menu closed): stop; ensureClosed copes
          break;
        }
      }
    }

    const reloaded = !(await ensureClosed(c, via));
    if (reloaded && verbose) console.log(`[snapshot]   ${key}: reloaded after trigger ${i} (${cd.desc})`);
  };
  for (let i = 0; i < cands.length; i++) {
    try {
      await one(i);
    } catch (e) {
      if (!/context was destroyed|navigat/i.test(String(e))) throw e;
      // the click left the page (a link, a form submit): come back and rebuild the lists, same order
      cov.unopened.push({ desc: cands[i].desc, reason: "navigates-away" });
      cov.reloads++;
      await initPage(c, true);
      await page.evaluate(([p]) => (window as any).__snap.listCands(p), [plain] as const);
      await page.evaluate((max) => (window as any).__snap.listHover(max), MAX_HOVER_TARGETS);
    }
  }
  if (found.capped > 0) cov.unopened.push({ desc: `(+${found.capped} more triggers beyond the cap of ${MAX_TRIGGERS})`, reason: "capped" });
  return out;
}

