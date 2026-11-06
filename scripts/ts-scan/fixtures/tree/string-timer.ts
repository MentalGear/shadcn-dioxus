// expect: string-sink
// a string handed to a timer is code; a function is not
export function later(): void {
  window.setTimeout("alert(1)", 10);
  window.setTimeout(() => {}, 10);
  setInterval(() => {}, 10);
}
