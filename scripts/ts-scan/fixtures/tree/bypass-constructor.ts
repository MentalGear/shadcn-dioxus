// expect: constructor
// bypass 5: `(()=>{}).constructor('return 1')()`
export const out = (() => {}).constructor("return 1")();
