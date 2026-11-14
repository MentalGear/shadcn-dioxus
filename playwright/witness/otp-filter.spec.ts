// `otpFilter`'s witnesses (`prevent`, `setValue`), pre-authored as `test.fixme` keyed to the generated site list: they
// go live, with no edit here, when the library builds the site `input_otp.filter` (row 448). The vector in the binding
// is provisional: the first live run of the pair is the behaviour author's to confirm red.

import { test } from "../fixtures";
import { witness } from "./witness";

test.describe("otpFilter", () => {
  for (const t of [...witness("defaultAction", "input_otp.filter", "prevent"), ...witness("wrongPaint", "input_otp.filter", "setValue")]) test(t.title, t.run);
});
