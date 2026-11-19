// usage: node heapdiff.js <url> <inputSelector-or-trigger> <mode> <cycles>; mode: full|click
const {chromium}=require('playwright');
const [,,URL,SEL,MODE,CYC='200']=process.argv;
(async()=>{const b=await chromium.launch();const ctx=await b.newContext({viewport:{width:1280,height:900}});const p=await ctx.newPage();const cdp=await ctx.newCDPSession(p);await cdp.send('HeapProfiler.enable');
 await p.goto(URL);await p.waitForSelector('html[data-hydrated=true]');const t=p.locator(SEL).first();
 const cyc=async()=>{await t.click();await p.waitForTimeout(40);if(MODE==='full'){await p.keyboard.type('sv');await p.keyboard.press('Control+a');await p.keyboard.press('Backspace');}await p.keyboard.press('Escape');await p.waitForTimeout(40)};
 async function snap(){for(let g=0;g<3;g++){await cdp.send('HeapProfiler.collectGarbage');await p.waitForTimeout(200)};let chunks=[];const h=e=>chunks.push(e.chunk);cdp.on('HeapProfiler.addHeapSnapshotChunk',h);await cdp.send('HeapProfiler.takeHeapSnapshot',{reportProgress:false});cdp.off('HeapProfiler.addHeapSnapshotChunk',h);
  const j=JSON.parse(chunks.join(''));const m=j.snapshot.meta;const nf=m.node_fields.length;const ti=m.node_fields.indexOf('type'),ni=m.node_fields.indexOf('name');const types=m.node_types[0];const cnt=new Map();
  for(let i=0;i<j.nodes.length;i+=nf){const ty=types[j.nodes[i+ti]];const nm=j.strings[j.nodes[i+ni]];if(ty==='object'||ty==='native'){const k=ty+' '+nm.slice(0,60);cnt.set(k,(cnt.get(k)||0)+1)}}return cnt}
 for(let i=0;i<5;i++)await cyc();const a=await snap();for(let i=0;i<+CYC;i++)await cyc();const z=await snap();
 const d=[];for(const [k,v] of z){const dv=v-(a.get(k)||0);if(dv>=+CYC*0.5)d.push([dv,k])}d.sort((x,y)=>y[0]-x[0]);
 console.log('objects growing >=0.5 per cycle over',CYC,'cycles (count delta, type name):');for(const [dv,k] of d.slice(0,15))console.log(dv,k);
 await b.close()})();
