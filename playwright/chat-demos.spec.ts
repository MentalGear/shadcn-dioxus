/**
 * The richer chat-kit demos (message, bubble, marker, attachment, message_scroller), each mirroring
 * a shadcn/ui example from `apps/v4/examples/base/` (research: dev-docs, "chat kit demos" gap
 * report). Every demo here is built from the existing component API only; the specs check that
 * each one renders and that its option control changes what the reader observes:
 *
 * - message_scroller: `commands` (the "Jump to..." menu scrolls the target row into view),
 *   `anchoring` (the toggle changes which role settles near the top), `opening_position` (the tabs
 *   change where the thread opens), `previous_context` (the slider changes how much of the previous
 *   row peeks above a new turn), `group_chat` (a join marker is an anchored row), `animation` (the
 *   preset picked at send time is stored with the row), `full_chat` (the hero's Card shell, Empty
 *   state, Reset tooltip and composer).
 * - attachment: `dialog`, `image_gallery`, `removable_group` (the main demo, shadcn's attachment-demo, is attachment-demo.spec.ts).
 * - bubble: `tooltip`, `popover`, `collapsible`, `link_group`.
 * - marker: `icon`. message: `demo`, `download`.
 *
 * No assertion is a millisecond threshold: scroll positions are compared in pixels with a margin
 * that absorbs one frame of lag, and a debug build is fine (functional spec).
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const url = (name: string) => `${BASE_URL}/component/?name=${name}&`;

const frame = (page: Page, variant: string) => page.locator(`#component-preview-frame-${variant}`).first();
const viewportOf = (page: Page, variant: string) => frame(page, variant).locator("[data-message-scroller-viewport]");

/** Open a component page and wait until the named demo frame has its stylesheet applied. */
async function open(page: Page, name: string, variant: string, readySelector: string) {
  await gotoHydrated(page, url(name), GOTO_OPTS);
  const root = frame(page, variant);
  await root.scrollIntoViewIfNeeded();
  await expect(root.locator(readySelector).first()).toBeVisible({ timeout: 60_000 });
  return root;
}

/** Pixel distance of a row's top below its viewport's top edge. */
const offsetOf = (vp: Locator, id: string) =>
  vp.evaluate((el, id) => {
    const row = el.querySelector(`[data-message-id="${id}"]`) as HTMLElement;
    return Math.round(row.getBoundingClientRect().top - el.getBoundingClientRect().top);
  }, id);

