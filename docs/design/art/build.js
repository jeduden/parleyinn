'use strict';
// Builds the GitHub page (docs/index.html), the design system's page (docs/design/index.html), the design tokens and
// every image under docs/design/assets from the picture's source, with the standard library of Node.js alone.
//   node docs/design/art/build.js           writes them
//   node docs/design/art/build.js --check   writes nothing, and fails naming each file that differs from the source
// The page copy that repeats the README is read from README.md, and the invariants from docs/srs/invariants.md, so the
// page says what they say.
const fs = require('fs'), path = require('path'), zlib = require('zlib');
const { Fb, png, PAL, NAMES } = require('./pix.js');
const { scene, W, H, FRAMES, wordmark, keyGlyph } = require('./scene.js');
const { halo } = require('./kit.js');
const INTERIOR = require('./interior.js'), YARD = require('./yard.js');
const M = require('./marks.js');
const { CONCEPTS, INN } = require('./concepts.js');

const DOCS = path.resolve(__dirname, '..', '..');
const ROOT = path.resolve(DOCS, '..');
const REPO = 'https://github.com/jeduden/parleyinn';
const SITE = 'https://jeduden.github.io/parleyinn/';
const TICK = 140, TS = 8, AC = 32;

// ---------- reading the repository ----------
const read = rel => fs.readFileSync(path.join(ROOT, rel), 'utf8');
const flat = s => s.replace(/\s+/g, ' ').trim();
const esc = s => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
// Inline Markdown as the README writes it: bold, code and links, a relative link pointing into the repository.
const inline = s => esc(flat(s))
  .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
  .replace(/`([^`]+)`/g, '<code>$1</code>')
  .replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (m, t, u) => `<a href="${/^https?:/.test(u) ? u : REPO + '/blob/main/' + u}">${t}</a>`);
function section(md, heading) {
  const lines = md.split('\n'), a = lines.indexOf(heading);
  if (a < 0) throw new Error(`README.md has no "${heading}" section`);
  const rest = lines.slice(a + 1), b = rest.findIndex(l => /^#{1,2} /.test(l));
  return (b < 0 ? rest : rest.slice(0, b)).join('\n');
}
const bullets = md => md.split(/\n(?=- )/).filter(b => b.startsWith('- ')).map(b => b.split(/\n\s*\n/)[0].slice(2));
const paragraphs = md => md.split(/\n\s*\n/).map(p => p.trim()).filter(p => p && !/^[-|<!#`]/.test(p));

const README = read('README.md');
const LEDE = paragraphs(section(README, '# ParleyInn'))[0];
const VISION = bullets(section(README, '## Vision')).map(b => { const m = /^\*\*(.+?)\*\*\s+([\s\S]+)$/.exec(b); if (!m) throw new Error('a Vision bullet has no bold lead: ' + b); return [m[1], m[2]]; });
const UNDERNEATH = paragraphs(section(README, '## Vision')).find(p => p.startsWith('Underneath'));
const HOLDS = bullets(section(README, '## Status').split('This repository holds:')[1] || '');
if (!LEDE || VISION.length !== 3 || !UNDERNEATH || !HOLDS.length) throw new Error('README.md no longer has the lede, three Vision bullets, the Underneath paragraph or the Status list the page repeats');
const INVARIANTS = read('docs/srs/invariants.md').split(/\n## (I\d+)\n/).slice(1).reduce((out, part, i, all) => {
  if (i % 2) return out;
  const m = /\*\*([\s\S]+?)\*\*/.exec(all[i + 1]); if (!m) throw new Error(part + ' has no bold headline');
  return out.concat([[part, flat(m[1])]]);
}, []);
if (INVARIANTS.length !== 10) throw new Error('docs/srs/invariants.md no longer holds ten invariants');

