'use strict';
// Pixel-art toolkit: an indexed framebuffer, a fixed palette, primitives, dithering, palette-aware light, bitmap text and a PNG writer.
const zlib = require('zlib'), fs = require('fs'), path = require('path');

const PAL = {
  ink: '#15101C', n1: '#231C2D', n2: '#372F45', n3: '#544C66', n4: '#7F7892', n5: '#B3ADC2',
  t0: '#0B1A29', t1: '#0F2A3A', t2: '#164250', t3: '#1F5E68', t4: '#2F8C8A', t5: '#6CC9B4',
  cy: '#1FE5D5', cyw: '#D9FFF5',
  v1: '#353B63', v2: '#62487A',
  d0: '#38203A', d1: '#5A2E48', d2: '#7F4152', d3: '#A95A59', d4: '#CC6E57', d5: '#E98F59', d6: '#F5B46A',
  am: '#FFC857', d8: '#FFE7A3', wh: '#FFF6DC',
  w0: '#2C1B22', w1: '#472C2D', w2: '#694235', w3: '#8E5E44', w4: '#B98A63', w5: '#D9B387',
  p0: '#A08D6E', p1: '#CDB78E', p2: '#EAD9B5',
  m0: '#2A3540', m1: '#4A5B66', m2: '#7B9199', m3: '#B2C5C5', m4: '#E4F1ED',
  r0: '#5A1B2B', r1: '#96302E', r2: '#D8483E',
  pk0: '#8C2654', pk: '#FF5C8A', pkw: '#FFC4D6',
  g0: '#1F483C', g1: '#3D7B58', g2: '#7DB06A',
};
const NAMES = Object.keys(PAL);
const IDX = {}; NAMES.forEach((n, i) => { IDX[n] = i + 1; });
const hex = h => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
const RGB = [[0, 0, 0], ...NAMES.map(n => hex(PAL[n]))];
const ci = c => { if (typeof c === 'number') return c; const i = IDX[c]; if (!i) throw new Error('unknown colour ' + c); return i; };

// OKLab, for nearest-colour lookups when light is blended into the palette.
const lin = u => { u /= 255; return u <= 0.04045 ? u / 12.92 : Math.pow((u + 0.055) / 1.055, 2.4); };
function oklab([r, g, b]) {
  r = lin(r); g = lin(g); b = lin(b);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s, 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s, 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s];
}
const LAB = RGB.map(oklab);
function nearest(rgb, exclude) {
  const q = oklab(rgb); let best = 1, bd = 1e9;
  for (let i = 1; i < LAB.length; i++) { if (exclude && exclude.has(i)) continue; const a = LAB[i]; const d = (a[0] - q[0]) ** 2 + (a[1] - q[1]) ** 2 + (a[2] - q[2]) ** 2; if (d < bd) { bd = d; best = i; } }
  return best;
}
const mixCache = new Map();
// The neon and near-white colours are kept for light sources themselves, so blending never lands on them by accident.
const NO_BLEND = new Set(['cy', 'cyw', 'pk', 'pkw'].map(n => IDX[n]));
function mix(c, target, level, steps) {
  if (level <= 0) return c; const key = c + '|' + target.join(',') + '|' + level + '|' + steps; let v = mixCache.get(key);
  if (v === undefined) { const a = level / steps, s = RGB[c]; v = nearest([s[0] + (target[0] - s[0]) * a, s[1] + (target[1] - s[1]) * a, s[2] + (target[2] - s[2]) * a], NO_BLEND); mixCache.set(key, v); }
  return v;
}
const B2 = [[0, 2], [3, 1]];
const B4 = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

function rng(seed) { let s = seed >>> 0 || 1; return () => { s ^= s << 13; s >>>= 0; s ^= s >>> 17; s ^= s << 5; s >>>= 0; return s / 4294967296; }; }

let FONTS = null;
function fonts() {
  if (FONTS) return FONTS; FONTS = JSON.parse(fs.readFileSync(path.join(__dirname, 'glyphs.json'), 'utf8'));
  for (const k in FONTS) { const f = FONTS[k], H = f.glyphs.H.rows; let top = 0; while (top < H.length && !H[top].includes('#')) top++; let bot = H.length - 1; while (bot > 0 && !H[bot].includes('#')) bot--; f.capTop = top; f.capH = bot - top + 1; }
  return FONTS;
}

