/**
 * Self-hosted fonts (backlog rows 162, 261): the docs site loads Geist / Geist Mono from its own origin,
 * makes no request to Google, reads the fonts through the theme's two tokens, and keeps the
 * preload + `font-display: optional` pair that holds the hard-load CLS at 0.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const ROUTES = ["/", "/component/form/", "/component/markdown/"];
const GOOGLE = /(^|\.)(fonts\.googleapis\.com|fonts\.gstatic\.com)$/;

for (const route of ROUTES) {
  test(`no request to Google fonts hosts on ${route}`, async ({ page }) => {
    const hosts: string[] = [];
    page.on("request", (r) => {
      try {
        hosts.push(new URL(r.url()).hostname);
      } catch {
        /* data: / blob: */
      }
    });
    await gotoHydrated(page, `${BASE_URL}${route}`, { waitUntil: "networkidle" });
    expect(hosts.filter((h) => GOOGLE.test(h))).toEqual([]);
  });
}

test("fonts come from our origin and the tokens drive body and code", async ({ page }) => {
  const fontUrls: string[] = [];
  page.on("response", (r) => {
    if (/\.woff2(\?|$)/.test(r.url())) fontUrls.push(r.url());
  });
  await gotoHydrated(page, `${BASE_URL}/component/markdown/`, { waitUntil: "networkidle" });
  const origin = new URL(page.url()).origin;
  expect(fontUrls.length).toBeGreaterThanOrEqual(1);
  for (const u of fontUrls) expect(new URL(u).origin).toBe(origin);

  const r = await page.evaluate(() => {
    const root = getComputedStyle(document.documentElement);
    const code = document.querySelector("main code, main pre, .dx-markdown code");
    return {
      sans: root.getPropertyValue("--dx-font-sans").trim(),
      mono: root.getPropertyValue("--dx-font-mono").trim(),
      body: getComputedStyle(document.body).fontFamily,
      code: code ? getComputedStyle(code).fontFamily : null,
      loaded: [...document.fonts].filter((f) => f.status === "loaded").map((f) => f.family.replace(/"/g, "")),
    };
  });
  expect(r.sans).toMatch(/^"?Geist"?,/);
  expect(r.mono).toMatch(/^"Geist Mono",/);
  const norm = (v: string | null) => (v ?? "").replace(/["\s]/g, "");
  expect(norm(r.body)).toBe(norm(r.sans));
  expect(r.code).not.toBeNull();
  expect(norm(r.code)).toBe(norm(r.mono));
  expect(r.loaded).toContain("Geist");
});

test("the head preloads both self-hosted woff2 files (crossorigin) with font-display: optional", async ({ page }) => {
  // goto-ok: reads the prerendered <head> only (preload links), no interaction
  await page.goto(`${BASE_URL}/component/form/`, { waitUntil: "domcontentloaded" });
  const info = await page.evaluate(() => {
    const links = [...document.querySelectorAll<HTMLLinkElement>('link[rel="preload"][as="font"]')];
    return {
      links: links.map((l) => ({ href: l.getAttribute("href") ?? "", type: l.type, cross: l.crossOrigin })),
      css: [...document.querySelectorAll("style")].map((s) => s.textContent ?? "").join("\n"),
    };
  });
  expect(info.links.length).toBeGreaterThanOrEqual(2);
  for (const l of info.links) {
    expect(l.href.startsWith("http")).toBe(false);
    expect(l.type).toBe("font/woff2");
    expect(l.cross).not.toBeNull();
    expect(info.css).toContain(l.href);
  }
  expect(info.css.match(/font-display:\s*optional/g)?.length).toBe(2);
});

test("hard-load CLS stays under 0.01 on /component/form/", async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as { __cls: number };
    w.__cls = 0;
    new PerformanceObserver((l) => {
      for (const e of l.getEntries()) {
        const s = e as unknown as { value: number; hadRecentInput: boolean };
        if (!s.hadRecentInput) w.__cls += s.value;
      }
    }).observe({ type: "layout-shift", buffered: true });
  });
  await gotoHydrated(page, `${BASE_URL}/component/form/`, { waitUntil: "networkidle" });
  await page.waitForTimeout(500);
  const cls = await page.evaluate(() => (window as unknown as { __cls: number }).__cls);
  expect(cls).toBeLessThan(0.01);
});
