(function () {
  'use strict';
  // The explorer on the GitHub page. The picture is 480 x 330. It is shown at a whole number of screen pixels per
  // picture pixel wherever that fits, so every pixel stays the same size. Frames of the loop are frame 0 plus the
  // 8 x 8 tiles that differ from it. docs/design/art/build.js writes the data into the page beside this script.
  var src = document.getElementById('inn-data');
  if (!src) return;
  var D = JSON.parse(src.textContent);
  var W = 480, H = 330, TS = 8, TC = 60, AC = D.ac;
  var $ = function (id) { return document.getElementById(id); };
  var ex = $('explore'), stage = $('stage'), plate = $('plate'), still = $('still'), cv = $('live'), marksEl = $('marks');
  var cap = $('cap'), capN = $('cap-n'), capT = $('cap-t'), col = $('keycol');
  var bPrev = $('b-prev'), bNext = $('b-next'), bAll = $('b-all'), bPlay = $('b-play'), bNums = $('b-nums');
  var ctx = cv.getContext ? cv.getContext('2d') : null;
  if (!ctx) return;
  var mqTwo = window.matchMedia('(min-width: 960px)');
  var reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var frame = document.createElement('canvas'); frame.width = W; frame.height = H;
  var fctx = frame.getContext('2d');
  var tmp = document.createElement('canvas'), tctx = tmp.getContext('2d');
  var atlas = new Image(), ready = false, need = 2;
  var cw = 0, ch = 0, dpr = 1, soft = false, s0 = 1;
  var view = [0, 0, W], raf = 0, cur = null, peek = null, fi = 0, playing = !reduce, timer = 0, onScreen = true;
  var lastW = -1, lastH = -1, lastD = -1;
  var keys = {}, marks = {};

  Array.prototype.forEach.call(document.querySelectorAll('.kb'), function (k) { keys[k.getAttribute('data-id')] = k; });
  D.order.forEach(function (id, i) {
    var m = document.createElement('span'), b = document.createElement('b');
    m.className = 'mk'; b.textContent = String(i + 1); m.appendChild(b); marksEl.appendChild(m); marks[id] = m;
  });

  function compose(i) {
    fctx.drawImage(still, 0, 0);
    if (i > 0 && D.frames[i - 1]) {
      var L = D.frames[i - 1];
      for (var k = 0; k < L.length; k += 2) {
        var a = L[k], p = L[k + 1];
        fctx.drawImage(atlas, (a % AC) * TS, ((a / AC) | 0) * TS, TS, TS, (p % TC) * TS, ((p / TC) | 0) * TS, TS, TS);
      }
    }
  }

  // The part of the picture a place zooms to: [left, top, width] in picture pixels, at a whole-number scale.
  function boxFor(id) {
    if (!id) return [0, 0, W];
    var b = D.box[id], s = Math.max(Math.round(cw / b[2]), Math.floor(s0 + 1e-4) + 1);
    var vw = cw / s, vh = ch / s;
    var sx = Math.min(W - vw, Math.max(0, b[0] - vw / 2)), sy = Math.min(H - vh, Math.max(0, b[1] - vh / 2));
    return [Math.round(sx * s) / s, Math.round(sy * s) / s, vw];
  }

  function placeMarks() {
    var s = cw / view[2] / dpr, rest = !raf;
    marksEl.className = 'marks' + (s < 1.3 ? ' dots' : '');
    for (var i = 0; i < D.order.length; i++) {
      var id = D.order[i], p = D.pos[id];
      var x = (p[0] + 0.5 - view[0]) * s, y = (p[1] + 0.5 - view[1]) * s;
      if (rest) { x = Math.round(x); y = Math.round(y); }
      marks[id].style.transform = 'translate(' + x.toFixed(1) + 'px,' + y.toFixed(1) + 'px)';
    }
  }

  function paint() {
    var vw = view[2], vh = vw * ch / cw;
    if (!raf && !cur && soft) {
      // The whole picture at a scale between whole numbers: enlarge by a whole number first, then let the browser
      // smooth the last step down, which keeps every pixel the same width.
      var K = Math.max(1, Math.ceil(s0 - 1e-4));
      ctx.imageSmoothingEnabled = true; ctx.imageSmoothingQuality = 'high';
      if (K > 1) {
        if (tmp.width !== W * K) { tmp.width = W * K; tmp.height = H * K; }
        tctx.imageSmoothingEnabled = false; tctx.drawImage(frame, 0, 0, W * K, H * K);
        ctx.drawImage(tmp, 0, 0, cw, ch);
      } else ctx.drawImage(frame, 0, 0, cw, ch);
    } else {
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(frame, view[0], view[1], vw, vh, 0, 0, cw, ch);
    }
    placeMarks();
  }

  function fit(force) {
    var d = window.devicePixelRatio || 1, wc = ex.clientWidth, ih = window.innerHeight || 0;
    if (!wc) return;
    if (!force && wc === lastW && d === lastD && Math.abs(ih - lastH) < 100) return;
    lastW = wc; lastD = d; lastH = ih; dpr = d;
    var two = mqTwo.matches, maxW = two ? wc - 28 - 290 : wc;
    if (ih > 360) maxW = Math.min(maxW, Math.floor((two ? ih - 54 : ih * 0.5) * W / H));
    maxW = Math.max(160, maxW);
    var s = maxW * d / W, k = Math.floor(s + 1e-4), cssW;
    if (k >= 2 || (k === 1 && s < 1.3)) { cw = k * W; ch = k * H; cssW = cw / d; soft = false; }
    else { cssW = maxW; cw = Math.round(maxW * d); ch = Math.round(cw * H / W); soft = true; }
    s0 = cw / W;
    plate.style.width = cssW + 'px';
    plate.style.marginLeft = two ? '' : (Math.max(0, Math.round((wc - cssW) / 2 * d) / d) + 'px');
    ex.style.setProperty('--pw', cssW + 'px');
    if (cv.width !== cw || cv.height !== ch) { cv.width = cw; cv.height = ch; }
    cancelAnimationFrame(raf); raf = 0;
    view = boxFor(cur);
    if (ready) paint(); else placeMarks();
  }

  function go(to) {
    cancelAnimationFrame(raf); raf = 0;
    if (reduce || !ready) { view = to.slice(); if (ready) paint(); else placeMarks(); return; }
    var from = view.slice(), t0 = 0;
    raf = requestAnimationFrame(function step(now) {
      if (!t0) t0 = now;
      var t = Math.min(1, (now - t0) / 720), e = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
      if (t < 1) {
        view = [from[0] + (to[0] - from[0]) * e, from[1] + (to[1] - from[1]) * e, from[2] + (to[2] - from[2]) * e];
        raf = requestAnimationFrame(step);
      } else { view = to.slice(); raf = 0; }
      paint();
    });
  }

  function caption() {
    var id = peek || cur;
    if (id) { capN.textContent = String(D.order.indexOf(id) + 1); capT.textContent = D.names[id]; }
    cap.hidden = !id;
    cap.className = 'cap' + (peek && peek !== cur ? ' peek' : '');
  }

  function reveal(el) {
    var beh = reduce ? 'auto' : 'smooth';
    if (col.scrollHeight > col.clientHeight + 2) {
      var li = el.parentNode, top = li.offsetTop - (col.clientHeight - li.offsetHeight) / 2;
      col.scrollTo({ top: Math.max(0, top), behavior: beh });
    } else {
      var r = el.getBoundingClientRect(), sb = stage.getBoundingClientRect().bottom;
      if (r.top < sb + 6 || r.bottom > window.innerHeight - 6) window.scrollBy({ top: r.top - sb - 10, behavior: beh });
    }
  }

  function select(id, fromKey) {
    cur = id; peek = null;
    for (var k in keys) {
      var on = k === id;
      keys[k].setAttribute('aria-pressed', on ? 'true' : 'false');
      keys[k].querySelector('.k-def').hidden = !on;
    }
    for (var m in marks) marks[m].className = 'mk' + (m === id ? ' on' : '');
    plate.classList.toggle('zoomed', !!id);
    bAll.disabled = !id;
    caption();
    go(boxFor(id));
    if (id && !fromKey) reveal(keys[id]);
  }
  function stepBy(d) {
    var n = D.order.length, i = cur ? D.order.indexOf(cur) : (d > 0 ? -1 : 0);
    select(D.order[(i + d + n) % n], false);
  }

  // The numbered place nearest a pointer, if one is close enough to mean it.
  function nearest(e) {
    var r = cv.getBoundingClientRect();
    if (!r.width) return null;
    var s = r.width / view[2], x = view[0] + (e.clientX - r.left) / s, y = view[1] + (e.clientY - r.top) / s, best = null, bd = 1e9;
    for (var i = 0; i < D.order.length; i++) {
      var p = D.pos[D.order[i]], d = Math.sqrt(Math.pow(p[0] + 0.5 - x, 2) + Math.pow(p[1] + 0.5 - y, 2)) * s;
      if (d < bd) { bd = d; best = D.order[i]; }
    }
    return bd < Math.min(44, Math.max(22, 7 * s)) ? best : null;
  }

  function tick() { fi = (fi + 1) % D.n; compose(fi); if (!raf) paint(); }
  function run() {
    clearInterval(timer); timer = 0;
    if (playing && ready && onScreen && !document.hidden && D.n > 1) timer = setInterval(tick, D.tick);
  }
  function setPlay(on) {
    playing = on;
    bPlay.setAttribute('aria-label', on ? 'Pause the picture' : 'Play the picture');
    bPlay.querySelector('.i-pause').hidden = !on;
    bPlay.querySelector('.i-play').hidden = on;
    run();
  }

  for (var id in keys) (function (id) { keys[id].addEventListener('click', function () { select(cur === id ? null : id, true); }); })(id);
  bPrev.addEventListener('click', function () { stepBy(-1); });
  bNext.addEventListener('click', function () { stepBy(1); });
  bAll.addEventListener('click', function () { select(null, true); });
  bPlay.addEventListener('click', function () { setPlay(!playing); });
  bNums.addEventListener('click', function () {
    var show = marksEl.hidden;
    marksEl.hidden = !show; bNums.setAttribute('aria-pressed', show ? 'true' : 'false');
  });
  plate.addEventListener('click', function (e) {
    var id = nearest(e);
    if (id) select(id === cur ? null : id, false); else if (cur) select(null, true);
  });
  plate.addEventListener('mousemove', function (e) {
    var id = nearest(e);
    plate.style.cursor = id ? 'pointer' : '';
    if (id !== peek) { peek = id; caption(); }
  });
  plate.addEventListener('mouseleave', function () { plate.style.cursor = ''; if (peek) { peek = null; caption(); } });

  function got() {
    if (--need > 0) return;
    ready = true;
    fctx.imageSmoothingEnabled = false;
    compose(0); fit(true);
    cv.hidden = false;
    run();
  }
  atlas.onload = got;
  atlas.onerror = function () { D.frames = []; D.n = 1; got(); };
  atlas.src = D.atlas;
  if (still.complete && still.naturalWidth) got(); else still.addEventListener('load', got);

  setPlay(playing);
  fit(true);
  if (window.ResizeObserver) new ResizeObserver(function () { fit(false); }).observe(ex);
  window.addEventListener('resize', function () { fit(false); });
  document.addEventListener('visibilitychange', run);
  if (window.IntersectionObserver) new IntersectionObserver(function (es) { onScreen = es[es.length - 1].isIntersecting; run(); }).observe(plate);
})();
