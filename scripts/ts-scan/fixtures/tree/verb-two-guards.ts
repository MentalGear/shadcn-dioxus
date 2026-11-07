// expect: verb-row
const METHODS: Record<string, { run: (el: HTMLDialogElement) => void }> = {
  twoGuards: {
    run(el) {
      if (el.open) {
        if (el.id) el.close();
      }
    },
  },
};
export { METHODS };
