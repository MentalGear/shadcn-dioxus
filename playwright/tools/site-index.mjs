// class/id -> pages index for the spec selector (backlog row 323).
//
// Built from the SSG HTML of the build under test: every `class="..."` and `id="..."` token of every prerendered
// page. The build is found, in order, from an explicit directory (--ssg-dir / SELECT_SSG_DIR, the lane's
// `$LANE_TGT/pub-<lane>`), or by crawling the served site at PLAYWRIGHT_BASE_URL (the build `test:changed` is about to
// run against). With neither, there is no index and the selector keeps the broad set (the pre-row-323 behaviour).
//
// Why HTML and not the rsx sources: the pages that contain a class are exactly what the SSG rendered (the home page
// shows the first demo of every component; the docs shell wraps every component page; a `format!`-built class name is
// just a string). Re-deriving that from Rust text would need a call graph of the whole preview crate and would still be
// a guess. The price is that the SSG renders only what is in the DOM at first paint: a class that exists only inside a
// closed popup, or that script adds later, is NOT in the HTML. Those classes fall back to their source siblings (the
// classes of the same component directory or site file that ARE in the HTML), and to the broad set when even that is
// empty (`pagesOfClass`). Each (class, route) pair costs nothing to re-derive, so there is no cache to go stale: the
// index is rebuilt on every run, from the build being tested, in about a second.
import fs from "node:fs";
import path from "node:path";

export function routeName(dir, root) {
  const rel = path.relative(root, dir).split(path.sep).join("/");
  return "/" + rel + (rel ? "/" : "");
}

function tokens(html) {
  const classes = new Set(), ids = new Set();
  for (const m of html.matchAll(/\bclass=(?:"([^"]*)"|'([^']*)')/g)) for (const c of (m[1] ?? m[2]).split(/\s+/)) if (c) classes.add(c);
  for (const m of html.matchAll(/\bid=(?:"([^"]*)"|'([^']*)')/g)) { const v = m[1] ?? m[2]; if (v) ids.add(v); }
  return { classes, ids };
}

function finish(pages, origin) {
  const routes = [...pages.keys()].sort();
  const classes = new Map(), ids = new Map();
  routes.forEach((r, i) => {
    const t = pages.get(r);
    for (const c of t.classes) { if (!classes.has(c)) classes.set(c, new Set()); classes.get(c).add(r); }
    for (const c of t.ids) { if (!ids.has(c)) ids.set(c, new Set()); ids.get(c).add(r); }
  });
  return { routes, classes, ids, origin };
}

export function fromDir(dir) {
  const pages = new Map();
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) { if (e.name !== "assets") walk(p); }
      else if (e.name === "index.html") pages.set(routeName(d, dir), tokens(fs.readFileSync(p, "utf8")));
    }
  };
  walk(dir);
  if (pages.size === 0) throw new Error(`no index.html under ${dir}`);
  return finish(pages, `dir ${dir}`);
}

export async function fromUrl(base, { limit = 400, concurrency = 8 } = {}) {
  const origin = new URL(base);
  const prefix = origin.pathname.replace(/\/$/, "");
  const pages = new Map(), seeds = ["/", "/component/", "/component/block/"], seen = new Set(seeds), queue = [...seeds]; // seeds: pages nothing links to
  const worker = async () => {
    while (queue.length) {
      const route = queue.shift();
      let html;
      try {
        const r = await fetch(origin.origin + prefix + route, { headers: { accept: "text/html" }, signal: AbortSignal.timeout(20000) });
        if (!r.ok || !(r.headers.get("content-type") ?? "").includes("html")) continue;
        html = await r.text();
      } catch { continue; }
      pages.set(route, tokens(html));
      for (const m of html.matchAll(/\b(?:href|src)="([^"#?]*)/g)) {
        let h = m[1];
        if (prefix && h.startsWith(prefix + "/")) h = h.slice(prefix.length);
        if (!h.startsWith("/") || h.startsWith("//") || /\.[a-z0-9]{2,5}$/i.test(h) || h.startsWith("/assets/")) continue;
        if (!h.endsWith("/")) h += "/"; // the same spelling routeName() gives a prerendered directory
        if (!seen.has(h) && seen.size < limit) { seen.add(h); queue.push(h); }
      }
    }
  };
  await Promise.all(Array.from({ length: concurrency }, worker));
  if (pages.size === 0) throw new Error(`could not fetch any page from ${base}`);
  return finish(pages, `url ${base}`);
}

// Find the index for this run, or null (reason in `why`).
export async function loadIndex({ ssgDir = process.env.SELECT_SSG_DIR, baseUrl = process.env.PLAYWRIGHT_BASE_URL } = {}) {
  try {
    if (ssgDir) return { index: fromDir(path.resolve(ssgDir)) };
    if (baseUrl) return { index: await fromUrl(baseUrl) };
    return { index: null, why: "no SSG build to read (pass --ssg-dir, or set SELECT_SSG_DIR or PLAYWRIGHT_BASE_URL)" };
  } catch (e) {
    return { index: null, why: `class index unavailable: ${e.message}` };
  }
}
