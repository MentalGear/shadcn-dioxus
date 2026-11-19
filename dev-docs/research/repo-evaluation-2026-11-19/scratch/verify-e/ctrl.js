const {chromium}=require('playwright');
(async()=>{const b=await chromium.launch();const ctx=await b.newContext();const p=await ctx.newPage();const cdp=await ctx.newCDPSession(p);await cdp.send('HeapProfiler.enable');
const dc=async()=>{for(let g=0;g<3;g++){await cdp.send('HeapProfiler.collectGarbage');await p.waitForTimeout(150)};return (await cdp.send('Memory.getDOMCounters')).nodes};
async function run(label,url,sel,steps,n=150){await p.goto(url);if(url.startsWith('http'))await p.waitForSelector('html[data-hydrated=true]');const t=p.locator(sel).first();const a=await dc();for(let i=0;i<n;i++)await steps(t);const z=await dc();console.log(label,'nodes',a,'->',z,'per cycle',((z-a)/n).toFixed(2))}
await run('PLAIN input click+type+ctrl-a+bksp+esc','data:text/html,<input id=i><p>x</p>','input',async t=>{await t.click();await p.keyboard.type('sv');await p.keyboard.press('Control+a');await p.keyboard.press('Backspace');await p.keyboard.press('Escape')});
const U='http://127.0.0.1:8741/shadcn-dioxus/component/combobox/';
await run('combobox click only','http://127.0.0.1:8741/shadcn-dioxus/component/combobox/','main input[role=combobox]',async t=>{await t.click();await p.waitForTimeout(40);await p.keyboard.press('Escape');await p.waitForTimeout(40)});
await run('combobox click+Escape+type sv (no clear)','http://127.0.0.1:8741/shadcn-dioxus/component/combobox/','main input[role=combobox]',async t=>{await t.click();await p.keyboard.type('sv');await p.waitForTimeout(40);await p.keyboard.press('Escape');await p.waitForTimeout(40)});
await run('combobox full (type, ctrl-a, bksp, esc)',U,'main input[role=combobox]',async t=>{await t.click();await p.keyboard.type('sv');await p.waitForTimeout(40);await p.keyboard.press('Control+a');await p.keyboard.press('Backspace');await p.keyboard.press('Escape');await p.waitForTimeout(40)});
await b.close()})();
