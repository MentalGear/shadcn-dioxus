// expect: none
// the one allowed computed read: a closed table through `own`
const hasOwn = (table: object, key: string): boolean => Object.prototype.hasOwnProperty.call(table, key);
export function own<T>(table: Record<string, T>, key: unknown): T | undefined {
  return typeof key === "string" && hasOwn(table, key) ? table[key] : undefined;
}
