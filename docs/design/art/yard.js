'use strict';
// Everything outside the inn: the forge, the stage platform and the visiting stage, the gate, and the street in front.
const { Fb, rng, IDX, ci, B2 } = require('./pix.js');
const { halo, boards, vboards, plate, lamp, FRAMES } = require('./kit.js');
const { figure, PON, HAT, STEEL, BRASS, DARK, vkey } = require('./interior.js');
const ob = (g, x, y, w, h, f) => g.R(x - 1, y - 1, w + 2, h + 2, 'ink').R(x, y, w, h, f); // box with an outline outside it

function puff(g, cx, cy, r, c, thin) { g.fn(Math.floor(cx - r), Math.floor(cy - r), Math.ceil(r * 2) + 1, Math.ceil(r * 2) + 1, (i, j) => { const d = (i + 0.5 - cx) ** 2 + (j + 0.5 - cy) ** 2; if (d > r * r) return 0; if (thin && B2[j & 1][i & 1] >= thin) return 0; return c; }); }

function forge(g, t) {
  // smoke from the stack
  for (let k = 0; k < 4; k++) { const p = ((t / FRAMES) + k / 4) % 1; puff(g, 15 + p * 13 + Math.sin(p * 9) * 2, 160 - p * 40, 2 + p * 3.4, p < 0.5 ? 'n3' : 'n4', p < 0.3 ? 0 : p < 0.65 ? 3 : 2); }
  g.R(10, 165, 11, 30, 'ink'); g.R(11, 166, 9, 28, 'd2'); for (let j = 169; j < 194; j += 4) { g.H(11, j, 9, 'd1'); g.px(13 + ((j >> 2) & 1) * 3, j - 2, 'd1'); } g.R(10, 163, 11, 3, 'n2'); g.H(10, 163, 11, 'n3'); g.V(19, 166, 28, 'd3');
  // shed
  vboards(g, 8, 198, 70, 38, 'w2', 'w1', 5, 1);
  g.H(36, 204, 38, 'w3'); g.H(36, 205, 38, 'w0');
  g.V(40, 206, 7, 'm1'); g.V(42, 206, 7, 'm1'); g.px(41, 208, 'm1'); g.V(47, 206, 6, 'w3'); g.R(46, 211, 3, 2, 'm2');
  for (const x of [53, 60]) { g.V(x, 206, 5, 'm2'); g.V(x + 3, 206, 5, 'm2'); g.H(x + 1, 210, 2, 'm2'); }
  vkey(g, 69, 206, 'm2', 7);
  // hearth with a white-hot column between the hood and the coals
  ob(g, 11, 214, 21, 22, 'd1'); for (let j = 217; j < 236; j += 4) { g.H(11, j, 21, 'd0'); for (let i = 13 + ((j >> 2) & 1) * 4; i < 32; i += 8) g.V(i, j - 3, 3, 'd0'); } g.R(10, 212, 23, 2, 'n3'); g.H(10, 212, 23, 'n4');
  g.PG([[12, 198], [31, 198], [27, 203], [16, 203]], 'm1', 'ink'); g.H(14, 199, 16, 'm2');
  g.R(14, 209, 15, 3, 'd4'); for (let i = 0; i < 7; i++) g.px(14 + i * 2 + ((t + i) % 2), 209 + ((i * 3 + t) % 3), (i + t) % 3 ? 'am' : 'wh');
  g.V(21, 203, 6, 'wh'); g.V(22, 203, 6, 'd8'); g.px(21 + (t % 2), 205 + (t % 3), 'wh'); g.px(19, 205, 'm3'); g.px(24, 205, 'm3'); g.px(19, 207, 'm3'); g.px(24, 207, 'm3');
  // anvil, with work on it
  ob(g, 41, 228, 9, 8, 'w2'); g.V(41, 228, 8, 'w3'); g.H(41, 231, 9, 'w1');
  g.S(37, 221, ['..KKKKKKKKKKKK.', 'KKmmmmmmmmmmmK.', '.KKKMMMMMMMMK..', '....KMMMMMK....', '....KMMMMMK....', '...KMMMMMMMK...', '..KKKKKKKKKKK..'], { K: 'ink', m: 'm3', M: 'm1' });
  g.R(42, 220, 8, 2, 'am'); g.H(43, 220, 5, 'wh'); g.px(49, 219, 'am');
  const sp = [[40, 216], [52, 214], [46, 212], [55, 219], [37, 219], [50, 210]]; sp.forEach(([x, y], i) => { if ((t + i * 2) % 4 < 2) g.px(x + (t % 2), y - ((t + i) % 3), i % 2 ? 'am' : 'wh'); });
  // the smith, hammer up: dark iron, for the forge is next door and the house's own staff are brass
  g.R(44, 199, 9, 5, 'ink'); g.R(45, 200, 7, 3, 'm1'); g.H(45, 200, 7, 'm3'); g.L(52, 209, 50, 204, 'w3'); g.L(53, 209, 51, 204, 'w2');
  figure(g, 60, 236, { metal: DARK, flip: true, arm: 'up', apron: 'w2', gog: true });
  ob(g, 67, 224, 8, 12, 'w2'); g.H(67, 227, 8, 'm1'); g.H(67, 233, 8, 'm1'); g.H(68, 224, 6, 't4'); g.V(67, 224, 12, 'w3');
  // light inside the shed
  g.clip(8, 198, 70, 38); g.shade(8, 198, 70, 38, (i, j) => 0.1 + Math.max(0, (i - 40) / 38) * 0.16, 6, true); g.glow(22, 208, 60, 40, 'd5', 0.8, 6, true); g.glow(22, 205, 12, 10, 'd8', 0.5, 6, true); g.unclip();
  // posts and roof
  ob(g, 6, 200, 3, 36, 'w2'); g.V(6, 200, 36, 'w3'); ob(g, 76, 196, 3, 40, 'w2'); g.V(76, 196, 40, 'w3');
  g.PG([[2, 195], [82, 188], [82, 195], [2, 202]], 'm1', 'ink'); for (let x = 6; x < 80; x += 5) { const yt = Math.round(195 - (x - 2) * 7 / 80); g.V(x, yt + 1, 5, 'm0'); } g.L(3, 196, 81, 189, 'm2');
  // sign on the roof, with an iron horseshoe hung from it: the forge carries none of the house's neon
  g.V(30, 185, 9, 'w2'); g.V(62, 185, 6, 'w2'); ob(g, 23, 171, 46, 14, 'r1'); g.B(24, 172, 44, 12, 'r0'); g.T(46, 173, 'FORGE', 'p2', 'j10', { align: 'c', sh: 'r0' });
  g.H(69, 175, 4, 'ink'); g.H(69, 176, 4, 'n3');
  const shoe = new Fb(11, 11).S(1, 1, ['##.....##', '##.....##', '##.....##', '##.....##', '##.....##', '##.....##', '###...###', '.#######.', '..#####..'], { '#': 'n2' });
  shoe.V(1, 1, 6, 'n3'); shoe.V(8, 1, 6, 'n3'); shoe.px(5, 8, 'd8');
  g.blit(shoe.outlined('ink'), 71, 170);
}

