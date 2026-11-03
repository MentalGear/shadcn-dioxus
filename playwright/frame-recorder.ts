/**
 * A recorder of what the page would show, from the first frame of a navigation on.
 *
 * A spec that asserts "no frame ever showed X" while a document loads and hydrates installs its sampler before any page
 * script runs. A raw `requestAnimationFrame` loop in an `addInitScript` reads the DOM while the HTML parser is still
 * inside the subject: a streamed document ends a chunk between an avatar's `<img>` and its fallback sibling, and a
 * loaded machine, which parses in more and smaller steps, shows a frame with the half-built subtree that no hydrated
 * page ever has. So the sampler gets a `parsed(el)` predicate, true once the parser has moved past `el` (the document is
 * parsed, or `el` or an ancestor has a following sibling), and returns `null` for a state it must not judge, typically
 * `parsed(subject) ? state : null`.
 *
 *     await installFrameRecorder(page, "__navFrames", ({ parsed }) => {
 *       const el = document.querySelector(".x");
 *       return el && parsed(el) ? { shown: getComputedStyle(el).display } : null;
 *     });
 *     await gotoHydrated(page, url);
 *     const frames = await readFrames<{ shown: string }>(page, "__navFrames");
 *
 * It samples on every animation frame, which is what a style-based claim ("no frame painted the A-Z list") is about.
 * `{ perTask: true }` also samples at the end of every task that changed the DOM (a `MutationObserver` callback), for a
 * claim about discrete states of a component whatever the frame cadence: every state a frame could show is judged, so
 * the verdict does not depend on which states the frames happened to land on. A state that lives for less than a frame
 * counts then, so use it only for a subject whose DOM goes through no benign intermediate state.
 *
 * The sampler is serialised into the page, so it can use only its argument (and the page's globals), never a closure.
 * `scripts/check-timing-tags.sh` fails a spec that starts its own `requestAnimationFrame` loop in an `addInitScript`.
 */
import type { Page } from "@playwright/test";

/** What a sampler is given: `parsed(el)` is true once the HTML parser is done with `el`. */
export interface SampleTools {
  parsed: (el: Node) => boolean;
}

export async function installFrameRecorder<T>(
  page: Page,
  name: string,
  sample: (tools: SampleTools) => T | null | undefined,
  options: { perTask?: boolean } = {},
): Promise<void> {
  await page.addInitScript(
    ({ name, src, perTask }) => {
      const w = window as unknown as Record<string, unknown>;
      const out: unknown[] = [];
      w[name] = out;
      const parsed = (el: Node): boolean => {
        if (document.readyState !== "loading") return true;
        for (let n: Node | null = el; n; n = n.parentNode) if (n.nextSibling) return true;
        return false;
      };
      const sampler = new Function(`return (${src});`)() as (tools: { parsed: typeof parsed }) => unknown;
      const take = () => {
        const s = sampler({ parsed });
        if (s !== null && s !== undefined) out.push(s);
      };
      if (perTask) new MutationObserver(take).observe(document, { subtree: true, childList: true, attributes: true, characterData: true });
      const tick = () => {
        requestAnimationFrame(tick);
        take();
      };
      requestAnimationFrame(tick);
    },
    { name, src: sample.toString(), perTask: !!options.perTask },
  );
}

/** Every state the recorder judged so far, in order. */
export async function readFrames<T>(page: Page, name: string): Promise<T[]> {
  return page.evaluate((n) => (window as unknown as Record<string, unknown[]>)[n].slice(), name) as Promise<T[]>;
}
