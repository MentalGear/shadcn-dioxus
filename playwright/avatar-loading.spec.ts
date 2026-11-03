import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoPrerendered, gotoReference } from "./page-kinds";
import { installFrameRecorder, readFrames } from "./frame-recorder";

// Avatar loading semantics (backlog row 235), Base UI's: the fallback stands in for the picture in every state
// but `loaded` -- while it loads, after an error, with no picture -- and only a loaded picture removes it, so
// there is no blank frame before the picture (removing the fallback when the status becomes
// `loading` would give initials, then a blank pulsing disc, then the picture). `AvatarFallback { delay }` holds the
// fallback back for that many milliseconds.
//
// The `states` demo's slow avatars all request `https://httpbin.org/delay/3600`, which never resolves offline;
// each test answers it itself, after the delay it wants.
const URL = `${BASE_URL}/component/avatar/`;
const SLOW_SRC = "https://httpbin.org/delay/3600";
const WITH_FALLBACK = "Loading avatar with fallback";
const DELAYED_FALLBACK = "Loading avatar with delayed fallback";
const PICTURE = `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#0d9488"/></svg>`;

async function answerSlowImageAfter(page: Page, ms: number) {
  await page.route(SLOW_SRC, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, ms));
    await route.fulfill({ status: 200, contentType: "image/svg+xml", body: PICTURE }).catch(() => {});
  });
}

/** Navigate without waiting for the network to go idle (the slow image keeps it busy on purpose). */
async function open(page: Page) {
  await gotoPrerendered(page, URL, { waitUntil: "commit", timeout: 20 * 60 * 1000 });
  await page.waitForSelector('html[data-hydrated="true"]', { state: "attached", timeout: 5 * 60 * 1000 });
}

/** What one avatar shows right now: its fallback, a decoded picture, both or neither (null: the parser is still inside it). */
interface Showing {
  fallback: boolean;
  loaded: boolean;
}
interface Sample {
  t: number;
  main: Showing | null;
  delayed: Showing | null;
}

/**
 * From the first frame of the navigation on, judge what each avatar shows after every DOM-changing task and on every
 * animation frame (`frame-recorder.ts`, `perTask`): a state is judged only once the HTML parser is past the avatar, so a document
 * streamed in two chunks never reads as an avatar without its fallback.
 */
async function startSampling(page: Page) {
  await installFrameRecorder(page, "__avatarStates", ({ parsed }) => {
    const showing = (label: string) => {
      const root = document.querySelector(`[aria-label="${label}"]`);
      if (!root || !parsed(root)) return null;
      const img = root.querySelector("img");
      return { fallback: !!root.querySelector(".dx-avatar-fallback"), loaded: !!img && img.complete && img.naturalWidth > 0 };
    };
    const main = showing("Loading avatar with fallback");
    const delayed = showing("Loading avatar with delayed fallback");
    return main || delayed ? { t: performance.now(), main, delayed } : null;
  }, { perTask: true });
}

const sampled = (page: Page) => readFrames<Sample>(page, "__avatarStates");
const isBlank = (s: Showing | null) => s !== null && !s.fallback && !s.loaded;

test("while the picture loads the fallback stays, the picture then replaces it, and no frame is blank", async ({ page }) => {
  await startSampling(page);
  await answerSlowImageAfter(page, 9_000);
  await open(page);

  const avatar = page.getByLabel(WITH_FALLBACK);
  await avatar.scrollIntoViewIfNeeded();
  // Hydrated and still loading: the fallback is there (a removed fallback would leave a blank pulsing disc).
  await expect(avatar).toHaveAttribute("data-state", "loading");
  await expect(avatar.locator(".dx-avatar-fallback")).toHaveText("LF");
  await expect(avatar.locator(".dx-avatar-fallback")).toBeVisible();
  // With the fallback standing in, nothing pulses.
  expect(await avatar.evaluate((el) => getComputedStyle(el).animationName)).toBe("none");

  // The picture arrives: it replaces the fallback.
  await expect(avatar).toHaveAttribute("data-state", "loaded", { timeout: 30_000 });
  await expect(avatar.locator(".dx-avatar-fallback")).toHaveCount(0);
  await expect(avatar.locator("img")).toBeVisible();
  expect(await avatar.locator("img").evaluate((el: HTMLImageElement) => el.naturalWidth)).toBeGreaterThan(0);

  const states = (await sampled(page)).map((s) => s.main).filter((m): m is Showing => m !== null);
  expect(states.some((m) => m.fallback && !m.loaded), "the recorder saw the fallback standing in").toBe(true);
  expect(states.some((m) => !m.fallback && m.loaded), "the recorder saw the picture replace it").toBe(true);
  expect(states.filter(isBlank), "states with neither the fallback nor a decoded picture").toEqual([]);
});

test("the recorder flags a blank avatar: a fallback removed by hand while the picture loads", async ({ page }) => {
  await startSampling(page);
  await answerSlowImageAfter(page, 60_000);
  await open(page);
  const avatar = page.getByLabel(WITH_FALLBACK);
  await expect(avatar).toHaveAttribute("data-state", "loading");
  await avatar.evaluate((el) => el.querySelector(".dx-avatar-fallback")!.remove());
  await expect.poll(async () => (await sampled(page)).filter((s) => isBlank(s.main)).length).toBeGreaterThan(0);
});

