#!/usr/bin/env python3
"""Recordings of consoles running the 240p Test Suite's MDFourier sequence, measured against Nephrite's own run of it
(Nephrite_Disputes.md D-27, Nephrite_Native.md §28). The recordings are MDFourier's (junkerhq.net/MDFourier, "MegaDrive
/Sega Genesis and Sega CD recordings 2020-06-14", FLAC, decoded to WAV); Nephrite's run is `examples/wav` with the pad
presses that start the sequence. Needs numpy and scipy.

The sequence, from the first sync pulse, in frames: 20 of pulses (the PSG at 9,322 Hz, a frame on and a frame off), 20
of silence, 96 FM notes of 20 (eight octaves of twelve, the key released at frame 16; channels 1 to 3 on the left with
one voice, 4 to 6 on the right with another), 40 PSG tones of 20, a PSG ramp of 400, 16 noises of 20, 20 of silence
and the pulses again: 3,500 frames between the two pulse trains.

  align      <wav>...                      where the pulse trains are, and the frames between them
  response   <ref> <rec>...                each recording's level over the reference's at the FM partials, by band
  fit        <ref> <rec>... [--held]       a gain, a first-order low-pass and a first-order high-pass fitted to that level;
                                           --held takes out the droop of a reference that holds each 53 kHz sample
  fit2       <ref> <rec>...                the same with a second-order low-pass (corner and Q)
  fit3       <ref> <rec>...                the same with a second-order and a first-order low-pass
  level      <ref> <rec>...                a gain only: what is left, each side, and the PSG's level against FM's
  ladder     <with> <without> <rec> <low-pass Hz>...
                                           the partials the reference has only with the ladder: their level in a recording
  design                                   the digital filters of sound.rs, fitted to the analogue ones at 48 kHz
  dac-steps  <diagram.png>                 the steps between DAC levels read off Kabuto's two diagrams (D-27)
"""
import sys, wave
import numpy as np

FRAME = 896040 / 53693175            # an NTSC frame, seconds
YM_RATE = 53693175 / 7 / 144          # the YM2612's sample rate

def load(path):
    w = wave.open(path, "rb")
    x = np.frombuffer(w.readframes(w.getnframes()), dtype="<i2").reshape(-1, w.getnchannels()).astype(np.float64)
    return w.getframerate(), x

def pulse_rises(x, sr):
    """The rises of the sync tone: its envelope by quadrature at the PSG's 9,322 Hz, rises under 0.6 of a frame apart
    taken as one."""
    m = x.mean(axis=1)
    z = m * np.exp(-2j * np.pi * 9322 * np.arange(len(m)) / sr)
    box = lambda v, seconds: np.convolve(v, np.ones(int(sr * seconds)) / int(sr * seconds), "same")
    env = box(np.abs(box(z, 0.001)), 0.003)
    on = env > np.percentile(env, 99.5) * 0.4
    rises = []
    for e in np.flatnonzero(np.diff(on.astype(int)) == 1):
        if not rises or e - rises[-1] > 0.6 * FRAME * sr:
            rises.append(e)
    return np.array(rises)

def align(x, sr):
    """The first sample of the opening pulse train and of the closing one: ten rises two frames apart."""
    rises, two = pulse_rises(x, sr), 2 * FRAME * sr
    train = lambda i: len(rises) - i >= 10 and np.all(np.abs(np.diff(rises[i:i + 10]) - two) < 0.15 * two / 2)
    first = next(i for i in range(len(rises)) if train(i))
    last = next(i for i in range(len(rises) - 10, first, -1) if train(i))
    return rises[first], rises[last]

