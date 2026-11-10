'use strict';
// The inn's open rooms: record, print shop, wire office and office upstairs, the saloon below, and the figures in them.
const { Fb, rng, IDX, ci, B2 } = require('./pix.js');
const { halo, boards, vboards, plate, lamp } = require('./kit.js');

const PON = { red: ['r1', 'r0', 'r2', 'p1'], teal: ['t3', 't2', 't4', 'am'], ochre: ['d5', 'd3', 'd6', 'r1'], violet: ['v2', 'v1', 'n4', 'cy'], bone: ['p1', 'p0', 'p2', 'r1'] };
const HAT = { brown: ['w2', 'w3'], grey: ['n3', 'n4'], black: ['n1', 'n2'], tan: ['w4', 'w5'] };
const STEEL = ['m2', 'm1', 'm3', 'm0'], BRASS = ['d5', 'd3', 'd6', 'w1'], DARK = ['m1', 'm0', 'm2', 'n1'];

// A robot, side on, facing right unless flipped. x is the middle of the body and y the floor it stands or sits over.
function figure(g, x, y, o) {
  o = Object.assign({ metal: STEEL, eye: 'cy', arm: 'rest' }, o || {});
  const L = new Fb(44, 42), [mb, md, ml, mk] = o.metal, sit = !!o.sit;
  if (o.cell) { L.R(11, 23, 3, 6, 'm1'); L.V(12, 24, 4, 'cy'); L.px(12, 23, 'am'); }
  if (sit) {
    L.R(25, 30, 2, 6, md); L.R(25, 36, 3, 2, 'w0');
    L.R(22, 27, 8, 2, mb); L.px(29, 27, ml); L.px(29, 28, mk);
    L.R(28, 29, 2, 7, mb); L.px(28, 32, mk); L.R(28, 36, 4, 2, 'w1'); L.px(31, 36, 'w2');
  } else {
    L.R(17, 29, 2, 7, md); L.R(16, 36, 4, 2, 'w0');
    L.R(20, 29, 2, 7, mb); L.px(20, 32, mk); L.px(21, 32, mk); L.R(20, 36, 4, 2, 'w1'); L.px(23, 36, 'w2');
  }
  const rowsW = [8, 8, 10, 10, 10, 12, 12, 12, 14, 14, 14];
  if (o.pon) {
    const [pb, pd, pl, ps] = o.pon;
    for (let k = 0; k < 11; k++) { const w = rowsW[k], x0 = 20 - w / 2, j = 18 + k; L.R(x0, j, w, 1, pb); L.R(x0, j, 2, 1, pd); L.px(x0 + w - 1, j, pl); if (k === 6 || k === 8) L.R(x0, j, w, 1, ps); if (k === 10) for (let i = x0; i < x0 + w; i += 2) L.px(i, j, ps); }
    L.px(19, 18, pd); L.px(20, 18, pd);
  } else {
    L.R(16, 18, 8, 11, mb); L.R(16, 18, 2, 11, md); L.V(23, 18, 11, ml); L.R(18, 20, 4, 4, ml); L.px(19, 21, o.eye); L.px(20, 22, mk); L.H(16, 27, 8, 'w1'); L.px(20, 27, 'am'); L.H(16, 28, 8, mk);
    if (o.apron) { L.R(17, 22, 7, 7, o.apron); L.px(18, 21, mk); L.px(22, 21, mk); }
    if (o.vest) { L.R(16, 18, 2, 9, o.vest); L.R(22, 18, 2, 9, o.vest); }
  }
  const A = o.arm;
  if (A === 'table') { L.R(24, 22, 5, 2, mb); L.R(29, 22, 2, 2, ml); }
  else if (A === 'fwd') { L.R(24, 21, 6, 2, mb); L.R(30, 21, 2, 2, ml); }
  else if (A === 'crank') { L.L(24, 23, 29, 19, mb); L.L(24, 24, 29, 20, mb); L.R(29, 17, 2, 2, ml); }
  else if (A === 'up') { L.L(23, 20, 27, 13, mb); L.L(24, 20, 28, 13, mb); L.R(27, 11, 2, 2, ml); }
  else if (A === 'type') { L.R(24, 24, 5, 2, mb); L.R(29, 25, 2, 2, ml); }
  else if (o.pon) { L.R(25, 29, 2, 2, ml); }
  else { L.R(24, 20, 2, 7, mb); L.R(24, 27, 2, 2, ml); }
  L.R(19, 17, 2, 1, mk);
  L.R(17, 12, 6, 5, mb); L.H(17, 12, 6, ml); L.V(17, 13, 4, md); L.px(19, 14, mk); L.R(20, 14, 3, 1, o.eye); L.px(21, 16, mk); L.px(22, 16, mk);
  if (o.hat) { const [hb, hl] = o.hat, band = o.band || 'am'; L.R(17, 7, 6, 1, hl); L.R(16, 8, 8, 2, hb); L.px(16, 8, hl); L.H(16, 10, 8, band); L.R(12, 11, 16, 1, hb); L.R(21, 11, 6, 1, hl); L.px(12, 10, hb); L.px(27, 10, hb); }
  if (o.shade) { L.H(18, 11, 4, ml); L.H(17, 12, 6, 't3'); L.R(23, 12, 4, 1, 't4'); }
  if (o.gog) { L.R(18, 13, 5, 1, 'n1'); L.px(21, 13, 'am'); L.px(22, 13, 'am'); }
  const O = L.outlined('ink');
  if (o.flip) g.blit(O, x - 23, y - 38, { flip: true }); else g.blit(O, x - 20, y - 38);
}
function chair(g, x, y, o) { // side on; x is the sitter's middle
  o = o || {}; const f = o.flip ? -1 : 1, seat = y - 9; const X = d => x + f * d, lo = (a, b) => Math.min(X(a), X(b));
  g.R(lo(-7, -6), seat - 12, 2, 21, 'w2'); g.V(X(-7), seat - 12, 21, 'w1'); g.H(lo(-7, -6), seat - 13, 2, 'w3');
  g.R(lo(-6, 3), seat, 10, 2, 'w3'); g.H(lo(-6, 3), seat + 1, 10, 'w1'); g.V(X(3), seat + 2, 7, 'w2'); g.H(lo(-6, 3), seat + 5, 10, 'w1');
  if (o.no) { const p0 = lo(-16, -10); g.R(p0, seat - 14, 7, 9, 'p2'); g.B(p0, seat - 14, 7, 9, 'ink'); g.T(p0 + 2, seat - 12, String(o.no), 'ink', 'm5'); g.px(o.flip ? p0 - 1 : p0 + 7, seat - 11, 'ink'); }
}
function chairFront(g, x, y, no) {
  const seat = y - 9; g.R(x - 4, seat - 18, 9, 18, 'w2'); g.B(x - 4, seat - 18, 9, 18, 'ink'); g.H(x - 3, seat - 17, 7, 'w3'); g.V(x - 4, seat, 9, 'w1'); g.V(x + 4, seat, 9, 'w1'); g.R(x - 4, seat, 9, 2, 'w3');
  g.R(x - 3, seat - 17, 7, 9, 'p2'); g.B(x - 3, seat - 17, 7, 9, 'ink'); g.T(x - 1, seat - 15, String(no), 'ink', 'm5');
}
function cardTable(g, x, y) {
  const top = y - 14; g.R(x - 19, top, 39, 2, 'w3'); g.H(x - 17, top, 35, 't3'); g.px(x - 19, top, 'w4'); g.px(x - 18, top, 'w4'); g.px(x + 18, top, 'w4'); g.px(x + 19, top, 'w4'); g.H(x - 18, top + 2, 37, 'ink');
  g.R(x - 1, top + 3, 3, 9, 'w2'); g.V(x - 1, top + 3, 9, 'w1'); g.R(x - 6, y - 2, 13, 2, 'w2'); g.H(x - 6, y - 2, 13, 'w3');
}
function spike(g, x, top, n) { g.V(x, top - 10, 10, 'm3'); for (let k = 0; k < n; k++) { g.H(x - 2 + (k % 2), top - 2 - k, 4 + ((k * 3) % 2), k % 2 ? 'p1' : 'p2'); } }
function vkey(g, x, y, c, len) { if (len >= 6) { g.B(x - 1, y, 3, 3, c); g.V(x, y + 3, len - 3, c); g.px(x + 1, y + len - 1, c); g.px(x + 1, y + len - 3, c); } else { g.px(x, y, c); g.px(x - 1, y, c); g.V(x, y + 1, len - 1, c); g.px(x + 1, y + len - 1, c); } }
function paper(g, x, y, w, h, lit, kind) {
  g.R(x, y, w, h, lit ? 'p2' : 'n4'); g.B(x, y, w, h, lit ? 'p0' : 'n3'); g.H(x + 2, y + 2, w - 4, kind); g.H(x + 2, y + 4, w - 5, lit ? 'n3' : 'n3'); if (h > 7) g.H(x + 2, y + 6, w - 4, lit ? 'n3' : 'n3'); g.px(x + (w >> 1), y, lit ? 'r2' : 'n5');
}
function bottle(g, x, y, c, h) { g.R(x, y - h + 2, 2, h - 2, c); g.px(x, y - h, 'p1'); g.px(x, y - h + 1, c); g.px(x + 1, y - h + 3, 'p2'); }
const DK = { r1: 'r0', t3: 't2', p1: 'p0', w3: 'w2', d3: 'd2', n3: 'n2', t2: 't1', r0: 'w0', p0: 'w3', d2: 'd1', w2: 'w1', n4: 'n3', d4: 'd3' };

