#!/usr/bin/env python3
"""Audio compared as a feature stream, because samples never compare directly.

See EmuSen_Debugging_Tools_Reference_v5.md §3.51 and Mercury_HardwareTests.md §1.

Why not compare samples
-----------------------
`screen.py` can compare pixel for pixel: two emulators drawing the same frame
either agree on a pixel or one of them is wrong. Audio has no equivalent. Two
correct emulators produce different sample streams because they mix at different
rates, start their resampler at a different phase, and round differently at the
DAC. A byte-wise diff of two correct captures reports a wall of differences and
teaches nothing, which is the failure mode this module exists to avoid.

What is compared instead is a **feature stream**: the signal reduced to
per-window measurements that survive resampling and phase, and that still change
when the emulation is wrong. Three of them, chosen because each fails
differently:

- **RMS envelope**, in dB per window. Catches a channel that never starts, a
  note that holds when it should decay, a mix that is too loud or silent.
- **Onsets** - windows where the envelope jumps. Catches a sound driver running
  at the wrong tempo, or notes that are missing entirely.
- **Zero-crossing rate**. A cheap pitch proxy: catches a channel playing at the
  wrong frequency while the envelope looks right.

None of these is a proof of correctness. They are a *tripwire*, and the verdict
they produce is "these two captures do not describe the same performance", which
is a much weaker and much more defensible claim than sample equality. For an
actual assertion about sound hardware, use the test corpus instead - it needs no
reference at all (`Mercury_HardwareTests.md`).

On tolerances
-------------
Every threshold here is a judgement, not a measurement, and they are named
constants so that a later run can argue with them. The defaults are deliberately
loose: this is a screen for gross divergence, and a comparator that cries wolf
gets switched off.
"""
import math
import struct
import sys

# 20 ms at 44.1 kHz. Short enough to place a note, long enough that phase
# differences between two resamplers average out inside one window.
WINDOW_MS = 20.0

# Below this a window is silence, and its dB value would be meaningless.
SILENCE_FLOOR_DB = -70.0

# Two envelopes may differ by this much per window and still be "the same".
ENVELOPE_TOLERANCE_DB = 6.0

# A window whose RMS rises by this much over the previous one starts a note.
ONSET_RISE_DB = 9.0

# Onsets may be this many windows apart and still count as the same event.
ONSET_ALIGN_WINDOWS = 2

# Fraction of windows allowed to exceed ENVELOPE_TOLERANCE_DB before failing.
ENVELOPE_FAIL_FRACTION = 0.10

# Zero-crossing rates within this ratio of each other are the same pitch.
ZCR_TOLERANCE_RATIO = 0.25

# How far either way find_lag() searches: 60 windows is 1.2 seconds.
LAG_SEARCH_WINDOWS = 60

# A candidate lag needs at least this much overlap to be scored at all.
MIN_LAG_OVERLAP = 100


