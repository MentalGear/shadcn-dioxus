// The one test fixture behaviour (row 434 step 3.0): proves install, epoch, lease, ref-counted
// registration, passive derivation and teardown end to end. No component uses it. A real behaviour is
// exactly this shape: the closed event fields and attribute reads its Rust `decide` needs, registered under its
// kind; the decision itself is the decision kernel's (`decision-kernel/src/fixture.rs`), and the listener, its
// phase and passivity come from the Rust `Behaviour`.

import { kernelDecide, register } from "../lib/register";

register("fixture", {
  fields: ["type"],
  reads: ["data-dx-fixture-block"],
  decide: kernelDecide("fixture"),
});
