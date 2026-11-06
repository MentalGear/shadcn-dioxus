//! The one definition of a controlled pair (backlog rows 392 and 387).
//!
//! `build.rs` and `src/main.rs` include this file with `#[path]`, so the "Controlled" block a component page
//! gets, the build failure for a half-present pair and the tests that pin both come from the same rules and
//! cannot drift apart. The block is generated from the component's own props, so it cannot be missing for a
//! new component and has no per-page prose to go stale; the explanation is written once, on `/docs#controlled`.
//!
//! A **controlled pair** is a state prop and the callback that reports its changes:
//! `X` + `on_X_change` (+ `default_X`, the uncontrolled start value). The state prop is `X`, else `selected_X`
//! (Calendar's `selected_date` with `on_date_change`), else the one name [`STATE_ALIASES`] gives for that struct.
//! An `on_X_change` with no state prop is an event (Avatar's `on_state_change`), not a pair.
//!
//! Two shapes are inconsistent and fail the build instead of being hidden:
//! - `default_X` with `on_X_change` but no state prop to control (the page would offer a start value and a
//!   change report for nothing that can be controlled), unless listed in [`OBSERVERS`];
//! - `X` with `default_X` but no `on_X_change` (a controlled prop whose changes are never reported), unless
//!   listed in [`UNREPORTED`].
//!
//! `std` only.
#![allow(
    dead_code,
    reason = "build.rs and main.rs include this file and each uses a different subset"
)]

/// The `h3` id of the generated section and its heading. The heading is a page-specific section, not a
/// recurring topic, so it sits with the page's own sections.
pub const SECTION_ID: &str = "docs-controlled-state";
pub const SECTION_HEADING: &str = "Controlled state";
/// Where the one explanation lives (the `Docs()` section "Controlled", whose anchor is `controlled`).
pub const EXPLANATION_HREF: &str = "/docs#controlled";

/// One props struct: its name and the names of its fields.
#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub fields: Vec<String>,
}

/// A controlled pair found in one struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    pub owner: String,
    pub state: String,
    pub callback: String,
    pub default: Option<String>,
}

/// `(struct, callback's X, state prop)`: a pair whose state prop is not named `X` or `selected_X`.
/// An alias that matches no struct field is a stale entry and `pairs` reports it.
pub const STATE_ALIASES: &[(&str, &str, &str)] = &[("DatePickerProps", "value", "selected_date")];

/// `(struct, X)`: `default_X` and `on_X_change` with no state prop, by design (DataTable's sort: a start sort
/// and a change report, the live sort being the view's own state).
pub const OBSERVERS: &[(&str, &str)] = &[("DataTableProps", "sort")];

/// `(struct, X)`: `X` and `default_X` without `on_X_change`, by design (the checkbox's internal bubble input
/// mirrors the checkbox's value to the form and never changes it).
pub const UNREPORTED: &[(&str, &str)] = &[("BubbleInput", "checked")];

/// The controlled pairs of `items` and every inconsistency, each as a sentence naming the struct and prop.
pub fn pairs(items: &[Item]) -> (Vec<Pair>, Vec<String>) {
    let mut found = Vec::new();
    let mut problems = Vec::new();
    for item in items {
        let has = |n: &str| item.fields.iter().any(|f| f == n);
        for alias in STATE_ALIASES.iter().filter(|a| a.0 == item.name) {
            if !has(alias.2) || !has(&format!("on_{}_change", alias.1)) {
                problems.push(format!(
                    "`{}`: STATE_ALIASES names `{}` for `on_{}_change`, but the struct has no such pair",
                    item.name, alias.2, alias.1
                ));
            }
        }
        let mut seen = Vec::new();
        for field in &item.fields {
            let Some(base) = field
                .strip_prefix("on_")
                .and_then(|rest| rest.strip_suffix("_change"))
            else {
                continue;
            };
            seen.push(base.to_string());
            let selected = format!("selected_{base}");
            let alias = STATE_ALIASES
                .iter()
                .find(|a| a.0 == item.name && a.1 == base)
                .map(|a| a.2);
            let state = if has(base) {
                Some(base.to_string())
            } else if let Some(alias) = alias.filter(|a| has(a)) {
                Some(alias.to_string())
            } else if has(&selected) {
                Some(selected)
            } else {
                None
            };
            let default = has(&format!("default_{base}")).then(|| format!("default_{base}"));
            match state {
                Some(state) => found.push(Pair {
                    owner: item.name.clone(),
                    state,
                    callback: field.clone(),
                    default,
                }),
                None if default.is_some()
                    && !OBSERVERS.iter().any(|o| o.0 == item.name && o.1 == base) =>
                {
                    problems.push(format!(
                        "`{}`: `default_{base}` and `on_{base}_change` have no `{base}` prop to control",
                        item.name
                    ));
                }
                None => {}
            }
        }
        for field in &item.fields {
            let Some(base) = field.strip_prefix("default_") else {
                continue;
            };
            if has(base)
                && !seen.iter().any(|s| s == base)
                && !UNREPORTED.iter().any(|u| u.0 == item.name && u.1 == base)
            {
                problems.push(format!(
                    "`{}`: `{base}` and `default_{base}` have no `on_{base}_change` to report changes",
                    item.name
                ));
            }
        }
    }
    (found, problems)
}

