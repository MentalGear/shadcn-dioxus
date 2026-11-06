//! The one reset path for a text control whose value the owner's Rust signal holds.
//!
//! `<form>` reset restores a control's own DOM state (a text field's `value` goes back to its
//! `value` content attribute, a checkbox's `checked` to `defaultChecked`) and tells no Rust state
//! about it. Without a reset path, a controlled text `Input` or `Textarea` shows the default while its
//! owner's signal keeps the old text, and the next render of the owner writes the old text back.
//!
//! [`use_form_reset_value`] closes that. It listens for the owning form's `reset` through
//! [`use_form_reset_listener`], the listener every other
//! form-associated primitive already uses (`Checkbox`, `Switch`, `RadioGroup`, `Select`, `Slider`,
//! `InputOtp`, `Combobox`), then reads the control's value after the browser restored it and hands
//! it to `on_reset`.
//!
//! **The rule.** A reset is a value change to the control's default, so the text control reports it
//! the way an edit is reported: the wrapper calls its `oninput` and then its `onchange` with the
//! restored value, and an owner that mirrors the field into a signal follows without knowing about
//! forms. An uncontrolled field already holds its default in the DOM; its handlers see the same
//! event and ignore it. Reading the DOM after the reset (not predicting the default) means a
//! `reset` the page cancelled with `preventDefault` reports the unchanged value, which is a no-op
//! for the owner.
//!
//! **One read for the whole form.** A reset restores every field at once, and a controlled field
//! whose owner re-renders writes the owner's text back into the DOM. If each field read its own
//! value in its own task, the first field's report would re-render the owner and put the old text
//! back into the fields not read yet. The fields that heard the reset therefore queue
//! themselves, one task (a task after the browser restored the values) reads all of them in one
//! bridge transaction, and the handlers then run back to back, so the owner sees every restored
//! value before it renders once.

use std::cell::RefCell;

use dioxus::html::{FileData, FormData, FormValue, HasFileData, HasFormData};
use dioxus::prelude::*;

use crate::bridge::{self, Prop, Target};
use crate::{use_form_reset_listener, use_unique_id};

/// The `id` the control carries: the caller's own, else the one this hook generated.
fn caller_id(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .find(|attribute| attribute.name == "id")
        .and_then(|attribute| match &attribute.value {
            dioxus_core::AttributeValue::Text(text) => Some(text.to_string()),
            _ => None,
        })
}

/// The data of a synthetic form event: the control's value, no files, no other fields.
struct ResetFormData(String);

impl HasFileData for ResetFormData {
    fn files(&self) -> Vec<FileData> {
        Vec::new()
    }
}

impl HasFormData for ResetFormData {
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

/// A synthetic `input`/`change` event carrying `value`, for the handlers of a reset field.
pub fn reset_form_event(value: String) -> FormEvent {
    Event::new(std::rc::Rc::new(FormData::new(ResetFormData(value))), false)
}

/// A control that heard a reset: its id and what to do with its restored value.
type Waiting = (String, Box<dyn FnMut(String)>);

thread_local! {
    /// The controls that heard a reset and wait for the one read; `None` while no read is scheduled.
    static QUEUE: RefCell<Option<Vec<Waiting>>> = const { RefCell::new(None) };
}

/// Adds a control to the next read, scheduling it when it is the first.
fn queue_read(id: String, on_reset: Box<dyn FnMut(String)>) {
    let first = QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();
        match queue.as_mut() {
            Some(waiting) => {
                waiting.push((id, on_reset));
                false
            }
            None => {
                *queue = Some(vec![(id, on_reset)]);
                true
            }
        }
    });
    if first {
        // interval-ok: one task per reset, ends after it has reported; it must outlive the first field's component
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
                transaction = transaction.get(Target::id(id.clone()), [Prop::Value]);
            }
            let Ok(results) = transaction.run().await else {
                return;
            };
            let values: Vec<Option<String>> = results
                .into_iter()
                .map(|result| match result {
                    bridge::OpResult::Ok { ok, .. } => serde_json::from_value::<bridge::Props>(ok)
                        .ok()
                        .and_then(|props| props.string(&Prop::Value).map(str::to_owned)),
                    _ => None,
                })
                .collect();
            for ((_, mut on_reset), value) in waiting.into_iter().zip(values) {
                if let Some(value) = value {
                    on_reset(value);
                }
            }
        });
    }
}

/// Reports a form reset of the text control `attributes` belongs to: after the browser restored the
/// control's value, `on_reset` runs with it. Returns the attributes to add to the control: an `id`
/// when the caller passed none (the listener finds the control and its form by id).
///
/// Call it in the wrapper component that renders the `<input>` or `<textarea>`, pass `attributes`
/// as the caller supplied them and merge the returned ones in.
pub fn use_form_reset_value(
    attributes: &[Attribute],
    on_reset: impl FnMut(String) + Clone + 'static,
) -> Vec<Attribute> {
    let mut id = use_unique_id();
    let user_id = caller_id(attributes);
    use_effect(use_reactive!(|user_id| {
        if let Some(user_id) = user_id {
            id.set(user_id);
        }
    }));
    use_form_reset_listener(id, move || {
        let on_reset = on_reset.clone();
        queue_read(id.peek().clone(), Box::new(on_reset));
    });
    if caller_id(attributes).is_some() {
        Vec::new()
    } else {
        vec![Attribute::new("id", id(), None, false)]
    }
}
