'use strict';
// The cowboy-future ParleyInn as pixel art, 480 x 330. scene(t) draws frame t of a short loop; frame 0 is the still picture.
const { Fb, png, rng, IDX, RGB, PAL, ci, B2 } = require('./pix.js');
const W = 480, H = 330, HZ = 206, GY = 236;

const { halo, neonMask, boards, vboards, plate, lamp, FRAMES } = require('./kit.js');
const INTERIOR = require('./interior.js');
const YARD = require('./yard.js');

// ---------- sky ----------
function ring(g, cx, cy, r, back) {
  const a = r * 1.78, b = r * 0.34, ph = -0.3, cp = Math.cos(ph), sp = Math.sin(ph);
  for (let s = 0.74; s <= 1.0001; s += 0.012) {
    if (s > 0.865 && s < 0.905) continue; const col = s >= 0.905 ? (s > 0.955 ? 'd6' : 'd8') : (s > 0.8 ? 'd4' : 'd3');
    for (let th = 0; th < Math.PI * 2; th += 0.006) { const sn = Math.sin(th); if (back ? sn >= 0 : sn < 0) continue; const X = a * s * Math.cos(th), Y = b * s * sn; g.px(Math.round(cx + X * cp - Y * sp), Math.round(cy + X * sp + Y * cp), col); }
  }
}
function planet(g, cx, cy, r) {
  ring(g, cx, cy, r, true);
  const bands = ['d5', 'd5', 'd6', 'd4', 'd6', 'd8', 'd6', 'd5', 'd4', 'd5', 'd6', 'd4'];
  for (let j = -r; j < r; j++) for (let i = -r; i < r; i++) {
    const u = (i + 0.5) / r, v = (j + 0.5) / r, dd = u * u + v * v; if (dd > 1) continue; const z = Math.sqrt(1 - dd);
    const lat = v * 0.955 + u * 0.295; let c = bands[Math.max(0, Math.min(11, Math.floor((lat + 1) * 6)))];
    const frac = (lat + 1) * 6 % 1; if (frac > 0.72 && B2[(cy + j) & 1][(cx + i) & 1] < 2) c = bands[Math.max(0, Math.min(11, Math.floor((lat + 1) * 6) + 1))];
    const s = u * 0.6 + v * 0.5 + z * 0.42; const bz = B2[(cy + j) & 1][(cx + i) & 1];
    if (s < -0.12) c = 'v1'; else if (s < 0.04) c = bz < 2 ? 'v1' : 'v2'; else if (s < 0.2) c = bz < 2 ? 'v2' : 'd2'; else if (s < 0.36) c = ({ d8: 'd5', d6: 'd4', d5: 'd3', d4: 'd2' })[c];
    g.px(cx + i, cy + j, c);
  }
  ring(g, cx, cy, r, false);
}
function moon(g, cx, cy, r, lit, dark) { for (let j = -r; j <= r; j++) for (let i = -r; i <= r; i++) { if (i * i + j * j > r * r + 1) continue; const k = (i + r * 0.55) ** 2 + (j + r * 0.4) ** 2 < r * r * 0.95; g.px(cx + i, cy + j, k ? dark : lit); } }
function sky(g, t) {
  g.VG(0, 0, W, HZ, [[0, 't0'], [28, 't1'], [56, 't2'], [84, 'v1'], [110, 'v2'], [132, 'd2'], [152, 'd3'], [168, 'd4'], [182, 'd5'], [193, 'd6'], [202, 'd8']], 9);
  g.glow(404, 210, 170, 70, 'd8', 0.62, 6);
  const r = rng(11);
  for (let i = 0; i < 170; i++) { const x = Math.floor(r() * W), y = Math.floor(r() * r() * 128), k = r(); let c = k < 0.14 ? 'wh' : k < 0.5 ? 'n5' : 'n4'; if (y > 86) c = 'n4'; if ((i * 5 + t) % FRAMES === 0 && c !== 'n4') c = 'n4'; g.px(x, y, c); }
  [[26, 12], [142, 24], [252, 9], [304, 30], [456, 22], [386, 10], [196, 44], [118, 6]].forEach(([x, y], i) => { const on = (t + i * 5) % FRAMES < FRAMES - 2; g.px(x, y, 'wh'); if (on) { g.px(x - 1, y, 'n5'); g.px(x + 1, y, 'n5'); g.px(x, y - 1, 'n5'); g.px(x, y + 1, 'n5'); } });
  planet(g, 58, 70, 31);
  moon(g, 404, 40, 5, 'p2', 'v2'); moon(g, 430, 60, 2, 'p1', 'v2');
  // a stage far out, climbing
  const tr = [[152, 46], [176, 37], [200, 29], [222, 22], [238, 17]];
  for (let k = 0; k + 1 < tr.length; k++) g.L(tr[k][0], tr[k][1], tr[k + 1][0], tr[k + 1][1], k < 1 ? 't2' : k < 2 ? 't3' : k < 3 ? 'n4' : 'n5');
  g.px(239 + (t % 2), 16, 'wh'); g.px(240, 16, 'cy'); g.px(241, 15, 'wh');
  // streak clouds, lit from underneath
  const cl = (x, y, w, a, b) => { g.H(x + 2, y, w - 5, a); g.H(x, y + 1, w, a); g.H(x + 3, y + 2, w - 9, b); };
  cl(392, 150, 60, 'd2', 'd4'); cl(426, 163, 54, 'd3', 'd5'); cl(360, 171, 40, 'd3', 'd5'); cl(-6, 158, 52, 'd2', 'd4'); cl(10, 176, 64, 'd3', 'd5'); cl(96, 168, 30, 'd3', 'd4');
}