function recordRoom(g, t) {
  const x0 = 131, x1 = 199, y0 = 112, y1 = 162; g.R(x0, y0, x1 - x0, y1 - y0, 'w0');
  const r = rng(77), ups = [132, 153, 174, 195], names = ['A1', 'A2', 'B1'], cols = ['r1', 't3', 'p1', 'w3', 'd3', 'n3', 't2', 'r0', 'p0', 'd2', 'w2', 'n4', 'd4'], late = [];
  for (let b = 0; b < 3; b++) {
    const bx = ups[b] + 2, bw = ups[b + 1] - ups[b] - 2; g.R(bx, y0 + 9, bw, y1 - y0 - 9, 'n1');
    for (let s = 0; s < 5; s++) {
      const sy = 129 + s * 8; g.H(bx, sy, bw, 'w3'); g.H(bx, sy + 1, bw, 'w1');
      const end = (b === 2 && s === 3) ? bx + 8 : (b === 2 && s === 4) ? bx : bx + bw; let x = bx;
      while (x < end) { const k = r(), w = k < 0.22 ? 1 : k < 0.86 ? 2 : 3; if (x + w > end) break; if (r() < 0.06) { x += 1; continue; } const hh = 5 + Math.floor(r() * 2.3), c = cols[Math.floor(r() * cols.length)];
        g.R(x, sy - hh, w, hh, c); if (w > 1) g.V(x + w - 1, sy - hh, hh, DK[c] || 'n2'); if (r() < 0.45) g.px(x, sy - 3, r() < 0.5 ? 'am' : 'p2');
        if (r() < 0.15 && hh < 7) { const xx = x, yy = sy - hh - 2; late.push(() => { g.R(xx, yy + 1, 2, 3, 'cy'); g.px(xx, yy + 1, 'cyw'); }); }
        x += w; }
      if (b === 2 && s === 3) { g.R(bx + 9, sy - 2, 6, 2, 'p1'); g.H(bx + 9, sy - 2, 6, 'r1'); }
    }
    const pc = bx + (bw >> 1) + 1; late.push(() => plate(g, pc, y0 + 1, names[b], 'p2', 'ink', 'm5', { h: 8, pad: 2 }));
  }
  for (const u of ups) { g.R(u, y0, 2, y1 - y0, 'w2'); g.V(u, y0, y1 - y0, 'w3'); }
  g.L(181, 161, 187, 123, 'w4'); g.L(186, 161, 192, 123, 'w4'); for (let k = 0; k < 7; k++) { const yy = 157 - k * 5, xx = 181 + Math.round((161 - yy) * 6 / 38); g.H(xx + 1, yy, 4, 'w3'); }
  g.clip(x0, y0, x1 - x0, y1 - y0); g.glow(154, 120, 44, 38, 'am', 0.5, 6, true); g.unclip();
  late.forEach(f => f());
  halo(g, x0, y0, x1 - x0, y1 - y0, ['cy'], 'cy', [0.3], 8);
  lamp(g, 154, y0, t, { drop: 2 });
}
function printShop(g, t) {
  const x0 = 200, x1 = 260, y0 = 112, y1 = 162; g.R(x0, y0, x1 - x0, y1 - y0, 'p0');
  g.R(x0, 150, x1 - x0, 12, 'w2'); g.H(x0, 150, x1 - x0, 'w4'); g.H(x0, 151, x1 - x0, 'w1'); for (let x = x0 + 3; x < x1; x += 6) g.V(x, 152, 10, 'w1');
  g.clip(x0, y0, x1 - x0, y1 - y0); g.glow(243, 120, 52, 44, 'd8', 0.55, 6, true); g.shade(x0, 150, x1 - x0, 12, 0.14, 6, true); g.unclip();
  // handbills drying on a line
  const sag = x => 116 + Math.round(Math.sin((x - x0) / (x1 - x0) * Math.PI) * 2);
  for (let x = x0; x < x1; x++) g.px(x, sag(x), 'ink');
  for (const bx of [203, 212, 221, 250]) { const by = sag(bx + 3) + 1; g.R(bx, by, 7, 9, 'p2'); g.B(bx, by, 7, 9, 'w3'); g.H(bx + 1, by + 1, 5, 'r1'); g.H(bx + 1, by + 3, 5, 'n4'); g.H(bx + 1, by + 5, 4, 'n4'); g.H(bx + 1, by + 7, 5, 'n4'); g.px(bx + 3, by - 1, 'w2'); }
  // hand press
  g.R(201, 157, 15, 5, 'w1'); g.H(201, 157, 15, 'w3'); g.B(201, 157, 15, 5, 'ink');
  g.R(202, 132, 2, 25, 'm1'); g.R(212, 132, 2, 25, 'm1'); g.V(202, 132, 25, 'm2'); g.V(212, 132, 25, 'm2');
  g.R(201, 129, 14, 4, 'm0'); g.H(201, 129, 14, 'm2'); g.B(201, 129, 14, 4, 'ink'); g.R(207, 133, 2, 8, 'm3'); g.R(204, 141, 8, 2, 'm1'); g.H(204, 141, 8, 'm2');
  g.R(203, 147, 10, 3, 'w3'); g.H(203, 147, 10, 'w4'); g.H(204, 146, 8, 'p2'); g.R(203, 150, 10, 1, 'm0'); g.V(204, 151, 6, 'm1'); g.V(211, 151, 6, 'm1');
  g.L(208, 135, 215, 132, 'w4'); g.px(216, 132, 'am'); g.px(216, 131, 'am');
  // windlass over the hatch, lowering the handbill
  g.R(231, 162, 11, 6, 'ink'); g.H(232, 162, 9, 'n2');
  g.R(230, 137, 2, 25, 'w2'); g.V(230, 137, 25, 'w3'); g.R(241, 137, 2, 25, 'w2'); g.V(241, 137, 25, 'w3'); g.R(228, 134, 16, 3, 'w3'); g.H(228, 134, 16, 'w4'); g.B(228, 134, 16, 3, 'ink');
  g.R(232, 141, 9, 6, 'w3'); g.B(232, 141, 9, 6, 'ink'); for (const xx of [234, 236, 238]) g.V(xx, 142, 4, 'p1');
  g.px(229, 143, 'm3'); g.px(228, 143, 'm3'); g.px(227, 142, 'm3'); g.px(227, 141, 'am'); g.px(226, 141, 'am');
  g.V(236, 147, 21, 'p1');
  // type case
  for (let i = 0; i < 12; i++) { const tp = 147 - Math.round(i * 7 / 12); g.V(247 + i, tp, 6, i % 3 === 0 ? 'w1' : 'w3'); g.px(247 + i, tp, 'w4'); if (i % 3) { g.px(247 + i, tp + 2, 'm2'); g.px(247 + i, tp + 4, 'm1'); } }
  g.V(247, 153, 9, 'w2'); g.V(258, 146, 16, 'w2'); g.H(247, 157, 12, 'w1');
  figure(g, 217, y1, { metal: BRASS, arm: 'crank', shade: true, vest: 't2' });
  lamp(g, 243, y0, t, { drop: 3 });
}
function wireOffice(g, t) {
  const x0 = 262, x1 = 314, y0 = 112, y1 = 162; vboards(g, x0, y0, x1 - x0, y1 - y0, 't2', 't1', 5, 2);
  g.R(x0, 153, x1 - x0, 9, 't1'); g.H(x0, 153, x1 - x0, 't3');
  g.clip(x0, y0, x1 - x0, y1 - y0); g.glow(306, 122, 50, 46, 'am', 0.5, 6, true); g.unclip();
  plate(g, 284, 114, 'UNTRUSTED', 'r1', 'p2', 'm5', { h: 9, pad: 3 });
  const px0 = 265, py0 = 127; g.R(px0 - 1, py0 - 1, 25, 19, 'w3'); g.B(px0 - 1, py0 - 1, 25, 19, 'ink');
  for (let rr = 0; rr < 3; rr++) for (let c = 0; c < 4; c++) { const cx = px0 + c * 6, cy = py0 + rr * 6; g.R(cx, cy, 5, 5, 'w0'); if ((rr * 4 + c) % 3 !== 1) { g.R(cx + 1, cy + 3, 4, 2, 'p2'); g.px(cx + 1, cy + 3, 'r2'); g.px(cx + 1, cy + 4, 'r2'); if ((rr + c) % 2) g.H(cx, cy + 2, 4, 'p1'); } }
  // desk, sounder, envelopes with their red warning band, and the tube that drops them to the call box
  g.R(299, 151, 9, 11, 'm1'); g.B(299, 151, 9, 11, 'ink'); g.H(300, 152, 7, 'm2'); g.px(305, 155, t % 6 < 3 ? 'cy' : 't4'); g.R(301, 157, 5, 3, 'ink'); g.H(302, 158, 3, 'p2');
  g.R(288, 147, 25, 2, 'w3'); g.H(288, 147, 25, 'w4'); g.H(288, 149, 25, 'ink'); g.V(289, 150, 12, 'w2'); g.V(311, 150, 12, 'w2');
  g.R(301, 136, 11, 11, 'm1'); g.B(301, 136, 11, 11, 'ink'); g.R(303, 138, 7, 6, 't1'); g.H(304, 139, 4, 'cy'); g.H(304, 141, 5, 't4'); if (t % 4 < 2) g.px(304, 143, 'cy'); g.px(310, 145, 'r2');
  for (let k = 0; k < 3; k++) { g.R(291, 145 - k * 2, 8, 2, 'p2'); g.H(291, 146 - k * 2, 8, 'p1'); g.R(291, 145 - k * 2, 2, 2, 'r2'); }
  g.R(275, 153, 8, 2, 'w3'); g.V(276, 155, 7, 'w2'); g.V(281, 155, 7, 'w2'); g.H(276, 158, 6, 'w1');
  figure(g, 279, y1, { metal: BRASS, sit: true, arm: 'type', shade: true, vest: 't4' });
  lamp(g, 308, y0, t, { drop: 3 });
}
function office(g, t) {
  const x0 = 316, x1 = 368, y0 = 112, y1 = 162; g.R(x0, y0, x1 - x0, y1 - y0, 'd1');
  for (let j = y0 + 3; j < 150; j += 6) for (let i = x0 + (((j - y0) / 6) & 1 ? 2 : 5); i < x1; i += 6) g.px(i, j, 'd0');
  g.R(x0, 150, x1 - x0, 12, 'w1'); g.H(x0, 150, x1 - x0, 'w3'); for (let x = x0 + 2; x < x1; x += 7) g.V(x, 152, 10, 'w0');
  g.clip(x0, y0, x1 - x0, y1 - y0); g.glow(339, 146, 40, 38, 'am', 0.5, 6, true); g.unclip();
  // the key board under glass: principal key, device keys hung from it, seat keys hung from those
  const kx = 318, ky = 116; g.R(kx, ky, 27, 28, 'w3'); g.B(kx, ky, 27, 28, 'ink'); g.R(kx + 2, ky + 2, 23, 24, 'n1'); g.H(kx + 1, ky + 1, 25, 'w4');
  g.L(kx + 3, ky + 9, kx + 8, ky + 4, 'n2'); g.L(kx + 18, ky + 23, kx + 22, ky + 19, 'n2');
  vkey(g, kx + 13, ky + 3, 'am', 8);
  for (const dx of [5, 13, 21]) { if (dx !== 13) g.L(kx + 13, ky + 11, kx + dx, ky + 13, 'n4'); else g.px(kx + 13, ky + 12, 'n4'); vkey(g, kx + dx, ky + 13, 'd6', 6); for (const s of [-2, 2]) { g.px(kx + dx + s, ky + 19, 'n4'); vkey(g, kx + dx + s, ky + 20, 'm3', 4); } }
  // the safe
  const sx = 346, sy = 134; g.R(sx, sy, 21, 28, 'm1'); g.B(sx, sy, 21, 28, 'ink');
  g.R(sx + 1, sy + 1, 19, 8, 'm2'); g.H(sx + 1, sy + 1, 19, 'm3'); g.H(sx + 1, sy + 9, 19, 'm0'); g.T(sx + 2, sy + 3, 'HOME', 'ink', 'm5');
  g.R(sx + 2, sy + 11, 17, 14, 'm2'); g.B(sx + 2, sy + 11, 17, 14, 'm0'); g.H(sx + 3, sy + 12, 15, 'm3');
  g.OV(sx + 4, sy + 14, 7, 7, 'm3', 'ink'); g.px(sx + 7, sy + 17, 'ink'); g.px(sx + 7, sy + 15, 'r2'); g.R(sx + 13, sy + 15, 2, 6, 'am'); g.px(sx + 14, sy + 15, 'd8');
  g.px(sx + 1, sy + 13, 'm0'); g.px(sx + 1, sy + 22, 'm0'); g.R(sx + 1, sy + 26, 3, 2, 'm0'); g.R(sx + 17, sy + 26, 3, 2, 'm0');
  plate(g, sx + 10, 120, 'B0', 't1', 'cy', 'm5', { h: 9, pad: 3 });
  // cabinet with a ledger and a banker's lamp
  g.R(318, 150, 26, 12, 'w2'); g.B(318, 150, 26, 12, 'ink'); g.H(319, 151, 24, 'w4'); g.R(321, 153, 9, 3, 'w1'); g.R(332, 153, 9, 3, 'w1'); g.px(325, 154, 'am'); g.px(336, 154, 'am'); g.R(321, 157, 20, 3, 'w1'); g.px(331, 158, 'am');
  g.R(335, 144, 8, 2, 't4'); g.H(336, 144, 6, 't5'); g.V(339, 146, 3, 'am'); g.R(337, 149, 5, 1, 'am'); g.px(338, 146, 'wh'); g.px(340, 146, 'wh');
  g.R(321, 147, 11, 3, 'p2'); g.V(326, 147, 3, 'p0'); g.H(322, 148, 3, 'n4'); g.H(328, 148, 3, 'n4');
}
function upstairs(g, t) {
  recordRoom(g, t); printShop(g, t); wireOffice(g, t); office(g, t);
  for (const x of [198, 260, 314]) { g.R(x, 112, 2, 50, 'w1'); g.V(x, 112, 50, 'ink'); }
}

