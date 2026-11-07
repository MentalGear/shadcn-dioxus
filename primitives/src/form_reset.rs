//! The one path by which a DOM-side value change reaches the owner of a form control.
//!
//! A form control's value has two writers: the browser (the DOM) and the owner's Rust signal.
//! Whenever the browser changes the value and the owner is not told, the two disagree and the next
//! render writes the owner's stale value back over the DOM. The browser changes a value without an
//! `input` event in two cases:
//!
//! - **a `<form>` reset** (`form.reset()`, a `<button type=reset>`): a text field's `value` goes
//!   back to its `value` content attribute, a checkbox's `checked` to `defaultChecked`;
//! - **form-state restore**: history navigation (and a reload in some engines) puts the previous
//!   page's field values back into the server-rendered controls before the wasm hydrates, so the
//!   restored text is in the DOM while the owner's signal holds the default. (A back/forward-cache
//!   restore keeps the whole page alive, DOM and signals alike, so it cannot disagree.)
//!
//! **The rule.** Every DOM-side value change reaches the owner through the control's ONE change
//! callback, for controlled and uncontrolled alike: a text control reports it the way an edit is
//! reported (`oninput`, then `onchange`, with the new value); a checkbox, switch, radio group,
//! select, slider, OTP field and combobox call their value-change callback with the default. An
//! owner that mirrors the control into a signal follows without knowing about forms, and there is
//! no separate `onreset` an owner could forget to wire. React differs: it reports no reset to a
//! controlled input (a long-standing React issue), so the state keeps the old value and React
//! rewrites the DOM from it.
//!
//! Two kinds of control are covered here:
//!
//! - a **text-like control the owner holds** (`Input`, `Textarea`, `NativeSelect`, and the OTP
//!   field) reports through [`use_form_value_report`] (attributes) or [`use_dom_value_report`]
//!   (an id): the DOM value after a reset, and the DOM value at mount when it differs from the one
//!   the owner rendered (a restored field). Both read the DOM, so a `reset` the page cancelled with
//!   `preventDefault` reports the unchanged value, a no-op for the owner.
//! - a **mirror input** a primitive renders only so a form submits the primitive's state (the
//!   checkbox, switch, radio and select mirrors) is derived output: Rust is its only writer. It
//!   carries `autocomplete="off"` ([`MIRROR_AUTOCOMPLETE`]) so the browser never restores a value
//!   into it, and a reset reaches the primitive through its own value-change callback.
//!
//! **One read for the whole form.** A reset or a restore changes many fields at once, and a
//! controlled field whose owner re-renders writes the owner's text back into the DOM. If each field
//! read its own value in its own task, the first field's report would re-render the owner and put
//! the old text back into the fields not read yet. The fields that heard the event therefore queue
//! themselves, one task (after the browser restored the values) reads all of them in one bridge
//! transaction, and the handlers then run back to back, so the owner sees every value before it
//! renders once.

use std::cell::RefCell;

use dioxus::html::{FileData, FormData, FormValue, HasFileData, HasFormData};
use dioxus::prelude::*;

use crate::bridge::{self, Prop, Target};
use crate::caller_id::caller_id;
use crate::{use_form_reset_listener, use_unique_id};

/// The text of the `value` attribute the caller passed, if it passed one as text.
fn text_value(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .find(|attribute| attribute.name == "value")
        .and_then(|attribute| match &attribute.value {
            dioxus_core::AttributeValue::Text(text) => Some(text.to_string()),
            _ => None,
        })
}

/// The data of a synthetic form event: the control's value, no files, no other fields.
struct ReportFormData(String);

impl HasFileData for ReportFormData {
    fn files(&self) -> Vec<FileData> {
        Vec::new()
    }
}

impl HasFormData for ReportFormData {
    fn value(&self) -> String {
        self.0.clone()
    }

    fn valid(&self) -> bool {
        true
    }

    fn values(&self) -> Vec<(String, FormValue)> {
        Vec::new()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A synthetic `input`/`change` event carrying `value`, for the handlers of a control whose value
/// the browser changed.
fn value_report_event(value: String) -> FormEvent {
    Event::new(
        std::rc::Rc::new(FormData::new(ReportFormData(value))),
        false,
    )
}

/// What a read found in a control: its `value` and its `defaultValue` (what the server rendered).
struct Read {
    value: String,
    default: Option<String>,
}

/// A control that heard a reset or mounted: its id and what to do with what is read.
type Waiting = (String, Box<dyn FnMut(Read)>);

thread_local! {
    /// The controls waiting for the one read; `None` while no read is scheduled.
    static QUEUE: RefCell<Option<Vec<Waiting>>> = const { RefCell::new(None) };
}

/// Adds a control to the next read, scheduling it when it is the first.
fn queue_read(id: String, on_value: Box<dyn FnMut(Read)>) {
    let first = QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();
        match queue.as_mut() {
            Some(waiting) => {
                waiting.push((id, on_value));
                false
            }
            None => {
                *queue = Some(vec![(id, on_value)]);
                true
            }
        }
    });
    if first {
        // interval-ok: one task per read batch, ends after it has reported; it must outlive the first field's component
        crate::dioxus_core::spawn_forever(async {
            // The `reset` event is dispatched before the browser restores the values, and the
            // microtasks that carried every field's message run in between: the next task is after
            // the restore.
            // motion-ok: a task boundary, not a duration; it orders this read after the browser's restore
            crate::sleep::sleep(std::time::Duration::from_millis(0)).await;
            let waiting = QUEUE
                .with(|queue| queue.borrow_mut().take())
                .unwrap_or_default();
            let mut transaction = bridge::Transaction::new();
            for (id, _) in &waiting {
                transaction =
                    transaction.get(Target::id(id.clone()), [Prop::Value, Prop::DefaultValue]);
            }
            let Ok(results) = transaction.run().await else {
                return;
            };
            let values: Vec<Option<Read>> = results
                .into_iter()
                .map(|result| match result {
                    bridge::OpResult::Ok { ok, .. } => serde_json::from_value::<bridge::Props>(ok)
                        .ok()
                        .and_then(|props| {
                            Some(Read {
                                value: props.string(&Prop::Value)?.to_owned(),
                                default: props.string(&Prop::DefaultValue).map(str::to_owned),
                            })
                        }),
                    _ => None,
                })
                .collect();
            for ((_, mut on_value), value) in waiting.into_iter().zip(values) {
                if let Some(value) = value {
                    on_value(value);
                }
            }
        });
    }
}