/** Resolves once scrollTop has been the same for a few frames. */
const stable = (vp: Locator) =>
  vp.evaluate(
    (el) =>
      new Promise<number>((resolve) => {
        let last = el.scrollTop;
        let frames = 0;
        const tick = () => {
          if (Math.abs(el.scrollTop - last) < 0.5) frames++;
          else frames = 0;
          last = el.scrollTop;
          if (frames >= 10) resolve(Math.round(el.scrollTop));
          else requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      }),
  );

/**
 * Where an anchored turn settles: the content padding (16px) plus the default
 * `scroll_previous_item_peek` (64px) of the previous row, which stays visible above it.
 */
const ANCHOR_OFFSET = 80;
const atAnchor = (offset: number) => Math.abs(offset - ANCHOR_OFFSET) <= 8;

const dist = (vp: Locator) =>
  vp.evaluate((el) => Math.round(el.scrollHeight - el.scrollTop - el.clientHeight));

/** The row is fully inside the viewport's visible box (a pixel of slack for rounding). */
const rowInView = (vp: Locator, id: string) =>
  vp.evaluate((el, id) => {
    const row = el.querySelector(`[data-message-id="${id}"]`) as HTMLElement | null;
    if (!row) return false;
    const r = row.getBoundingClientRect();
    const v = el.getBoundingClientRect();
    return r.top >= v.top - 1 && r.top < v.bottom - 1;
  }, id);

// ---------------------------------------------------------------------------------------------
// message_scroller
// ---------------------------------------------------------------------------------------------

test.describe("message_scroller: commands", () => {
  test("Jump to... lists the user turns and scrolls the chosen one to the top", async ({ page }) => {
    const root = await open(page, "message_scroller", "commands", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "commands");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    await stable(vp);

    // The thread opens at the end, so the first user turn is out of view above.
    expect(await rowInView(vp, "command-activation")).toBe(false);
    const before = await stable(vp);

    await root.getByRole("button", { name: "Jump to..." }).click();
    const items = page.getByRole("menuitem");
    await expect(items).toHaveCount(4);
    await expect(items.first()).toContainText("We're seeing activation dip after works...");
    await items.first().click();

    await expect.poll(() => rowInView(vp, "command-activation"), { timeout: 10_000 }).toBe(true);
    const after = await stable(vp);
    expect(after).toBeLessThan(before);
    // `align: Start`: the row's top sits at the viewport's top (the content padding is 16px).
    expect(await offsetOf(vp, "command-activation")).toBeLessThanOrEqual(24);
  });

  test("a later command moves the transcript again", async ({ page }) => {
    const root = await open(page, "message_scroller", "commands", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "commands");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");

    await root.getByRole("button", { name: "Jump to..." }).click();
    await page.getByRole("menuitem").nth(1).click(); // "What should I compare ..."
    await expect.poll(() => offsetOf(vp, "command-compare"), { timeout: 10_000 }).toBeLessThanOrEqual(24);
    const mid = await stable(vp);

    await root.getByRole("button", { name: "Jump to..." }).click();
    await page.getByRole("menuitem").nth(0).click();
    await expect.poll(() => stable(vp), { timeout: 10_000 }).toBeLessThan(mid);
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "message_scroller", "commands", "[data-message-scroller-viewport]");
    await expectNoAxeViolations(page, "message_scroller/commands: loaded", {
      include: "#component-preview-frame-commands",
    });
    await root.getByRole("button", { name: "Jump to..." }).click();
    await expect(page.getByRole("menuitem").first()).toBeVisible();
    await expectNoAxeViolations(page, "message_scroller/commands: menu open", {
      include: ["#component-preview-frame-commands", '[role="menu"]'],
    });
  });
});

test.describe("message_scroller: anchoring", () => {
  const send = (page: Page) => frame(page, "anchoring").getByRole("button", { name: "Send Message" });

  test("starts empty, sends scripted turns and resets", async ({ page }) => {
    const root = await open(page, "message_scroller", "anchoring", '[data-slot="card"]');
    await expect(root.getByText("No anchored messages yet")).toBeVisible();
    await expect(root.getByRole("button", { name: "Reset anchored turns" })).toBeDisabled();
    await send(page).click();
    await expect(root.getByText("No anchored messages yet")).toHaveCount(0);
    await expect(root.getByText("Can you show me how anchoring behaves")).toBeVisible();
    await root.getByRole("button", { name: "Reset anchored turns" }).click();
    await expect(root.getByText("No anchored messages yet")).toBeVisible();
  });

  test("the toggle picks which role settles near the top", async ({ page }) => {
    const root = await open(page, "message_scroller", "anchoring", '[data-slot="card"]');
    const vp = viewportOf(page, "anchoring");
    const group = root.getByRole("group", { name: "Select scroll anchor role" });
    const user = group.getByRole("button", { name: "Anchor user messages" });
    const assistant = group.getByRole("button", { name: "Anchor assistant messages" });
    await expect(user).toHaveAttribute("aria-pressed", "true");

    // User anchors: the first prompt is the anchor, so after the reply arrives the viewport still
    // shows the prompt at the top and nothing was scrolled past it.
    await send(page).click();
    await send(page).click();
    await expect(root.getByText("Append the user prompt first")).toBeVisible();
    await stable(vp);
    const scrolledUnderUser = await stable(vp);
    const replyUnderUser = await offsetOf(vp, "anchor-1-assistant");
    expect(await rowInView(vp, "anchor-1-user")).toBe(true);

    // Assistant anchors: choosing it resets the thread (the toggle's own behaviour) ...
    await assistant.click();
    await expect(assistant).toHaveAttribute("aria-pressed", "true");
    await expect(user).toHaveAttribute("aria-pressed", "false");
    await expect(root.getByText("No anchored messages yet")).toBeVisible();

    // ... and after the same two turns the assistant's reply is the anchor: it has moved up to the
    // anchor position and the prompt above it has scrolled partly out of view.
    await send(page).click();
    await send(page).click();
    await expect(root.getByText("Append the user prompt first")).toBeVisible();
    await expect.poll(async () => atAnchor(await offsetOf(vp, "anchor-1-assistant")), { timeout: 10_000 }).toBe(true);
    const scrolledUnderAssistant = await stable(vp);
    expect(scrolledUnderAssistant).toBeGreaterThan(scrolledUnderUser + 10);
    expect(replyUnderUser).toBeGreaterThan(ANCHOR_OFFSET + 8);
  });

  test("pressing the active role again keeps it selected", async ({ page }) => {
    const root = await open(page, "message_scroller", "anchoring", '[data-slot="card"]');
    const user = root.getByRole("button", { name: "Anchor user messages" });
    await user.click();
    await expect(user).toHaveAttribute("aria-pressed", "true");
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    await open(page, "message_scroller", "anchoring", '[data-slot="card"]');
    await send(page).click();
    await expectNoAxeViolations(page, "message_scroller/anchoring: with a message", {
      include: "#component-preview-frame-anchoring",
    });
  });
});