const PROP = ['....KKKKKKKK....', '...KnnnnnnnnK...', '...KnNNNNNNnK...', '...KrrrrrrrrK...', 'KKKKnnnnnnnnKKKK', 'KnnnNNNNNNNNnnnK', '.KKKKKKKKKKKKKK.', '...KsSSSSSSsK...', '...KSeSSSSeSK...', '...KsSSSSSSsK...', '..KmmmmmmmmmmK..', '...KmsSSSSsmK...', '....KsSSSSsK....', '...KKKbbbbKKK...', '.KKwwvvbbvvwwKK.', 'KwwwwvvwwvvwwwwK', 'KwgwwvvwgvvwwgwK', 'KwwwKvvvvvvKwwwK', 'KSSwKvvvvvvKwSSK'];
const PROPL = { K: 'ink', n: 'n1', N: 'n2', r: 'r2', s: 'w4', S: 'w5', e: 'ink', m: 'w0', b: 'r2', w: 'p2', v: 't3', g: 'am' };

function saloon(g, t) {
  const x0 = 131, x1 = 368, y0 = 168, y1 = 230;
  g.R(x0, y0, x1 - x0, 44, 'd2'); for (let x = x0 + 3; x < x1; x += 8) g.R(x, y0, 2, 44, 'd1');
  g.H(x0, y0, x1 - x0, 'w1'); g.H(x0, y0 + 1, x1 - x0, 'w3');
  g.R(x0, 212, x1 - x0, 18, 'w2'); g.H(x0, 212, x1 - x0, 'w4'); g.H(x0, 213, x1 - x0, 'w1'); for (let x = x0 + 3; x < x1; x += 6) g.V(x, 214, 16, 'w1'); g.H(x0, 229, x1 - x0, 'w0');
  g.clip(x0, y0, x1 - x0, y1 - y0);
  g.shade(x0, 214, x1 - x0, 16, (i, j) => (j - 212) / 18 * 0.3, 6, true);
  g.glow(200, 178, 70, 56, 'am', 0.3, 6, true); g.glow(270, 178, 70, 56, 'am', 0.3, 6, true); g.glow(346, 198, 50, 38, 'am', 0.28, 6, true);
  g.unclip();
  // the notice board: one intent on top, lit constraints and a preference, and the kinds that stay behind
  const bx = 134, by = 173; g.R(bx, by, 44, 33, 'w1'); g.B(bx, by, 44, 33, 'ink'); g.B(bx + 1, by + 1, 42, 31, 'w3'); g.H(bx + 1, by + 1, 42, 'w4');
  paper(g, bx + 3, by + 23, 12, 7, false, 'n3'); paper(g, bx + 16, by + 23, 12, 7, false, 'n3'); paper(g, bx + 29, by + 23, 12, 7, false, 'n3');
  paper(g, bx + 12, by + 3, 20, 8, true, 'am');
  paper(g, bx + 3, by + 13, 12, 8, true, 'r2'); paper(g, bx + 16, by + 13, 12, 8, true, 'r2'); paper(g, bx + 29, by + 13, 12, 8, true, 't4');
  halo(g, bx + 2, by + 2, 40, 29, ['p2'], 'am', [0.4], 8);
  // canary
  const cx = 189; g.V(cx, y0 + 2, 3, 'm1'); g.H(cx - 2, y0 + 5, 5, 'am'); g.px(cx - 3, y0 + 6, 'am'); g.px(cx + 3, y0 + 6, 'am'); g.px(cx - 4, y0 + 7, 'am'); g.px(cx + 4, y0 + 7, 'am');
  g.R(cx - 3, y0 + 7, 7, 10, 'd1');
  for (const dx of [-4, -2, 0, 2, 4]) g.V(cx + dx, y0 + 7, 10, 'am'); g.H(cx - 4, y0 + 17, 9, 'am'); g.H(cx - 4, y0 + 18, 9, 'w3'); g.H(cx - 3, y0 + 14, 7, 'w3');
  const hop = (t % 6 === 2 || t % 6 === 3) ? -1 : 0; g.R(cx - 1, y0 + 11 + hop, 3, 3, 'd8'); g.px(cx + 1, y0 + 10 + hop, 'd8'); g.px(cx, y0 + 10 + hop, 'd8'); g.px(cx + 2, y0 + 11 + hop, 'd5'); g.px(cx + 1, y0 + 11 + hop, 'ink'); g.px(cx - 2, y0 + 13 + hop, 'am');
  // the room: table, three seats, a tab on a spike for each
  const tx = 236;
  chairFront(g, tx, y1, 3); cardTable(g, tx, y1);
  chair(g, 212, y1, { no: 1 }); figure(g, 212, y1, { sit: true, arm: 'table', pon: PON.red, hat: HAT.brown, band: 'am', cell: true });
  chair(g, 260, y1, { no: 2, flip: true }); figure(g, 260, y1, { sit: true, arm: 'table', pon: PON.teal, hat: HAT.grey, band: 'r2', flip: true, cell: true });
  spike(g, 226, 216, 5); spike(g, 247, 216, 3); spike(g, 242, 216, 1);
  // the handbill coming down from the print shop
  const hx = 221, hy = 175; g.V(236, 168, hy - 168, 'p1'); g.px(236, hy - 1, 'm3');
  g.R(hx, hy, 31, 25, 'p2'); g.B(hx, hy, 31, 25, 'ink'); g.T(hx + 2, hy + 2, 'RESTORE', 'r1', 'm5'); g.H(hx + 2, hy + 8, 27, 'ink');
  [['am', 20], ['r2', 22], ['t4', 17]].forEach(([c, w], k) => { const yy = hy + 10 + k * 4; g.R(hx + 2, yy, 2, 2, c); g.H(hx + 6, yy, w, 'n3'); g.H(hx + 6, yy + 2, w - 6, 'n4'); });
  g.px(hx + 3, hy + 22, 'cy'); g.px(hx + 6, hy + 22, 'cy'); g.px(hx + 9, hy + 22, 'cy'); g.H(hx + 13, hy + 22, 14, 'n4');
  // the call box and its tube
  g.R(303, y0, 2, 22, 't4'); g.V(303, y0, 22, 't5'); g.R(302, y0 + 1, 4, 2, 'm2'); g.H(302, y0 + 1, 4, 'm3'); g.R(303, y0 + 4, 2, 3, 'p2'); g.H(303, y0 + 5, 2, 'r2');
  plate(g, 304, 177, 'RECALL', 'n1', 'am', 'm5', { h: 9, pad: 3 });
  g.R(303, 186, 2, 4, 't4'); g.V(303, 186, 4, 't5');
  g.R(297, 190, 14, 24, 'w2'); g.B(297, 190, 14, 24, 'ink'); g.H(298, 191, 12, 'w4'); g.R(299, 193, 10, 7, 'w1'); g.H(300, 194, 8, 'am'); g.H(300, 196, 8, 'am'); g.H(300, 198, 8, 'am'); g.px(308, 192, t % 6 < 2 ? 'cy' : 't4');
  g.R(299, 203, 10, 6, 'ink'); g.H(300, 207, 6, 'p2'); g.px(300, 206, 'r2'); g.H(301, 206, 5, 'p1');
  g.px(296, 207, 'm3'); g.px(295, 208, 'm3'); g.px(294, 209, 'm3'); g.px(294, 210, 'am');
  figure(g, 284, y1, { arm: 'crank', pon: PON.ochre, hat: HAT.black, band: 'cy', cell: true });
  // the bar
  const b0 = 320;
  g.R(336, 189, 30, 12, 'n2'); g.B(335, 188, 32, 14, 'w3'); g.L(358, 199, 364, 193, 'n3'); g.H(336, 201, 30, 'w4');
  [[336, 't4', 8], [357, 'g1', 8], [360, 'r2', 6], [363, 'am', 7]].forEach(([x, c, h]) => bottle(g, x, 201, c, h));
  plate(g, 343, 170, 'NEEDS YOU', 'n1', 'am', 'm5', { h: 8, pad: 3 });
  g.H(321, 179, 46, 'am'); g.H(321, 180, 46, 'w2');
  for (let k = 0; k < 4; k++) { const x = 323 + k * 11, y = 181 + (k === 2 ? 1 : 0); g.R(x, y, 9, 8, 'p2'); g.B(x, y, 9, 8, 'p0'); g.T(x + 1, y + 2, String(41 + k), 'ink', 'm5'); g.px(x + 4, y - 1, 'am'); g.px(x + 4, y, 'am'); }
  g.S(340, 191, PROP, PROPL);
  g.R(b0, 212, x1 - b0, 18, 'w2'); g.B(b0, 212, x1 - b0, 18, 'ink'); g.R(b0 - 1, 210, x1 - b0 + 1, 2, 'w4'); g.H(b0 - 1, 211, x1 - b0 + 1, 'w3');
  for (let k = 0; k < 3; k++) { g.R(b0 + 3 + k * 14, 215, 12, 9, 'w1'); g.H(b0 + 3 + k * 14, 223, 12, 'w3'); } g.H(b0 + 1, 226, x1 - b0 - 2, 'am'); g.H(b0 + 1, 227, x1 - b0 - 2, 'w0');
  // the register, where every refusal rings up
  const rx = 322; g.R(rx + 1, 196, 10, 8, 'n1'); g.B(rx + 1, 196, 10, 8, 'ink'); g.T(rx + 3, 198, '07', 'cy', 'm5'); g.R(rx, 204, 12, 6, 'w3'); g.B(rx, 204, 12, 6, 'ink'); g.H(rx + 1, 204, 10, 'am'); for (let i = 0; i < 5; i++) { g.px(rx + 1 + i * 2, 206, 'p2'); g.px(rx + 2 + i * 2, 208, 'p2'); } g.px(rx + 12, 205, 'm3'); g.px(rx + 13, 204, 'm3'); g.px(rx + 13, 203, 'am');
  lamp(g, 200, y0, t, { drop: 4 }); lamp(g, 270, y0, t, { drop: 4 });
  halo(g, x0, 200, x1 - x0, 30, ['cy'], 'cy', [0.26], 8);
}
module.exports = { figure, upstairs, saloon, PON, HAT, STEEL, BRASS, DARK, vkey, bottle, chair, PROP, PROPL };
