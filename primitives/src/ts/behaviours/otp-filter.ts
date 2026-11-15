// The one-time-passcode field's filter: registers the generated adapter of the kernel's `otp-filter` kind. Its event
// fields (the edit's text, the selection and the value) come from the input types of its Rust `decide`
// (`decision-kernel/src/otp_filter.rs`), the decision is the kernel's, and the effects it asks for (cancel the edit, write
// a range of the value) are applied by the bridge's apply table. The listener, its phase and passivity come from the Rust
// `Behaviour`, installed once per event (`beforeinput`, `input`, `compositionend`).

import { ADAPTER, kernelBehaviour, register } from "../lib/register";

register("otp-filter", kernelBehaviour("otp-filter", ADAPTER["otp-filter"]));