test.describe("message_scroller: opening_position", () => {
  const tab = (page: Page, name: string) => frame(page, "opening_position").getByRole("tab", { name, exact: true });

  test("opens at the last user turn, then start and end on demand", async ({ page }) => {
    await open(page, "message_scroller", "opening_position", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "opening_position");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    await expect(tab(page, "last-anchor")).toHaveAttribute("aria-selected", "true");

    // last-anchor: `open-3` (the last user message) is the anchor, its reply below it.
    await expect.poll(async () => atAnchor(await offsetOf(vp, "open-3")), { timeout: 10_000 }).toBe(true);
    expect(await stable(vp)).toBeGreaterThan(0);

    // start: the thread opens at the very top.
    await tab(page, "start").click();
    await expect(tab(page, "start")).toHaveAttribute("aria-selected", "true");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    expect(await stable(vp)).toBe(0);

    // end: the thread opens at the bottom.
    await tab(page, "end").click();
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    await expect.poll(() => dist(vp), { timeout: 10_000 }).toBeLessThanOrEqual(2);
    expect(await stable(vp)).toBeGreaterThan(0);

    // and back to last-anchor.
    await tab(page, "last-anchor").click();
    await expect.poll(async () => atAnchor(await offsetOf(vp, "open-3")), { timeout: 10_000 }).toBe(true);
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    await open(page, "message_scroller", "opening_position", "[data-message-scroller-viewport]");
    await expectNoAxeViolations(page, "message_scroller/opening_position: loaded", {
      include: "#component-preview-frame-opening_position",
    });
  });
});

