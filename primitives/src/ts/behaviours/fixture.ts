// The one test fixture behaviour (row 434 step 3.0): proves install, epoch, lease, ref-counted
// registration, passive derivation and teardown end to end. No component uses it. A real behaviour is
// exactly this shape: a pure `decide` over closed event fields and closed attribute reads, registered
// under its kind; the listener, its phase and passivity come from the Rust `Behaviour`.

import { register } from "../lib/register";

register("fixture", {
  fields: ["type"],
  reads: ["data-dx-fixture-block"],
  decide(behaviour, _event, state) {
    // an attribute Rust mirrored before the event makes the behaviour stand down
    if (state["data-dx-fixture-block"] !== null) return {};
    return {
      ...(behaviour.cancel ? { prevent: true } : {}),
      attrs: { [behaviour.attr]: behaviour.value },
      send: { behaviour: "fixture", event: behaviour.event },
    };
  },
});