// ---------- the loop: frame 0 whole, and the 8 x 8 tiles of each later frame that differ from it ----------
const frames = []; for (let t = 0; t < FRAMES; t++) frames.push(scene(t));
const f0 = frames[0], TC = W / TS, TR = Math.ceil(H / TS);
const tile = (g, tx, ty) => { let s = ''; for (let j = 0; j < TS; j++) for (let i = 0; i < TS; i++) s += String.fromCharCode(g.get(tx * TS + i, ty * TS + j) || 1); return s; };
const base = []; for (let ty = 0; ty < TR; ty++) for (let tx = 0; tx < TC; tx++) base.push(tile(f0, tx, ty));
const tiles = new Map(), lists = [];
for (let f = 1; f < FRAMES; f++) {
  const L = [];
  for (let ty = 0; ty < TR; ty++) for (let tx = 0; tx < TC; tx++) {
    const k = tile(frames[f], tx, ty); if (k === base[ty * TC + tx]) continue;
    if (!tiles.has(k)) tiles.set(k, tiles.size); L.push(tiles.get(k), ty * TC + tx);
  }
  lists.push(L);
}
const atlas = new Fb(AC * TS, Math.max(1, Math.ceil(tiles.size / AC)) * TS, 'ink');
for (const [k, n] of tiles) { const ox = (n % AC) * TS, oy = Math.floor(n / AC) * TS; for (let j = 0; j < TS; j++) for (let i = 0; i < TS; i++) atlas.d[(oy + j) * atlas.w + ox + i] = k.charCodeAt(j * TS + i); }
// Rebuild every frame from frame 0 and its tiles, the way the page does, and make sure it is the frame that was drawn.
for (let f = 1; f < FRAMES; f++) {
  const r = f0.copy(), L = lists[f - 1];
  for (let k = 0; k < L.length; k += 2) {
    const a = L[k], p = L[k + 1], ox = (a % AC) * TS, oy = Math.floor(a / AC) * TS, dx = (p % TC) * TS, dy = Math.floor(p / TC) * TS;
    for (let j = 0; j < TS; j++) for (let i = 0; i < TS; i++) if (dx + i < W && dy + j < H) r.d[(dy + j) * W + dx + i] = atlas.d[(oy + j) * atlas.w + ox + i];
  }
  for (let i = 0; i < r.d.length; i++) if (r.d[i] !== frames[f].d[i]) throw new Error('frame ' + f + ' does not rebuild from its tiles');
}

// ---------- the kit, cut from the picture ----------
const crop = (x, y, w, h) => { const o = new Fb(w, h); for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) o.d[j * w + i] = f0.get(x + i, y + j); return o; };
function trim(g) {
  let x0 = g.w, y0 = g.h, x1 = -1, y1 = -1;
  for (let j = 0; j < g.h; j++) for (let i = 0; i < g.w; i++) if (g.d[j * g.w + i]) { if (i < x0) x0 = i; if (i > x1) x1 = i; if (j < y0) y0 = j; if (j > y1) y1 = j; }
  const o = new Fb(x1 - x0 + 1, y1 - y0 + 1); for (let j = 0; j < o.h; j++) for (let i = 0; i < o.w; i++) o.d[j * o.w + i] = g.d[(y0 + j) * g.w + x0 + i]; return o;
}
const sprite = draw => { const g = new Fb(W, H); draw(g); return trim(g); };
const SIGN = crop(160, 64, 142, 40), BANNER = crop(60, 8, 360, 104), SOCIAL = crop(40, 40, 400, 210);
const STACKED = (() => { const g = new Fb(100, 62, 'n1'); g.B(0, 0, 100, 62, 'ink'); g.B(1, 1, 98, 60, 'w2'); wordmark(g, 12, 8, 'PARLEY', 'cy', 'cyw', -1); wordmark(g, 34, 27, 'INN', 'cy', 'cyw', -1); keyGlyph(g, 12, 50, 76, 'pk', 'pkw'); halo(g, 2, 2, 96, 42, ['cy', 'cyw'], 'cy', [0.34, 0.2, 0.1, 0.05], 8); halo(g, 2, 45, 96, 13, ['pk', 'pkw'], 'pk', [0.3, 0.16, 0.07], 8); return g; })();
const FLAT = (() => { const g = new Fb(126, 33); wordmark(g, 4, 3, 'PARLEY INN', 'p2', 'p2', -1); keyGlyph(g, 7, 24, 112, 'p2', 'p2'); return g; })();
const ICON = (() => { const g = new Fb(16, 16); g.RR(0, 0, 16, 16, 'n1', 'cy'); g.H(4, 4, 4, 'pk'); g.H(4, 9, 4, 'pk'); g.V(3, 5, 4, 'pk'); g.V(8, 5, 4, 'pk'); g.H(9, 6, 4, 'pkw'); g.H(9, 7, 4, 'pk'); g.px(13, 6, 'pk'); g.px(13, 7, 'pk'); g.R(10, 8, 2, 2, 'pk'); g.V(13, 8, 3, 'pk'); return g; })();
const { figure, PON, HAT, BRASS, DARK, PROP, PROPL } = INTERIOR;
const CAST = [
  ['hand', 'A hand', 'agent', sprite(g => figure(g, 60, 80, { pon: PON.red, hat: HAT.brown, band: 'am', cell: true })), 'A steel robot in a red poncho and a brown hat, with a power cell on its back'],
  ['proprietor', 'The proprietor', 'principal', sprite(g => g.S(40, 40, PROP, PROPL)), 'A person in a black hat, a white shirt and a teal waistcoat'],
  ['compositor', 'The compositor', 'restore builder', sprite(g => figure(g, 60, 80, { metal: BRASS, arm: 'crank', shade: true, vest: 't2' })), 'A brass robot in an eyeshade and waistcoat, one arm raised to a crank'],
  ['strongbox', 'A strongbox', 'sealed range', sprite(g => YARD.strongbox(g, 40, 40, 13, 9)), 'A steel strongbox with a red seal'],
  ['horse', 'The outfit’s horse', 'harness', sprite(g => YARD.horse(g, 40, 80, DARK, { blanket: true })), 'A dark iron robot horse with a striped saddle blanket'],
  ['stage', 'The stage', 'peer', sprite(g => YARD.coach(g, 0)), 'A red hover coach with a rocket at the back, strongboxes on the roof and a key on its door'],
];
const LETTERING = [
  ['hooks', crop(90, 252, 78, 28), 'The hook board', 'A board lettered SessionStart, UserPromptSubmit, PermissionRequest'],
  ['needs-you', crop(319, 168, 49, 23), 'Needs you', 'A sign lettered NEEDS YOU over four numbered tickets'],
  ['tombstone', crop(397, 285, 49, 32), 'A tombstone', 'A headboard lettered PURGED, A2, 4812-4830'],
];