function strongbox(g, x, y, w, h) {
  ob(g, x, y, w, h, 'm1'); g.H(x, y, w, 'm2'); g.V(x + 2, y, h, 'm0'); g.V(x + w - 3, y, h, 'm0'); g.H(x, y + h - 1, w, 'm0');
  const cx = x + (w >> 1); g.R(cx - 1, y + 2, 3, 3, 'r2'); g.px(cx, y + 3, 'r1'); g.px(cx - 1, y + 5, 'r1'); g.px(cx + 1, y + 5, 'r1'); g.px(cx - 1, y + 2, 'd4');
}
function stageStop(g, t) {
  ob(g, 379, 227, 54, 8, 'w2'); g.H(379, 227, 54, 'w4'); g.H(379, 228, 54, 'w3'); for (let x = 384; x < 432; x += 9) g.V(x, 229, 6, 'w1');
  for (const x of [382, 427]) { ob(g, x, 190, 2, 37, 'w2'); g.V(x, 190, 37, 'w3'); }
  ob(g, 377, 185, 58, 4, 'w1'); g.H(377, 185, 58, 'w3');
  for (let x = 377; x < 435; x += 4) { g.R(x, 190, 2, 2, 'r1'); g.R(x + 2, 190, 2, 1, 'p1'); }
  g.V(388, 181, 4, 'w2'); g.V(422, 181, 4, 'w2'); ob(g, 383, 168, 46, 13, 't2'); g.B(384, 169, 44, 11, 't1'); g.T(406, 170, 'STAGE', 'p2', 'j10', { align: 'c', sh: 't1' });
  g.V(416, 190, 3, 'ink'); ob(g, 415, 193, 3, 4, 'am'); g.px(416, 194, 'wh'); g.H(414, 192, 5, 'n3');
  g.clip(378, 190, 56, 37); g.glow(416, 196, 20, 18, 'am', 0.5, 6, true); g.unclip();
  strongbox(g, 396, 215, 14, 12); strongbox(g, 412, 218, 12, 9); strongbox(g, 400, 206, 12, 8);
  plate(g, 393, 197, 'B2', 't1', 'cy', 'm5', { h: 9, pad: 3 });
}
function gate(g, t) {
  // telegraph pole and the line back to the inn
  for (let x = 374; x <= 472; x++) { const u = (x - 374) / 98; g.px(x, Math.round(66 + u * 86 + Math.sin(u * Math.PI) * 7), 'n1'); g.px(x, Math.round(70 + u * 86 + Math.sin(u * Math.PI) * 8), 'n1'); }
  ob(g, 476, 148, 2, 88, 'w1'); g.V(476, 148, 88, 'w2'); ob(g, 471, 152, 9, 2, 'w1'); g.px(472, 150, 'm3'); g.px(478, 150, 'm3'); g.px(472, 155, 'm3');
  for (const x of [443, 469]) { ob(g, x, 178, 3, 58, 'w2'); g.V(x, 178, 58, 'w3'); g.V(x + 2, 178, 58, 'w1'); }
  ob(g, 439, 178, 37, 4, 'w2'); g.H(439, 178, 37, 'w4'); g.H(439, 181, 37, 'w1');
  g.V(449, 182, 3, 'ink'); g.V(465, 182, 3, 'ink'); ob(g, 444, 185, 27, 16, 'w1'); g.H(444, 185, 27, 'w2'); g.T(458, 187, 'PUBLIC', 'p1', 'm5', { align: 'c' }); g.T(458, 194, 'ROAD', 'p1', 'm5', { align: 'c' });
  g.V(455, 174, 4, 'ink'); g.V(460, 174, 4, 'ink'); plate(g, 458, 166, 'B3', 't1', 'cy', 'm5', { h: 9, pad: 3 });
  plate(g, 437, 203, 'B1', 't1', 'cy', 'm5', { h: 9, pad: 3 }); g.px(442, 207, 'ink');
  // a neon wire marks the lot's edge
  g.H(472, 212, 8, 'cy'); g.H(433, 212, 10, 'cy'); g.V(436, 213, 14, 'w2'); g.H(472, 220, 8, 'w2'); g.H(433, 220, 10, 'w2');
  halo(g, 430, 210, 50, 5, ['cy'], 'cy', [0.25], 8);
}

