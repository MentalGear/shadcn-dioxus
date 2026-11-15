// The page executor (row 449): the one writer of the open state of every element a Rust owner registers with an
// `executor` command. Plumbing only: Rust decides the level, this module applies it through the apply table and
// reports what the page did on its own. No decision is made here.
//
//   identity   an entry is keyed by the element's `id` (rendered by the server, so the same before and after
//              hydration) and carries the nonce of the newest command applied to it: the owner's command count
//   epoch      a command older than the entry's nonce is dropped, so a late command never undoes a newer one
//   one writer the entry's show and hide go through `apply` (the `show` / `hide` rows of APPLY) and nothing else
//              shows or hides a registered element; the transition the write itself causes is not reported
//   reports    a transition the page caused (light dismiss, Escape) is reported as `{ n, open }`: the level, stamped
//              with the nonce the entry had applied, so Rust drops a report older than its newest command
//   discovery  a command for an element that is not in the document yet waits for it on one `childList` observer
//              and runs when it appears (the owner's effect can run before a keyed re-render lands)

/** What the bridge gives the executor at boot: its apply table and the Rust-caused marker. */
export interface ExecutorPorts {
  apply(site: string, effect: "show" | "hide", el: Element): void;
  asRust<T>(run: () => T): T;
}

type ExecutorCommand = Extract<DxCommand, { cmd: "executor" }>;

type Entry = { sc: DxScope; n: number; level: boolean; mode: ExecutorCommand["mode"]; site: string; own: number };

export function createExecutor(W: any, ports: ExecutorPorts) {
  const entries = new Map<string, Entry>();
  const waiting = new Map<string, Entry>();
  let observer: { disconnect(): void } | null = null;

  const write = (entry: Entry, el: Element, effect: "show" | "hide"): void => {
    entry.own++;
    try {
      ports.asRust(() => ports.apply(entry.site, effect, el));
    } catch (error) {
      console.warn("dx executor:", String(error));
    } finally {
      entry.own--;
    }
  };

  const place = (id: string, entry: Entry): void => {
    const el = W.document.getElementById(id);
    if (!el) {
      waiting.set(id, entry);
      if (!observer) {
        const mo = new W.MutationObserver(() => {
          for (const [wid, wentry] of [...waiting]) if (W.document.getElementById(wid)) waiting.delete(wid), place(wid, wentry);
          if (!waiting.size && observer) observer.disconnect(), (observer = null);
        });
        mo.observe(W.document.documentElement, { childList: true, subtree: true });
        observer = mo;
      }
      return;
    }
    waiting.delete(id);
    const shown = !!el.matches && el.matches(":popover-open");
    if (entry.level && !shown) write(entry, el, "show");
    else if (!entry.level && shown && entry.mode === "immediate") write(entry, el, "hide");
  };

  // listener-ok: page-lifetime capture listener of the executor, installed once at boot
  W.document.addEventListener(
    "beforetoggle",
    (event: any) => {
      const id = event.target && event.target.id;
      const entry = id ? entries.get(id) : undefined;
      if (entry && !entry.own) entry.sc.send({ n: entry.n, open: event.newState === "open" });
    },
    true,
  );

  return {
    command(sc: DxScope, spec: ExecutorCommand): void {
      let entry = entries.get(spec.target);
      if (!entry || entry.sc !== sc) {
        const made: Entry = { sc, n: 0, level: false, mode: spec.mode, site: spec.site, own: 0 };
        entries.set(spec.target, made);
        sc.cleanup(() => {
          if (entries.get(spec.target) === made) entries.delete(spec.target);
          if (waiting.get(spec.target) === made) waiting.delete(spec.target);
        });
        entry = made;
      }
      if (spec.nonce < entry.n) return;
      entry.n = spec.nonce;
      entry.level = !!spec.level;
      entry.mode = spec.mode;
      entry.site = spec.site;
      place(spec.target, entry);
    },
  };
}
