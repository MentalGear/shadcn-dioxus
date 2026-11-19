const {chromium}=require('playwright');
(async()=>{const b=await chromium.launch();const p=await b.newPage();
await p.goto('http://127.0.0.1:8741/shadcn-dioxus/component/dialog/');await p.waitForSelector('html[data-hydrated=true]');
const n=()=>p.evaluate(()=>[performance.getEntriesByType('mark').length,performance.getEntriesByType('measure').length]);
console.log('after load marks/measures',await n());
const t=p.locator('main button:has-text("Show Dialog")').first();
for(let i=0;i<20;i++){await t.click();await p.waitForTimeout(40);await p.keyboard.press('Escape');await p.waitForTimeout(40)}
console.log('after 20 dialog cycles',await n());
console.log(JSON.stringify(await p.evaluate(()=>{const e=performance.getEntriesByType('measure');const c={};for(const x of e.slice(-400)){const k=x.name.replace(/\d+/g,'N').slice(0,90);c[k]=(c[k]||0)+1}return Object.entries(c).sort((a,b)=>b[1]-a[1]).slice(0,6)})));
console.log(JSON.stringify(await p.evaluate(()=>performance.getEntriesByType('mark').slice(-3).map(m=>m.name.slice(0,100)))));
await b.close()})();
