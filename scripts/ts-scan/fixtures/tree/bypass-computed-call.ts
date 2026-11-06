// expect: computed-access
// root 5.3 item 4, bypass 1: `e[name]()`
export function call(e: Record<string, () => void>, name: string): void {
  e[name]();
}