class Fb {
  constructor(w, h, fill) { this.w = w; this.h = h; this.d = new Uint8Array(w * h); if (fill) this.d.fill(ci(fill)); this.cl = [0, 0, w, h]; this.cls = []; }
  clip(x, y, w, h) { this.cls.push(this.cl); const c = this.cl; const x0 = Math.max(c[0], x), y0 = Math.max(c[1], y), x1 = Math.min(c[0] + c[2], x + w), y1 = Math.min(c[1] + c[3], y + h); this.cl = [x0, y0, Math.max(0, x1 - x0), Math.max(0, y1 - y0)]; return this; }
  unclip() { this.cl = this.cls.pop() || [0, 0, this.w, this.h]; return this; }
  inb(x, y) { const c = this.cl; return x >= c[0] && y >= c[1] && x < c[0] + c[2] && y < c[1] + c[3]; }
  px(x, y, c) { x |= 0; y |= 0; if (this.inb(x, y)) this.d[y * this.w + x] = ci(c); return this; }
  get(x, y) { return (x < 0 || y < 0 || x >= this.w || y >= this.h) ? 0 : this.d[y * this.w + x]; }
  R(x, y, w, h, c) { c = ci(c); x |= 0; y |= 0; const k = this.cl; const x0 = Math.max(k[0], x), y0 = Math.max(k[1], y), x1 = Math.min(k[0] + k[2], x + w), y1 = Math.min(k[1] + k[3], y + h); for (let j = y0; j < y1; j++) this.d.fill(c, j * this.w + x0, j * this.w + Math.max(x0, x1)); return this; }
  H(x, y, n, c) { return this.R(x, y, n, 1, c); }
  V(x, y, n, c) { return this.R(x, y, 1, n, c); }
  B(x, y, w, h, c) { return this.R(x, y, w, 1, c).R(x, y + h - 1, w, 1, c).R(x, y, 1, h, c).R(x + w - 1, y, 1, h, c); }
  RB(x, y, w, h, f, b) { return this.R(x, y, w, h, f).B(x, y, w, h, b); }
  // rounded box: border with the four corner pixels left open
  RR(x, y, w, h, f, b) { this.R(x + 1, y, w - 2, h, f).R(x, y + 1, w, h - 2, f); if (b) this.R(x + 1, y, w - 2, 1, b).R(x + 1, y + h - 1, w - 2, 1, b).R(x, y + 1, 1, h - 2, b).R(x + w - 1, y + 1, 1, h - 2, b); return this; }
  L(x0, y0, x1, y1, c) { c = ci(c); x0 |= 0; y0 |= 0; x1 |= 0; y1 |= 0; const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1; let e = dx + dy; for (;;) { this.px(x0, y0, c); if (x0 === x1 && y0 === y1) break; const e2 = 2 * e; if (e2 >= dy) { e += dy; x0 += sx; } if (e2 <= dx) { e += dx; y0 += sy; } } return this; }
  // polyline through points
  PL(pts, c) { for (let i = 0; i + 1 < pts.length; i++) this.L(pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1], c); return this; }
  // filled ellipse inscribed in the box x,y,w,h; optional border colour
  OV(x, y, w, h, f, b) {
    const rx = w / 2, ry = h / 2, cx = x + rx, cy = y + ry; const inside = (i, j) => { const u = (i + 0.5 - cx) / rx, v = (j + 0.5 - cy) / ry; return u * u + v * v <= 1.0001; };
    for (let j = y; j < y + h; j++) for (let i = x; i < x + w; i++) { if (!inside(i, j)) continue; const edge = b && !(inside(i - 1, j) && inside(i + 1, j) && inside(i, j - 1) && inside(i, j + 1)); if (edge) this.px(i, j, b); else if (f) this.px(i, j, f); }
    return this;
  }
  // filled polygon (pixel centres), optional border
  PG(pts, f, b) {
    if (f) { f = ci(f); let y0 = 1e9, y1 = -1e9; for (const p of pts) { y0 = Math.min(y0, p[1]); y1 = Math.max(y1, p[1]); }
      for (let j = Math.floor(y0); j <= Math.ceil(y1); j++) { const yc = j + 0.5, xs = []; for (let i = 0; i < pts.length; i++) { const a = pts[i], q = pts[(i + 1) % pts.length]; if ((a[1] <= yc && q[1] > yc) || (q[1] <= yc && a[1] > yc)) xs.push(a[0] + (yc - a[1]) / (q[1] - a[1]) * (q[0] - a[0])); } xs.sort((u, v) => u - v); for (let k = 0; k + 1 < xs.length; k += 2) { const xa = Math.ceil(xs[k] - 0.5), xb = Math.floor(xs[k + 1] - 0.5); if (xb >= xa) this.R(xa, j, xb - xa + 1, 1, f); } } }
    if (b) this.PL(pts.concat([pts[0]]), b);
    return this;
  }
  // per-pixel shader over a box: fn(x,y,current) returns a colour (name or index) or nothing
  fn(x, y, w, h, f) { for (let j = y; j < y + h; j++) for (let i = x; i < x + w; i++) { if (!this.inb(i, j)) continue; const r = f(i, j, this.d[j * this.w + i]); if (r) this.d[j * this.w + i] = ci(r); } return this; }
  // ordered dither of one colour over a box at lvl/4 coverage (2x2 Bayer), or lvl/16 with fine=true
  DI(x, y, w, h, c, lvl, fine) { c = ci(c); return this.fn(x, y, w, h, (i, j) => (fine ? B4[j & 3][i & 3] < lvl : B2[j & 1][i & 1] < lvl) ? c : 0); }
  // vertical bands with dithered joins: stops = [[y, colour], ...] ascending; soft = height of each join in px
  VG(x, y0, w, y1, stops, soft) {
    soft = soft || 8;
    for (let j = y0; j < y1; j++) for (let i = x; i < x + w; i++) {
      let k = 0; while (k + 1 < stops.length && stops[k + 1][0] <= j) k++;
      let c = stops[k][1];
      if (k + 1 < stops.length) { const d = stops[k + 1][0] - j; if (d <= soft) { const t = 1 - d / soft; const lv = t < 0.34 ? 1 : t < 0.67 ? 2 : 3; if (B2[j & 1][i & 1] < lv) c = stops[k + 1][1]; } }
      this.px(i, j, c);
    }
    return this;
  }
  // blend light of an RGB colour into what is there: a(x,y) in 0..1, snapped to palette, dithered between steps
  // nd = no dither: the light falls in clean bands, which keeps small detail legible
  // target may also be a function (colour, step, steps) => colour, for light whose steps are chosen by hand
  light(x, y, w, h, target, a, steps, nd) {
    steps = steps || 6; const pick = typeof target === 'function' ? target : null; if (typeof target === 'string') target = RGB[ci(target)];
    return this.fn(x, y, w, h, (i, j, cur) => { if (!cur || NO_BLEND.has(cur)) return 0; let v = a(i, j); if (!(v > 0)) return 0; if (v > 1) v = 1; const A = v * steps, b = Math.floor(A), f = A - b; const lv = nd ? (Math.abs(f - 0.5) < 0.13 ? b + (B2[j & 1][i & 1] < 2 ? 1 : 0) : Math.round(A)) : b + (f * 4 > B2[j & 1][i & 1] + 0.5 ? 1 : 0); return lv > 0 ? (pick ? pick(cur, lv, steps) : mix(cur, target, lv, steps)) : 0; });
  }
  glow(cx, cy, rx, ry, target, strength, steps, nd) {
    return this.light(Math.floor(cx - rx), Math.floor(cy - ry), Math.ceil(rx * 2) + 1, Math.ceil(ry * 2) + 1, target, (i, j) => { const u = (i + 0.5 - cx) / rx, v = (j + 0.5 - cy) / ry, d = Math.sqrt(u * u + v * v); return d >= 1 ? 0 : strength * (1 - d) * (1 - d); }, steps, nd);
  }
  shade(x, y, w, h, a, steps, nd) { return this.light(x, y, w, h, RGB[IDX.ink], typeof a === 'number' ? () => a : a, steps, nd); }
  // sprite from rows of characters; legend maps characters to colours; '.' and ' ' are clear
  S(x, y, rows, legend, o) {
    o = o || {}; const hh = rows.length;
    for (let j = 0; j < hh; j++) { const r = rows[j]; for (let i = 0; i < r.length; i++) { const ch = r[i]; if (ch === '.' || ch === ' ') continue; const c = legend[ch]; if (c === undefined) throw new Error('sprite legend lacks "' + ch + '"'); this.px(x + (o.flip ? r.length - 1 - i : i), y + j, c); } }
    return this;
  }
  tw(str, font, sp) { const f = fonts()[font]; let w = 0; for (const ch of String(str)) { const g = f.glyphs[ch] || f.glyphs['?']; w += g.adv + (sp || 0); } return w - (sp || 0) - 1; }
  // text with y at the top of the capitals; align 'l' | 'c' | 'r' about x
  T(x, y, str, c, font, o) {
    o = o || {}; font = font || 't5'; const f = fonts()[font]; str = String(str); const sp = o.sp || 0; const w = this.tw(str, font, sp);
    let cx = o.align === 'c' ? Math.round(x - w / 2) : o.align === 'r' ? x - w : x;
    for (const ch of str) { const g = f.glyphs[ch] || f.glyphs['?']; for (let r = 0; r < g.rows.length; r++) { const row = g.rows[r]; for (let k = 0; k < row.length; k++) if (row[k] === '#') { if (o.sh) this.px(cx + g.ox + k + (o.shx === undefined ? 1 : o.shx), y + r - f.capTop + (o.shy === undefined ? 1 : o.shy), o.sh); } } cx += g.adv + sp; }
    cx = o.align === 'c' ? Math.round(x - w / 2) : o.align === 'r' ? x - w : x;
    for (const ch of str) { const g = f.glyphs[ch] || f.glyphs['?']; for (let r = 0; r < g.rows.length; r++) { const row = g.rows[r]; for (let k = 0; k < row.length; k++) if (row[k] === '#') this.px(cx + g.ox + k, y + r - f.capTop, c); } cx += g.adv + sp; }
    return this;
  }
  // copy another buffer in; index 0 is clear
  blit(src, x, y, o) { o = o || {}; for (let j = 0; j < src.h; j++) for (let i = 0; i < src.w; i++) { const c = src.d[j * src.w + (o.flip ? src.w - 1 - i : i)]; if (c) this.px(x + i, y + j, c); } return this; }
  // a copy with a one-pixel outline around everything opaque (4-neighbours)
  outlined(c) { c = ci(c); const o = new Fb(this.w, this.h); o.d.set(this.d); for (let j = 0; j < this.h; j++) for (let i = 0; i < this.w; i++) { if (this.d[j * this.w + i]) continue; if (this.get(i - 1, j) || this.get(i + 1, j) || this.get(i, j - 1) || this.get(i, j + 1)) o.d[j * this.w + i] = c; } return o; }
  copy() { const o = new Fb(this.w, this.h); o.d.set(this.d); return o; }
}