class Run:
    """A recording cut by its own frame length."""
    def __init__(self, path):
        self.sr, self.x = load(path)
        self.start, end = align(self.x, self.sr)
        self.frame = (end - self.start) / 3500

    def spectrum(self, frame, frames, side=None):
        n = int(frames * self.frame)
        a = int(self.start + frame * self.frame)
        seg = self.x[a:a + n].mean(axis=1) if side is None else self.x[a:a + n, side]
        return np.abs(np.fft.rfft((seg - seg.mean()) * np.hanning(n))) ** 2

    def notes(self, side):
        """The 96 FM notes' spectra, twelve frames from the note's third."""
        return [self.spectrum(40 + 20 * k + 2, 12, side) for k in range(96)], self.sr / int(12 * self.frame)

    def tones(self):
        """The 40 PSG tones' spectra, fourteen frames from the tone's fourth."""
        return [self.spectrum(1960 + 20 * j + 3, 14) for j in range(40)], self.sr / int(14 * self.frame)

def energy(sp, i):
    return sp[i - 2:i + 3].sum()

def partial_levels(ref, rec, side):
    """For each partial of the reference within 40 dB of its note's strongest: its frequency, the recording's level
    over the reference's there in dB, and the partial's strength in the reference against the note's strongest."""
    (pr, df), (pc, _) = ref.notes(side), rec.notes(side)
    out = []
    for k in range(96):
        sp, c = pr[k], pc[k]
        top = sp.max()
        for i in range(3, len(sp) - 3):
            if sp[i] > top * 1e-4 and sp[i] == sp[i - 2:i + 3].max():
                j = i - 3 + int(np.argmax(c[i - 3:i + 4]))
                out.append((i * df, 10 * np.log10(energy(c, j) / energy(sp, i)), 10 * np.log10(sp[i] / top)))
    return out

def psg_levels(ref, rec, top=12000):
    """Each PSG tone's fundamental: its frequency and the recording's level over the reference's."""
    (tr, df), (tc, _) = ref.tones(), rec.tones()
    out = []
    for j in range(40):
        i = int(np.argmax(tr[j][5:])) + 5
        if i * df <= top:
            k = i - 3 + int(np.argmax(tc[j][i - 3:i + 4]))
            out.append((i * df, 10 * np.log10(energy(tc[j], k) / energy(tr[j], i))))
    return out

def low_pass_db(f, corner):
    return -10 * np.log10(1 + (f / corner) ** 2)

def high_pass_db(f, corner):
    return -10 * np.log10(1 + (corner / f) ** 2)

def second_order_db(f, corner, q):
    w = f / corner
    return -10 * np.log10((1 - w * w) ** 2 + (w / q) ** 2)

def name(path):
    return path.split("/")[-1][:24].ljust(24)

def cmd_align(paths):
    for p in paths:
        sr, x = load(p)
        a, b = align(x, sr)
        print(name(p), sr, "Hz; pulse trains at samples", a, "and", b, "; %.3f frames between" % ((b - a) / sr / FRAME))

BANDS = [20, 40, 80, 160, 320, 640, 1000, 1500, 2000, 3000, 4000, 6000, 8000, 12000, 16000, 20000]

def cmd_response(ref, recs):
    ref = Run(ref)
    print("bands from", BANDS)
    for p in recs:
        rec = Run(p)
        for side in (0, 1):
            pts = partial_levels(ref, rec, side)
            row = []
            for lo, hi in zip(BANDS, BANDS[1:]):
                v = [d for f, d, s in pts if lo <= f < hi and s > -25]
                row.append("%6.1f" % np.median(v) if len(v) > 3 else "    --")
            print(name(p), "LR"[side], " ".join(row))

