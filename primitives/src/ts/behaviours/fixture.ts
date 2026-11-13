// The one test fixture behaviour (row 434 step 3.0): proves install, epoch, lease, ref-counted
// registration, passive derivation and teardown end to end. No component uses it. A real behaviour is
// exactly this shape: its event fields and attribute reads are generated from the input types of its Rust
// `decide` (`decision-kernel/src/fixture.rs`), and the decision itself is the decision kernel's, so this module
// holds neither a list nor a `decide`. The listener, its phase and passivity come from the Rust `Behaviour`.

import { ADAPTER, kernelBehaviour, register } from "../lib/register";

register("fixture", kernelBehaviour("fixture", ADAPTER.fixture));
