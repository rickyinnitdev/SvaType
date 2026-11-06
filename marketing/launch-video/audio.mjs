// Fully synthesized soundtrack: background music + sound effects placed on the
// animation's cue list. No samples, no downloads, no licensing questions.
const SR = 44100;

export function synthesize(cues, duration) {
  const n = Math.ceil(duration * SR);
  const music = [new Float32Array(n), new Float32Array(n)];
  const sfx = [new Float32Array(n), new Float32Array(n)];
  const verb = [new Float32Array(n), new Float32Array(n)]; // reverb send
  let seed = 12345;
  const rnd = () =>
    (seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff;
  const noise = () => rnd() * 2 - 1;
  const mtof = (m) => 440 * Math.pow(2, (m - 69) / 12);

  function put(bus, i, v, pan = 0, send = 0) {
    if (i < 0 || i >= n) return;
    const l = Math.cos(((pan + 1) * Math.PI) / 4),
      r = Math.sin(((pan + 1) * Math.PI) / 4);
    bus[0][i] += v * l;
    bus[1][i] += v * r;
    if (send) {
      verb[0][i] += v * l * send;
      verb[1][i] += v * r * send;
    }
  }

  // ---------------- music ----------------
  // Tempo chosen so bar 12 lands exactly on the end-card impact (26.35 s).
  const BAR = 26.35 / 12,
    BEAT = BAR / 4;
  const CHORDS = [
    // D – A – Bm – G (I–V–vi–IV in D major)
    { pad: [62, 66, 69], bass: 38, arp: [74, 78, 81, 86] },
    { pad: [61, 64, 69], bass: 33, arp: [73, 76, 81, 85] },
    { pad: [62, 66, 71], bass: 35, arp: [74, 78, 83, 86] },
    { pad: [62, 67, 71], bass: 31, arp: [74, 79, 83, 86] },
  ];
  const END_BAR = 12;
  const kickTimes = [];

  function padNote(m, t0, dur, gain) {
    const f = mtof(m),
      a = 0.5,
      rel = 0.9;
    const s0 = Math.floor(t0 * SR),
      len = Math.floor((dur + rel) * SR);
    const harm = 9;
    for (const det of [-0.004, 0.004]) {
      const ff = f * (1 + det),
        pan = det > 0 ? 0.35 : -0.35;
      for (let k = 0; k < len; k++) {
        const t = k / SR;
        const env =
          Math.min(1, t / a) * (t > dur ? Math.max(0, 1 - (t - dur) / rel) : 1);
        let v = 0;
        for (let h = 1; h <= harm; h++)
          v +=
            Math.sin(2 * Math.PI * ff * h * t + h) / (h * (1 + h * h * 0.06));
        put(music, s0 + k, v * env * gain, pan, 0.25);
      }
    }
  }
  function pluck(m, t0, gain, pan) {
    // Karplus–Strong
    const f = mtof(m),
      period = Math.round(SR / f);
    const ring = new Float32Array(period);
    for (let i = 0; i < period; i++) ring[i] = noise();
    const s0 = Math.floor(t0 * SR),
      len = Math.floor(1.4 * SR);
    let idx = 0,
      prev = 0;
    for (let k = 0; k < len; k++) {
      const cur = ring[idx];
      const next = 0.5 * (cur + ring[(idx + 1) % period]) * 0.996;
      ring[idx] = next;
      idx = (idx + 1) % period;
      const v = (cur + prev) * 0.5;
      prev = cur;
      put(music, s0 + k, v * gain * Math.min(1, k / 40), pan, 0.35);
    }
  }
  function bassNote(m, t0, dur, gain) {
    const f = mtof(m),
      s0 = Math.floor(t0 * SR),
      len = Math.floor(dur * SR);
    for (let k = 0; k < len; k++) {
      const t = k / SR,
        env =
          Math.min(1, t / 0.01) *
          Math.exp(-t * 3.2) *
          Math.min(1, (dur - t) / 0.03);
      const v = Math.tanh(
        1.6 *
          (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(4 * Math.PI * f * t)),
      );
      put(music, s0 + k, v * env * gain, 0);
    }
  }
  function kick(t0, gain) {
    kickTimes.push(t0);
    const s0 = Math.floor(t0 * SR),
      len = Math.floor(0.45 * SR);
    let ph = 0;
    for (let k = 0; k < len; k++) {
      const t = k / SR,
        f = 45 + 95 * Math.exp(-t * 28);
      ph += (2 * Math.PI * f) / SR;
      const v =
        Math.sin(ph) * Math.exp(-t * 7) +
        (k < 90 ? noise() * 0.25 * (1 - k / 90) : 0);
      put(music, s0 + k, v * gain, 0);
    }
  }
  function hat(t0, gain, open = false) {
    const s0 = Math.floor(t0 * SR),
      len = Math.floor((open ? 0.25 : 0.06) * SR);
    let p = 0;
    for (let k = 0; k < len; k++) {
      const x = noise(),
        hp = x - p;
      p = x;
      put(
        music,
        s0 + k,
        hp * Math.exp((-k / SR) * (open ? 14 : 60)) * gain,
        0.25,
      );
    }
  }
  function clap(t0, gain) {
    const s0 = Math.floor(t0 * SR),
      len = Math.floor(0.3 * SR);
    let lp = 0,
      p = 0;
    for (let k = 0; k < len; k++) {
      const t = k / SR;
      const burst =
        t < 0.03
          ? Math.floor(t / 0.01) % 2 === 0
            ? 1
            : 0.4
          : Math.exp(-(t - 0.03) * 18);
      const x = noise();
      lp += 0.35 * (x - lp);
      const bp = lp - p;
      p = lp;
      put(music, s0 + k, bp * burst * gain * 2.2, -0.1, 0.3);
    }
  }
  function crash(t0, gain) {
    const s0 = Math.floor(t0 * SR),
      len = Math.floor(2.6 * SR);
    let p = 0;
    for (let k = 0; k < len; k++) {
      const x = noise(),
        hp = x - p;
      p = x;
      put(
        music,
        s0 + k,
        hp * Math.exp((-k / SR) * 1.8) * gain,
        rnd() * 0.6 - 0.3,
        0.3,
      );
    }
  }

  for (let bar = 0; bar < END_BAR; bar++) {
    const c = CHORDS[bar % 4],
      t0 = bar * BAR;
    for (const m of c.pad) padNote(m, t0, BAR, bar < 2 ? 0.022 : 0.018);
    const full = bar >= 2;
    // arpeggio: quarters in the intro, eighths afterwards
    const pattern = [0, 1, 2, 3, 2, 1, 2, 3];
    const steps = full ? 8 : 4;
    for (let s = 0; s < steps; s++) {
      const note = c.arp[pattern[(s * (8 / steps)) % 8]];
      pluck(note, t0 + s * (BAR / steps), 0.11, s % 2 ? 0.45 : -0.45);
    }
    if (!full) continue;
    for (let b = 0; b < 4; b++) {
      kick(t0 + b * BEAT, 0.5);
      hat(t0 + b * BEAT + BEAT / 2, 0.06, b === 3 && bar % 2 === 1);
      if (b === 1 || b === 3) clap(t0 + b * BEAT, 0.16);
      for (let e = 0; e < 2; e++)
        bassNote(
          c.bass + (e ? 12 : 0),
          t0 + b * BEAT + (e * BEAT) / 2,
          (BEAT / 2) * 0.92,
          0.17,
        );
    }
    if (bar === 2) crash(t0, 0.08);
  }
  // build into the end card: snare-ish clap roll on the last half bar
  for (let r = 0; r < 8; r++)
    clap((END_BAR - 0.5) * BAR + (r * BEAT) / 4, 0.05 + r * 0.012);
  // ending: one big sustained D chord + bass + crash, then fade
  const tEnd = END_BAR * BAR;
  for (const m of [50, 57, 62, 66, 69, 74]) padNote(m, tEnd, 3.6, 0.02);
  kick(tEnd, 0.6);
  crash(tEnd, 0.12);
  bassNote(26, tEnd, 3.5, 0.2);
  [86, 81, 78, 74, 78, 81, 86, 90].forEach((m, i) =>
    pluck(
      m,
      tEnd + 0.15 + (i * BEAT) / 2,
      0.08 * (1 - i / 10),
      i % 2 ? 0.5 : -0.5,
    ),
  );

  // sidechain-style ducking of the music under each kick
  const duck = new Float32Array(n).fill(1);
  for (const t of kickTimes) {
    const s0 = Math.floor(t * SR),
      len = Math.floor(0.22 * SR);
    for (let k = 0; k < len && s0 + k < n; k++)
      duck[s0 + k] = Math.min(duck[s0 + k], 0.55 + 0.45 * (k / len));
  }

  // ---------------- sound effects ----------------
  function sweep(
    t0,
    len,
    f0,
    f1,
    f2,
    gain,
    q = 3,
    panFrom = -0.7,
    panTo = 0.7,
  ) {
    const s0 = Math.floor(t0 * SR),
      L = Math.floor(len * SR);
    let low = 0,
      band = 0;
    for (let k = 0; k < L; k++) {
      const x = k / L;
      const fc =
        x < 0.5 ? f0 + (f1 - f0) * (x * 2) : f1 + (f2 - f1) * ((x - 0.5) * 2);
      const F = 2 * Math.sin((Math.PI * Math.min(fc, SR / 6)) / SR);
      const high = noise() - low - band / q;
      band += F * high;
      low += F * band;
      const env = Math.sin(Math.PI * x) ** 1.5;
      put(sfx, s0 + k, band * env * gain, panFrom + (panTo - panFrom) * x, 0.2);
    }
  }
  function bell(t0, f, gain, decay = 1.2, pan = 0) {
    const s0 = Math.floor(t0 * SR),
      L = Math.floor(decay * 2.5 * SR);
    for (let k = 0; k < L; k++) {
      const t = k / SR;
      const v =
        Math.sin(2 * Math.PI * f * t) * Math.exp((-t / decay) * 2.2) +
        0.35 *
          Math.sin(2 * Math.PI * f * 2.76 * t) *
          Math.exp((-t / decay) * 5) +
        0.15 * Math.sin(2 * Math.PI * f * 5.4 * t) * Math.exp((-t / decay) * 9);
      put(sfx, s0 + k, v * gain * Math.min(1, k / 30), pan, 0.45);
    }
  }
  function blip(t0, f0, f1, len, gain, pan = 0) {
    const s0 = Math.floor(t0 * SR),
      L = Math.floor(len * SR);
    let ph = 0;
    for (let k = 0; k < L; k++) {
      const x = k / L,
        f = f0 * Math.pow(f1 / f0, Math.min(1, x * 2.2));
      ph += (2 * Math.PI * f) / SR;
      put(
        sfx,
        s0 + k,
        Math.sin(ph) * Math.exp(-x * 5) * Math.min(1, k / 20) * gain,
        pan,
        0.15,
      );
    }
  }
  function keyClick(t0, shift) {
    const s0 = Math.floor(t0 * SR),
      vary = 0.85 + rnd() * 0.3,
      pan = rnd() * 0.4 - 0.2;
    let p = 0,
      lp = 0;
    const L = Math.floor(0.06 * SR);
    for (let k = 0; k < L; k++) {
      const t = k / SR,
        x = noise(),
        hp = x - p;
      p = x;
      lp += 0.2 * (x - lp);
      const v =
        hp * Math.exp(-t * 260) * 0.8 + // sharp top click
        lp * Math.exp(-t * 90) * 0.9 + // plastic body
        Math.sin(2 * Math.PI * 170 * vary * t) * Math.exp(-t * 70) * 0.5; // thock
      put(sfx, s0 + k, v * 0.42 * vary, pan, 0.05);
    }
    if (shift) keyClickSoft(t0 - 0.035);
  }
  function keyClickSoft(t0) {
    const s0 = Math.floor(t0 * SR),
      L = Math.floor(0.04 * SR);
    let lp = 0;
    for (let k = 0; k < L; k++) {
      const x = noise();
      lp += 0.25 * (x - lp);
      put(sfx, s0 + k, lp * Math.exp((-k / SR) * 120) * 0.35, -0.3);
    }
  }
  function impact(t0) {
    const s0 = Math.floor(t0 * SR),
      L = Math.floor(1.6 * SR);
    let ph = 0,
      lp = 0;
    for (let k = 0; k < L; k++) {
      const t = k / SR,
        f = 32 + 60 * Math.exp(-t * 6);
      ph += (2 * Math.PI * f) / SR;
      lp += 0.08 * (noise() - lp);
      put(
        sfx,
        s0 + k,
        Math.tanh(2 * Math.sin(ph)) * Math.exp(-t * 2.4) * 0.55 +
          lp * Math.exp(-t * 9) * 1.4,
        0,
        0.3,
      );
    }
  }

  for (const c of cues) {
    switch (c.type) {
      case 'whoosh':
        sweep(c.t, c.len || 0.8, 250, 3200, 700, 0.55 * (c.gain ?? 1));
        break;
      case 'swish':
        sweep(c.t, 0.32, 900, 5200, 2500, 0.28, 2, -0.3, 0.3);
        break;
      case 'riser':
        sweep(c.t, c.len || 1, 400, 2500, 7000, 0.18, 4, 0, 0);
        break;
      case 'pop':
        blip(
          c.t,
          900 * (c.pitch || 1),
          300 * (c.pitch || 1),
          0.12,
          0.42,
          rnd() * 0.6 - 0.3,
        );
        break;
      case 'tick':
        blip(
          c.t,
          2400 * (c.pitch || 1),
          2200 * (c.pitch || 1),
          0.025,
          0.12,
          0.2,
        );
        break;
      case 'key':
        keyClick(c.t, c.shift);
        break;
      case 'ding':
        bell(c.t, 1318.5, 0.2, 1.1, -0.15);
        bell(c.t + 0.1, 1760, 0.2, 1.3, 0.15);
        break;
      case 'chime':
        [1174.7, 1480, 1760, 2349.3].forEach((f, i) =>
          bell(c.t + i * 0.07, f, 0.1, 0.9, -0.45 + i * 0.3),
        );
        break;
      case 'sparkle':
        for (let i = 0; i < 6; i++)
          bell(
            c.t + i * 0.05,
            2600 + rnd() * 2200,
            0.05,
            0.4,
            rnd() * 1.6 - 0.8,
          );
        break;
      case 'shimmer':
        for (let i = 0; i < 12; i++)
          bell(
            c.t + rnd() * 1.1,
            2200 + rnd() * 3200,
            0.04,
            0.6,
            rnd() * 1.8 - 0.9,
          );
        break;
      case 'impact':
        impact(c.t);
        break;
      default:
        break;
    }
  }

  // ---------------- reverb (Schroeder) on the send bus ----------------
  const wet = [new Float32Array(n), new Float32Array(n)];
  for (let ch = 0; ch < 2; ch++) {
    const combs = [1557, 1617, 1491, 1422].map((d) => d + ch * 23);
    for (const d of combs) {
      const buf = new Float32Array(d);
      let i = 0,
        lp = 0;
      for (let k = 0; k < n; k++) {
        const out = buf[i];
        lp = out * 0.75 + lp * 0.25;
        buf[i] = verb[ch][k] + lp * 0.8;
        i = (i + 1) % d;
        wet[ch][k] += out * 0.25;
      }
    }
    for (const d of [225, 556]) {
      const buf = new Float32Array(d);
      let i = 0;
      for (let k = 0; k < n; k++) {
        const b = buf[i],
          x = wet[ch][k];
        const y = -x + b;
        buf[i] = x + b * 0.5;
        i = (i + 1) % d;
        wet[ch][k] = y;
      }
    }
  }

  // ---------------- mix + master ----------------
  const outL = new Float32Array(n),
    outR = new Float32Array(n);
  let peak = 0;
  for (let k = 0; k < n; k++) {
    const t = k / SR;
    const fade =
      Math.min(1, t / 0.3) * Math.min(1, Math.max(0, (duration - t) / 1.4));
    const l =
      (music[0][k] * duck[k] * 0.8 + sfx[0][k] + wet[0][k] * 0.6) * fade;
    const r =
      (music[1][k] * duck[k] * 0.8 + sfx[1][k] + wet[1][k] * 0.6) * fade;
    outL[k] = l;
    outR[k] = r;
    peak = Math.max(peak, Math.abs(l), Math.abs(r));
  }
  const g = 1.15 / peak; // drive slightly into a soft limiter
  const wav = Buffer.alloc(44 + n * 4);
  wav.write('RIFF', 0);
  wav.writeUInt32LE(36 + n * 4, 4);
  wav.write('WAVE', 8);
  wav.write('fmt ', 12);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(2, 22);
  wav.writeUInt32LE(SR, 24);
  wav.writeUInt32LE(SR * 4, 28);
  wav.writeUInt16LE(4, 32);
  wav.writeUInt16LE(16, 34);
  wav.write('data', 36);
  wav.writeUInt32LE(n * 4, 40);
  for (let k = 0; k < n; k++) {
    wav.writeInt16LE(
      Math.round(Math.tanh(outL[k] * g) * 0.93 * 32767),
      44 + k * 4,
    );
    wav.writeInt16LE(
      Math.round(Math.tanh(outR[k] * g) * 0.93 * 32767),
      46 + k * 4,
    );
  }
  return wav;
}