def read_wav(path):
    """(samples, channels, rate) with samples as a flat interleaved list of ints.

    Walks the chunk list rather than assuming the data starts at byte 44, for the
    same reason the Rust side does: a WAV written by a decoder carries a LIST
    chunk and a naive reader silently reads metadata as audio.
    """
    with open(path, "rb") as handle:
        raw = handle.read()

    if len(raw) < 12 or raw[0:4] != b"RIFF" or raw[8:12] != b"WAVE":
        raise ValueError(f"{path} is not a RIFF/WAVE file")

    channels = 0
    rate = 0
    bits = 0
    data = None

    at = 12
    while at + 8 <= len(raw):
        chunk = raw[at:at + 4]
        size = struct.unpack_from("<I", raw, at + 4)[0]
        body = at + 8

        if chunk == b"fmt " and size >= 16:
            _, channels, rate, _, _, bits = struct.unpack_from("<HHIIHH", raw, body)
        elif chunk == b"data":
            data = raw[body:body + size]

        at = body + size + (size & 1)

    if data is None:
        raise ValueError(f"{path} has no data chunk")
    if bits != 16:
        raise ValueError(f"{path} is {bits}-bit; only 16-bit PCM is read here")

    samples = list(struct.unpack(f"<{len(data) // 2}h", data[:len(data) // 2 * 2]))
    return samples, max(1, channels), rate


def to_mono(samples, channels):
    """Downmix by averaging. Panning differences are not what this module judges."""
    if channels <= 1:
        return list(samples)

    return [
        sum(samples[i:i + channels]) / channels
        for i in range(0, len(samples) - channels + 1, channels)
    ]


def _db(value):
    return SILENCE_FLOOR_DB if value <= 1e-9 else max(SILENCE_FLOOR_DB, 20.0 * math.log10(value / 32768.0))


def envelope(mono, rate, window_ms=WINDOW_MS):
    """RMS per window, in dB relative to full scale."""
    size = max(1, int(rate * window_ms / 1000.0))
    out = []

    for start in range(0, len(mono) - size + 1, size):
        window = mono[start:start + size]
        mean_square = sum(s * s for s in window) / size
        out.append(_db(math.sqrt(mean_square)))

    return out


def zero_crossing_rates(mono, rate, window_ms=WINDOW_MS):
    """Crossings per second per window - a pitch proxy that survives resampling."""
    size = max(1, int(rate * window_ms / 1000.0))
    out = []

    for start in range(0, len(mono) - size + 1, size):
        window = mono[start:start + size]
        crossings = sum(
            1 for i in range(1, size)
            if (window[i - 1] < 0) != (window[i] < 0)
        )
        out.append(crossings * rate / size)

    return out


def onsets(env, rise_db=ONSET_RISE_DB):
    """Indices of windows where the envelope jumps - where a note starts."""
    return [i for i in range(1, len(env)) if env[i] - env[i - 1] >= rise_db]


def _align(a, b):
    """Trim to the shorter. A capture that ran longer is not a difference in kind."""
    n = min(len(a), len(b))
    return a[:n], b[:n], n


def gain_offset(left, right):
    """The constant dB difference between two envelopes, or 0.0 if there is none.

    Two emulators do not agree on how much of a 16-bit range a Game Boy should
    use, and neither is wrong: the console's output level is analogue and the
    scaling is a frontend convention. Measured against gambatte, Mercury runs
    about 9 dB hotter across every ROM tried (`Mercury_HardwareTests.md` §6.1).

    Subtracting it is the same move `screen.py` makes for a whole-image vertical
    shift - find the systematic offset, report it, and compare what is left.
    Without this the comparator reports every ROM as divergent and stops being
    read, which is the failure mode this module was written to avoid.
    """
    a, b, n = _align(left, right)
    deltas = [
        a[i] - b[i]
        for i in range(n)
        if a[i] > SILENCE_FLOOR_DB + 1.0 and b[i] > SILENCE_FLOOR_DB + 1.0
    ]

    if not deltas:
        return 0.0

    deltas.sort()
    middle = len(deltas) // 2
    return deltas[middle] if len(deltas) % 2 else (deltas[middle - 1] + deltas[middle]) / 2.0


def find_lag(left, right, limit=LAG_SEARCH_WINDOWS):
    """(lag, mean abs dB) at the offset where two envelopes agree best.

    Distinguishes the two ways an envelope comparison fails, which otherwise look
    identical in the totals: a sound driver running at the wrong *tempo* agrees
    well at some non-zero lag, and a mixer at the wrong *level* does not improve
    at any lag. The first Mercury-versus-gambatte run was the second kind - best
    lag 0 to 2 windows and no meaningful improvement - which is what ruled out the
    documented four-frame boot offset as the explanation
    (`Mercury_HardwareTests.md` §6.1).

    The same idea as `screen.py`'s vertical-shift search, one dimension down.
    """
    if not left or not right:
        return 0, SILENCE_FLOOR_DB

    # Scaled to the input rather than fixed: a flat 100-window floor rejects every
    # candidate on a short capture, and the empty result then reads as 0.0 dB -
    # which looks like perfect agreement rather than like no answer.
    needed = min(MIN_LAG_OVERLAP, max(1, min(len(left), len(right)) // 2))

    gain = gain_offset(left, right)
    best = (0, float("inf"))

    for lag in range(-limit, limit + 1):
        deltas = [
            abs((left[i] - gain) - right[i + lag])
            for i in range(len(left))
            if 0 <= i + lag < len(right)
        ]

        if len(deltas) < needed:
            continue

        score = sum(deltas) / len(deltas)
        if score < best[1]:
            best = (lag, score)

    return best


def compare_envelopes(left, right, tolerance_db=ENVELOPE_TOLERANCE_DB, normalise=True):
    a, b, n = _align(left, right)
    if n == 0:
        return {"windows": 0, "exceeded": 0, "fraction": 0.0,
                "worst_db": 0.0, "worst_window": -1, "gain_db": 0.0}

    # Removed before comparing, and reported, so a level convention is never a verdict.
    gain = gain_offset(a, b) if normalise else 0.0

    worst = 0.0
    worst_at = -1
    exceeded = 0

    for i in range(n):
        # Silence on both sides is agreement, whatever the gain would have made of it.
        if a[i] <= SILENCE_FLOOR_DB + 1.0 and b[i] <= SILENCE_FLOOR_DB + 1.0:
            continue

        delta = abs((a[i] - gain) - b[i])
        if delta > tolerance_db:
            exceeded += 1
        if delta > worst:
            worst, worst_at = delta, i

    return {
        "windows": n,
        "exceeded": exceeded,
        "fraction": exceeded / n,
        "worst_db": worst,
        "worst_window": worst_at,
        "gain_db": gain,
    }


def compare_onsets(left, right, slack=ONSET_ALIGN_WINDOWS):
    """How many onsets on each side have no partner within `slack` windows."""
    unmatched_left = sum(1 for i in left if not any(abs(i - j) <= slack for j in right))
    unmatched_right = sum(1 for j in right if not any(abs(i - j) <= slack for i in left))

    total = max(1, max(len(left), len(right)))
    return {
        "left": len(left),
        "right": len(right),
        "unmatched_left": unmatched_left,
        "unmatched_right": unmatched_right,
        "fraction": (unmatched_left + unmatched_right) / (2 * total),
    }


def compare_pitch(left, right, floor_left, floor_right, ratio=ZCR_TOLERANCE_RATIO):
    """Zero-crossing agreement, over the windows where both sides have signal."""
    a, b, n = _align(left, right)
    fa, fb, _ = _align(floor_left, floor_right)

    compared = 0
    disagreed = 0

    for i in range(n):
        # Silence has no pitch, so comparing it would only add noise to the verdict.
        if fa[i] <= SILENCE_FLOOR_DB + 1.0 or fb[i] <= SILENCE_FLOOR_DB + 1.0:
            continue

        compared += 1
        louder = max(a[i], b[i])
        if louder > 0 and abs(a[i] - b[i]) / louder > ratio:
            disagreed += 1

    return {
        "compared": compared,
        "disagreed": disagreed,
        "fraction": disagreed / compared if compared else 0.0,
    }


def compare(path_a, path_b):
    """The whole verdict for two WAV files."""
    samples_a, channels_a, rate_a = read_wav(path_a)
    samples_b, channels_b, rate_b = read_wav(path_b)

    mono_a = to_mono(samples_a, channels_a)
    mono_b = to_mono(samples_b, channels_b)

    env_a = envelope(mono_a, rate_a)
    env_b = envelope(mono_b, rate_b)

    zcr_a = zero_crossing_rates(mono_a, rate_a)
    zcr_b = zero_crossing_rates(mono_b, rate_b)

    result = {
        "rate_a": rate_a,
        "rate_b": rate_b,
        "seconds_a": len(mono_a) / rate_a if rate_a else 0.0,
        "seconds_b": len(mono_b) / rate_b if rate_b else 0.0,
        "envelope": compare_envelopes(env_a, env_b),
        "onsets": compare_onsets(onsets(env_a), onsets(env_b)),
        "pitch": compare_pitch(zcr_a, zcr_b, env_a, env_b),
    }

    lag, lag_score = find_lag(env_a, env_b)
    result["lag"] = {"windows": lag, "seconds": lag * WINDOW_MS / 1000.0, "mean_abs_db": lag_score}

    result["agrees"] = (
        result["envelope"]["fraction"] <= ENVELOPE_FAIL_FRACTION
        and result["onsets"]["fraction"] <= ENVELOPE_FAIL_FRACTION
        and result["pitch"]["fraction"] <= ENVELOPE_FAIL_FRACTION
    )
    return result


def format_report(result):
    lines = [
        f"rate      {result['rate_a']} Hz vs {result['rate_b']} Hz",
        f"length    {result['seconds_a']:.2f}s vs {result['seconds_b']:.2f}s",
        "gain      {gain_db:+.1f} dB systematic offset, removed before comparing".format(**result["envelope"]),
        "envelope  {exceeded}/{windows} windows over tolerance ({fraction:.1%}), "
        "worst {worst_db:.1f} dB at window {worst_window}".format(**result["envelope"]),
        "onsets    {left} vs {right}, {unmatched_left}+{unmatched_right} unmatched "
        "({fraction:.1%})".format(**result["onsets"]),
        "pitch     {disagreed}/{compared} windows disagree ({fraction:.1%})".format(**result["pitch"]),
        "lag       best at {windows:+d} windows ({seconds:+.2f}s), "
        "mean {mean_abs_db:.1f} dB there".format(**result["lag"]),
        f"verdict   {'agrees' if result['agrees'] else 'DIVERGES'}",
    ]
    return "\n".join(lines)


def main(argv):
    if len(argv) != 3:
        print("usage: audio.py <a.wav> <b.wav>", file=sys.stderr)
        return 2

    result = compare(argv[1], argv[2])
    print(format_report(result))
    return 0 if result["agrees"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
