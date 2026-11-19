// usage: node retain2.js <base> <comp> <cycles> : created nodes (createElement/createTextNode/cloneNode/importNode/innerHTML-less) still alive but not connected after forced GC
const {chromium}=require('playwright');
const [,,BASE,COMP,CYC='60']=process.argv;
const init=`window.__made=[];window.__on=false;
const reg=(n,how)=>{if(window.__on&&n)window.__made.push([new WeakRef(n),how]);return n};
const P=(o,m,how)=>{const f=o[m];o[m]=function(...a){return reg(f.apply(this,a),how+':'+(a[0]&&a[0].toString?String(a[0]).slice(0,20):''))}};
P(Document.prototype,'createElement','createElement');P(Document.prototype,'createElementNS','createElementNS');P(Document.prototype,'createTextNode','createTextNode');P(Document.prototype,'createComment','createComment');
P(Node.prototype,'cloneNode','cloneNode');P(Document.prototype,'importNode','importNode');`;
(async()=>{const b=await chromium.launch();const ctx=await b.newContext({viewport:{width:1280,height:900}});const p=await ctx.newPage();
 await p.addInitScript(init);const cdp=await ctx.newCDPSession(p);await cdp.send('HeapProfiler.enable');
 await p.goto(`${BASE}/component/${COMP}/`);await p.waitForSelector('html[data-hydrated=true]');
 const sel={dialog:'main button:has-text("Show Dialog")',popover:'main button:has-text("Show Popover")',dropdown_menu:'main button:has-text("Open Menu")',combobox:'main input[role=combobox]'}[COMP];
 const t=p.locator(sel).first();
 const cyc=async()=>{await t.click();if(COMP==='combobox')await p.keyboard.type('sv');await p.waitForTimeout(40);if(COMP==='combobox'){await p.keyboard.press('Control+a');await p.keyboard.press('Backspace');}await p.keyboard.press('Escape');await p.waitForTimeout(40);};
 for(let i=0;i<3;i++)await cyc();
 await p.evaluate(()=>{window.__made.length=0;window.__on=true});
 const c0=(await cdp.send('Memory.getDOMCounters')).nodes;
 for(let i=0;i<+CYC;i++)await cyc();
 await p.waitForTimeout(500);
 for(let i=0;i<3;i++){await cdp.send('HeapProfiler.collectGarbage');await p.waitForTimeout(200)}
 const c1=(await cdp.send('Memory.getDOMCounters')).nodes;
 const r=await p.evaluate(()=>{const g={};let made=0,alive=0,aliveDetached=0;for(const [w,how] of window.__made){made++;const n=w.deref();if(!n)continue;alive++;if(n.isConnected)continue;aliveDetached++;
   let root=n;while(root.parentNode)root=root.parentNode;const rk=root===n?'SELF-ROOT':(root.nodeType===1?root.tagName+'.'+(root.getAttribute('class')||'').slice(0,40)+'|id='+root.id:'#'+root.nodeType);const k=rk+' <= '+how+'|'+(n.nodeType===1?n.tagName+'|cls='+(n.getAttribute('class')||'').slice(0,40)+'|slot='+(n.getAttribute('data-slot')||'')+'|role='+(n.getAttribute('role')||''):'#'+n.nodeType)+'|hasParent='+!!n.parentNode;g[k]=(g[k]||0)+1}
  return {made,alive,aliveDetached,g}});
 console.log('cycles',CYC,'CDP nodes before/after',c0,c1,'delta/cycle',((c1-c0)/+CYC).toFixed(1));
 console.log('created while observing',r.made,'alive after GC',r.alive,'alive & detached',r.aliveDetached);
 for(const [k,v] of Object.entries(r.g).sort((a,b)=>b[1]-a[1]).slice(0,12))console.log(v,k);
 await b.close()})().catch(e=>{console.log('FAIL',e.message);process.exit(1)});