/// The `autocomplete` value of a mirror input: the browser never restores a form-state value into
/// it, because the primitive that renders it is its only writer.
pub const MIRROR_AUTOCOMPLETE: &str = "off";

/// Reports the value of the control with `id` once after mount when it differs from `owner_value` and
/// from the control's `defaultValue`: a form-state restore put text into the server-rendered control
/// before the wasm hydrated, or the user typed before it did. A control still holding what the server
/// rendered is not reported, even when that is less than the owner holds.
///
/// `owner_value` is the value the owner's Rust state holds, `None` when the owner holds none the
/// control could be compared with (then nothing is reported); it is read once, at mount. `report`
/// runs with the DOM's value. A control that resets itself through its own value-change callback
/// (`Slider`, `InputOtp`) uses this and not [`use_dom_value_report`].
pub fn use_dom_value_restore(
    id: impl Readable<Target = String> + Copy + 'static,
    owner_value: impl Fn() -> Option<String> + Clone + 'static,
    report: impl FnMut(String) + Clone + 'static,
) {
    use_effect(move || {
        let owner_value = owner_value.clone();
        let mut report = report.clone();
        queue_read(
            id.peek().clone(),
            Box::new(move |read: Read| {
                // Only a value that differs from the server's default is a browser restore or an edit the
                // owner was not told about; a control the server rendered with less than the owner holds
                // (a `<textarea>` has no `value` attribute) still holds its default.
                if read.default.as_deref() != Some(read.value.as_str())
                    && owner_value().is_some_and(|owner| owner != read.value)
                {
                    report(read.value);
                }
            }),
        );
    });
}

/// Reports every DOM-side value change of the control with `id` to `report`: the value after a form
/// reset, and what [`use_dom_value_restore`] reports at mount. `report` runs after the browser
/// restored the DOM, with the DOM's value.
pub fn use_dom_value_report(
    id: impl Readable<Target = String> + Copy + 'static,
    owner_value: impl Fn() -> Option<String> + Clone + 'static,
    report: impl FnMut(String) + Clone + 'static,
) {
    let on_reset = report.clone();
    use_form_reset_listener(id, move || {
        let mut on_reset = on_reset.clone();
        queue_read(
            id.peek().clone(),
            Box::new(move |read: Read| on_reset(read.value)),
        );
    });
    use_dom_value_restore(id, owner_value, report);
}

/// [`use_dom_value_report`] for a text control the wrapper renders from `attributes` (an `<input>`,
/// a `<textarea>` or a `<select>`): the report is the control's `oninput` and then its `onchange`,
/// each with a [`FormEvent`] carrying the DOM value, and the owner's value is the `value`
/// attribute (`owner_default` when it passed none: `Some("")` for an `<input>` or `<textarea>`,
/// `None` for a `<select>`, whose DOM value is its first option's and so says nothing). Returns the attributes to add to the control: an `id` when the caller passed none
/// (the listener and the read find the control by id).
///
/// Call it in the wrapper component that renders the control, pass `attributes` as the caller
/// supplied them and merge the returned ones in.
pub fn use_form_value_report(
    attributes: &[Attribute],
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
    owner_default: Option<&'static str>,
) -> Vec<Attribute> {
    let mut id = use_unique_id();
    let user_id = caller_id(attributes);
    use_effect(use_reactive!(|user_id| {
        if let Some(user_id) = user_id {
            id.set(user_id);
        }
    }));
    let mut owner_value = use_hook(|| CopyValue::new(None::<String>));
    owner_value
        .set(text_value(attributes).or_else(|| owner_default.map(str::to_owned)));
    use_dom_value_report(
        id,
        move || owner_value.cloned(),
        move |value: String| {
            if let Some(handler) = oninput {
                handler.call(value_report_event(value.clone()));
            }
            if let Some(handler) = onchange {
                handler.call(value_report_event(value));
            }
        },
    );
    if caller_id(attributes).is_some() {
        Vec::new()
    } else {
        vec![Attribute::new("id", id(), None, false)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::with_runtime;

    fn id_attribute(value: &str) -> Attribute {
        Attribute::new("id", value.to_string(), None, false)
    }

    /// The attributes `use_form_value_report` hands back for a control given `attributes`.
    fn owned_by_report(attributes: Vec<Attribute>) -> Vec<Attribute> {
        with_runtime(move || use_form_value_report(&attributes, None, None, Some("")))
    }

    #[test]
    fn a_control_with_an_empty_id_gets_a_generated_one() {
        // `id=""` names nothing, so the listener and the read need an id of the report's own.
        let owned = owned_by_report(vec![id_attribute("")]);
        assert!(
            owned.iter().any(|a| a.name == "id"),
            "an empty caller id must not count as the caller's id"
        );
    }

    #[test]
    fn a_control_with_a_caller_id_keeps_it() {
        assert!(owned_by_report(vec![id_attribute("name")]).is_empty());
    }
}