def cmd_fit(ref, recs, held, second):
    from scipy.optimize import least_squares
    ref = Run(ref)
    for p in recs:
        rec, out = Run(p), []
        for side in (0, 1):
            pts = [(f, d) for f, d, s in partial_levels(ref, rec, side) if s > -20 and 25 < f < (14000 if second == 2 else 16000)]
            f, d = np.array([q[0] for q in pts]), np.array([q[1] for q in pts])
            droop = 20 * np.log10(np.abs(np.sinc(f / YM_RATE))) if held else 0
            if second == 3:
                model = lambda v: v[0] + second_order_db(f, v[1], v[2]) + low_pass_db(f, v[3]) + high_pass_db(f, abs(v[4]) + 1e-9)
                r = least_squares(lambda v: model(v) - d, [0, 6000, 0.7, 9000, 10], loss="soft_l1", f_scale=0.5)
                out.append("%s: gain %+.2f dB, corner %.0f Hz, Q %.2f, low-pass %.0f Hz, high-pass %.1f Hz, rms %.2f dB" % ("LR"[side], r.x[0], r.x[1], abs(r.x[2]), r.x[3], abs(r.x[4]), np.sqrt(np.mean((model(r.x) - d) ** 2))))
            elif second:
                model = lambda v: v[0] + second_order_db(f, v[1], v[2]) + high_pass_db(f, abs(v[3]) + 1e-9)
                r = least_squares(lambda v: model(v) - d, [0, 8000, 0.7, 20], loss="soft_l1", f_scale=0.5)
                out.append("%s: gain %+.2f dB, corner %.0f Hz, Q %.2f, high-pass %.1f Hz, rms %.2f dB" % ("LR"[side], r.x[0], r.x[1], abs(r.x[2]), abs(r.x[3]), np.sqrt(np.mean((model(r.x) - d) ** 2))))
            else:
                model = lambda v: v[0] + low_pass_db(f, v[1]) + high_pass_db(f, abs(v[2]) + 1e-9) - droop
                r = least_squares(lambda v: model(v) - d, [0, 3000, 20], loss="soft_l1", f_scale=0.5)
                out.append("%s: gain %+.2f dB, low-pass %.0f Hz, high-pass %.1f Hz, rms %.2f dB" % ("LR"[side], r.x[0], r.x[1], abs(r.x[2]), np.sqrt(np.mean((model(r.x) - d) ** 2))))
        print(name(p), " | ".join(out))

def cmd_level(ref, recs):
    ref = Run(ref)
    for p in recs:
        rec, row, gains = Run(p), [], []
        for side in (0, 1):
            d = np.array([d for f, d, s in partial_levels(ref, rec, side) if s > -20 and 40 < f < 16000])
            g = np.median(d)
            gains.append(g)
            row.append("%s: gain %+.2f dB, rms %.2f dB, %d%% within 1 dB" % ("LR"[side], g, np.sqrt(np.mean((d - g) ** 2)), 100 * np.mean(np.abs(d - g) < 1)))
        psg = [d for f, d in psg_levels(ref, rec)]
        print(name(p), " | ".join(row), "| PSG over FM %+.2f dB (spread %.2f)" % (np.median(psg) - np.mean(gains), np.std(psg)))

def cmd_ladder(with_ladder, without, pairs):
    """The partials 30 times stronger with the ladder than without, 60 Hz to 8 kHz: the recording's level over each
    reference's there, the note's strongest partial fixing the gain and the low-pass taken out."""
    lad, plain = Run(with_ladder), Run(without)
    for p, corner in zip(pairs[::2], map(float, pairs[1::2])):
        rec = Run(p)
        for side in (0, 1):
            (pl, df), (pp, _), (pr, _) = lad.notes(side), plain.notes(side), rec.notes(side)
            over, over_plain = [], []
            for k in range(96):
                a, b, c = pl[k], pp[k], pr[k]
                i0 = int(np.argmax(a))
                gain = 10 * np.log10(energy(c, i0) / energy(a, i0)) - low_pass_db(i0 * df, corner)
                for i in range(5, len(a) - 5):
                    f = i * df
                    if 60 < f < 8000 and a[i] == a[i - 2:i + 3].max() and a[i] > a.max() * 10 ** -4.5 and energy(a, i) > 30 * energy(b, i):
                        over.append(10 * np.log10(energy(c, i) / energy(a, i)) - gain - low_pass_db(f, corner))
                        over_plain.append(10 * np.log10(energy(c, i) / energy(b, i)) - gain - low_pass_db(f, corner))
            print(name(p), "LR"[side], "%d partials; the recording over the ladder's: median %+.1f dB, quartiles %+.1f and %+.1f; over the plain run's: median %+.1f dB"
                  % (len(over), np.median(over), np.percentile(over, 25), np.percentile(over, 75), np.median(over_plain)))