// ---------- land ----------
function butte(g, pts, face, dark, rim) {
  g.PG(pts, face);
  let x0 = 1e9, x1 = -1e9, y0 = 1e9; for (const p of pts) { x0 = Math.min(x0, p[0]); x1 = Math.max(x1, p[0]); y0 = Math.min(y0, p[1]); }
  const F = ci(face);
  // shadowed left faces, strata, and a lit right edge
  g.fn(x0, y0, x1 - x0 + 1, HZ - y0, (i, j, c) => { if (c !== F) return 0; let run = 0; while (g.get(i - run - 1, j) === F && run < 40) run++; let rr = 0; while (g.get(i + rr + 1, j) === F && rr < 40) rr++; const wd = run + rr + 1; if (rr === 0) return rim; if (run < wd * 0.34) return (j % 7 === 3 || B2[j & 1][i & 1] < 3) ? dark : 0; if (j % 7 === 3 && (i * 7 + j) % 11 < 8) return dark; return 0; });
}
function land(g) {
  // far range
  const far = [[0, 206], [0, 199], [18, 198], [26, 194], [58, 194], [64, 199], [112, 200], [118, 196], [150, 196], [156, 201], [230, 201], [330, 200], [372, 197], [380, 193], [398, 193], [404, 198], [480, 199], [480, 206]];
  g.PG(far, 'd3'); g.fn(0, 190, W, 16, (i, j, c) => (c === IDX.d3 && g.get(i, j - 1) !== IDX.d3 && g.get(i, j - 1) !== IDX.d4) ? 'd4' : 0);
  butte(g, [[-6, 206], [2, 190], [9, 187], [12, 163], [16, 154], [47, 154], [50, 161], [53, 185], [61, 189], [72, 206]], 'd2', 'd1', 'd4');
  butte(g, [[384, 206], [393, 191], [400, 188], [403, 167], [407, 160], [431, 160], [434, 168], [437, 188], [446, 192], [455, 206]], 'd2', 'd1', 'd5');
  butte(g, [[456, 206], [461, 193], [463, 172], [465, 169], [468, 169], [470, 173], [472, 193], [479, 206]], 'd2', 'd1', 'd5');
  // plain
  g.R(0, HZ, W, GY - HZ, 'd2');
  g.VG(0, HZ, W, GY, [[HZ, 'd3'], [HZ + 5, 'd2'], [GY - 8, 'd1']], 5);
  const r = rng(5);
  for (let i = 0; i < 90; i++) { const x = Math.floor(r() * W), y = HZ + 1 + Math.floor(r() * (GY - HZ - 2)), w = 3 + Math.floor(r() * 9); g.H(x, y, w, r() < 0.5 ? 'd1' : 'd3'); }
  // the public road running out to the horizon
  g.PG([[447, 236], [470, 236], [447, 208], [445, 208]], 'd3'); g.fn(440, HZ + 2, 34, 28, (i, j, c) => (c === IDX.d3 && (i * 3 + j * 5) % 7 === 0) ? 'd2' : 0);
  for (let i = 0; i < 26; i++) { const x = Math.floor(r() * W), y = HZ + 2 + Math.floor(r() * 24); g.px(x, y, 'g0'); g.px(x + 1, y, 'g0'); g.px(x, y - 1, 'g1'); }
}
function street(g) {
  g.VG(0, GY, W, H, [[GY, 'd1'], [GY + 34, 'd0'], [H - 26, 'n1']], 12);
  const r = rng(23);
  for (let i = 0; i < 260; i++) { const x = Math.floor(r() * W), y = GY + 2 + Math.floor(r() * (H - GY - 3)), w = 2 + Math.floor(r() * 7), k = r(); g.H(x, y, w, k < 0.4 ? 'd0' : k < 0.75 ? 'n1' : 'd1'); }
  for (let i = 0; i < 70; i++) { const x = Math.floor(r() * W), y = GY + 3 + Math.floor(r() * (H - GY - 5)); g.px(x, y, 'd2'); if (r() < 0.4) g.px(x + 1, y, 'd1'); }
  // wheel ruts
  for (const yy of [276, 283]) for (let x = 0; x < W; x += 1) { if ((x * 13 + yy) % 17 < 11) g.px(x, yy + Math.round(Math.sin(x / 37) * 1.2), 'n1'); }
}