// ---------- colours: every one the picture uses, in ramps, and the page's design tokens, which are picture colours too ----------
const used = new Set(); for (const g of frames) for (const c of g.d) used.add(c);
const RAMPS = [['ink', 'n1', 'n2', 'n3', 'n4', 'n5'], ['t0', 't1', 't2', 't3', 't4', 't5', 'cy', 'cyw'], ['v1', 'v2', 'd0', 'd1', 'd2', 'd3', 'd4', 'd5', 'd6', 'am', 'd8', 'wh'], ['w0', 'w1', 'w2', 'w3', 'w4', 'w5', 'p0', 'p1', 'p2'], ['m0', 'm1', 'm2', 'm3'], ['r0', 'r1', 'r2', 'pk0', 'pk', 'pkw'], ['g0', 'g1', 'g2']];
const RAMP_NAMES = ['Ink', 'Night and neon cyan', 'Dusk to lamplight', 'Timber and bone', 'Steel', 'Red and neon pink', 'Sage'];
{ const u = [...used].map(i => NAMES[i - 1]).sort().join(), s = RAMPS.flat().sort().join(); if (u !== s) throw new Error('the ramps differ from the colours the picture uses: used ' + u + ', ramps ' + s); }
const NCOL = RAMPS.flat().length;
const NAMED = [['Night', 't0'], ['Bone', 'p2'], ['Dusk coral', 'd4'], ['Mesa plum', 'd1'], ['Teal', 't4'], ['Amber', 'am'], ['Neon cyan', 'cy'], ['Neon pink', 'pk']];
const PAGE = { bg: 't0', bg2: 't1', ink: 'ink', line: 't2', line2: 't3', fg: 'p2', fg2: 'm3', mute: 'm2', pink: 'pk', cyan: 'cy', amber: 'am' };
const USES = [
  ['cy', 'Neon cyan is the house’s own light: the name on the sign, ROOMS over the balcony and the beams of the pen.'],
  ['pk', 'Neon pink is the working light: the key under the name, power cells, the ledgers’ bookmarks and boundary plates.'],
  ['am', 'Amber is lamplight and the proprietor’s business: the Needs you sign and the rail its tickets hang on.'],
  ['r2', 'Red is for marks that must not be missed: UNTRUSTED, QUARANTINE, the seals and the band on every envelope.'],
];
const FONTS = [
  ['display', 'Jersey 15', 'jersey-15-400.woff2', "'Jersey 15','Chakra Petch','Trebuchet MS',sans-serif", 'Headings, at 27 and 54 pixels: one or two of its 15-pixel cells.'],
  ['label', 'Tiny5', 'tiny5-400.woff2', "'Tiny5','Chakra Petch','Trebuchet MS',sans-serif", 'Names, numbers and captions, at 16 pixels.'],
  ['text', 'Chakra Petch', 'chakra-petch-400.woff2', "'Chakra Petch','Trebuchet MS',Verdana,sans-serif", 'Everything read at length, at 15 to 17 pixels.'],
];
const PALETTE = (() => { const w = Math.max(...RAMPS.map(r => r.length)), g = new Fb(w * 8, RAMPS.length * 8); RAMPS.forEach((r, j) => r.forEach((c, i) => g.R(i * 8, j * 8, 8, 8, c))); return g; })();