def cmd_design():
    """sound.rs's coefficients: least-squares fits in decibels from 20 Hz to 20 kHz at 48 kHz, as 24-bit fractions."""
    from scipy.optimize import least_squares
    fs = 48000.0
    f = np.geomspace(20, 20000, 400)
    z = np.exp(-2j * np.pi * f / fs)
    db = lambda h: 20 * np.log10(np.abs(h))
    biquad = lambda p: (p[0] + p[1] * z + p[2] * z * z) / (1 - p[3] * z - p[4] * z * z)
    for label, target, start in (
        ("model 1, low-pass 3,216 Hz", low_pass_db(f, 3216.0), [0.30, 0.05, 0.0, 0.65, 0.0]),
        ("model 2, low-pass 5,805 Hz, Q 1.16", second_order_db(f, 5805.0, 1.16), [0.09, 0.22, -0.02, 1.03, -0.32]),
        ("model 2, low-pass 4,111 Hz", low_pass_db(f, 4111.0), [0.30, 0.05, 0.0, 0.65, 0.0]),
    ):
        p = least_squares(lambda p: db(biquad(p)) - target, start).x
        p[:3] /= sum(p[:3]) / (1 - p[3] - p[4])
        q = [round(v * (1 << 24)) for v in p]
        print(label, "b", q[:3], "a", q[3:], "largest error %.3f dB" % np.abs(db(biquad(np.array(q) / (1 << 24))) - target).max())
    for hz in (23, 16):
        print("high-pass", hz, "Hz: pole", round(np.exp(-2 * np.pi * hz / fs) * (1 << 24)))
    f = np.linspace(20, 20000, 500)
    w = 2 * np.pi * f / fs
    target = -20 * np.log10(np.abs(np.sinc(f / YM_RATE)))
    gain = lambda c: 1 - 2 * sum(c) + 2 * sum(c[k] * np.cos((k + 1) * w) for k in range(3))
    c = least_squares(lambda c: 20 * np.log10(gain(c)) - target, [-0.07, 0.01, 0.0]).x
    q = [round(v * (1 << 24)) for v in c]
    print("unhold: centre", (1 << 24) - 2 * sum(q), "then", q, "largest error %.3f dB" % np.abs(20 * np.log10(gain(np.array(q) / (1 << 24))) - target).max())

def cmd_dac_steps(path):
    """Kabuto's diagram of a console's DAC levels (two panels of 256 levels, `$2C` bit 3 clear and set, two pixels a
    level): each step that is far from the panel's median step, as (level, step over the median)."""
    from PIL import Image
    import statistics
    im = Image.open(path).convert("RGB")
    px = im.load()
    for panel in (0, 1):
        ys = []
        for x in range(panel * 512, panel * 512 + 512, 2):
            col = [y for y in range(im.size[1]) if sum(px[x, y]) < 120] or [y for y in range(im.size[1]) if sum(px[x + 1, y]) < 120]
            ys.append(statistics.mean(col))
        steps = [ys[i] - ys[i + 1] for i in range(255)]
        med = statistics.median(steps)
        print("panel", panel, "median step", med, "px;", [(i, round(s / med, 2)) for i, s in enumerate(steps) if abs(s / med - 1) > 0.6])

if __name__ == "__main__":
    what, args = sys.argv[1], sys.argv[2:]
    if what == "align": cmd_align(args)
    elif what == "response": cmd_response(args[0], args[1:])
    elif what in ("fit", "fit2", "fit3"): cmd_fit(args[0], [a for a in args[1:] if a != "--held"], "--held" in args, {"fit": 0, "fit2": 2, "fit3": 3}[what])
    elif what == "level": cmd_level(args[0], args[1:])
    elif what == "ladder": cmd_ladder(args[0], args[1], args[2:])
    elif what == "design": cmd_design()
    elif what == "dac-steps": cmd_dac_steps(args[0])
    else: sys.exit(__doc__)
