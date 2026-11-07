// expect: behaviour-import
import { helper } from "./elsewhere";
import type { Thing } from "./elsewhere";
export const decide = (t: Thing) => helper(t);