/// The generated section's body for `pairs` (never empty), as HTML. `EXPLANATION_HREF` is root-absolute like
/// every docs link, so the site's base-path rewrite applies to it.
pub fn render(pairs: &[Pair]) -> String {
    let mut out = format!(
        "<p>Some props here come in controlled pairs. <a href=\"{EXPLANATION_HREF}\">Controlled and uncontrolled state</a> says how they work, and what freezes a component.</p>\n<ul class=\"dx-docs-controlled-pairs\">\n"
    );
    let mut seen = Vec::new();
    for pair in pairs {
        if seen.contains(&(&pair.owner, &pair.state)) {
            continue;
        }
        seen.push((&pair.owner, &pair.state));
        let start = match &pair.default {
            Some(default) => format!("uncontrolled start value <code>{default}</code>"),
            None => "no start value: the app always owns it".to_string(),
        };
        out.push_str(&format!(
            "<li data-controlled-pair=\"{state}\"><code>{owner}</code>: <code>{state}</code> with <code>{callback}</code>; {start}.</li>\n",
            owner = pair.owner,
            state = pair.state,
            callback = pair.callback,
        ));
    }
    out.push_str("</ul>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, fields: &[&str]) -> Item {
        Item {
            name: name.to_string(),
            fields: fields.iter().map(|f| f.to_string()).collect(),
        }
    }

    #[test]
    fn a_full_pair_is_found() {
        let (found, problems) = pairs(&[item(
            "DialogProps",
            &["open", "on_open_change", "default_open", "children"],
        )]);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].state, "open");
        assert_eq!(found[0].default.as_deref(), Some("default_open"));
    }

    #[test]
    fn a_pair_without_a_default_is_still_a_pair() {
        let (found, problems) = pairs(&[item("ColorPickerProps", &["color", "on_color_change"])]);
        assert!(problems.is_empty());
        assert_eq!(found[0].default, None);
    }

    #[test]
    fn selected_and_aliased_state_props_are_found() {
        let (found, problems) = pairs(&[
            item("CalendarProps", &["selected_date", "on_date_change"]),
            item("DatePickerProps", &["selected_date", "on_value_change"]),
        ]);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(found.len(), 2);
        assert_eq!(found[1].state, "selected_date");
    }

    #[test]
    fn an_event_is_not_a_pair() {
        let (found, problems) = pairs(&[item("AvatarProps", &["on_state_change", "src"])]);
        assert!(found.is_empty() && problems.is_empty());
    }

    #[test]
    fn half_present_pairs_are_reported() {
        let (_, problems) = pairs(&[item("A", &["on_open_change", "default_open"])]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("no `open` prop to control"));
        let (_, problems) = pairs(&[item("B", &["value", "default_value"])]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("no `on_value_change`"));
    }

    #[test]
    fn listed_exceptions_are_quiet_and_stale_aliases_are_not() {
        let (_, problems) = pairs(&[
            item("DataTableProps", &["on_sort_change", "default_sort"]),
            item("BubbleInput", &["checked", "default_checked"]),
        ]);
        assert!(problems.is_empty(), "{problems:?}");
        let (_, problems) = pairs(&[item("DatePickerProps", &["on_value_change"])]);
        assert_eq!(problems.len(), 1, "{problems:?}");
    }

    #[test]
    fn the_block_links_the_explanation_once_per_state_prop() {
        let pair = Pair {
            owner: "SwitchProps".into(),
            state: "checked".into(),
            callback: "on_checked_change".into(),
            default: Some("default_checked".into()),
        };
        let html = render(&[pair.clone(), pair]);
        assert_eq!(html.matches(EXPLANATION_HREF).count(), 1);
        assert_eq!(html.matches("data-controlled-pair").count(), 1);
    }
}
