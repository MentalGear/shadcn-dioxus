// `otpFilter`'s witnesses at the site `input_otp.filter`, which the OTP field builds with `site!("input_otp.filter")`:
// they are live because the generated site list names it. Each pair is a control (two real runs agree) and an
// expected-failure witness (the effect moved to its next-best placement must leave the real arm's observables).
//
// `setValue` is red: a write that lands after the next keystroke paints states the real arm never painted. The binding
// types a rejected character into the alphanumeric field and the next key one paint later, which is when the delayed
// write overwrites it.
//
// `prevent` is green on Chromium: the next-best placement is a cancel in the bubble phase at the host, and a
// `beforeinput` cancelled there is cancelled all the same, so the counterfactual leaves every default-action observable
// where it was. The witness is `test.fixme` until the owner decides the admission of this effect (a platform rule row,
// or the cancel moves to an rsx handler when Dioxus delivers `onbeforeinput`); the control still runs.

import { test } from "../fixtures";
import { witness } from "./witness";

test.describe("otpFilter", () => {
  const [prevent, preventWitness] = witness("defaultAction", "input_otp.filter", "prevent");
  test(prevent.title, prevent.run);
  test.fixme(preventWitness.title, preventWitness.run);
  for (const t of witness("wrongPaint", "input_otp.filter", "setValue")) test(t.title, t.run);
});
