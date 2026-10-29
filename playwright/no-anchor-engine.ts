/**
 * Simulates an engine without CSS Anchor Positioning (iOS 18 Safari): removes the `@supports (anchor-name: --a)`
 * block from the engine stylesheet (`primitives/src/top_layer/anchor.css`, injected as `#dx-anchor-positioning-styles`),
 * so every anchored panel is placed by the script (`use_anchor_position_fallback`) from the first open on.
 * Call it after the page has hydrated and before the overlay opens.
 */
import type { Page } from "@playwright/test";

export async function stripAnchorSupportsBlock(page: Page): Promise<void> {
  await page.waitForFunction(() => !!document.getElementById("dx-anchor-positioning-styles"), { timeout: 15000 });
  const result = await page.evaluate(() => {
    const tag = document.getElementById("dx-anchor-positioning-styles") as HTMLStyleElement;
    const src = tag.textContent ?? "";
    const start = src.indexOf("@supports (anchor-name: --a)");
    if (start === -1) return { ok: false, reason: "marker not found" };
    let depth = 0;
    let end = -1;
    for (let i = src.indexOf("{", start); i < src.length; i++) {
      if (src[i] === "{") depth++;
      else if (src[i] === "}" && --depth === 0) {
        end = i + 1;
        break;
      }
    }
    if (end === -1) return { ok: false, reason: "unbalanced braces" };
    tag.textContent = src.slice(0, start) + src.slice(end);
    return { ok: true };
  });
  if (!result.ok) throw new Error(`stripAnchorSupportsBlock failed: ${JSON.stringify(result)}`);
}