test.describe("message_scroller: previous_context", () => {
  const composerSend = (page: Page) => frame(page, "previous_context").getByRole("button", { name: "Send", exact: true });

  /** Send the next queued message and wait until its reply has finished streaming. */
  async function sendAndSettle(page: Page) {
    const send = composerSend(page);
    await send.click();
    // The send button is disabled while the reply streams and enabled again at the end (or, with
    // the queue drained, stays disabled -- in which case the reset button is the signal).
    const reset = frame(page, "previous_context").getByRole("button", { name: "Reset context example" });
    await expect(reset).toBeEnabled({ timeout: 60_000 });
  }

  test("the slider writes the peek and the readout follows", async ({ page }) => {
    const root = await open(page, "message_scroller", "previous_context", '[data-slot="card"]');
    const readout = root.getByTestId("peek-value");
    await expect(readout).toHaveText("64px");
    const slider = root.getByRole("slider", { name: "Previous context peek" });
    await slider.focus();
    await page.keyboard.press("End");
    await expect(readout).toHaveText("128px");
    await page.keyboard.press("Home");
    await expect(readout).toHaveText("64px");
  });

  test("a larger peek keeps more of the previous reply above the new turn", async ({ page }) => {
    const root = await open(page, "message_scroller", "previous_context", '[data-slot="card"]');
    const vp = viewportOf(page, "previous_context");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");

    // Default peek (64px): send the first queued message; the new user row is the anchor.
    await sendAndSettle(page);
    const atDefault = await offsetOf(vp, "m2");

    // Reset, widen the peek to 128px, send the same message again.
    await root.getByRole("button", { name: "Reset context example" }).click();
    await expect(root.getByTestId("peek-value")).toHaveText("64px");
    const slider = root.getByRole("slider", { name: "Previous context peek" });
    await slider.focus();
    await page.keyboard.press("End");
    await expect(root.getByTestId("peek-value")).toHaveText("128px");
    await sendAndSettle(page);
    const atWide = await offsetOf(vp, "m2");

    // The anchored row sits (peek) pixels below the top edge, so +64px of peek is ~+64px of offset.
    expect(atWide - atDefault).toBeGreaterThan(40);
    expect(atWide - atDefault).toBeLessThan(88);
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    await open(page, "message_scroller", "previous_context", '[data-slot="card"]');
    await expectNoAxeViolations(page, "message_scroller/previous_context: loaded", {
      include: "#component-preview-frame-previous_context",
    });
  });
});

test.describe("message_scroller: group_chat", () => {
  test("adding Rocky appends an anchored join marker, then his reply", async ({ page }) => {
    const root = await open(page, "message_scroller", "group_chat", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "group_chat");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    await expect(root.getByText("Rocky has joined the chat")).toHaveCount(0);
    const reset = root.getByRole("button", { name: "Reset conversation" });
    await expect(reset).toBeDisabled();

    await root.getByRole("button", { name: "Add Rocky" }).click();
    const marker = root.getByText("Rocky has joined the chat");
    await expect(marker).toBeVisible();
    // The marker is an anchored row: it settles near the top of the viewport.
    await expect.poll(async () => atAnchor(await offsetOf(vp, "group-4")), { timeout: 10_000 }).toBe(true);
    await expect(root.getByRole("button", { name: "Send Message as Rocky" })).toBeEnabled();

    await root.getByRole("button", { name: "Send Message as Rocky" }).click();
    await expect(root.getByText("Amaze. Astrophage eats light")).toBeVisible();
    await expect(root.getByRole("button", { name: "Send Message as Rocky" })).toBeDisabled();

    // Reset returns to the three opening messages.
    await reset.click();
    await expect(root.getByText("Rocky has joined the chat")).toHaveCount(0);
    await expect(root.getByRole("button", { name: "Add Rocky" })).toBeEnabled();
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "message_scroller", "group_chat", "[data-message-scroller-viewport]");
    await root.getByRole("button", { name: "Add Rocky" }).click();
    await expectNoAxeViolations(page, "message_scroller/group_chat: with the join marker", {
      include: "#component-preview-frame-group_chat",
    });
  });
});