test("the recorder does not judge a subtree the parser has not finished", async ({ page }) => {
  await installFrameRecorder(page, "__half", ({ parsed }) => {
    const root = document.querySelector("#root");
    return root && parsed(root) ? { children: root.children.length } : null;
  });
  await gotoReference(page, "about:blank");
  // A document that is still being parsed: <div id=root> holds one child and its end tag has not arrived.
  await page.evaluate(() => {
    document.open();
    document.write('<div id="root"><i></i>');
  });
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  expect(await readFrames(page, "__half"), "no state is judged while the parser is inside the subject").toEqual([]);
  await page.evaluate(() => {
    document.write("<b></b></div><p>after</p>");
    document.close();
  });
  await expect.poll(async () => (await readFrames<{ children: number }>(page, "__half")).at(-1)?.children).toBe(2);
});

test("the picture-less avatar keeps its pulsing placeholder while it loads", async ({ page }) => {
  await answerSlowImageAfter(page, 60_000);
  await open(page);
  const placeholder = page.getByLabel("Loading avatar", { exact: true });
  await placeholder.scrollIntoViewIfNeeded();
  await expect(placeholder).toHaveAttribute("data-state", "loading");
  expect(await placeholder.evaluate((el) => getComputedStyle(el).animationName)).toBe("dx-pulse");
});

test("a broken picture falls back at once and the img is gone", async ({ page }) => {
  await answerSlowImageAfter(page, 60_000);
  await open(page);
  const broken = page.getByLabel("Error avatar", { exact: true });
  await expect(broken).toHaveAttribute("data-state", "error", { timeout: 60_000 });
  await expect(broken.getByText("JR")).toBeVisible();
  await expect(broken.locator("img")).toHaveCount(0);
});

test("delay: the fallback is held back that long, and a picture that loads within it never shows it", async ({ page }) => {
  await startSampling(page);
  await answerSlowImageAfter(page, 9_000);
  await open(page);
  const avatar = page.getByLabel(DELAYED_FALLBACK);
  await avatar.scrollIntoViewIfNeeded();
  // Still loading at hydration: no fallback yet (the root's placeholder stands in) ...
  await expect(avatar).toHaveAttribute("data-state", "loading");
  const shownAt = async () => (await sampled(page)).find((s) => s.delayed?.fallback)?.t ?? null;
  // ... and it appears once the 600 ms are up (not instantly, not never).
  await expect(avatar.locator(".dx-avatar-fallback")).toHaveText("DF", { timeout: 10_000 });
  // The delay runs from the component's client mount, which is not observable from the DOM and happens during
  // hydration, up to a machine-dependent gap BEFORE the `data-hydrated` flag (row 247: anchoring on `hydratedAt` fails when the gap exceeds its 450 ms
  // floor). So anchor on something that
  // strictly precedes the mount: the wasm module's last byte (the app cannot mount before it is downloaded). Then
  // the delay itself is the floor, with no slack to tune: shown >= wasmEnd + 600 on every machine.
  const wasmEnd = await page.evaluate(() => {
    const e = performance.getEntriesByType("resource").find((r) => /\.wasm(\?|$)/.test(r.name));
    return e ? e.responseEnd : null;
  });
  expect(wasmEnd, "the wasm module's resource timing entry").not.toBeNull();
  const waited = (await shownAt())! - wasmEnd!;
  // 600 ms delay; 5 ms for timer coarsening (a timer may fire a tick early against performance.now()). A lower bound
  // only: that the fallback does appear is the 10 s wait above, and an upper bound would measure the machine.
  expect(waited).toBeGreaterThanOrEqual(595);
});

test("delay: a picture that loads within the delay never shows the fallback", async ({ page }) => {
  await startSampling(page);
  await answerSlowImageAfter(page, 50);
  await open(page);
  const avatar = page.getByLabel(DELAYED_FALLBACK);
  await expect(avatar).toHaveAttribute("data-state", "loaded", { timeout: 30_000 });
  // Outlast the 600 ms timer: a late fallback would show up now.
  await page.waitForTimeout(1_200);
  await expect(avatar.locator(".dx-avatar-fallback")).toHaveCount(0);
  expect((await sampled(page)).filter((s) => s.delayed?.fallback), "states in which the delayed avatar showed its fallback").toEqual([]);
});

test("the prerendered HTML holds the fallback (no hydration mismatch), and none when a delay holds it back", async ({ request }) => {
  const response = await request.get(`${BASE_URL}/component/avatar/`);
  expect(response.ok()).toBeTruthy();
  const html = await response.text();
  const around = (label: string) => {
    const at = html.indexOf(`aria-label="${label}"`);
    expect(at, `${label} in the served page`).toBeGreaterThan(-1);
    return html.slice(at, at + 900);
  };
  // The image and the initials are both in the server markup ...
  const instant = around(WITH_FALLBACK);
  expect(instant).toContain("<img");
  expect(instant).toContain(">LF<");
  // ... and with a delay the server markup has the image but not the fallback (the timer exists only in the browser).
  const delayed = around(DELAYED_FALLBACK);
  expect(delayed).toContain("<img");
  expect(delayed).not.toContain(">DF<");
});