// ---------- the files ----------
const files = new Map();
const put = (rel, data) => { if (files.has(rel)) throw new Error(rel + ' is written twice'); files.set(rel, Buffer.isBuffer(data) ? data : Buffer.from(data)); };
const A = 'design/assets/';
const IMG = {};
const asset = (name, g, o) => { put(A + name, png(g, o)); return (IMG[name] = { name, w: g.w * ((o && o.scale) || 1), h: g.h * ((o && o.scale) || 1) }); };
asset('inn.png', f0);
asset('inn-loop.png', atlas);
asset('sign.png', SIGN, { transparent: true });
asset('banner.png', BANNER);
asset('banner-2x.png', BANNER, { scale: 2 });
asset('social.png', SOCIAL, { scale: 3 });
asset('mark-stacked.png', STACKED);
asset('mark-flat.png', FLAT, { transparent: true });
asset('icon.png', ICON, { transparent: true });
asset('icon-32.png', ICON, { transparent: true, scale: 2 });
asset('palette.png', PALETTE);
for (const [id, , , g] of CAST) asset('cast-' + id + '.png', g, { transparent: true });
for (const [id, g] of LETTERING) asset('lettering-' + id + '.png', g);

const fontFace = ([, fam, file]) => `@font-face{font-family:'${fam}';font-style:normal;font-weight:400;font-display:swap;src:url(fonts/${file}) format('woff2')}`;
put('design/tokens.css', `/* ParleyInn's design tokens, generated by docs/design/art/build.js from the picture's palette: edit the source, not this file. */
${FONTS.map(fontFace).join('\n')}
:root{color-scheme:dark;
${Object.entries(PAGE).map(([k, c]) => `--${k}:${PAL[c]};`).join('')}
${FONTS.map(([k, , , stack]) => `--${k}:${stack};`).join('')}
${RAMPS.flat().map(c => `--px-${c}:${PAL[c]};`).join('')}}
${RAMPS.flat().map(c => `.c-${c}{background-color:var(--px-${c})}`).join('\n')}
`);
put('design/tokens.json', JSON.stringify({
  name: 'ParleyInn design tokens',
  generated_by: 'docs/design/art/build.js',
  picture: { width: W, height: H, colours: NCOL, frames: FRAMES, frame_ms: TICK },
  page: Object.fromEntries(Object.entries(PAGE).map(([k, c]) => [k, { value: PAL[c], colour: c }])),
  named: Object.fromEntries(NAMED.map(([n, c]) => [n, { value: PAL[c], colour: c }])),
  palette: Object.fromEntries(RAMPS.flat().map(c => [c, PAL[c]])),
  ramps: Object.fromEntries(RAMPS.map((r, i) => [RAMP_NAMES[i], r])),
  meanings: Object.fromEntries(USES.map(([c, t]) => [c, t])),
  type: Object.fromEntries(FONTS.map(([k, fam, file, stack, use]) => [k, { family: fam, file: 'docs/design/fonts/' + file, stack, use }])),
  shape: { border_px: 2, radius_px: 0, shadow: '2px 2px 0 ink', image_scale: 'whole numbers, no smoothing' },
}, null, 2) + '\n');

