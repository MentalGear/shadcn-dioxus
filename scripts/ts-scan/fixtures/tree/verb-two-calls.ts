// expect: verb-row
// a verb row is one member call; two calls are a program
const METHODS: Record<string, { run: (el: HTMLElement) => void }> = {
  blurThenFocus: { run: (el) => { el.blur(); el.focus(); } },
};
export { METHODS };
