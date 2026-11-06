// expect: dynamic-key-call
// bypass 3: `Reflect.get(e, 'prevent'+'Default')`
export function read(e: object): unknown {
  return Reflect.get(e, "prevent" + "Default");
}
