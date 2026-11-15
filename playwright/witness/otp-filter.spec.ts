// `otpFilter`'s witnesses (`prevent`, `setValue`) at the site `input_otp.filter`, which the OTP field builds with
// `site!("input_otp.filter")`: they are live because the generated site list names it. The binding's vector replays
// a trusted text insertion on the real page; each pair is a control (two real runs agree) and an expected-failure
// witness (the effect moved to its next-best placement must leave the real arm's observables).

import { test } from "../fixtures";
import { witness } from "./witness";

test.describe("otpFilter", () => {
  for (const t of [...witness("defaultAction", "input_otp.filter", "prevent"), ...witness("wrongPaint", "input_otp.filter", "setValue")]) test(t.title, t.run);
});
