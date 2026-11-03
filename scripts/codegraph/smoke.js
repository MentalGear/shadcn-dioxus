// Smoke test for the built page (optional, not a gate). Opens explorer.html from file:, aborts and reports every
// non-file request, fails on a console error, and renders component picker -> select -> each level.
// usage: node scripts/codegraph/smoke.js [path/to/explorer.html]   (playwright resolved from playwright/node_modules)
const path = require('path');
const { chromium } = require(path.resolve(__dirname, '../../playwright/node_modules/playwright'));
const { sandboxLaunchOptions } = require(path.resolve(__dirname, '../../playwright/sandbox-chromium.cjs'));
const page_path = path.resolve(process.argv[2] || path.resolve(__dirname, '../../target-codegraph/explorer.html'));
(async () => {
  const b = await chromium.launch(sandboxLaunchOptions());
  const ctx = await b.newContext({ viewport: { width: 1280, height: 900 } });
  const p = await ctx.newPage();
  const errs = [], nets = [];
  await ctx.route('**/*', r => { const u = r.request().url(); if (u.startsWith('file:')) return r.continue(); nets.push(u); return r.abort(); });
  p.on('pageerror', e => errs.push('pageerror: ' + e.message));
  p.on('console', m => { if (m.type() === 'error') errs.push('console: ' + m.text()); });
  await p.goto('file://' + page_path, { waitUntil: 'load' });
  await p.waitForFunction(() => window.__cy && window.__cy.nodes().length > 0, null, { timeout: 30000 });
  const stat = async name => { const s = await p.evaluate(() => ({ msg: document.getElementById('msg').textContent, nodes: __cy.nodes().length, edges: __cy.edges().length, sw: document.documentElement.scrollWidth, cw: innerWidth })); console.log(name, JSON.stringify(s)); return s; };
  const lvl = async v => { await p.evaluate(v => { const e = document.getElementById('lvl'); e.value = v; e.dispatchEvent(new Event('input')); }, v); await p.waitForTimeout(500); };
  const results = [await stat('initial (select, component level)')];
  await p.fill('#pick', 'button'); await p.dispatchEvent('#pick', 'change'); await p.waitForTimeout(500);
  results.push(await stat('picked button, component'));
  await lvl(0); results.push(await stat('crate'));
  await lvl(2); results.push(await stat('file'));
  await p.evaluate(() => { const e = document.getElementById('rad'); e.value = 2; e.dispatchEvent(new Event('input')); }); await p.waitForTimeout(800);
  results.push(await stat('file, radius 2'));
  const dir = process.env.CODEGRAPH_SHOTS; if (dir) await p.screenshot({ path: path.join(dir, 'file-r2.png') });
  console.log('network attempts', JSON.stringify(nets)); console.log('errors', JSON.stringify(errs));
  await b.close();
  const bad = errs.length || nets.length || results.some(r => r.nodes === 0 || r.sw > r.cw);
  process.exit(bad ? 1 : 0);
})();
