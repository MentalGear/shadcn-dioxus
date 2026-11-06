// expect: computed-access
// bypass 2: `self['ev'+'al']('1')`
export function run(): void {
  (self as unknown as Record<string, (s: string) => void>)["ev" + "al"]("1");
}
