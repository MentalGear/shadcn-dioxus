// The order contract's page side in a real browser: the page-minted sequence number, the slot fence against a late Rust
// write, the Rust-caused rule and the focus owner. Each page is hand-written on the shipped bridge bundle and kernel
// (no site build); the inbound channel is a stub that records every frame the page sends to Rust.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "../fixtures";
import { gotoReference } from "../page-kinds";
import { witnessPage } from "../../primitives/ts-test/witness";

const JS = join(__dirname, "..", "..", "primitives", "src");
const BRIDGE = readFileSync(join(JS, "js", "bridge.js"), "utf8");
const FIXTURE = readFileSync(join(JS, "js", "behaviours", "fixture.js"), "utf8");
const OTP = readFileSync(join(JS, "js", "behaviours", "otp-filter.js"), "utf8");
const KERNEL_B64 = readFileSync(join(JS, "kernel", "decision-kernel.b64"), "utf8").trim();
const URL = "http://order.fixture/page.html";

const PAGE = (markup: string, script = ""): string => `<!doctype html><meta charset="utf-8"><body style="margin:0">${markup}
<script>${BRIDGE}</script><script>${FIXTURE}</script><script>${OTP}</script>
<script>window.__dxBridge.kernel.load(${JSON.stringify(KERNEL_B64)});
window.frames_ = [];
window.__dxBridge.inbound({ send: (m) => window.frames_.push(m), recv: () => new Promise(() => {}) });
${script}</script>`;

const DIALOG = `<button id="open">open</button><dialog id="d"><button id="inside">inside</button></dialog>`;

const frames = (page: import("@playwright/test").Page) => page.evaluate(() => (window as any).frames_ as any[]);
const write = (page: import("@playwright/test").Page, method: string, basedOn?: number) =>
  page.evaluate(([m, b]) => (window as any).__dxBridge.transaction([{ op: "call", target: "d", method: m, args: [], ...(b === null ? {} : { basedOn: b }) }])[0], [method, basedOn ?? null] as const);

async function open(page: import("@playwright/test").Page, markup: string, script = ""): Promise<void> {
  await page.route(URL, (route) => route.fulfill({ contentType: "text/html", body: PAGE(markup, script) }));
  await gotoReference(page, URL);
  await page.waitForFunction(() => document.documentElement.dataset.dxBridge === "ready");
}

test.describe("the order contract in a browser", () => {
  test("the bridge is ready once the inbound eval runs", async ({ page }) => {
    await open(page, DIALOG);
    expect(await page.evaluate(() => document.documentElement.dataset.dxBridge)).toBe("ready");
  });

  test("a dialog dismissed by Escape fences the open slot: a Rust reopen decided before it is dropped, one decided after applies", async ({ page }) => {
    await open(page, DIALOG);
    expect(await write(page, "showModal", 0)).toEqual({ ok: null });
    expect(await page.evaluate(() => (document.getElementById("d") as HTMLDialogElement).open)).toBe(true);
    await page.keyboard.press("Escape");
    expect(await page.evaluate(() => (document.getElementById("d") as HTMLDialogElement).open)).toBe(false);
    const open_ = (await frames(page)).filter((f) => f.t === "open" && !f.c);
    expect(open_.length).toBeGreaterThan(0);
    const seq = open_[open_.length - 1].seq;
    // the late write of a decision made before Rust saw the dismissal
    expect(await write(page, "showModal", seq - 1)).toEqual({ ok: null, dropped: true });
    expect(await page.evaluate(() => (document.getElementById("d") as HTMLDialogElement).open)).toBe(false);
    // the counterfactual: with no fence the stale write reopens the dialog
    expect(await write(page, "showModal")).toEqual({ ok: null });
    expect(await page.evaluate(() => (document.getElementById("d") as HTMLDialogElement).open)).toBe(true);
    // a decision made after Rust saw every page transition (the dismissal moves focus too) applies
    await write(page, "close");
    await page.evaluate(() => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))));
    const latest = Math.max(...(await frames(page)).filter((f) => f.t && !f.c).map((f) => f.seq));
    expect(await write(page, "showModal", latest)).toEqual({ ok: null });
  });

  test("a transition a Rust write causes is reported with its cause and does not fence", async ({ page }) => {
    await open(page, DIALOG);
    expect(await write(page, "showModal", 0)).toEqual({ ok: null });
    const caused = (await frames(page)).filter((f) => f.t === "open");
    expect(caused.length).toBeGreaterThan(0);
    expect(caused.every((f) => f.c === 1)).toBe(true);
    expect(await write(page, "close", 0)).toEqual({ ok: null });
  });

  test("a user focus change fences the one document-wide focus slot against a call that touches focus", async ({ page }) => {
    await open(page, `<input id="a"><input id="b"><dialog id="d"></dialog>`);
    await page.focus("#a");
    await page.focus("#b");
    const focus = (await frames(page)).filter((f) => f.t === "focus");
    expect(focus.length).toBe(2);
    expect(await write(page, "showModal", focus[1].seq - 1)).toEqual({ ok: null, dropped: true });
    await page.evaluate(() => (document.getElementById("d") as HTMLDialogElement).close());
    expect(await write(page, "showModal", focus[1].seq)).toEqual({ ok: null });
  });

  test("sequence numbers rise across the facts the page reports", async ({ page }) => {
    await open(page, `<input id="a"><input id="b">`);
    await page.focus("#a");
    await page.focus("#b");
    await page.evaluate(() => document.getElementById("b")?.blur());
    const seqs = (await frames(page)).map((f) => f.seq);
    expect(seqs.length).toBeGreaterThan(2);
    expect([...seqs].sort((x, y) => x - y)).toEqual(seqs);
    expect(new Set(seqs).size).toBe(seqs.length);
  });

  test("a decision's focus slot write focuses the host without scrolling; a plain focus scrolls", async ({ page }) => {
    const install = `window.__dxBridge.scope({ send() {}, recv: () => new Promise(() => {}) }).run({ cmd: "behaviour", target: "far",
      behaviour: { kind: "fixture", event: "click", attr: "data-dx-fixture", value: "on", capture: false, cancel: false, focus: true, leaseMs: null },
      declared: { target: "element", event: "click", capture: false, passive: true, leaseMs: null }, epoch: { key: "behaviour:fixture", n: 1 } });`;
    await open(page, `<div style="height:3000px"></div><button id="far">far</button><div style="height:3000px"></div>`, install);
    await page.evaluate(() => document.getElementById("far")!.click());
    expect(await page.evaluate(() => document.activeElement?.id)).toBe("far");
    expect(await page.evaluate(() => window.scrollY)).toBe(0);
    // the counterfactual: the browser's own focus step scrolls the page
    await page.evaluate(() => (document.activeElement as HTMLElement).blur());
    await page.evaluate(() => document.getElementById("far")!.focus());
    expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
  });
});