// ---- PNG (8-bit indexed) ----
const CRC = (() => { const t = new Uint32Array(256); for (let n = 0; n < 256; n++) { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xEDB88320 ^ (c >>> 1) : c >>> 1; t[n] = c >>> 0; } return t; })();
function crc32(buf) { let c = 0xFFFFFFFF; for (let i = 0; i < buf.length; i++) c = CRC[(c ^ buf[i]) & 255] ^ (c >>> 8); return (c ^ 0xFFFFFFFF) >>> 0; }
function chunk(type, data) { const len = Buffer.alloc(4); len.writeUInt32BE(data.length); const td = Buffer.concat([Buffer.from(type, 'ascii'), data]); const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td)); return Buffer.concat([len, td, crc]); }
// opts: scale (integer), crop [x,y,w,h], transparent (index 0 becomes clear)
function png(fb, opts) {
  opts = opts || {}; const s = opts.scale || 1; const [cx, cy, cw, ch] = opts.crop || [0, 0, fb.w, fb.h]; const W = cw * s, H = ch * s;
  const raw = Buffer.alloc((W + 1) * H);
  for (let j = 0; j < H; j++) { const row = j * (W + 1); raw[row] = 0; const sy = cy + Math.floor(j / s); for (let i = 0; i < W; i++) { const sx = cx + Math.floor(i / s); raw[row + 1 + i] = (sx < 0 || sy < 0 || sx >= fb.w || sy >= fb.h) ? 1 : fb.d[sy * fb.w + sx]; } }
  const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(W, 0); ihdr.writeUInt32BE(H, 4); ihdr[8] = 8; ihdr[9] = 3; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  const plte = Buffer.alloc(RGB.length * 3); RGB.forEach((c, i) => { plte[i * 3] = c[0]; plte[i * 3 + 1] = c[1]; plte[i * 3 + 2] = c[2]; });
  const parts = [Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('PLTE', plte)];
  if (opts.transparent) parts.push(chunk('tRNS', Buffer.from([0])));
  parts.push(chunk('IDAT', zlib.deflateSync(raw, { level: 9 })), chunk('IEND', Buffer.alloc(0)));
  return Buffer.concat(parts);
}

module.exports = { PAL, NAMES, IDX, RGB, ci, Fb, png, rng, mix, nearest, fonts, B2, B4 };
