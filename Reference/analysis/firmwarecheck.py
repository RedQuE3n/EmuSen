#!/usr/bin/env python3
"""How much of an original firmware image an open replacement repeats, in numbers and offsets only.

See EmuSen_Firmware.md §0 for the policy it serves and EmuSen_Debugging_Tools_Reference_v5.md §3.61 for the method.
Every core carries an open replacement for its firmware, written from documents, and this is the measurement each
replacement's record cites.

    firmwarecheck.py <replacement> <original> [--window START:END] [--max-equal PERCENT] [--max-run N] [--min-run N] [--limit N]
                     [--forced FORMULA] [--ignore-fill N] [--control CONTROL] [--explained FILE]

What it reports, over the window (both images, offsets START to END, END exclusive; the whole of each by default):

- **Equal at the same offset**: the count of offsets where the two images hold the same byte, and its share of the
  offsets both cover; with the longest such run and where it starts.
- **The longest common run**: the longest byte string found in both at any offsets, with its offset in each,
  from a suffix automaton of the original (exact, linear in the two sizes).
- **Common runs**: every maximal run of at least --min-run bytes (default 4) the two share at any offsets, longest
  first, at most --limit of them listed (default 50) with the count of all.
- **The verdict**: PASS when the share equal at the same offset is at most --max-equal (default 25) and no common
  run is longer than --max-run (default 6); FAIL otherwise.

With --forced FORMULA, for a replacement made of derived tables (VenusRT_DspHle.md §5.3): FORMULA is the same tables
written by an independent generator from the formulas alone. The tool finds every common run between FORMULA and the
original, and calls an original byte forced where such a run covers it. It reports the **residual**: the parts of
the replacement's common runs with the original that no formula run covers. It also reports the **formula
coverage**, the share of the original's bytes that some formula run reproduces. The verdict is then PASS when no
residual run is longer than --max-run. The equal-share test does not apply, since a correct table must equal the
original's where the original's is that formula.

With --ignore-fill N, for a replacement that is code of its own size rather than a table: a stretch of N or more of
one byte in the replacement is fill (zeroed memory, padding to the image's size), and its offsets are left out of every
measure; no aligned or common run is counted across or inside it. A shared string of zeros says nothing about where a
program came from (EmuSen_Firmware.md §0, decided 2026-10-11).

With --control CONTROL, for a replacement that is compiled code: CONTROL is code of the same processor and about the
same size that has nothing to do with the original (EmuSen_Firmware.md §0 names it for each core). The same measures
are taken for it against the original, and the verdict is then PASS when the replacement's share equal at the same
offset is at most --max-equal, it shares no more common runs of --min-run bytes or more than the control does, and every
common run longer than the control's longest lies inside a pattern of --explained FILE: lines of the replacement's own
32-bit words in hex, little-endian, each line a pattern, '#' starting a comment that gives the reason. Compiled code
shares register saves and returns with any other compiled code, so a literal run limit cannot tell copying from
compiling; the control measures what compiling alone shares (decided 2026-10-11).

The exit code is the verdict: 0 PASS, 1 FAIL, 2 a usage or file error. Nothing of either image's bytes is printed,
only counts and offsets, so a writer under a clean-room protocol can run it against an image they may not read.
"""
import argparse
import sys

# Pairs of k-gram matches examined while listing runs; past it the list is marked incomplete, the longest stays exact.
PAIR_BUDGET = 5_000_000


def window(data, start, end):
    end = len(data) if end is None else min(end, len(data))
    return data[start:end] if start < end else b""


def fill_mask(data, n):
    """1 at every offset of a stretch of n or more of one byte, 0 elsewhere; all 0 when n is 0."""
    mask = bytearray(len(data))
    if n <= 0:
        return mask
    i = 0
    while i < len(data):
        j = i
        while j < len(data) and data[j] == data[i]:
            j += 1
        if j - i >= n:
            mask[i:j] = b"\x01" * (j - i)
        i = j
    return mask


def aligned(a, b, mask=None):
    """(equal offsets, compared offsets, longest aligned run, its start) of a and b laid over each other, fill left out."""
    n = min(len(a), len(b))
    same = best = run = compared = 0
    at = -1
    for i in range(n):
        if mask and mask[i]:
            run = 0
            continue
        compared += 1
        if a[i] == b[i]:
            same += 1
            run += 1
            if run > best:
                best, at = run, i - run + 1
        else:
            run = 0
    return same, compared, best, at


