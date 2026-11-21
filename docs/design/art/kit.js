'use strict';
// Helpers shared by every part of the scene: light that spreads from bright pixels, neon lettering, boards, plates and lamps.
const { Fb, rng, IDX, RGB, ci, B2, mix } = require('./pix.js');
// ---------- shared helpers ----------
// Light spreading from pixels of the given source colours: alphas[d-1] is the strength at distance d.
function halo(g, x, y, w, h, src, target, alphas, steps) {
  const S = new Set(src.map(ci)), r = alphas.length, pts = [];
  for (let j = y - r; j < y + h + r; j++) for (let i = x - r; i < x + w + r; i++) if (S.has(g.get(i, j))) pts.push([i, j]);
  if (!pts.length) return g;
  const D = new Float32Array((w + 2 * r) * (h + 2 * r)).fill(99), ww = w + 2 * r;
  for (const [px, py] of pts) for (let dj = -r; dj <= r; dj++) for (let di = -r; di <= r; di++) { const i = px + di - (x - r), j = py + dj - (y - r); if (i < 0 || j < 0 || i >= ww || j >= h + 2 * r) continue; const d = Math.sqrt(di * di + dj * dj); if (d < D[j * ww + i]) D[j * ww + i] = d; }
  return g.light(x - r, y - r, w + 2 * r, h + 2 * r, target, (i, j) => { if (S.has(g.get(i, j))) return 0; const d = D[(j - (y - r)) * ww + (i - (x - r))]; if (d > r) return 0; return alphas[Math.max(0, Math.ceil(d) - 1)]; }, steps || 8);
}
// Neon: a mask drawn with a bright core where strokes are thick enough, and a halo on whatever is behind.
function neonMask(g, x, y, rows, edge, core) {
  const at = (i, j) => j >= 0 && j < rows.length && i >= 0 && i < rows[j].length && rows[j][i] === '#';
  for (let j = 0; j < rows.length; j++) for (let i = 0; i < rows[j].length; i++) { if (!at(i, j)) continue; const inner = at(i - 1, j) && at(i + 1, j) && at(i, j - 1) && at(i, j + 1); g.px(x + i, y + j, inner ? core : edge); }
}
function boards(g, x, y, w, h, base, line, pitch, r, hi) { // horizontal boards with staggered butt joints
  g.R(x, y, w, h, base);
  for (let j = y + pitch - 1; j < y + h; j += pitch) { g.H(x, j, w, line); if (hi) g.H(x, j + 1 < y + h ? j + 1 : j, w, hi); }
  if (r) for (let j = y; j < y + h; j += pitch) { let i = x + Math.floor(r() * 30) + 6; while (i < x + w - 2) { g.V(i, j, Math.min(pitch - 1, y + h - j), line); i += 18 + Math.floor(r() * 26); } }
}
function vboards(g, x, y, w, h, base, line, pitch, off) { g.R(x, y, w, h, base); for (let i = x + (off || 0); i < x + w; i += pitch) g.V(i, y, h, line); }
function plate(g, cx, y, str, bg, fg, font, o) { // small lettered plate centred on cx; returns [x, y, w, h]
  o = o || {}; font = font || 'm5'; const tw = g.tw(str, font), pad = o.pad === undefined ? 2 : o.pad, w = tw + pad * 2, x = Math.round(cx - w / 2), h = (o.h || 9);
  g.R(x, y, w, h, bg); g.B(x, y, w, h, o.edge || 'ink'); g.T(x + pad, y + Math.floor((h - 5) / 2), str, fg, font); return [x, y, w, h];
}
// Light whose steps are chosen by hand: a colour named in rows climbs its own row, step by step; any other blends toward the fallback.
function ramped(rows, fallback) {
  const T = new Map(Object.entries(rows).map(([c, r]) => [ci(c), r.map(ci)])), F = RGB[ci(fallback)];
  return (cur, lv, steps) => { const r = T.get(cur); return r ? r[Math.min(lv, r.length) - 1] : mix(cur, F, lv, steps); };
}
function lamp(g, x, y, t, o) { // hanging oil lamp: cord from the ceiling at y, flame flickers
  o = o || {}; const drop = o.drop || 5; g.V(x, y, drop, 'ink');
  g.R(x - 2, y + drop, 5, 1, 'n3'); g.R(x - 1, y + drop + 1, 3, 3, 'd8'); g.px(x, y + drop + 2, 'wh'); if (t % 5 === 3) g.px(x, y + drop + 1, 'wh'); g.px(x - 2, y + drop + 2, 'am'); g.px(x + 2, y + drop + 2, 'am'); g.R(x - 1, y + drop + 4, 3, 1, 'n3');
  return [x, y + drop + 2];
}

const FRAMES = 24;
module.exports = { halo, neonMask, boards, vboards, plate, lamp, ramped, FRAMES };