// The value slot: the OTP filter decides and writes inside the edit's own dispatch, so a paste, a form reset and a
// keystroke in one gesture end on the keystroke, and the last `input` event every listener heard is the DOM value.
const OTP_PAGE = `<form id="f"><input id="otp" value=""><button type="reset" id="r">reset</button></form>`;
const OTP_INSTALL = `window.inputs = [];
document.getElementById("otp").addEventListener("input", (e) => window.inputs.push(e.target.value));
const otp = window.__dxBridge.scope({ send() {}, recv: () => new Promise(() => {}) });
for (const event of ["beforeinput", "input", "compositionend"]) {
  const behaviour = { kind: "otp-filter", site: "input_otp.filter", event, ranges: "09", max: 6 };
  otp.run({ cmd: "behaviour", target: "otp", behaviour,
    declared: { target: "element", event, capture: true, passive: event !== "beforeinput", leaseMs: event === "beforeinput" ? 600000 : null },
    epoch: { key: "behaviour:otp-filter:" + event, n: 1 } });
}`;

/** One gesture: a paste of "123-456", a reset of the form, a keystroke "7"; the value and the inputs heard after the paints settle. */
async function gesture(page: import("@playwright/test").Page): Promise<{ value: string; inputs: string[] }> {
  await page.focus("#otp");
  await page.keyboard.insertText("123-456");
  await page.evaluate(() => (document.getElementById("f") as HTMLFormElement).reset());
  await page.keyboard.type("7");
  await page.evaluate(() => new Promise<void>((done) => (window as any).__dxWitness.paints(8, done)));
  return page.evaluate(() => ({ value: (document.getElementById("otp") as HTMLInputElement).value, inputs: (window as any).inputs as string[] }));
}

test.describe("the value slot", () => {
  test("a paste, a reset and a keystroke in one gesture end on the keystroke, and the last input heard is the DOM value", async ({ page }) => {
    await page.addInitScript(witnessPage);
    await open(page, OTP_PAGE, OTP_INSTALL);
    const { value, inputs } = await gesture(page);
    expect(inputs[0]).toBe("123456");
    expect(value).toBe("7");
    expect(inputs.at(-1)).toBe(value);
  });

  test("the counterfactual: a value write that lands after the keystroke overwrites it and no input event reports it", async ({ page }) => {
    await page.addInitScript(witnessPage);
    await open(page, OTP_PAGE, OTP_INSTALL);
    await page.evaluate(() => {
      const w = (window as any).__dxWitness;
      w.arm({ site: "input_otp.filter", effect: "setValue", mode: "delay", frames: 3, host: "#otp" });
      w.freeze();
    });
    const { value, inputs } = await gesture(page);
    expect(value).toBe("123456");
    expect(inputs.at(-1)).not.toBe(value);
  });
});