def longest_common(a, b, mask=None):
    """(length, offset in a, offset in b) of the longest string both hold, by a suffix automaton of b."""
    link, length, nxt, end = [-1], [0], [{}], [0]
    last = 0
    for pos, c in enumerate(b):
        cur = len(length)
        length.append(length[last] + 1)
        link.append(-1)
        nxt.append({})
        end.append(pos)
        p = last
        while p != -1 and c not in nxt[p]:
            nxt[p][c] = cur
            p = link[p]
        if p == -1:
            link[cur] = 0
        else:
            q = nxt[p][c]
            if length[p] + 1 == length[q]:
                link[cur] = q
            else:
                clone = len(length)
                length.append(length[p] + 1)
                link.append(link[q])
                nxt.append(dict(nxt[q]))
                end.append(end[q])
                while p != -1 and nxt[p].get(c) == q:
                    nxt[p][c] = clone
                    p = link[p]
                link[q] = link[cur] = clone
        last = cur
    state = matched = 0
    best = (0, -1, -1)
    for i, c in enumerate(a):
        if mask and mask[i]:
            state = matched = 0
            continue
        while state and c not in nxt[state]:
            state = link[state]
            matched = length[state]
        if c in nxt[state]:
            state = nxt[state][c]
            matched += 1
        if matched > best[0]:
            best = (matched, i - matched + 1, end[state] - matched + 1)
    return best


def common_runs(a, b, k, mask=None):
    """Every maximal common run of at least k bytes as (length, offset in a, offset in b), and whether all were seen."""
    if k <= 0 or len(a) < k or len(b) < k:
        return [], True
    where = {}
    for j in range(len(b) - k + 1):
        where.setdefault(b[j:j + k], []).append(j)
    runs, pairs = [], 0
    for i in range(len(a) - k + 1):
        if mask and any(mask[i:i + k]):
            continue
        for j in where.get(a[i:i + k], ()):
            pairs += 1
            if pairs > PAIR_BUDGET:
                return runs, False
            if i and j and a[i - 1] == b[j - 1] and not (mask and mask[i - 1]):
                continue
            n = k
            while i + n < len(a) and j + n < len(b) and a[i + n] == b[j + n] and not (mask and mask[i + n]):
                n += 1
            runs.append((n, i, j))
    runs.sort(key=lambda r: (-r[0], r[1], r[2]))
    return runs, True


def explained_patterns(path):
    """The byte strings of FILE's lines: hex 32-bit words, little-endian, '#' starting a comment."""
    patterns = []
    with open(path) as f:
        for line in f:
            words = line.split("#", 1)[0].split()
            if words:
                patterns.append(b"".join(int(w, 16).to_bytes(4, "little") for w in words))
    return patterns


def span(text):
    start, _, end = text.partition(":")
    return int(start or "0", 0), (int(end, 0) if end else None)