// ---------- the pages ----------
const CSP = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; base-uri 'none'; form-action 'none'";
const img = (name, zoom, alt, cls) => { const i = IMG[name]; if (!i) throw new Error('no image ' + name); return `<img class="px${cls ? ' ' + cls : ''}${zoom ? ' z' + zoom : ''}" src="${A}${name}" width="${i.w}" height="${i.h}" alt="${esc(alt)}">`; };
const svg = rects => `<svg viewBox="0 0 8 8" width="16" height="16" aria-hidden="true" shape-rendering="crispEdges" fill="currentColor">${rects.map(r => `<rect x="${r[0]}" y="${r[1]}" width="${r[2]}" height="${r[3]}"/>`).join('')}</svg>`;
const I_PREV = svg([[1, 3, 1, 2], [2, 2, 1, 4], [3, 1, 1, 6], [4, 3, 4, 2]]), I_NEXT = svg([[6, 3, 1, 2], [5, 2, 1, 4], [4, 1, 1, 6], [0, 3, 4, 2]]);
const I_PAUSE = svg([[1, 1, 2, 6], [5, 1, 2, 6]]), I_PLAY = svg([[2, 0, 1, 8], [3, 1, 1, 6], [4, 2, 1, 4], [5, 3, 1, 2]]);
const DESCRIPTION = flat(LEDE);
const SIGN_ALT = 'ParleyInn in cyan neon capitals over one long pink key';
const ALT = 'Pixel art of a frontier saloon at dusk under a ringed planet, with its front wall removed. Robot hands sit at a card table in the saloon, and upstairs are the ledger shelves, a print shop, a wire office and an office. Around it stand a forge, a hitching rail, a wagon, a stage stop, a gate and Boot Hill. The key lists its 28 numbered places.';

function page({ base, title, description, canonical, body, script }) {
  // base is the path from the page to docs/: '' for docs/index.html, '../' for docs/design/index.html.
  const rel = s => s.startsWith(A) ? base + s : s;
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta http-equiv="Content-Security-Policy" content="${CSP}">
<meta name="referrer" content="no-referrer">
<title>${esc(title)}</title>
<meta name="description" content="${esc(description)}">
<meta name="color-scheme" content="dark">
<meta name="theme-color" content="${PAL.t0}">
<link rel="canonical" href="${canonical}">
<link rel="icon" type="image/png" sizes="16x16" href="${base}${A}icon.png">
<link rel="icon" type="image/png" sizes="32x32" href="${base}${A}icon-32.png">
<meta property="og:type" content="website">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(description)}">
<meta property="og:url" content="${canonical}">
<meta property="og:image" content="${SITE}${A}social.png">
<meta property="og:image:width" content="${IMG['social.png'].w}">
<meta property="og:image:height" content="${IMG['social.png'].h}">
<meta property="og:image:alt" content="The front of ParleyInn at dusk: a neon sign over a saloon with its front wall removed">
<meta name="twitter:card" content="summary_large_image">
${FONTS.map(f => `<link rel="preload" href="${base}design/fonts/${f[2]}" as="font" type="font/woff2" crossorigin>`).join('\n')}
<link rel="stylesheet" href="${base}design/tokens.css">
<link rel="stylesheet" href="${base}design/page.css">
${script ? `<script src="${base}design/inn.js" defer></script>\n` : ''}</head>
<body>
<a class="skip" href="#main">Skip to the content</a>
<header class="top"><div class="wrap top-in">
<a class="brand" href="${base || './'}">${img('icon.png', 2, '').replace(`src="${A}`, `src="${base}${A}`)}<span>ParleyInn</span></a>
<nav aria-label="Site"><ul class="nav">
<li><a href="${base}#inn">The inn</a></li><li><a href="${base}#contract">The contract</a></li><li><a href="${base}design/"${base ? ' aria-current="page"' : ''}>Design system</a></li><li><a href="${REPO}">GitHub</a></li>
</ul></nav>
</div></header>
<main id="main" class="wrap">
${body.replace(new RegExp(`src="${A}`, 'g'), `src="${base}${A}`)}
</main>
<footer class="foot"><div class="wrap foot-in">
<p>ParleyInn is MIT licensed and specified in the open: the <a href="${REPO}/blob/main/docs/srs/index.md">requirements</a>, the <a href="${REPO}/blob/main/docs/domain-model/index.md">domain model</a> and the <a href="${REPO}/blob/main/PLAN.md">plans</a>. Report a vulnerability privately, as <a href="${REPO}/blob/main/SECURITY.md">SECURITY.md</a> says.</p>
<p>The picture is drawn in code under <a href="${REPO}/tree/main/docs/design/art">docs/design/art</a>, and every file this page loads comes from the repository. Its fonts, Jersey 15, Tiny5 and Chakra Petch, are their authors’ work under the SIL Open Font License.</p>
</div></footer>
${script || ''}</body>
</html>
`;
}

const key = CONCEPTS.map(([id, name, def], i) => `<li><button type="button" class="kb" data-id="${id}" aria-pressed="false"><span class="k-n">${i + 1}</span><span class="k-b"><span class="k-name">${esc(name)}</span><span class="k-inn">${esc(INN[id])}</span><span class="k-def" hidden><span class="k-lab">In the domain model</span>${esc(def)}</span></span></button></li>`).join('\n');
const ids = CONCEPTS.map(c => c[0]);
if (ids.join() !== M.order.join()) throw new Error('marks.js numbers the concepts in another order than concepts.js');
for (const id of ids) if (!M.pos[id] || !M.box[id] || !INN[id]) throw new Error('no place, close-up or inn text for ' + id);
const DATA = { names: Object.fromEntries(CONCEPTS.map(c => [c[0], c[1]])), order: ids, pos: M.pos, box: M.box, n: FRAMES, tick: TICK, ac: AC, frames: lists, atlas: A + 'inn-loop.png' };
const json = JSON.stringify(DATA).replace(/</g, '\\u003c');

const home = page({
  base: '', title: 'ParleyInn', description: DESCRIPTION, canonical: SITE, script: `<script type="application/json" id="inn-data">${json}</script>\n`,
  body: `<section class="intro" aria-labelledby="name">
