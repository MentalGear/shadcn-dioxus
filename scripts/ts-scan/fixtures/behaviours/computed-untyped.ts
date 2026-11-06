// expect: computed-access
// `{ a: number }` has no index signature, so a string key is an untyped computed read
export const decide = (o: { a: number }, k: string) => (o as unknown as { [key: string]: number })[k] + (o as { a: number })[k as "a"];
