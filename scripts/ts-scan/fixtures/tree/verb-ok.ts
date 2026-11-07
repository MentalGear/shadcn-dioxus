// expect: none
// the allowed shapes: one member read, one call, one guard, a branch on an argument
function pick(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}
const METHODS: Record<string, { run: (el: HTMLDialogElement, args: unknown[]) => void }> = {
  blur: { run: (el) => el.blur() },
  showModal: {
    run(el) {
      if (!el.open) el.showModal();
    },
  },
  close: {
    run(el, args) {
      if (el.open) typeof args[0] === "string" ? el.close(args[0]) : el.close();
    },
  },
  select: { run: (el, args) => el.setAttribute("x", pick(args[0]) ?? "") },
};
const PROPS: Record<string, { get: (el: HTMLElement) => unknown }> = {
  offsetWidth: { get: (el) => el.offsetWidth },
  open: { get: (el) => !!(el as HTMLDialogElement).open },
};
export { METHODS, PROPS };
