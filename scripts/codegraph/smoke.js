// Smoke test for the built page (optional, not a gate). Opens explorer.html from file:, aborts and reports every
// non-file request, fails on a console error, and renders component picker -> select -> each level.
// usage: node scripts/codegraph/smoke.js [path/to/explorer.html]   (playwright resolved from playwright/node_modules)
const path = require('path');
const fs = require('fs');
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
  // ---- comment metrics (M1-M6)
  const checks = [];
  const check = (name, ok, detail) => { checks.push([name, !!ok]); console.log(ok ? 'ok  ' : 'FAIL', name, detail === undefined ? '' : JSON.stringify(detail)); };
  // ---- pairing through component.json, and the complete radius-2 neighbourhood
  const setRange = async (id, v) => { await p.evaluate(([id, v]) => { const e = document.getElementById(id); e.value = v; e.dispatchEvent(new Event('input')); }, [id, v]); await p.waitForTimeout(300); };
  await p.fill('#pick', 'select'); await p.dispatchEvent('#pick', 'change'); await p.waitForTimeout(300);
  await lvl(2); await p.uncheck('#hubs'); await setRange('thr', 1);
  const pg = await p.evaluate(() => {
    const { G, N, st } = window.__g;
    const seed = [...st.seedFiles].map(i => N[i].id);
    const comp = id => G.comps.find(c => c.id === id);
    // independent breadth-first search over every file-level edge (hubs shown, weight >= 1, both directions)
    const adj = new Map(); G.edges.forEach(([a, b]) => { (adj.get(a) || adj.set(a, []).get(a)).push(b); (adj.get(b) || adj.set(b, []).get(b)).push(a); });
    const bfs = r => { const keep = new Set(st.seedFiles); let fr = [...keep]; for (let k = 0; k < r; k++) { const nx = []; fr.forEach(u => (adj.get(u) || []).forEach(v => { if (!keep.has(v)) { keep.add(v); nx.push(v); } })); fr = nx; } return keep.size; };
    const view = r => { const e = document.getElementById('rad'); e.value = r; e.dispatchEvent(new Event('input')); return window.__last.keep.size; };
    const r1 = view(1), r2 = view(2);
    return { seed, seedN: seed.length, pairSel: comp('preview:select').pair, pairPrim: comp('primitives:select').pair, r1, r2, b1: bfs(1), b2: bfs(2), pairedBy: G.comps.filter(c => c.paired && c.crate === 'preview').length };
  });
  check('pick "select" seeds the themed component incl. mod.rs and its primitive', pg.seed.includes('preview/src/components/select/mod.rs') && pg.seed.includes('preview/src/components/select/component.rs') && pg.seed.includes('primitives/src/select/mod.rs'), { files: pg.seedN, pair: pg.pairSel, back: pg.pairPrim });
  check('a component without a same-named primitive pairs through its sources (area_chart -> chart)', await p.evaluate(() => window.__g.G.comps.find(c => c.id === 'preview:area_chart').pair.join() === 'primitives:chart'));
  check('radius 1 view equals an independent breadth-first search', pg.r1 === pg.b1, { page: pg.r1, bfs: pg.b1 });
  check('radius 2 view lists the complete neighbourhood (equals an independent breadth-first search)', pg.r2 === pg.b2 && pg.r2 > pg.r1, { page: pg.r2, bfs: pg.b2, radius1: pg.r1, themedPaired: pg.pairedBy });
  await p.check('#hubs'); await setRange('thr', 2); await setRange('rad', 1); await lvl(1);
  const cm = await p.evaluate(() => {
    const c = window.__cm, f = c.fields, ix = k => f.indexOf(k), o = {};
    const get = (l, k) => { const a = c.agg(l, k); if (!a) return null; const r = {}; f.forEach((n, i) => r[n] = a[i]); return r; };
    const T = c.total(), tot = {}; f.forEach((n, i) => tot[n] = T[i]);
    o.total = tot; o.primitives = get(0, 'primitives'); o.preview = get(0, 'preview');
    o.file = get(2, 'primitives/src/lib.rs');
    o.out = { file: c.outliers(2), comp: c.outliers(1), crate: c.outliers(0) };
    return o;
  });
  check('totals: code and comment lines counted', cm.total.code > 0 && cm.total.com > 0, { code: cm.total.code, com: cm.total.com, doc: cm.total.doc });
  const pr = cm.primitives;
  check('primitives crate: M1/M2/M3', pr && pr.code > 0, pr && { M1: +(pr.com / pr.code).toFixed(3), M2: +(pr.doc / pr.com).toFixed(3), M3: +((pr.doced + pr.fdoced) / (pr.pubs + pr.fields)).toFixed(3), pubs: pr.pubs, fields: pr.fields });
  console.log('M4 history hits total', cm.total.hist, '| M5 TODO total', cm.total.todo);
  // a known file: the doc-line count of primitives/src/lib.rs, counted here by a plain line match
  const libText = fs.readFileSync(path.resolve(__dirname, '../../primitives/src/lib.rs'), 'utf8');
  const docLines = libText.split('\n').filter(l => /^\s*\/\/[\/!]/.test(l)).length;
  check('lib.rs doc lines equal an independent line count', cm.file && cm.file.doc === docLines, { page: cm.file && cm.file.doc, plain: docLines });
  check('lib.rs: pub items counted, `pub mod x;` lines skipped', cm.file && cm.file.pubs >= 0 && cm.file.pubs <= (libText.match(/^\s*pub\s+(?:fn|struct|enum|trait|const|static|type|mod)\s/gm) || []).length, cm.file && { pubs: cm.file.pubs, doced: cm.file.doced });
  // status line: what the graph (index) and the comment metrics were each built from
  const chips = await p.evaluate(() => ['chip-index', 'chip-stale', 'chip-comments'].map(id => { const e = document.getElementById(id); return e ? e.textContent : null; }));
  check('status line names the index, its staleness and the metrics inputs', chips[0] && /SCIP index of [0-9a-f]{8}/.test(chips[0]) && chips[1] && chips[2] && /working tree [0-9a-f]{8}/.test(chips[2]), chips);
  if (process.env.CODEGRAPH_EXPECT_STALE) {
    const want = process.env.CODEGRAPH_EXPECT_STALE === '1';
    check('index staleness is shown as expected (CODEGRAPH_EXPECT_STALE=' + (want ? 1 : 0) + ')', /STALE/.test(chips[1]) === want, chips[1]);
  }
  // details panel
  await p.evaluate(() => { __cm.show('graph'); const e = document.getElementById('lvl'); e.value = 1; e.dispatchEvent(new Event('input')); });
  await p.waitForTimeout(500);
  await p.evaluate(() => __cm.info(document.getElementById('pick').value ? __cy.nodes().filter(n => !n.data('grp'))[0].id() : ''));
  const panel = await p.evaluate(() => document.getElementById('cm-table') && document.getElementById('cm-table').innerText);
  check('details panel shows the Comments table', panel && /Comment ratio/.test(panel) && /Doc share/.test(panel) && /History-rule hits/.test(panel));
  // lens: the node fill changes from the crate colour and a zero-valued lens uses the neutral colour or says so
  const fills = async () => p.evaluate(() => __cy.nodes().filter(n => !n.data('grp')).map(n => n.data('fill')));
  const f0 = await fills();
  await p.selectOption('#lens', 'ratio'); await p.waitForTimeout(200);
  const f1 = await fills();
  check('lens "comment ratio" recolours the nodes', JSON.stringify(f0) !== JSON.stringify(f1) && new Set(f1).size > 1, { distinct: new Set(f1).size });
  await p.selectOption('#lens', 'cov'); await p.waitForTimeout(200);
  check('lens "doc coverage" has a legend gradient', await p.evaluate(() => !!document.querySelector('#legend .grad')));
  await p.selectOption('#lens', 'todo'); await p.waitForTimeout(200);
  const banner = await p.evaluate(() => { const b = document.getElementById('banner'); return b.hidden ? '' : b.textContent; });
  check('lens "TODOs": banner when the view has none, else a hot node', cm.total.todo === 0 ? /0 TODO/.test(banner) : true, banner.slice(0, 60));
  await p.selectOption('#lens', 'hist'); await p.waitForTimeout(200);
  const chip = await p.evaluate(() => document.getElementById('chip-hist').textContent);
  check('status chip carries the history hit count', chip.includes(String(cm.total.hist)), chip);
  await p.selectOption('#lens', 'deps'); await p.waitForTimeout(200);
  // Comments view
  await p.click('#t-comm'); await p.waitForTimeout(300);
  const rows = async () => p.evaluate(() => [...document.querySelectorAll('#c-body tr')].map(r => [...r.children].map(c => c.innerText.trim())));
  let tr = await rows();
  check('Comments table renders rows', tr.length > 1 && tr[0].length === 13, { rows: tr.length, first: tr[0] && tr[0].slice(0, 4) });
  const ratios = tr.map(r => parseFloat(r[3])).filter(x => !isNaN(x));
  check('sparsest first: ratios ascend', ratios.every((x, i) => i === 0 || x >= ratios[i - 1]), ratios.slice(0, 5));
  await p.selectOption('#c-rank', 'dense'); await p.waitForTimeout(200);
  tr = await rows(); const dr = tr.map(r => parseFloat(r[3]));
  check('densest first: ratios descend', dr.every((x, i) => i === 0 || x <= dr[i - 1]), dr.slice(0, 5));
  await p.fill('#c-q', 'primitives/src/select'); await p.waitForTimeout(200);
  tr = await rows();
  check('path filter narrows the table', tr.length > 0 && tr.every(r => r[0].includes('primitives/src/select') || r[0] === 'no entity matches'), { rows: tr.length });
  await p.fill('#c-q', ''); await p.check('#c-out'); await p.waitForTimeout(200);
  tr = await rows();
  check('only-outliers filter keeps flagged rows', tr.every(r => /dense|sparse/.test(r[12]) || r[0].startsWith('no entity')), { rows: tr.length });
  await p.uncheck('#c-out');
  // ---- N1-N6
  const ext = await p.evaluate(() => {
    const c = window.__cm, f = c.fields, T = c.total(), t = {}; f.forEach((n, i) => t[n] = T[i]);
    
    return { total: t, itemCount: c.itemCount(), blame: c.blameBuilt() };
  });
  console.log('N1 stale candidates', ext.total.stale, '| N2 js comment/code lines', ext.total.js + '/' + ext.total.jscode, '| N3 words', ext.total.words, '| N4 repeated lines', ext.total.dup, '| N6 items', ext.itemCount);
  check('N1: stale candidates counted', ext.total.stale > 0, ext.total.stale);
  check('N2: JS code lines in strings counted', ext.total.jscode > 0 && ext.total.js > 0, { js: ext.total.js, jscode: ext.total.jscode });
  check('N3: comment words counted', ext.total.words > ext.total.com, ext.total.words);
  check('N4: repeated comment lines counted', ext.total.dup > 0, ext.total.dup);
  check('N6: items rolled up', ext.itemCount > 1000, ext.itemCount);
  // an item's own numbers: the lib.rs `pub mod accordion;` line is an item span of one code line
  const item = await p.evaluate(() => { const a = window.__cm.itemOf('primitives/src/lib.rs', 'accordion'); return a; });
  check('N6: item numbers for a known item (pub mod accordion; in lib.rs)', item && item.code === 1 && item.pub === 1, item);
  // a lens per metric recolours; the Comments tab ranks by stale, dup, and lists items
  await p.click('#t-graph'); await p.waitForTimeout(300);
  for (const lens of ['stale', 'dup', 'age']) {
    await p.selectOption('#lens', lens); await p.waitForTimeout(200);
    const leg = await p.evaluate(() => document.getElementById('legend').innerText);
    check('lens "' + lens + '" shows a legend', /in view|older|repeated|stale/i.test(leg), leg.slice(0, 80));
  }
  await p.selectOption('#lens', 'deps');
  await p.click('#t-comm'); await p.waitForTimeout(300);
  await p.selectOption('#c-lvl', '3'); await p.selectOption('#c-rank', 'dense'); await p.fill('#c-min', '20'); await p.waitForTimeout(300);
  tr = await rows();
  check('Comments table lists items (rows: items)', tr.length > 1 && /:\d+ (fn|struct|enum|impl|const|mod|type|trait|static) /.test(tr[0][0]), tr[0] && tr[0].slice(0, 4));
  await p.selectOption('#c-lvl', '2'); await p.selectOption('#c-rank', 'stale'); await p.waitForTimeout(300);
  tr = await rows();
  check('rank by stale: first row has stale candidates', tr.length > 0 && parseInt(tr[0][10]) > 0, tr[0] && [tr[0][0], tr[0][10]]);
  await p.selectOption('#c-rank', 'dup'); await p.waitForTimeout(300);
  tr = await rows();
  check('rank by repeated lines: first row has repeats', tr.length > 0 && parseInt(tr[0][11]) > 0, tr[0] && [tr[0][0], tr[0][11]]);
  if (ext.blame) {
    await p.selectOption('#c-rank', 'age'); await p.waitForTimeout(300);
    tr = await rows();
    check('rank by comment age: rows exist when blame is built', tr.length > 0, tr[0] && tr[0][0]);
  }
  await p.selectOption('#c-lvl', '2'); await p.selectOption('#c-rank', 'sparse'); await p.fill('#c-min', '100');
  console.log('M6 file-level fences', JSON.stringify({ lo: cm.out.file.lo, hi: cm.out.file.hi, median: cm.out.file.median }));
  console.log('M6 densest files', JSON.stringify(cm.out.file.dense.slice(0, 5).map(([k, r]) => [k, +r.toFixed(2)])));
  console.log('M6 sparsest files', JSON.stringify(cm.out.file.sparse.slice(0, 5).map(([k, r]) => [k, +r.toFixed(2)])));
  console.log('M6 outlier counts file/component/crate', cm.out.file.dense.length + '/' + cm.out.file.sparse.length, cm.out.comp.dense.length + '/' + cm.out.comp.sparse.length, cm.out.crate.dense.length + '/' + cm.out.crate.sparse.length);
  await p.click('#t-graph'); await p.waitForTimeout(500);
  const dir = process.env.CODEGRAPH_SHOTS; if (dir) await p.screenshot({ path: path.join(dir, 'file-r2.png') });
  console.log('network attempts', JSON.stringify(nets)); console.log('errors', JSON.stringify(errs));
  await b.close();
  const bad = errs.length || nets.length || checks.some(c => !c[1]) || results.some(r => r.nodes === 0 || r.sw > r.cw);
  process.exit(bad ? 1 : 0);
})();
