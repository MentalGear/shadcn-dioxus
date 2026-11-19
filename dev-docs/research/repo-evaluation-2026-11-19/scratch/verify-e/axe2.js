const {chromium}=require('playwright');const AxeBuilder=require('/home/user/shadcn-dioxus/playwright/node_modules/@axe-core/playwright').default;
const fs=require('fs');const axeSrc=fs.readFileSync('/home/user/shadcn-dioxus/playwright/node_modules/.pnpm/axe-core@4.10.3/node_modules/axe-core/axe.min.js','utf8');
(async()=>{const b=await chromium.launch();
for(const pg of ['component/sidebar/','component/dialog/']){const p=await (await b.newContext({viewport:{width:1280,height:800}})).newPage();
 await p.goto('https://mentalgear.github.io/shadcn-dioxus/'+pg);await p.waitForSelector('html[data-hydrated="true"]',{state:'attached'});await p.waitForTimeout(1500);
 const r=await new AxeBuilder({page:p}).analyze();console.log(pg,'AxeBuilder',r.violations.map(v=>v.id+'('+v.nodes.length+') '+v.nodes.map(n=>n.target.join('>')).slice(0,2).join(' ; ')));
 await p.evaluate(axeSrc);console.log(pg,'manual injection (main frame)',JSON.stringify(await p.evaluate(()=>axe.run(document).then(r=>r.violations.map(v=>v.id+'('+v.nodes.length+')')))),'iframes',await p.evaluate(()=>document.querySelectorAll('iframe').length));}
await b.close()})();
