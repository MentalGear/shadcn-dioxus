// The page executor (row 449): the one writer of the open state of every element whose Rust owner renders a level.
// Plumbing only: Rust decides the level and renders it, this module applies it through the apply table and reports
// what the page did on its own. No decision is made here.
//
//   level      an owner renders `data-dx-level="<nonce> <open|closed> <immediate|mounted> <site>"` on its element:
//              the nonce is the owner's command count, so the server's HTML and the first client render agree
//   identity   the applied nonce is kept per element node: a re-created element starts fresh, and a level is
//              applied once, when its nonce is newer than the one applied, so a page-caused close stands until the
//              owner decides again
//   in order   `execute` runs in the eval Rust issues after the render that changed a level, so a show lands before
//              a write the owner makes after it in the same effect pass (a focus moved into the shown panel)
//   one writer the show and hide go through `apply` (the `show` / `hide` rows of APPLY) and nothing else shows or
//              hides an element with a level; the transition the write itself causes is not reported
//   reports    a transition the page caused (light dismiss, Escape) is reported as `{ n, open }` on the session
//              of the element's owner (`watch`, by the element's id) from its `toggle` task, stamped with the nonce applied, so Rust
//              drops a report older than its newest level; two document capture listeners carry them while any
//              session watches

/** What the bridge gives the executor at boot: its apply table and the Rust-caused marker. */
export interface ExecutorPorts {
  apply(site: string, effect: "show" | "hide", el: Element): void;
  asRust<T>(run: () => T): T;
}

type Watch = Extract<DxCommand, { cmd: "executor" }>;
type Applied = { n: number; own: number; page: boolean };

export function createExecutor(W: Window, ports: ExecutorPorts) {
  const applied = new WeakMap<Element, Applied>();
  const sessions = new Map<string, DxScope>();

  const write = (state: Applied, el: Element, site: string, effect: "show" | "hide"): void => {
    state.own++;
    try {
      ports.asRust(() => ports.apply(site, effect, el));
    } catch (error) {
      console.warn("dx executor:", String(error));
    } finally {
      state.own--;
    }
  };

  // Who caused a transition is read in the synchronous `beforetoggle` (inside the executor's own write or not); the
  // report goes out from the `toggle` task the browser queues after it, as the platform reports the change, so a
  // trigger's own click in the same gesture reaches Rust first and the report is then older than its level.
  const onBefore = (event: Event): void => {
    const state = applied.get(event.target as Element);
    if (state && !state.own) state.page = true;
  };
  const onToggle = (event: Event): void => {
    const el = event.target as Element;
    const state = applied.get(el);
    const sc = sessions.get(el.id);
    if (!state || !state.page) return;
    state.page = false;
    if (sc) sc.send({ n: state.n, open: (event as ToggleEvent).newState === "open" });
  };
  const listening = (on: boolean): void => {
    const doc = W.document;
    // listener-ok: added with the first watching session and removed with the last
    for (const [type, fn] of [["beforetoggle", onBefore], ["toggle", onToggle]] as const) on ? doc.addEventListener(type, fn, true) : doc.removeEventListener(type, fn, true);
  };

  return {
    /** Applies every rendered level newer than the one applied to its element, in the calling turn. */
    execute(): void {
      for (const el of W.document.querySelectorAll("[data-dx-level]")) {
        const [nonce, level, mode, site] = (el.getAttribute("data-dx-level") ?? "").split(" ");
        const n = Number(nonce);
        let state = applied.get(el);
        if (!state) applied.set(el, (state = { n: 0, own: 0, page: false }));
        if (!(n > state.n)) continue;
        state.n = n;
        const shown = el.matches(":popover-open");
        if (level === "open" && !shown) write(state, el, site, "show");
        else if (level === "closed" && shown && mode === "immediate") write(state, el, site, "hide");
      }
    },
    /** Sends the page-caused transitions of the element `target` to the session of its owner until it ends. */
    watch(sc: DxScope, spec: Watch): void {
      if (!sessions.size) listening(true);
      sessions.set(spec.target, sc);
      sc.cleanup(() => {
        if (sessions.get(spec.target) === sc) sessions.delete(spec.target);
        if (!sessions.size) listening(false);
      });
    },
  };
}