test.describe("message_scroller: animation", () => {
  test("the preset chosen at send time is kept by that row", async ({ page }) => {
    const root = await open(page, "message_scroller", "animation", '[data-slot="card"]');
    await expect(root.getByText("No Messages Yet")).toBeVisible();
    const send = root.getByRole("button", { name: "Send Message" });
    const reset = root.getByRole("button", { name: "Reset animated messages" });
    await expect(reset).toBeDisabled();

    const userRow = (id: string) => root.locator(`[data-message-id="${id}"] [data-slot="message"]`);

    await send.click();
    await expect(root.getByText("Can user messages pop in like iMessage")).toBeVisible();
    await expect(userRow("m0")).toHaveClass(/dx-demo-enter-fade/);
    // The assistant's reply carries no entrance class.
    await expect(send).toBeEnabled({ timeout: 60_000 });

    // Pick "Pop" and send the next turn.
    await root.getByRole("combobox", { name: "Animation preset" }).click();
    await page.getByRole("option", { name: "Pop" }).click();
    await send.click();
    await expect(userRow("m2")).toHaveClass(/dx-demo-enter-pop/);
    // The earlier row did not take on the new preset.
    await expect(userRow("m0")).toHaveClass(/dx-demo-enter-fade/);
    await expect(userRow("m0")).not.toHaveClass(/dx-demo-enter-pop/);
    await expect(send).toBeEnabled({ timeout: 60_000 });

    // The entrance is a theme keyframe, not a layout animation.
    const name = await userRow("m2").evaluate((el) => getComputedStyle(el).animationName);
    expect(name).toBe("dx-pop-in");

    await reset.click();
    await expect(root.getByText("No Messages Yet")).toBeVisible();
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "message_scroller", "animation", '[data-slot="card"]');
    await expectNoAxeViolations(page, "message_scroller/animation: empty", {
      include: "#component-preview-frame-animation",
    });
    await root.getByRole("button", { name: "Send Message" }).click();
    await expect(root.getByText("Can user messages pop in like iMessage")).toBeVisible();
    await expectNoAxeViolations(page, "message_scroller/animation: with a message", {
      include: "#component-preview-frame-animation",
    });
  });
});

test.describe("message_scroller: full_chat", () => {
  test("empty state, send, plus menu and reset tooltip", async ({ page }) => {
    const root = await open(page, "message_scroller", "full_chat", '[data-slot="card"]');
    await expect(root.getByText("Morning, shadcn!")).toBeVisible();
    await expect(root.getByRole("textbox", { name: "Next message" })).toHaveValue(/scroll behavior is driving me nuts/);

    // The "+" menu offers the four composer actions and reports the one picked.
    await root.getByRole("button", { name: "Add files" }).click();
    const items = page.getByRole("menuitem");
    await expect(items).toHaveCount(4);
    await expect(items.nth(1)).toHaveText("Create Image");
    await items.nth(1).click();
    await expect(root.getByRole("status").last()).toContainText('Picked "Create Image"');

    // Send plays the queued user message and streams the reply.
    const send = root.getByRole("button", { name: "Send", exact: true });
    await send.click();
    await expect(root.getByText("Morning, shadcn!")).toHaveCount(0);
    await expect(root.getByText("That's the classic streaming scroll problem")).toBeVisible({ timeout: 60_000 });
    await expect(send).toBeEnabled({ timeout: 60_000 });
    // The composer now shows the next queued message.
    await expect(root.getByRole("textbox", { name: "Next message" })).toHaveValue(/Okay, but when someone sends/);

    // The reset button names itself and has a "Reset" tooltip.
    const reset = root.getByRole("button", { name: "Reset conversation" });
    await reset.focus();
    await expect(page.getByRole("tooltip").filter({ hasText: "Reset" })).toBeVisible();
    await reset.click();
    await expect(root.getByText("Morning, shadcn!")).toBeVisible();
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "message_scroller", "full_chat", '[data-slot="card"]');
    await expectNoAxeViolations(page, "message_scroller/full_chat: empty", {
      include: "#component-preview-frame-full_chat",
    });
    await root.getByRole("button", { name: "Send", exact: true }).click();
    await expect(root.getByText("That's the classic streaming scroll problem")).toBeVisible({ timeout: 60_000 });
    await expect(root.getByRole("button", { name: "Send", exact: true })).toBeEnabled({ timeout: 60_000 });
    await expectNoAxeViolations(page, "message_scroller/full_chat: with a conversation", {
      include: "#component-preview-frame-full_chat",
    });
  });
});

// ---------------------------------------------------------------------------------------------
// attachment
// ---------------------------------------------------------------------------------------------