// ---------- the inn: shell ----------
const LET = {
  P: ['##########.', '###########', '###########', '.###....###', '.###....###', '.###....###', '.##########', '.##########', '.#########.', '.###.......', '.###.......', '.###.......', '.###.......', '#####......', '#####......'],
  A: ['...#####...', '...#####...', '...#####...', '..###.###..', '..###.###..', '..###.###..', '..###.###..', '.#########.', '.#########.', '.#########.', '.###...###.', '.###...###.', '.###...###.', '####...####', '####...####'],
  R: ['##########.', '###########', '###########', '.###....###', '.###....###', '.###....###', '.#########.', '.########..', '.#########.', '.###..####.', '.###...###.', '.###...###.', '.###...###.', '#####..####', '#####..####'],
  L: ['#####......', '#####......', '.###.......', '.###.......', '.###.......', '.###.......', '.###.......', '.###.......', '.###.......', '.###.......', '.###.......', '.###....###', '###########', '###########', '###########'],
  E: ['###########', '###########', '###########', '.###....###', '.###.......', '.###.......', '.#######...', '.#######...', '.#######...', '.###.......', '.###.......', '.###....###', '###########', '###########', '###########'],
  Y: ['####...####', '####...####', '.###...###.', '.###...###.', '.###...###.', '..###.###..', '..#######..', '...#####...', '....###....', '....###....', '....###....', '....###....', '....###....', '...#####...', '...#####...'],
  I: ['#######', '#######', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '..###..', '#######', '#######'],
  N: ['####...####', '####...####', '.####..###.', '.####..###.', '.#####.###.', '.#####.###.', '.###.#####.', '.###.#####.', '.###.#####.', '.###..####.', '.###..####.', '.###..####.', '.###..####.', '####...####', '####...####'],
};
function wordmark(g, x, y, str, edge, core, dead) { // returns width; dead = index of a letter drawn unlit
  let cx = x, n = 0;
  for (const ch of str) { if (ch === ' ') { cx += 7; continue; } const rows = LET[ch]; if (n === dead) neonMask(g, cx, y, rows, 'pk0', 'pk0'); else neonMask(g, cx, y, rows, edge, core); cx += rows[0].length + 2; n++; }
  return cx - 2 - x;
}
function keyGlyph(g, x, y, len, c, core) { // a long key lying flat: bow at the left, bit at the right
  g.OV(x, y - 3, 8, 8, 0, c); g.px(x + 3, y, core); g.px(x + 4, y, core);
  g.H(x + 8, y, len - 8, core); g.H(x + 8, y + 1, len - 8, c);
  g.R(x + len - 12, y + 2, 2, 3, c); g.R(x + len - 8, y + 2, 2, 2, c); g.R(x + len - 4, y + 2, 2, 4, c); g.px(x + len, y, c); g.px(x + len, y + 1, c);
}
const INN = { x0: 86, x1: 372, cut0: 131, cut1: 368, up0: 112, up1: 162, sal0: 168, sal1: 230, p1: 198, p2: 259, p3: 313 };
function roofGear(g, t) {
  // water tank on its stand
  g.L(96, 60, 99, 44, 'w1'); g.L(116, 60, 113, 44, 'w1'); g.L(97, 58, 114, 46, 'w0'); g.L(115, 58, 98, 46, 'w0');
  g.R(92, 42, 28, 3, 'ink'); g.H(93, 42, 26, 'w3');
  g.R(93, 23, 26, 20, 'ink'); g.R(94, 24, 24, 18, 'w2'); for (let x = 96; x < 118; x += 3) g.V(x, 24, 18, 'w1'); g.V(117, 24, 18, 'w3'); g.V(94, 24, 18, 'w0'); g.H(94, 28, 24, 'n3'); g.H(94, 37, 24, 'n3');
  g.PG([[91, 24], [121, 24], [107, 14], [105, 14]], 'n2', 'ink'); g.L(108, 16, 119, 23, 'n3'); g.V(106, 11, 3, 'ink');
  // stovepipe and its smoke
  for (let k = 0; k < 4; k++) { const p = ((t / FRAMES) + k / 4) % 1; const cx = 291 + p * 20 + Math.sin(p * 8) * 2, cy = 26 - p * 24, rr = 1.6 + p * 3; g.fn(Math.floor(cx - rr), Math.floor(cy - rr), Math.ceil(rr * 2) + 1, Math.ceil(rr * 2) + 1, (i, j) => { const d = (i + 0.5 - cx) ** 2 + (j + 0.5 - cy) ** 2; if (d > rr * rr) return 0; if (p > 0.3 && B2[j & 1][i & 1] >= (p > 0.65 ? 2 : 3)) return 0; return p < 0.5 ? 'n3' : 'n2'; }); }
  g.R(289, 29, 5, 18, 'ink'); g.R(290, 30, 3, 17, 'm1'); g.V(290, 30, 17, 'm2'); g.R(288, 27, 7, 3, 'ink'); g.H(289, 28, 5, 'n3');
  // windpump
  const cx = 346, cy = 22;
  g.L(339, 58, 344, 26, 'ink'); g.L(353, 58, 348, 26, 'ink'); g.L(340, 58, 345, 26, 'n2'); g.L(352, 58, 347, 26, 'n2');
  g.H(342, 46, 9, 'n2'); g.H(343, 36, 7, 'n2'); g.L(341, 57, 350, 47, 'n2'); g.L(351, 57, 342, 47, 'n2'); g.L(342, 45, 349, 37, 'n2'); g.L(350, 45, 343, 37, 'n2');
  g.PG([[352, 20], [364, 16], [364, 28], [352, 24]], 'r1', 'ink'); g.L(354, 22, 362, 20, 'p1');
  const a0 = t / FRAMES * Math.PI / 4;
  for (let k = 0; k < 8; k++) { const a = a0 + k * Math.PI / 4, c = Math.cos, s = Math.sin; g.PG([[cx + c(a) * 3, cy + s(a) * 3], [cx + c(a) * 12, cy + s(a) * 12], [cx + c(a + 0.34) * 12, cy + s(a + 0.34) * 12]], k % 2 ? 'm2' : 'm3'); g.L(Math.round(cx + c(a) * 3), Math.round(cy + s(a) * 3), Math.round(cx + c(a) * 12), Math.round(cy + s(a) * 12), 'm1'); }
  g.R(cx - 1, cy - 1, 3, 3, 'am'); g.px(cx, cy, 'ink');
}
function innShell(g, t) {
  const r = rng(3);
  roofGear(g, t);
  // false front with a raised centre and a cornice
  boards(g, 84, 62, 290, 48, 'w2', 'w1', 4, r);
  boards(g, 150, 50, 158, 12, 'w2', 'w1', 4, r);
  g.R(82, 58, 68, 4, 'w3').H(82, 58, 68, 'w4').H(82, 61, 68, 'w0'); g.R(308, 58, 68, 4, 'w3').H(308, 58, 68, 'w4').H(308, 61, 68, 'w0');
  g.R(148, 46, 162, 4, 'w3').H(148, 46, 162, 'w4').H(148, 49, 162, 'w0');
  for (let x = 86; x < 148; x += 10) g.R(x, 62, 2, 3, 'w1'); for (let x = 312; x < 372; x += 10) g.R(x, 62, 2, 3, 'w1'); for (let x = 152; x < 308; x += 10) g.R(x, 50, 2, 3, 'w1');
  // small arch on top of the centre
  g.PG([[204, 46], [208, 40], [216, 36], [242, 36], [250, 40], [254, 46]], 'w2', 'w0'); g.H(209, 40, 40, 'w1'); g.H(216, 36, 27, 'w4'); g.OV(225, 38, 8, 8, 'w1', 'w0'); g.px(228, 41, 'am'); g.px(229, 41, 'am'); g.px(228, 42, 'd5'); g.px(229, 42, 'd5');
  g.V(84, 62, 48, 'w0'); g.V(373, 62, 48, 'w0'); g.V(150, 50, 12, 'w0'); g.V(307, 50, 12, 'w0');
  // dusk on the upper boards, faint sky light on the right
  g.light(84, 36, 290, 74, 'n1', (i, j) => 0.34 + (j < 62 ? 0.1 : 0), 6, true);
  // painted lettering either side of the sign
  g.T(124, 76, 'ROOMS', 'p0', 's5', { align: 'c' }); g.T(124, 84, '& SEATS', 'p0', 's5', { align: 'c' });
  g.T(337, 72, 'PINS', 'p0', 's5', { align: 'c' }); g.T(337, 80, 'KEPT', 'p0', 's5', { align: 'c' }); g.T(337, 88, 'VERBATIM', 'p0', 's5', { align: 'c' });
  // the sign board
  g.R(162, 66, 138, 36, 'n1').B(162, 66, 138, 36, 'ink').B(163, 67, 136, 34, 'w2');
  for (const x of [170, 290]) { g.R(x, 62, 2, 4, 'm1'); }
  const dead = (t === 15 || t === 17) ? 3 : -1;
  wordmark(g, 173, 71, 'PARLEY INN', 'pk', 'pkw', dead);
  keyGlyph(g, 174, 92, 112, 'cy', 'cyw');
  halo(g, 163, 67, 136, 34, ['pk', 'pkw'], 'pk', [0.34, 0.2, 0.1, 0.05], 8);
  halo(g, 163, 86, 136, 15, ['cy', 'cyw'], 'cy', [0.3, 0.16, 0.07], 8);
  g.glow(231, 84, 110, 40, 'pk', 0.2, 8);
  // header beam over the open floors, floor between them, end walls
  g.R(84, 108, 290, 4, 'w1').H(84, 108, 290, 'w3').H(84, 111, 290, 'ink');
  g.R(INN.cut0 - 3, INN.up0, 3, 118, 'n1'); for (let j = INN.up0; j < 230; j += 3) g.px(INN.cut0 - 2, j + 1, 'n3');
  g.R(INN.cut1, INN.up0, 4, 118, 'w1').V(INN.cut1, INN.up0, 118, 'ink').V(371, INN.up0, 118, 'w0'); g.R(372, 110, 2, 120, 't2').V(373, 110, 120, 't1');
  g.R(INN.cut0, 162, INN.cut1 - INN.cut0, 6, 'w1').H(INN.cut0, 162, INN.cut1 - INN.cut0, 'w4').H(INN.cut0, 163, INN.cut1 - INN.cut0, 'w2').H(INN.cut0, 167, INN.cut1 - INN.cut0, 'ink');
  for (let x = INN.cut0 + 4; x < INN.cut1 - 2; x += 9) g.R(x, 164, 2, 3, 'w0');
}
function facade(g, t) {
  const x0 = 86, x1 = 128; const r = rng(31);
  boards(g, x0, 112, x1 - x0, 118, 'w2', 'w1', 4, r);
  g.light(x0, 112, x1 - x0, 118, 'n1', () => 0.34, 6, true);
  // corner posts in teal paint carry the balcony
  g.R(x0 - 2, 112, 3, 118, 't2').V(x0 - 2, 112, 118, 't3').V(x0, 112, 118, 't1');
  g.R(x1 - 1, 112, 3, 118, 't2').V(x1 - 1, 112, 118, 't3').V(x1 + 1, 112, 118, 't1');
  // ROOMS in neon
  g.R(89, 114, 37, 13, 'n1').B(89, 114, 37, 13, 'ink');
  g.T(108, 116, 'ROOMS', (t === 6) ? 't4' : 'cy', 'j10', { align: 'c' });
  halo(g, 90, 115, 35, 11, ['cy'], 'cy', [0.3, 0.14], 8);
  // balcony door, lit from inside
  g.R(98, 130, 18, 29, 't2').B(98, 130, 18, 29, 'ink'); g.R(100, 132, 14, 25, 'd6'); g.R(100, 132, 14, 6, 'am'); g.V(107, 132, 25, 't2'); g.H(100, 143, 14, 't2'); g.R(100, 150, 14, 7, 'd5');
  g.R(101, 133, 2, 9, 'r1'); g.R(111, 133, 2, 9, 'r1'); // curtains
  // balcony
  g.R(x0 - 4, 159, x1 - x0 + 9, 3, 'w3').H(x0 - 4, 159, x1 - x0 + 9, 'w4').H(x0 - 4, 161, x1 - x0 + 9, 'ink');
  g.H(x0 - 4, 149, x1 - x0 + 9, 't4').H(x0 - 4, 150, x1 - x0 + 9, 't2'); for (let x = x0 - 3; x <= x1 + 3; x += 4) g.V(x, 151, 8, 't3'); g.V(x0 - 4, 148, 11, 't2'); g.V(x1 + 4, 148, 11, 't2');
  // OPEN board, doorway and batwing doors
  g.R(96, 190, 26, 40, 'w0').B(95, 189, 28, 42, 'ink'); g.R(97, 191, 24, 39, 'd4'); g.R(97, 191, 24, 10, 'd5'); g.R(97, 222, 24, 8, 'w2');
  g.R(97, 204, 11, 17, 'w3').B(97, 204, 11, 17, 'w1'); g.R(110, 204, 11, 17, 'w3').B(110, 204, 11, 17, 'w1');
  for (let j = 206; j < 220; j += 2) { g.H(99, j, 7, 'w2'); g.H(112, j, 7, 'w2'); } g.H(98, 204, 9, 'w4'); g.H(111, 204, 9, 'w4'); g.px(107, 212, 'am'); g.px(110, 212, 'am');
  plate(g, 109, 179, 'OPEN', 'r1', 'p2', 'm5', { h: 9, pad: 3 }); g.V(101, 176, 3, 'ink'); g.V(116, 176, 3, 'ink');
  // wall lantern and a rain barrel
  g.R(122, 196, 4, 2, 'n2'); g.R(122, 198, 4, 5, 'am'); g.px(123, 199, 'wh'); g.px(124, 200, 'wh'); g.B(121, 197, 6, 7, 'ink'); g.R(122, 203, 4, 1, 'n2');
  g.R(87, 219, 8, 11, 'w2').B(86, 218, 10, 12, 'ink'); g.H(87, 221, 8, 'm1'); g.H(87, 227, 8, 'm1'); g.H(88, 219, 6, 't3');
  g.clip(80, 170, 51, 70); g.glow(124, 200, 22, 20, 'am', 0.55, 6); g.unclip();
}
function boardwalk(g) {
  g.R(80, 230, 300, 6, 'w1'); g.H(80, 230, 300, 'w4'); g.H(80, 231, 300, 'w3'); g.H(80, 235, 300, 'ink');
  for (let x = 84; x < 380; x += 12) g.V(x, 232, 3, 'w0');
  g.R(100, 236, 18, 2, 'w2').H(100, 236, 18, 'w3').H(100, 237, 18, 'ink'); // step
}

