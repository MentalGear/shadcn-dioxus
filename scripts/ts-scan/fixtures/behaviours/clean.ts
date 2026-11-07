// expect: none
// a behaviour's shape: pure, closed fields, the allowed free identifiers only
type State = Record<string, string | null>;
export const decide = (behaviour: { cancel: boolean; attr: string; value: string }, state: State) => {
  if (state["data-dx-fixture-block"] !== null) return {};
  const keys = Object.keys(state);
  return {
    ...(behaviour.cancel ? { prevent: true } : {}),
    attrs: { [behaviour.attr]: String(Math.max(Number(behaviour.value), keys.length)) },
    frozen: Object.freeze(Array.from(keys)),
    pattern: new RegExp("^a$"),
    typed: state[behaviour.attr],
  };
};