test.describe("attachment: dialog", () => {
  test("the trigger opens a real dialog while the actions stay clickable", async ({ page }) => {
    const root = await open(page, "attachment", "dialog", '[data-slot="attachment"]');
    const trigger = root.getByRole("button", { name: "Preview research-summary.pdf" });
    await expect(trigger).toHaveAttribute("aria-haspopup", "dialog");
    await trigger.click();
    const dialog = page.getByRole("dialog", { name: "research-summary.pdf" });
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("The attachment trigger fills the card and opens the dialog");
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();

    // An action above the trigger does its own thing and does not open the dialog.
    await root.getByRole("button", { name: "Copy link to research-summary.pdf" }).click();
    await expect(root.getByRole("status")).toHaveText("Link copied");
    await expect(page.getByRole("dialog", { name: "research-summary.pdf" })).toBeHidden();

    await root.getByRole("button", { name: "Remove research-summary.pdf" }).click();
    await expect(root.locator('[data-slot="attachment"]')).toHaveCount(0);
  });

  test("the trigger opens by keyboard", async ({ page }) => {
    const root = await open(page, "attachment", "dialog", '[data-slot="attachment"]');
    await root.getByRole("button", { name: "Preview research-summary.pdf" }).focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog", { name: "research-summary.pdf" })).toBeVisible();
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "attachment", "dialog", '[data-slot="attachment"]');
    await expectNoAxeViolations(page, "attachment/dialog: closed", {
      include: "#component-preview-frame-dialog",
    });
    await root.getByRole("button", { name: "Preview research-summary.pdf" }).click();
    await expect(page.getByRole("dialog", { name: "research-summary.pdf" })).toBeVisible();
    await expectNoAxeViolations(page, "attachment/dialog: open", {
      include: '[role="dialog"]',
    });
  });
});

test.describe("attachment: image_gallery", () => {
  test("each image card is a vertical attachment with a link trigger and a remove action", async ({ page }) => {
    const root = await open(page, "attachment", "image_gallery", '[data-slot="attachment"]');
    const cards = root.locator('[data-slot="attachment"]');
    await expect(cards).toHaveCount(3);
    await expect(cards.first()).toHaveAttribute("data-orientation", "vertical");
    for (const name of ["workspace.svg", "desk-reference.svg", "office-reference.svg"]) {
      const link = root.getByRole("link", { name: `Open ${name}` });
      await expect(link).toHaveAttribute("target", "_blank");
      await expect(link).toHaveAttribute("rel", "noreferrer");
      expect(await link.getAttribute("href")).toMatch(new RegExp(`${name.replace(".svg", "")}.*\\.svg$`));
    }
    // The previews are bundled assets and actually load.
    const loaded = await root.locator("img").evaluateAll((imgs) =>
      imgs.map((i) => (i as HTMLImageElement).complete && (i as HTMLImageElement).naturalWidth > 0),
    );
    expect(loaded).toEqual([true, true, true]);

    await root.getByRole("button", { name: "Remove desk-reference.svg" }).click();
    await expect(cards).toHaveCount(2);
    await expect(root.getByRole("link", { name: "Open desk-reference.svg" })).toHaveCount(0);
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    await open(page, "attachment", "image_gallery", '[data-slot="attachment"]');
    await expectNoAxeViolations(page, "attachment/image_gallery: loaded", {
      include: "#component-preview-frame-image_gallery",
    });
  });
});

test.describe("attachment: removable_group", () => {
  test("every card has a working remove action and the count follows", async ({ page }) => {
    const root = await open(page, "attachment", "removable_group", '[data-slot="attachment"]');
    const cards = root.locator('[data-slot="attachment"]');
    await expect(cards).toHaveCount(4);
    await expect(root.getByRole("status")).toHaveText("4 of 4 attachments");
    await root.getByRole("button", { name: "Remove briefing-notes.pdf" }).click();
    await expect(cards).toHaveCount(3);
    await expect(root.getByRole("status")).toHaveText("3 of 4 attachments");
    await expect(root.getByText("briefing-notes.pdf")).toHaveCount(0);
    // The group scrolls horizontally: the cards are wider than a quarter of the group.
    const widths = await cards.evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().width)));
    expect(Math.min(...widths)).toBeGreaterThanOrEqual(200);
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    await open(page, "attachment", "removable_group", '[data-slot="attachment"]');
    await expectNoAxeViolations(page, "attachment/removable_group: loaded", {
      include: "#component-preview-frame-removable_group",
    });
  });
});