function scene(t) {
  t = t || 0; const g = new Fb(W, H, 't0');
  sky(g, t); land(g); YARD.gate(g, t); street(g); YARD.groundLight(g);
  YARD.forge(g, t); YARD.stageStop(g, t);
  innShell(g, t); INTERIOR.upstairs(g, t); INTERIOR.saloon(g, t); facade(g, t); boardwalk(g);
  YARD.hitch(g, t); YARD.rig(g, t); YARD.pen(g, t); YARD.stake(g); YARD.saguaro(g, 364, 296, 36); YARD.bootHill(g, t); YARD.coach(g, t); YARD.litter(g, t);
  return g;
}
module.exports = { scene, W, H, FRAMES, INN, wordmark, keyGlyph, LET };

if (require.main === module) {
  // node scene.js [frame] [x y w h] [scale]: writes the frame, or a crop of it, enlarged, to the system's temporary directory.
  const fs = require('fs'), os = require('os'), path = require('path'); const a = process.argv.slice(2); const t = +(a[0] || 0); const g = scene(t);
  const crop = a.length >= 5 ? a.slice(1, 5).map(Number) : null, sc = +(a[5] || (crop ? 4 : 2));
  const out = path.join(os.tmpdir(), crop ? `parleyinn-c-${crop.join('-')}.png` : `parleyinn-frame-${t}.png`);
  fs.writeFileSync(out, png(g, { scale: sc, crop })); console.log('wrote', out);
}
