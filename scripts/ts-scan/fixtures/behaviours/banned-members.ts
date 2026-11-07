// expect: banned-member, constructor
export const decide = (x: object, f: Function) => [x.constructor, (x as { __proto__: unknown }).__proto__, f.prototype];