<div class="i-a"><p class="eyebrow">Specified, not built yet</p>
<h1 id="name">${img('sign.png', 0, 'ParleyInn', 'sign')}</h1>
<p class="tag">Pins kept verbatim.</p></div>
<div class="i-b"><p class="lede">${inline(LEDE)}</p>
<p class="acts"><a class="btn" href="${REPO}/blob/main/docs/srs/index.md">Read the specification</a><a class="btn" href="${REPO}">See the repository</a></p></div>
</section>
<section class="inn" id="inn" aria-labelledby="inn-h">
<div class="sec-h"><h2 id="inn-h">The inn, numbered</h2>
<p>ParleyInn’s domain model reads like the inventory of a frontier inn, so the picture draws it as written. Robot hands sit at the card table, every seat runs a tab, and the stage at the platform belongs to another inn, enrolled by key.</p>
<p class="hint">Pick a name in the key or a number in the picture, and the picture takes you there.</p></div>
<div class="explore" id="explore">
<div class="stage" id="stage">
<div class="plate" id="plate"><img id="still" src="${A}inn.png" width="${W}" height="${H}" alt="${esc(ALT)}"><canvas id="live" aria-hidden="true" hidden></canvas><div class="marks" id="marks" aria-hidden="true"></div><p class="cap" id="cap" aria-live="polite" hidden><span class="cap-n" id="cap-n"></span><span id="cap-t"></span></p></div>
<div class="bar"><div class="ctl"><button type="button" class="btn ic" id="b-prev" aria-label="Previous place">${I_PREV}</button><button type="button" class="btn ic" id="b-next" aria-label="Next place">${I_NEXT}</button><button type="button" class="btn" id="b-all" disabled>Whole inn</button></div><div class="ctl"><button type="button" class="btn" id="b-nums" aria-pressed="true">Numbers</button><button type="button" class="btn ic" id="b-play" aria-label="Pause the picture"><span class="i-pause">${I_PAUSE}</span><span class="i-play" hidden>${I_PLAY}</span></button></div></div>
</div>
<div class="keycol" id="keycol"><h3 class="sr">Key to the picture</h3><ol class="key">
${key}
</ol></div>
</div>
</section>
<section class="vision" aria-labelledby="vision-h">
<div class="sec-h"><h2 id="vision-h">Your agents, part of your team</h2></div>
<ul class="cards">${VISION.map(([t, d]) => `<li><h3>${inline(t)}</h3><p>${inline(d)}</p></li>`).join('')}</ul>
<p class="under">${inline(UNDERNEATH)}</p>
</section>
<section class="contract" id="contract" aria-labelledby="contract-h">
<div class="sec-h"><h2 id="contract-h">The contract</h2>
<p>The invariants are ParleyInn’s contract. Every requirement serves at least one of them, and a change that would break one is a design change, not a bug fix.</p></div>
<ol class="inv">${INVARIANTS.map(([n, t]) => `<li><a href="${REPO}/blob/main/docs/srs/invariants.md#${n.toLowerCase()}"><span class="inv-n">${n}</span><span class="inv-t">${esc(t)}</span></a></li>`).join('')}</ol>
</section>
<section class="status" aria-labelledby="status-h">
<div class="sec-h"><h2 id="status-h">Where it stands</h2>
<p>Specified, not built yet. The repository holds:</p></div>
<ul class="holds">${HOLDS.map(h => `<li><span>${inline(h)}</span></li>`).join('')}</ul>
</section>`,
});
put('index.html', home);

const kit = `<section class="intro" aria-labelledby="name">
<div class="i-a"><p class="eyebrow">ParleyInn design system</p>
<h1 id="name">Cut from one picture</h1></div>
<div class="i-b"><p class="lede">Every colour, letter and figure here comes from the inn: ${W} by ${H} pixels in ${NCOL} colours, drawn in code one pixel at a time. Change the source under docs/design/art and run node docs/design/art/build.js, and this page, the GitHub page, the design tokens and the images are rebuilt from it.</p>
<p class="hint">The rules, the files and how to change them are in <a href="${REPO}/blob/main/docs/design-system.md">docs/design-system.md</a>.</p></div>
</section>
<div class="kit">
<section class="k-mark" aria-labelledby="mark-h"><h2 id="mark-h">The mark</h2>
${img('sign.png', 0, SIGN_ALT, 'sign')}
<p>The name in hand-set neon over one long key. Peers are enrolled by key and a principal trusts another principal by key, so the key is the one symbol that holds everywhere in the model. In text the name is always ParleyInn, one word; only the sign sets PARLEY and INN apart.</p>
<div class="shots"><figure>${img('mark-stacked.png', 2, 'ParleyInn stacked in cyan neon, PARLEY over INN, with the key beneath')}<figcaption class="cap2">Stacked</figcaption></figure><figure>${img('mark-flat.png', 2, 'ParleyInn and the key in one flat colour')}<figcaption class="cap2">One colour</figcaption></figure><figure><div class="icons">${img('icon.png', 4, 'The key as a square icon')}${img('icon.png', 2, '')}${img('icon.png', 1, '')}</div><figcaption class="cap2">Icon at 64, 32 and 16</figcaption></figure></div>
<div class="shots"><figure>${img('banner.png', 0, 'The inn’s false front and roofline under the night sky, with the neon sign in the middle', 'banner')}<figcaption class="cap2">Banner, for the top of a README</figcaption></figure></div>
<p class="tag">Pins kept verbatim.</p>
</section>
<section class="k-col" aria-labelledby="col-h"><h2 id="col-h">Colours</h2>
<ul class="named">${NAMED.map(([nm, c]) => `<li class="sw"><span class="sw-c c-${c}"></span><span class="sw-n">${esc(nm)}</span><span>${PAL[c]}</span></li>`).join('')}</ul>
<div class="ramps" role="img" aria-label="All ${NCOL} colours of the picture, in ${RAMPS.length} ramps">${RAMPS.map(r => `<div class="ramp">${r.map(c => `<i class="c-${c}" title="${c} ${PAL[c]}"></i>`).join('')}</div>`).join('')}</div>
<p class="cap2">All ${NCOL}, in ${RAMPS.length} ramps, and nothing else</p>
<ul class="uses">${USES.map(([c, t]) => `<li><i class="c-${c}"></i><p>${esc(t)}</p></li>`).join('')}</ul>
<p>The page is drawn in the same colours: night for the ground, bone for text, steel for quiet text, and the sign’s cyan, pink and amber for accents.</p>
</section>
<section class="k-cast" aria-labelledby="cast-h"><h2 id="cast-h">The cast</h2>
<div class="cast">${CAST.map(([id, nm, what, , alt]) => `<figure><span class="spr">${img('cast-' + id + '.png', 0, alt)}</span><figcaption>${esc(nm)}<span>${esc(what)}</span></figcaption></figure>`).join('')}</div>
<p>Hands are steel robots in poncho and hat. The house’s own staff are brass. The proprietor, the principal, keeps the house: a person or a service account. Figures from outside the house are dark iron: the outfit’s horses, the stage’s driver and the smith next door.</p>
</section>
<section class="k-type" aria-labelledby="type-h"><h2 id="type-h">Type</h2>
<p class="spec"><span class="spec-d">Pins kept verbatim</span><small>Jersey 15, for headings. The sign’s own eight letters are drawn by hand, 15 pixels tall.</small></p>
<p class="spec"><span class="spec-l">A2·4812–4830 · Needs you · Recall</span><small>Tiny5, for names, numbers and captions</small></p>
<p class="spec"><span class="spec-t">Every pin that restores is stored verbatim and restored verbatim after every compaction.</span><small>Chakra Petch, for everything read at length</small></p>
<div class="shots">${LETTERING.map(([id, , cap, alt]) => `<figure>${img('lettering-' + id + '.png', 3, alt)}<figcaption class="cap2">${esc(cap)}</figcaption></figure>`).join('')}</div>
<p>Lettering inside the picture is set from pixel fonts on their own grid: Micro 5, Silkscreen, Tiny5 and Jersey 10.</p>
</section>
<section class="k-voice" aria-labelledby="voice-h"><h2 id="voice-h">How it talks</h2>
<ul class="voice"><li>Pins that restore come back verbatim after every compaction.</li><li>Recalled content reaches an agent only when it asks, inside an envelope.</li><li>Quarantined content stays in the record and off every path to an agent.</li></ul>
<p>The inn is the picture, never the vocabulary: copy names the domain model’s concepts, and the frontier only lends them a place.</p>
</section>
<section class="k-files" aria-labelledby="files-h"><h2 id="files-h">Files</h2>
<ul class="files">
<li><a href="tokens.json">tokens.json</a><span>The colours, ramps, type and shape as data</span></li>
<li><a href="tokens.css">tokens.css</a><span>The same as CSS custom properties, with the fonts</span></li>
<li><a href="assets/palette.png">palette.png</a><span>The ${NCOL} colours as an image a pixel editor loads</span></li>
<li><a href="assets/banner-2x.png">banner-2x.png</a><span>The README banner at twice its size</span></li>
<li><a href="assets/social.png">social.png</a><span>The picture for link previews, ${IMG['social.png'].w} by ${IMG['social.png'].h}</span></li>
<li><a href="${REPO}/tree/main/docs/design">docs/design</a><span>Every image, font and licence, and the art source</span></li>
</ul>
</section>
</div>`;
put('design/index.html', page({ base: '../', title: 'ParleyInn design system', description: `The ParleyInn design system: the mark, ${NCOL} colours, the cast and the type, all cut from one pixel picture of the inn.`, canonical: SITE + 'design/', body: kit }));

// ---------- write, or check ----------
// A PNG is compared by what it holds: its header, palette, transparency and pixels, never by the deflate bytes.
function pixels(buf) {
  const out = []; let p = 8;
  while (p < buf.length) { const n = buf.readUInt32BE(p), t = buf.toString('latin1', p + 4, p + 8), d = buf.subarray(p + 8, p + 8 + n); out.push(t === 'IDAT' ? zlib.inflateSync(d).toString('hex') : t + d.toString('hex')); p += 12 + n; }
  return out.join('|');
}
const same = (rel, a, b) => rel.endsWith('.png') ? pixels(a) === pixels(b) : a.equals(b);
const stale = [];
for (const [rel, data] of files) {
  const at = path.join(DOCS, rel);
  if (process.argv.includes('--check')) { if (!fs.existsSync(at) || !same(rel, fs.readFileSync(at), data)) stale.push(rel); continue; }
  fs.mkdirSync(path.dirname(at), { recursive: true });
  if (!fs.existsSync(at) || !same(rel, fs.readFileSync(at), data)) fs.writeFileSync(at, data);
}
const known = new Set([...files.keys()].filter(r => r.startsWith(A)));
for (const f of fs.readdirSync(path.join(DOCS, A))) if (!known.has(A + f)) stale.push(A + f + ' (no longer drawn: remove it)');
if (stale.length) { console.error('these files differ from what docs/design/art draws; run node docs/design/art/build.js:\n  ' + stale.join('\n  ')); process.exit(1); }
console.log(process.argv.includes('--check') ? 'every file matches its source' : `wrote ${files.size} files: ${tiles.size} loop tiles, ${NCOL} colours, the page ${(home.length / 1024).toFixed(0)} KB`);
