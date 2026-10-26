// The rule interpreter (row 434 step 3, "rules as data, one interpreter"). Rust declares a rule from
// the closed `Rule` enum (`primitives/src/bridge/rules.rs`) and serialises it onto an element, either
// through `__dxBridge.rules.install(el, rule)` (static configuration) or as a
// `data-dx-rule-<name>="<json>"` attribute (state that changes with renders, read by
// `rules.sync`). This file is the only place that evaluates one: it owns the listener (with its
// options), decides at event or frame time from data already in the DOM, and emits only a native
// effect and/or a message to Rust, which arrives after the commit and so may be asynchronous.
//
// The dispatch below is a `switch` over the rule discriminant with NO default arm: a rule kind
// exists in Rust and here, or in neither (`scripts/check-bridge-rules.sh`, which also keeps
// `preventDefault`, `stopPropagation`, `requestAnimationFrame` and the three observers out of every
// other `ts/` file). The four marked regions are where the later lanes put their families: B1 event
// gates, B2 drag and swipe, B3 the measurement publisher, B5 reveal and settle. A new rule kind
// needs a backlog row and a `bun test` case.

import type { FixtureRule, Rule, RuleScope } from "./types";

/** The attribute prefix of a rule that Rust mirrors into the DOM. */
export const RULE_ATTR_PREFIX = "data-dx-rule-";

/** Installs `rule` on `el` under `scope`. Returns the function that removes it again. */
export function applyRule(rule: Rule, el: Element, scope: RuleScope): () => void {
  switch (rule.kind) {
    // region:B1 begin (event gates)
    // region:B1 end
    // region:B2 begin (drag and swipe)
    // region:B2 end
    // region:B3 begin (measurement publisher)
    // region:B3 end
    // region:B5 begin (reveal and settle)
    // region:B5 end
    // region:FIXTURE begin (test fixtures only)
    case "fixture":
      return fixtureRule(rule, el, scope);
    // region:FIXTURE end
  }
  return unknownRule(rule.kind);
}

/** The rule attributes of `el`: `[attribute name, rule]` for each `data-dx-rule-*` that parses. */
export function readRuleAttributes(el: Element): [string, Rule][] {
  const out: [string, Rule][] = [];
  for (const name of el.getAttributeNames()) {
    if (!name.startsWith(RULE_ATTR_PREFIX)) continue;
    try {
      const rule = JSON.parse(el.getAttribute(name) ?? "");
      if (rule && typeof rule.kind === "string") out.push([name, rule as Rule]);
    } catch {
      // a malformed attribute is no rule
    }
  }
  return out;
}

/** Reached only when Rust sent a kind this build has no case for (a version skew): nothing installs. */
function unknownRule(kind: never): () => void {
  console.warn("dx bridge: unknown rule kind", kind);
  return () => {};
}

function fixtureRule(rule: FixtureRule, el: Element, scope: RuleScope): () => void {
  let fired = 0;
  return scope.on(el, rule.event, () => {
    fired += 1;
    el.setAttribute(rule.attr, rule.value);
    scope.send({ rule: "fixture", fired });
  });
}
