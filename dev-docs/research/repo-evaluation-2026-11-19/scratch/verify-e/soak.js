// usage: node soak.js <base> <component> <cycles> [every=100] [diff=0]
const {chromium}=require('playwright');
const [,,BASE,COMP,CYC,EVERY='100',DIFF='0']=process.argv;
const TO={open:0,close:0};const cycles=+CYC,every=+EVERY;
const init=`window.__mems=[];const wrap=(o)=>{const i=o&&(o.instance||o);if(i&&i.exports&&i.exports.memory)window.__mems.push(i.exports.memory);return o};
const _i=WebAssembly.instantiate;WebAssembly.instantiate=function(...a){return _i.apply(this,a).then(wrap)};
const _s=WebAssembly.instantiateStreaming;if(_s)WebAssembly.instantiateStreaming=function(...a){return _s.apply(this,a).then(wrap)};`;
const snapFn=()=>{const m=[];for(const e of document.querySelectorAll('*')){const p=e.parentElement;m.push([e.tagName,e.id,(e.getAttribute('class')||'').slice(0,80),e.getAttribute('data-slot')||'',p?p.tagName+'#'+p.id+'.'+(p.getAttribute('class')||'').slice(0,60)+'['+(p.getAttribute('data-slot')||'')+']':''].join('|'))}return m};
(async()=>{
 const b=await chromium.launch();const ctx=await b.newContext({viewport:{width:1280,height:900}});const p=await ctx.newPage();
 await p.addInitScript(init);
 const errs=[];p.on('console',m=>{if(['error','warning'].includes(m.type()))errs.push(m.type()+': '+m.text())});p.on('pageerror',e=>errs.push('pageerror '+e));
 const cdp=await ctx.newCDPSession(p);await cdp.send('HeapProfiler.enable');await cdp.send('Performance.enable');
 await p.goto(`${BASE}/component/${COMP}/`);await p.waitForSelector('html[data-hydrated=true]',{timeout:60000});
 const sel={dialog:'main button:has-text("Show Dialog")',popover:'main button:has-text("Show Popover")',dropdown_menu:'main button:has-text("Open Menu")',combobox:'main input[role=combobox]'}[COMP];
 const trig=p.locator(sel).first();
 const openSel={dialog:'[role=dialog],dialog[open]',popover:'dialog',dropdown_menu:'[role=menu]',combobox:'[role=listbox]'}[COMP];
 const cycle=async()=>{
  await trig.click();
  if(COMP==='combobox'){await p.keyboard.type('sv');}
  await p.waitForFunction(s=>!!document.querySelector(s),openSel,{timeout:5000}).catch(()=>{TO.open++});
  if(COMP==='combobox'){await p.keyboard.press('Control+a');await p.keyboard.press('Backspace');}
  await p.keyboard.press('Escape');
  await p.waitForFunction(s=>{const e=document.querySelector(s);return !e||!e.checkVisibility||!e.checkVisibility()},openSel,{timeout:5000}).catch(()=>{TO.close++});
 };
 const sample=async(n)=>{
  for(let g=0;g<3;g++){await cdp.send('HeapProfiler.collectGarbage');await p.waitForTimeout(250);}
  const wasm=await p.evaluate(()=>window.__mems.reduce((a,m)=>a+m.buffer.byteLength,0));
  const c=await cdp.send('Memory.getDOMCounters');
  const heap=(await cdp.send('Performance.getMetrics')).metrics.find(x=>x.name==='JSHeapUsedSize').value;
  const all=await p.evaluate(()=>document.querySelectorAll('*').length);
  const row={cycle:n,wasmMB:+(wasm/1048576).toFixed(2),heapMB:+(heap/1048576).toFixed(2),nodes:c.nodes,qsaAll:all,listeners:c.jsEventListeners};
  console.log(JSON.stringify(row));return row};
 for(let i=0;i<3;i++)await cycle(); // warm-up, not counted
 const t0=Date.now();await sample(0);
 let before;
 for(let n=1;n<=cycles;n++){
  await cycle();
  if(n%every===0)await sample(n);
  if(DIFF==='1'&&n===50){await p.evaluate(()=>{});before=await p.evaluate(snapFn);}
  if(DIFF==='1'&&n===250){const after=await p.evaluate(snapFn);const cnt=new Map();for(const k of after)cnt.set(k,(cnt.get(k)||0)+1);for(const k of before)cnt.set(k,(cnt.get(k)||0)-1);
   const d=[...cnt].filter(([k,v])=>v!==0);console.log('DIFF 200 cycles (key=tag|id|class|slot|parent, delta):');for(const [k,v] of d)console.log(v,k)}
 }
 console.log('done secs',((Date.now()-t0)/1000).toFixed(0),'timeouts',JSON.stringify(TO),'errs',JSON.stringify(errs.slice(0,5)));
 await b.close();
})().catch(e=>{console.log('FAIL',e.message);process.exit(1)});
