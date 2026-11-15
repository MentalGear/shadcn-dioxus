// The executor's internals (the exit wait and the watchdog), admitted as a whole by the reds they already have:
// pre-authored as `test.fixme` keyed to the generated site list, live when the executor builds its sites (row 449).
// The exit wait: Rust waits for a `finished` fact, red because an invisible modal holds the page inert. The watchdog:
// dropped, so the CSS fail-open alone must keep a pending state from staying open. The popover show: the executor's
// one show of a registered popover (`executor.popover`), delayed to its next-best placement, paints the menu late.

import { test } from "../fixtures";
import { witness } from "./witness";

test.describe("the executor's internals", () => {
  for (const t of [...witness("wrongPaint", "executor.exit_wait", "hide"), ...witness("wrongPaint", "executor.watchdog", "attrs", { mode: "drop" }), ...witness("wrongPaint", "executor.popover", "show")]) test(t.title, t.run);
});