// ---------------------------------------------------------------------------------------------
// bubble
// ---------------------------------------------------------------------------------------------

test.describe("bubble: tooltip", () => {
  test("the read receipt is a button with a tooltip", async ({ page }) => {
    const root = await open(page, "bubble", "tooltip", '[data-slot="bubble"]');
    const receipt = root.getByRole("button", { name: "Read on Jan 5, 2026 at 4:32 PM" });
    await expect(receipt).toHaveClass(/dx-button/);
    await expect(page.getByRole("tooltip")).toHaveCount(0);
    await receipt.focus();
    await expect(page.getByRole("tooltip")).toContainText("Read on Jan 5, 2026 at 4:32 PM");
    await page.keyboard.press("Escape");
    await expect(page.getByRole("tooltip")).toHaveCount(0);
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "bubble", "tooltip", '[data-slot="bubble"]');
    await expectNoAxeViolations(page, "bubble/tooltip: closed", {
      include: "#component-preview-frame-tooltip",
    });
    await root.getByRole("button", { name: "Read on Jan 5, 2026 at 4:32 PM" }).focus();
    await expect(page.getByRole("tooltip")).toBeVisible();
    await expectNoAxeViolations(page, "bubble/tooltip: open", {
      include: ["#component-preview-frame-tooltip", '[role="tooltip"]'],
    });
  });
});

test.describe("bubble: popover", () => {
  test("the info trigger on the destructive bubble opens error details", async ({ page }) => {
    const root = await open(page, "bubble", "popover", '[data-slot="bubble"]');
    const trigger = root.getByRole("button", { name: "Show error details" });
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await trigger.click();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    const popover = page.getByRole("dialog").filter({ hasText: "Command failed with exit code 1" });
    await expect(popover).toBeVisible();
    await expect(popover).toContainText("ENOENT: no such file or directory, open pnpm-lock.yaml");
    await page.keyboard.press("Escape");
    await expect(popover).toBeHidden();
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "bubble", "popover", '[data-slot="bubble"]');
    await expectNoAxeViolations(page, "bubble/popover: closed", {
      include: "#component-preview-frame-popover",
    });
    await root.getByRole("button", { name: "Show error details" }).click();
    await expect(page.getByRole("dialog").filter({ hasText: "Command failed" })).toBeVisible();
    await expectNoAxeViolations(page, "bubble/popover: open", {
      include: ["#component-preview-frame-popover", '[role="dialog"]'],
    });
  });
});

test.describe("bubble: collapsible", () => {
  test("a long reply folds to a preview and expands with Show more / Show less", async ({ page }) => {
    const root = await open(page, "bubble", "collapsible", '[data-slot="bubble"]');
    const more = root.getByRole("button", { name: /Show more/ });
    await expect(more).toHaveAttribute("aria-expanded", "false");
    await expect(more).toHaveClass(/dx-button/);
    await expect(root.getByText("The accessibility review found two focus states")).toBeVisible();
    await expect(root.getByText("I also recommend keeping the change")).toHaveCount(0);
    const folded = (await root.locator('[data-slot="bubble"]').nth(1).boundingBox())!.height;

    await more.click();
    const less = root.getByRole("button", { name: /Show less/ });
    await expect(less).toHaveAttribute("aria-expanded", "true");
    await expect(root.getByText("I also recommend keeping the change")).toBeVisible();
    const expanded = (await root.locator('[data-slot="bubble"]').nth(1).boundingBox())!.height;
    expect(expanded).toBeGreaterThan(folded + 40);

    await less.click();
    await expect(root.getByText("I also recommend keeping the change")).toHaveCount(0);
    await expect(root.getByRole("button", { name: /Show more/ })).toBeVisible();
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "bubble", "collapsible", '[data-slot="bubble"]');
    await expectNoAxeViolations(page, "bubble/collapsible: folded", {
      include: "#component-preview-frame-collapsible",
    });
    await root.getByRole("button", { name: /Show more/ }).click();
    await expectNoAxeViolations(page, "bubble/collapsible: expanded", {
      include: "#component-preview-frame-collapsible",
    });
  });
});

