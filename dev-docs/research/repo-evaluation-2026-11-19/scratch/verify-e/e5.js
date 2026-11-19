const {chromium}=require('playwright');const fs=require('fs');
const BASE=process.argv[2];const pages=process.argv.slice(3);
const axeSrc=fs.readFileSync('/home/user/shadcn-dioxus/playwright/node_modules/.pnpm/axe-core@4.10.3/node_modules/axe-core/axe.min.js','utf8');
(async()=>{const b=await chromium.launch();
for(const pg of pages){
 const ctx=await b.newContext({viewport:{width:1280,height:900}});const p=await ctx.newPage();
 const bad=[];p.on('console',m=>{if(['error','warning'].includes(m.type()))bad.push(m.type()+': '+m.text().slice(0,120))});p.on('pageerror',e=>bad.push('pageerror '+e));p.on('requestfailed',r=>bad.push('reqfailed '+r.url()));
 await p.addInitScript(`window.__cls=0;new PerformanceObserver(l=>{for(const e of l.getEntries())if(!e.hadRecentInput)window.__cls+=e.value}).observe({type:'layout-shift',buffered:true});`);
 const cdp=await ctx.newCDPSession(p);await p.coverage.startCSSCoverage();
 let reqs=0,bytes=0;await cdp.send('Network.enable');cdp.on('Network.loadingFinished',e=>{reqs++;bytes+=e.encodedDataLength});
 const t0=Date.now();await p.goto(BASE+'/'+pg,{waitUntil:'load'});
 await p.waitForSelector('html[data-hydrated=true]',{timeout:60000});
 const hyd=await p.evaluate(()=>performance.now());
 await p.waitForTimeout(1500);
 const cls=await p.evaluate(()=>window.__cls);
 const cov=await p.coverage.stopCSSCoverage();let tot=0,usd=0;for(const e of cov){tot+=e.text.length;for(const r of e.ranges)usd+=r.end-r.start}
 const unusedPct=(100*(1-usd/tot)).toFixed(1);
 await p.evaluate(axeSrc);const ax=await p.evaluate(()=>axe.run(document).then(r=>r.violations.map(v=>v.id+'('+v.nodes.length+')').concat(['passes='+r.passes.length,'axe '+axe.version])));
 console.log(JSON.stringify({page:pg||'home',hydrateMs:Math.round(hyd),cls:+cls.toFixed(4),consoleBad:bad.length,bad:bad.slice(0,2),unusedCssPct:unusedPct,axe:ax,requests:reqs,transferKB:Math.round(bytes/1024)}));
 await ctx.close()}
await b.close()})();