function horse(g, x, y, metal, o) { // nose at x, hooves on y, facing left
  o = o || {}; const [mb, md, ml, mk] = metal, L = new Fb(40, 30);
  L.R(15, 16, 2, 10, md); L.R(15, 26, 3, 2, 'n1'); L.R(30, 15, 2, 11, md); L.R(30, 26, 3, 2, 'n1');
  L.L(32, 10, 35, 13, mk); L.L(35, 13, 35, 19, mk); L.px(35, 20, o.tip || 'cy'); L.L(33, 10, 36, 13, md);
  L.OV(9, 8, 24, 9, mb); L.H(12, 8, 18, ml); L.H(12, 16, 18, md); L.H(10, 15, 22, md);
  L.PG([[9, 10], [15, 9], [12, 2], [7, 3]], mb); L.PG([[1, 5], [7, 2], [10, 4], [9, 7], [3, 9], [1, 8]], mb); L.H(3, 3, 5, ml);
  L.px(6, 4, o.eye || 'cy'); L.px(2, 7, mk); L.px(9, 1, mb); L.px(10, 0, mb); L.L(11, 2, 15, 8, mk); L.px(11, 3, 'pk0'); L.px(12, 5, 'pk0'); L.px(14, 7, 'pk0');
  L.R(12, 16, 2, 10, mb); L.px(12, 21, mk); L.px(13, 21, mk); L.R(11, 26, 4, 2, 'n1'); L.R(27, 16, 2, 10, mb); L.px(27, 21, mk); L.px(28, 21, mk); L.R(26, 26, 4, 2, 'n1');
  L.V(22, 10, 5, md); L.px(19, 12, mk); L.px(25, 12, mk);
  if (o.blanket) { L.R(17, 7, 9, 5, 'r1'); L.H(17, 9, 9, 'p1'); L.H(17, 11, 9, 'am'); L.R(18, 5, 7, 2, 'w2'); L.H(18, 5, 7, 'w3'); }
  L.V(14, 7, 9, 'w1'); L.px(4, 6, 'w1'); L.px(5, 7, 'w1'); L.px(6, 8, 'w1');
  g.blit(L.outlined('ink'), x, y - 28);
}
function wheel(g, cx, cy, r, rot) {
  const d = 2 * r + 1; g.OV(cx - r - 1, cy - r - 1, d + 2, d + 2, 0, 'ink'); g.OV(cx - r, cy - r, d, d, 0, 'w3'); g.OV(cx - r + 1, cy - r + 1, d - 2, d - 2, 0, 'w2');
  for (let k = 0; k < 8; k++) { const a = rot + k * Math.PI / 4; g.L(cx, cy, Math.round(cx + Math.cos(a) * (r - 2)), Math.round(cy + Math.sin(a) * (r - 2)), 'w2'); }
  g.R(cx - 1, cy - 1, 3, 3, 'am'); g.px(cx, cy, 'ink');
}
function hitch(g, t) {
  ob(g, 90, 242, 3, 38, 'w2'); g.V(90, 242, 38, 'w3'); ob(g, 164, 242, 3, 38, 'w2'); g.V(164, 242, 38, 'w3');
  ob(g, 88, 244, 81, 3, 'w3'); g.H(88, 244, 81, 'w4'); g.H(88, 246, 81, 'w2');
  for (const x of [104, 128, 152]) { g.V(x, 247, 4, 'ink'); g.V(x + 1, 247, 3, 'm3'); g.px(x + 2, 250, 'm3'); g.px(x + 3, 249, 'm3'); g.px(x + 1, 250, 'ink'); g.px(x + 2, 251, 'ink'); g.px(x + 3, 250, 'ink'); g.px(x + 4, 249, 'ink'); }
  ob(g, 93, 255, 73, 23, 'w1'); g.H(93, 255, 73, 'w2'); g.px(94, 256, 'am'); g.px(164, 256, 'am'); g.px(94, 276, 'am'); g.px(164, 276, 'am');
  g.T(97, 257, 'SessionStart', 'p1', 't5'); g.T(97, 264, 'UserPromptSubmit', 'p1', 't5'); g.T(97, 271, 'PermissionRequest', 'p1', 't5');
}
function rig(g, t) {
  horse(g, 182, 286, DARK);
  g.L(184, 265, 168, 245, 'w0');
  horse(g, 171, 291, DARK, { blanket: true });
  g.L(173, 271, 167, 247, 'w0'); g.H(154, 243, 14, 'w0'); g.px(153, 244, 'w0');
  g.L(205, 280, 214, 275, 'w2'); g.L(205, 281, 214, 276, 'ink');
  // wagon
  ob(g, 213, 264, 68, 12, 'w2'); g.H(213, 264, 68, 'w4'); g.H(213, 275, 68, 'w1'); for (const x of [262, 272]) g.V(x, 265, 10, 'w1');
  g.R(216, 267, 44, 7, 'w1'); g.T(218, 268, 'TRANSCRIPTS', 'p1', 'm5');
  ob(g, 214, 258, 9, 2, 'w3'); g.R(213, 252, 2, 6, 'ink'); g.V(214, 253, 5, 'w2');
  ob(g, 226, 252, 20, 12, 'w3'); g.H(226, 252, 20, 'w4'); g.H(226, 256, 20, 'w1'); g.V(230, 252, 12, 'm1'); g.V(241, 252, 12, 'm1'); g.R(235, 255, 2, 3, 'am'); g.R(231, 250, 7, 2, 'p2'); g.px(238, 251, 'p1'); g.R(240, 249, 4, 3, 'p1');
  // the baler: loose pages in the hopper, a bale out the back
  g.PG([[249, 247], [269, 247], [272, 237], [246, 237]], 't2', 'ink'); g.H(247, 238, 25, 't3');
  g.R(249, 234, 6, 3, 'p2'); g.R(257, 233, 6, 3, 'p1'); g.R(263, 235, 6, 2, 'p2'); g.H(250, 235, 4, 'n4'); const py = 222 + (t % 12); g.R(254 + (t % 3), py, 4, 2, 'p2'); g.px(255 + (t % 3), py, 'n4');
  ob(g, 249, 247, 28, 17, 't3'); g.H(249, 247, 28, 't4'); g.V(249, 247, 17, 't4'); g.R(251, 258, 24, 4, 't2'); g.px(252, 250, 'am'); g.px(254, 250, 'r2');
  g.OV(262, 248, 13, 13, 'm1', 'ink'); for (let k = 0; k < 4; k++) { const a = t / FRAMES * Math.PI * 2 + k * Math.PI / 2; g.L(268, 254, Math.round(268 + Math.cos(a) * 4), Math.round(254 + Math.sin(a) * 4), 'm3'); } g.px(268, 254, 'am');
  ob(g, 277, 255, 5, 7, 'm1'); g.H(277, 255, 5, 'm2');
  const drop = t < 8 ? 0 : t < 10 ? 1 : t < 11 ? 2 : t < 12 ? 4 : t < 13 ? 7 : 10; ob(g, 283, 256 + drop, 9, 6, 'p1'); g.H(283, 256 + drop, 9, 'p2'); g.V(286, 256 + drop, 6, 'r1');
  wheel(g, 225, 282, 8, 0.2); wheel(g, 268, 281, 9, 0.5);
  // bales on the ground, the near one stencilled
  ob(g, 294, 271, 24, 9, 'p0'); g.H(294, 271, 24, 'p1'); g.V(300, 271, 9, 'r1'); g.V(311, 271, 9, 'r1');
  ob(g, 287, 281, 37, 11, 'p1'); g.H(287, 281, 37, 'p2'); g.R(287, 290, 37, 2, 'p0'); g.V(289, 281, 11, 'r1'); g.V(321, 281, 11, 'r1'); g.T(305, 284, 'SUMMARY', 'r1', 'm5', { align: 'c' });
}
function pen(g, t) {
  const post = (x, y, h) => { ob(g, x, y, 3, h, 'm1'); g.V(x, y, h, 'm2'); g.R(x, y - 2, 3, 2, 'pk'); g.px(x + 1, y - 3, 'pkw'); };
  const dim = (t === 3 || t === 17);
  g.H(17, 270, 60, 'pk0'); g.H(17, 278, 60, 'pk0'); g.L(8, 300, 15, 270, 'pk0'); g.L(8, 310, 15, 278, 'pk0'); g.L(84, 300, 77, 270, 'pk0'); g.L(84, 310, 77, 278, 'pk0');
  post(14, 264, 22); post(76, 264, 22);
  ob(g, 22, 284, 18, 14, 'w2'); g.H(22, 284, 18, 'w3'); g.V(31, 285, 13, 'w1'); for (let i = 22; i < 40; i++) for (let j = 289; j < 293; j++) g.px(i, j, ((i + j) >> 1) & 1 ? 'am' : 'n1');
  ob(g, 44, 288, 14, 10, 'w1'); g.H(44, 288, 14, 'w2'); g.L(44, 288, 57, 297, 'w3'); g.L(57, 288, 44, 297, 'w3');
  ob(g, 62, 277, 8, 9, 'm1'); g.H(62, 279, 8, 'm0'); g.H(62, 283, 8, 'm0'); g.V(62, 277, 9, 'm2');
  g.R(60, 296, 11, 2, 'ink'); g.R(60, 290, 11, 2, 'r1'); g.R(61, 292, 10, 2, 't3'); g.R(60, 294, 11, 2, 'p1'); g.V(70, 290, 6, 'p2'); g.R(59, 289, 13, 1, 'ink');
  g.H(9, 300, 75, dim ? 'pk0' : 'pk'); g.H(9, 310, 75, 'pk');
  post(6, 292, 30); post(83, 292, 30);
  plate(g, 46, 296, 'QUARANTINE', 'r1', 'p2', 'm5', { h: 9, pad: 3 });
}
function stake(g) {
  const pegs = [[330, 272], [354, 270], [357, 290], [326, 292]];
  for (let i = 0; i < 4; i++) { const a = pegs[i], b = pegs[(i + 1) % 4]; g.L(a[0], a[1] - 3, b[0], b[1] - 3, 'p0'); }
  for (const [x, y] of pegs) { g.R(x - 1, y - 6, 3, 7, 'ink'); g.V(x, y - 5, 5, 'w3'); g.px(x, y - 5, 'w4'); }
  g.fn(334, 275, 19, 12, (i, j) => ((i * 7 + j * 13) % 11 < 3) ? 'd2' : 0); g.H(338, 281, 9, 'd2'); g.H(340, 284, 8, 'ink'); g.H(341, 285, 6, 'd0');
  g.L(345, 277, 352, 285, 'w3'); g.L(342, 280, 348, 274, 'm2'); g.px(341, 281, 'm3'); g.px(349, 273, 'm3');
  ob(g, 337, 254, 2, 36, 'w3'); g.V(337, 254, 36, 'w4');
  g.PG([[340, 254], [353, 258], [340, 262]], 'r2', 'ink'); g.L(340, 258, 349, 258, 'p2'); g.px(340, 256, 'r2'); g.px(340, 260, 'r2');
  ob(g, 329, 266, 9, 10, 'p2'); g.H(331, 268, 5, 'r1'); g.H(331, 270, 5, 'n3'); g.H(331, 272, 4, 'n3'); g.px(333, 266, 'am');
}
function saguaro(g, x, y, h) {
  g.R(x - 3, y - h, 6, h, 'ink'); g.R(x - 2, y - h - 1, 4, 1, 'ink'); g.R(x - 2, y - h, 4, h, 'g1'); g.V(x - 2, y - h, h, 'g0'); g.V(x + 1, y - h + 1, h - 1, 'g2');
  g.R(x - 8, y - h + 10, 5, 11, 'ink'); g.R(x - 8, y - h + 17, 7, 5, 'ink'); g.R(x - 7, y - h + 11, 3, 10, 'g1'); g.V(x - 7, y - h + 11, 10, 'g0'); g.R(x - 7, y - h + 18, 6, 3, 'g1'); g.H(x - 7, y - h + 20, 6, 'g0');
  g.R(x + 4, y - h + 5, 5, 10, 'ink'); g.R(x + 2, y - h + 11, 7, 5, 'ink'); g.R(x + 5, y - h + 6, 3, 9, 'g1'); g.V(x + 7, y - h + 6, 9, 'g2'); g.R(x + 2, y - h + 12, 6, 3, 'g1');
  g.px(x - 1, y - h - 2, 'pk'); g.px(x, y - h - 2, 'pkw');
}
function bootHill(g, t) {
  g.PG([[362, 330], [372, 313], [390, 304], [420, 300], [452, 300], [472, 305], [480, 309], [480, 330]], 'd1');
  g.fn(362, 298, 118, 32, (i, j, c) => (c === IDX.d1 && g.get(i, j - 1) !== IDX.d1 && j < 316) ? 'd2' : (c === IDX.d1 && (i * 5 + j * 11) % 23 === 0) ? 'd0' : 0);
  // far boards and a cross
  ob(g, 449, 284, 17, 17, 'w2'); g.H(449, 284, 17, 'w3'); g.px(448, 283, 'd3'); g.T(458, 287, 'B1', 'ink', 'm5', { align: 'c' }); g.H(452, 294, 11, 'w1'); g.H(453, 297, 9, 'w1');
  ob(g, 472, 287, 2, 15, 'w3'); ob(g, 468, 291, 10, 2, 'w3');
  // the near headboard names what was removed
  ob(g, 399, 287, 45, 28, 'w3'); g.px(398, 286, 'd1'); g.px(444, 286, 'd1'); g.px(399, 287, 'ink'); g.px(443, 287, 'ink'); g.H(400, 287, 43, 'w4'); g.R(399, 312, 45, 3, 'w2'); g.px(421, 288, 'w2'); g.px(421, 289, 'w2'); g.px(421, 310, 'w2'); g.px(421, 311, 'w2');
  g.T(421, 290, 'PURGED', 'w1', 'm5', { align: 'c' }); g.T(422, 297, 'A2', 'ink', 's5b', { align: 'c' }); g.T(421, 305, '4812-4830', 'ink', 'm5', { align: 'c' });
  g.R(404, 315, 3, 5, 'w2'); g.R(436, 315, 3, 5, 'w2'); g.V(403, 315, 5, 'ink'); g.V(407, 315, 5, 'ink'); g.V(435, 315, 5, 'ink'); g.V(439, 315, 5, 'ink');
  // a shovel left in the ground
  g.L(385, 296, 381, 311, 'w3'); g.L(386, 296, 382, 311, 'ink'); g.R(378, 311, 5, 5, 'm1'); g.H(378, 311, 5, 'm2'); g.px(385, 294, 'w4'); g.px(386, 294, 'w4'); g.px(384, 295, 'w4');
  for (const [x, y] of [[376, 318], [392, 322], [446, 314], [462, 318], [430, 324]]) { g.px(x, y, 'g1'); g.px(x + 1, y - 1, 'g1'); g.px(x + 2, y, 'g0'); }
}
function coach(g, t) {
  const x = 388, y = 246, fl = t % 2;
  // hover pods and their glow on the ground
  g.glow(x + 26, y + 26, 34, 6, 'cy', 0.5 + fl * 0.1, 6);
  ob(g, x + 9, y + 20, 11, 3, 'm1'); g.H(x + 9, y + 20, 11, 'm2'); ob(g, x + 32, y + 20, 11, 3, 'm1'); g.H(x + 32, y + 20, 11, 'm2');
  g.H(x + 10, y + 24, 9, fl ? 'cy' : 'cyw'); g.H(x + 33, y + 24, 9, fl ? 'cyw' : 'cy'); g.V(x + 14, y + 18, 2, 'm0'); g.V(x + 37, y + 18, 2, 'm0');
  // rocket at the back
  g.PG([[x + 47, y + 5], [x + 54, y + 3], [x + 54, y + 14], [x + 47, y + 12]], 'm1', 'ink'); g.V(x + 53, y + 4, 10, 'm2'); g.H(x + 48, y + 8, 5, 'm0');
  g.H(x + 55, y + 8, 2 + fl, 'cy'); g.px(x + 55, y + 7, 'cyw'); g.px(x + 55, y + 9, 'cyw'); g.px(x + 57 + fl, y + 8, 't4');
  // body
  ob(g, x + 6, y, 40, 18, 'r1'); g.H(x + 6, y, 40, 'r2'); g.H(x + 6, y + 14, 40, 'am'); g.R(x + 6, y + 15, 40, 3, 'r0'); g.px(x + 6, y + 17, 'ink'); g.px(x + 45, y + 17, 'ink');
  ob(g, x + 4, y - 3, 44, 2, 'w1'); g.H(x + 4, y - 3, 44, 'w3');
  g.H(x + 12, y - 7, 30, 'm2'); for (const dx of [12, 26, 41]) g.V(x + dx, y - 6, 3, 'm1');
  strongbox(g, x + 15, y - 10, 9, 6); strongbox(g, x + 29, y - 9, 9, 5);
  g.R(x + 9, y + 3, 8, 7, 'd8'); g.B(x + 9, y + 3, 8, 7, 'r0'); g.px(x + 10, y + 4, 'wh'); g.V(x + 15, y + 4, 5, 'r2');
  g.R(x + 20, y + 2, 10, 13, 'r2'); g.B(x + 20, y + 2, 10, 13, 'r0'); g.R(x + 22, y + 4, 6, 4, 'd8'); vkey(g, x + 25, y + 9, 'am', 5);
  g.R(x + 33, y + 3, 9, 7, 'd8'); g.B(x + 33, y + 3, 9, 7, 'r0'); g.px(x + 34, y + 4, 'wh'); g.V(x + 40, y + 4, 5, 'r2');
  // driver's box and driver
  ob(g, x - 4, y + 5, 10, 4, 'w2'); g.H(x - 4, y + 5, 10, 'w3'); g.R(x - 9, y + 13, 14, 2, 'ink'); g.H(x - 9, y + 13, 14, 'w3'); g.V(x - 9, y + 8, 5, 'w2'); g.V(x - 10, y + 7, 7, 'ink');
  ob(g, x + 3, y + 1, 2, 3, 'am'); g.px(x + 3, y + 1, 'wh');
  figure(g, x + 1, y + 14, { sit: true, flip: true, metal: DARK, hat: HAT.tan, band: 'r2', pon: PON.violet, arm: 'fwd' });
}
const SKULL = ['.K.......K.', 'KpK.....KpK', '.KpKKKKKpK.', '..KpppppK..', '..KpKpKpK..', '..KpppppK..', '...KpKpK...', '....KKK....'];
function litter(g, t) {
  g.S(108, 300, SKULL, { K: 'ink', p: 'p1' });
  // barrel cactus
  g.OV(126, 308, 9, 8, 'g1', 'ink'); g.V(128, 310, 5, 'g0'); g.V(131, 310, 5, 'g2'); g.px(130, 307, 'pk');
  // tumbleweed
  const tx = 150 + Math.round(2 - 2 * Math.cos(t / FRAMES * Math.PI * 2)), ty = 311 - ((t % 12) === 5 ? 1 : 0), ph = t % 4;
  g.OV(tx, ty, 12, 11, 0, 'w2'); g.OV(tx + 1, ty + 1, 10, 9, 0, 'w3');
  [[3, 2, 8, 8], [8, 3, 3, 8], [2, 5, 9, 4], [5, 1, 6, 9], [2, 7, 7, 2]].forEach(([a, b, c, d], i) => g.L(tx + a, ty + b, tx + c, ty + d, (i + ph) % 3 ? 'w3' : 'w4'));
  g.px(tx + 5, ty + 5, 'w1'); g.px(tx + 7, ty + 4, 'w1'); g.px(tx + 4, ty + 7, 'w1');
  // dry grass along the near edge
  const r = rng(99); for (let x = 3; x < 480; x += 6 + Math.floor(r() * 10)) { const h = 3 + Math.floor(r() * 5); [h - 2, h, h - 1, h + 1, h - 3].forEach((hh, i) => { if (hh > 0) g.V(x + i, 330 - hh, hh, i % 2 ? 'ink' : 'n2'); }); }
  // stones
  for (const [x, y] of [[20, 326], [70, 250], [200, 318], [262, 304], [344, 316], [176, 246], [300, 300]]) { g.R(x, y, 3, 2, 'd2'); g.px(x, y, 'd3'); g.H(x, y + 2, 3, 'ink'); }
}
// light that falls on the ground before anything is put on it
function groundLight(g) {
  g.light(86, 236, 296, 30, 'd5', (i, j) => { const fall = Math.max(0, 1 - (j - 236) / 26); return 0.44 * fall * fall; }, 6);
  g.glow(109, 240, 30, 16, 'am', 0.6, 6); g.glow(42, 240, 52, 16, 'd5', 0.55, 6); g.glow(405, 240, 44, 12, 'am', 0.4, 6);
  g.glow(45, 316, 60, 16, 'pk', 0.34, 6); g.glow(46, 286, 40, 12, 'pk', 0.2, 6);
}
module.exports = { forge, stageStop, gate, hitch, rig, pen, stake, saguaro, bootHill, coach, litter, groundLight, horse, strongbox };