test.describe("bubble: link_group", () => {
  test("three tinted bubbles are buttons that answer with a toast", async ({ page }) => {
    const root = await open(page, "bubble", "link_group", '[data-slot="bubble"]');
    const buttons = root.getByRole("button");
    await expect(buttons).toHaveCount(3);
    await root.getByRole("button", { name: "I forgot my password" }).click();
    await expect(page.getByText("You clicked forgot password")).toBeVisible();
    await root.getByRole("button", { name: "I need help with my subscription" }).click();
    await expect(page.getByText("You clicked help with subscription")).toBeVisible();
    await root.getByRole("button", { name: "Something else. Talk to a human." }).click();
    await expect(page.getByText("You clicked something else. Talk to a human.")).toBeVisible();
  });

  test("has no automatically detectable a11y issues", async ({ page }) => {
    const root = await open(page, "bubble", "link_group", '[data-slot="bubble"]');
    await root.getByRole("button", { name: "I forgot my password" }).click();
    await expect(page.getByText("You clicked forgot password")).toBeVisible();
    await expectNoAxeViolations(page, "bubble/link_group: with a toast", {
      include: "#component-preview-frame-link_group",
    });
  });
});

// ---------------------------------------------------------------------------------------------
// marker, message
// ---------------------------------------------------------------------------------------------

test.describe("marker: icon", () => {
  test("the third marker is a column: icon above text", async ({ page }) => {
    const root = await open(page, "marker", "icon", '[data-slot="marker"]');
    const markers = root.locator('[data-slot="marker"]');
    await expect(markers).toHaveCount(3);
    const dirs = await markers.evaluateAll((els) => els.map((e) => getComputedStyle(e).flexDirection));
    expect(dirs).toEqual(["row", "row", "column"]);
    const third = markers.nth(2);
    const icon = (await third.locator('[data-slot="marker-icon"]').boundingBox())!;
    const text = (await third.locator('[data-slot="marker-content"]').boundingBox())!;
    expect(icon.y + icon.height).toBeLessThanOrEqual(text.y + 1);
    await expectNoAxeViolations(page, "marker/icon: loaded", { include: "#component-preview-frame-icon" });
  });
});

test.describe("message: demo", () => {
  test("composite: both sides with avatars, a delivery footer, a reaction and a typing status", async ({ page }) => {
    const root = await open(page, "message", "demo", '[data-slot="message"]');
    await expect(root.locator('[data-slot="message"]')).toHaveCount(4);
    await expect(root.getByText("Delivered")).toBeVisible();
    await expect(root.getByRole("img", { name: "Reactions: thumbs up" })).toBeVisible();
    const typing = root.getByRole("status");
    await expect(typing).toContainText("Oliver is typing...");
    await expect(typing.locator(".dx-shimmer")).toHaveCount(1);
    await expect(root.getByRole("img", { name: "Rabbit" })).toHaveCount(2);
    await expectNoAxeViolations(page, "message/demo: loaded", { include: "#component-preview-frame-demo" });
  });
});

test.describe("message: download", () => {
  test("the reply's file card has a Download action", async ({ page }) => {
    const root = await open(page, "message", "download", '[data-slot="message"]');
    await expect(root.locator('[data-slot="attachment"][data-orientation="vertical"]')).toHaveCount(1);
    await expect(root.locator("img")).toHaveAttribute("alt", "Workspace");
    const download = root.getByRole("button", { name: "Download sales-dashboard.pdf" });
    await expect(download).toHaveAttribute("title", "Download");
    await expect(download).toHaveClass(/dx-button/);
    await expect(root.getByRole("status")).toHaveText("");
    await download.click();
    await expect(root.getByRole("status")).toHaveText("Downloaded sales-dashboard.pdf (1x)");
    await expectNoAxeViolations(page, "message/download: loaded", { include: "#component-preview-frame-download" });
  });
});
