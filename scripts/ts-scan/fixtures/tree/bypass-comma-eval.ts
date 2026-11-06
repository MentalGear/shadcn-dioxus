// expect: as-any, string-sink
// bypass 4: `(0, W.eval)('1')` on an `any` receiver, which the checker cannot resolve
const W = window as any;
export function run(): void {
  (0, W.eval)("1");
}