def main(argv=None, out=sys.stdout):
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("replacement")
    p.add_argument("original")
    p.add_argument("--window", type=span, default=(0, None), metavar="START:END")
    p.add_argument("--max-equal", type=float, default=25.0, metavar="PERCENT")
    p.add_argument("--max-run", type=int, default=6, metavar="N")
    p.add_argument("--min-run", type=int, default=4, metavar="N")
    p.add_argument("--limit", type=int, default=50, metavar="N")
    p.add_argument("--forced", metavar="FORMULA")
    p.add_argument("--ignore-fill", type=int, default=0, metavar="N")
    p.add_argument("--control", metavar="CONTROL")
    p.add_argument("--explained", metavar="FILE")
    args = p.parse_args(argv)
    try:
        with open(args.replacement, "rb") as f:
            ours_whole = f.read()
        with open(args.original, "rb") as f:
            theirs_whole = f.read()
        control_whole = patterns = None
        if args.control:
            with open(args.control, "rb") as f:
                control_whole = f.read()
            patterns = explained_patterns(args.explained) if args.explained else []
        formula_whole = None
        if args.forced:
            with open(args.forced, "rb") as f:
                formula_whole = f.read()
    except OSError as e:
        print(f"firmwarecheck: {e}", file=sys.stderr)
        return 2
    start, end = args.window
    ours, theirs = window(ours_whole, start, end), window(theirs_whole, start, end)

    mask = fill_mask(ours, args.ignore_fill)
    same, compared, run, run_at = aligned(ours, theirs, mask)
    share = 100.0 * same / compared if compared else 0.0
    longest, in_ours, in_theirs = longest_common(ours, theirs, mask)
    runs, complete = common_runs(ours, theirs, args.min_run, mask)

    print(f"sizes: replacement {len(ours_whole)} bytes, original {len(theirs_whole)} bytes; window {start}:{start + max(len(ours), len(theirs))}", file=out)
    if args.ignore_fill:
        print(f"fill left out: {sum(mask)} bytes in stretches of {args.ignore_fill}+ of one byte", file=out)
    print(f"equal at the same offset: {same} of {compared} ({share:.1f}%); longest aligned run {run}" + (f" at {start + run_at}" if run else ""), file=out)
    print(f"longest common run: {longest}" + (f" bytes, replacement offset {start + in_ours}, original offset {start + in_theirs}" if longest else ""), file=out)
    print(f"common runs of {args.min_run}+ bytes: {len(runs)}{'' if complete else ' or more (listing stopped at the pair budget)'}", file=out)
    for n, i, j in runs[:args.limit]:
        print(f"  {n} bytes: replacement offset {start + i}, original offset {start + j}", file=out)
    if len(runs) > args.limit:
        print(f"  ... {len(runs) - args.limit} more", file=out)
    if formula_whole is not None:
        formula = window(formula_whole, start, end)
        forced_runs, forced_complete = common_runs(formula, theirs, args.min_run)
        covered = bytearray(len(theirs))
        for n, _, j in forced_runs:
            covered[j:j + n] = b"\x01" * n
        residual = []
        for n, i, j in runs:
            k = 0
            while k < n:
                if covered[j + k]:
                    k += 1
                    continue
                m = k
                while m < n and not covered[j + m]:
                    m += 1
                residual.append((m - k, i + k, j + k))
                k = m
        residual.sort(key=lambda r: (-r[0], r[1], r[2]))
        coverage = 100.0 * sum(covered) / len(theirs) if theirs else 0.0
        print(f"formula runs of {args.min_run}+ bytes with the original: {len(forced_runs)}{'' if forced_complete else ' or more'}; formula coverage {sum(covered)} of {len(theirs)} bytes ({coverage:.1f}%)", file=out)
        print(f"residual runs not covered by a formula run: {len(residual)}; longest {residual[0][0] if residual else 0}", file=out)
        for n, i, j in residual[:args.limit]:
            print(f"  {n} bytes: replacement offset {start + i}, original offset {start + j}", file=out)
        passed = (not residual or residual[0][0] <= args.max_run) and complete and forced_complete
        print(f"verdict: {'PASS' if passed else 'FAIL'} (forced: no residual run over {args.max_run})", file=out)
        return 0 if passed else 1
    if control_whole is not None:
        control = window(control_whole, start, end)
        cmask = fill_mask(control, args.ignore_fill)
        c_same, c_compared, _, _ = aligned(control, theirs, cmask)
        c_share = 100.0 * c_same / c_compared if c_compared else 0.0
        c_longest = longest_common(control, theirs, cmask)[0]
        c_runs, c_complete = common_runs(control, theirs, args.min_run, cmask)
        print(f"control: {len(control_whole)} bytes; equal at the same offset {c_same} of {c_compared} ({c_share:.1f}%); longest common run {c_longest}; common runs of {args.min_run}+ bytes: {len(c_runs)}{'' if c_complete else ' or more'}", file=out)
        over = [(n, i) for n, i, _ in runs if n > c_longest]
        unexplained = sorted({(n, i) for n, i in over if not any(ours[i:i + n] in pat for pat in patterns)}, key=lambda r: (-r[0], r[1]))
        print(f"runs longer than the control's longest: {len(over)}; unexplained: {len(unexplained)}", file=out)
        for n, i in unexplained[:args.limit]:
            print(f"  {n} bytes: replacement offset {start + i}", file=out)
        passed = share <= args.max_equal and len(runs) <= len(c_runs) and not unexplained and complete and c_complete
        print(f"verdict: {'PASS' if passed else 'FAIL'} (control: at most {args.max_equal:g}% equal at the same offset, no more runs of {args.min_run}+ than the control, every run past its longest explained)", file=out)
        return 0 if passed else 1
    passed = share <= args.max_equal and longest <= args.max_run
    print(f"verdict: {'PASS' if passed else 'FAIL'} (at most {args.max_equal:g}% equal at the same offset, no run over {args.max_run})", file=out)
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
