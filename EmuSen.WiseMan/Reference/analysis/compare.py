#!/usr/bin/env python3
"""Decide whether two emulators disagreeing about a frame means anything.

    compare.py <oursDir> <theirsDir> <frame> [phaseWindow] [--input F]... [--db P]

Both directories are dump sets - what `--probe ... --sig` writes. They are
ingested into the signature database on first use (see dumpdb.py); nothing else
has to be run first.

    dotnet run --project EmuSen.Pharaoh -c Release -- --probe <rom> ours 0 200 --sig
    ./build-probe.sh mesen && ~/.cache/emusen/probe mesen <rom> theirs 0 200 --sig
    ./compare.py ours theirs 194

The exit code is the verdict: 0 pass, 1 differs, 2 not comparable, 3 pass but
vacuous, 4 diverged.

Why the comparison is shaped this way
-------------------------------------
Three gates run before any difference is measured, because a raw pixel diff
between two emulators answers a question nobody asked.

Gate 1 is identity: if the two sides were not running the same machine - a
different board, a different region - then every pixel below describes that and
not a rendering bug.

Gate 2 is divergence, and it needs the streams aligned first. Two emulators do
not agree on how many frames a boot takes; this project runs two behind the
reference. Comparing frame N to frame N would report a divergence at the first
row for every ROM ever tested, which is a gate that only ever cries wolf. So the
offset that agrees longest is found first, and the divergence is whatever
survives it. Alignment uses only columns that actually move, since a constant
column agrees at every offset and can locate nothing.

Columns are then rated over a leading window rather than the whole run. Rating
over everything is self-defeating: a real divergence half way through drags a
column's rate down and disqualifies the one column that would have revealed it.
Two columns are rated but barred from carrying evidence - `nametable`, because
this project keeps a 4 KB mirror where the reference exposes the physical 2 KB,
and `chr`, whose banking differs in layout without differing in effect.

Gate 3 grades an *agreement*: a frame with six colours, or one colour covering
90% of it, agrees with almost anything, and calling that a pass is the false
positive the gate exists to stop. It only ever downgrades a pass - a frame that
plainly disagreed is never vacuous, however uniform it happens to be.

See EmuSen_Debugging_Tools_Reference_v5.md §3.48 and §3.48a, and §3.49 for the
known-differences dictionary that annotates - never suppresses - a finding.

On the arithmetic (a retired prediction)
----------------------------------------
The percentages here were expected to need special handling. The reasoning was
that .NET's "F" format rounds half away from zero while Python's format() rounds
half to even, so a value landing exactly on a midpoint would print differently -
and such values are reachable, since 384 differing pixels out of 256x240 is
exactly 0.625%. _fixed() was accordingly written to round half away from zero.

That was wrong, and the differential test caught it. Measured against .NET 10
across eighteen values chosen to include every dyadic midpoint at zero, one and
two decimals: .NET's "F" format rounds half to **even**, which is what Python
already does. 0.625 formats as "0.62" on both sides. The deliberate correction
disagreed with .NET on seven of the eighteen; plain format() disagreed on none.

So there is nothing to do, and _fixed() only exists to say so. The prediction is
kept rather than deleted because "the two languages must round differently" is
exactly the kind of plausible claim that would otherwise be re-derived, believed,
and re-implemented. See EmuSen_Stack.md §3.
"""
import os
import sys

import dictionary as dictionary_module
import dumpdb
from dumpset import DumpSet

PASS = 0
DIFFERS = 1
NOT_COMPARABLE = 2
PASS_VACUOUS = 3
DIVERGED = 4

# A near-uniform frame agrees with anything; below these a match is vacuous.
MINIMUM_COLOURS = 6
MAXIMUM_DOMINANT_FRACTION = 0.90

# How close after an input a divergence has to be to be blamed on it.
INPUT_BLAME_WINDOW = 30

# How far apart two emulators may be on the boot sequence before the search gives
# up; the observed offset against the reference is two frames.
MAX_STREAM_OFFSET = 16

# Above this a column tracks the machine; below the lower bound it is telling us
# about the emulators rather than about the game.
STABLE_RATE = 0.95
NEVER_RATE = 0.05

# The share of the run treated as known-good for rating purposes.
CALIBRATION_SHARE = 0.25
MINIMUM_CALIBRATION_FRAMES = 10

# Rated like any other column, but never allowed to carry divergence evidence.
STRUCTURAL = ("nametable", "chr")

STABLE = "stable"
INTERMITTENT = "intermittent"
NEVER = "never"


class Report:

    def __init__(self, verdict=PASS, summary=""):
        self.verdict = verdict
        self.summary = summary
        self.lines = []


class Divergence:

    def __init__(self, frame=None, columns=None, differing=None, frames_compared=0,
                 offset=0, agreed_frames=0, profiles=None, usable=None):
        self.frame = frame
        self.columns = columns or []
        self.differing = differing or []
        self.frames_compared = frames_compared
        self.offset = offset
        self.agreed_frames = agreed_frames
        self.profiles = profiles or []
        self.usable = usable or []

    @property
    def inconclusive(self):
        """Nothing agreed reliably enough to carry a verdict either way."""
        return len(self.usable) == 0


def compare(left, right, frame, phase_window, input_frames, dictionary=None):
    report = Report(PASS)

    # Proven entries only, and they annotate rather than suppress: a known
    # difference that silently removed a finding would hide the day its cause was
    # fixed.
    close_after = dictionary is None
    if dictionary is None:
        dictionary = dictionary_module.open_dictionary()

    try:
        mismatches = list(_identity_mismatches(left, right))
        for warning in _identity_warnings(left, right):
            report.lines.append(f"  [warn]  {warning}")

        if mismatches:
            not_comparable = Report(NOT_COMPARABLE, f"NOT-COMPARABLE: {mismatches[0]}")
            not_comparable.lines.extend(report.lines)
            for mismatch in mismatches:
                not_comparable.lines.append(f"  [ident] {mismatch}")
                subject = mismatch.split(" ")[0]
                for known in dictionary.proven("identity", subject, left.system,
                                               left.backend, right.backend):
                    not_comparable.lines.append(
                        f"          known: {known['slug']} - "
                        f"{known['cause'] or known['claim']}")
            not_comparable.lines.append(
                "  The two sides were not running the same machine, so any pixel")
            not_comparable.lines.append(
                "  difference below would describe that and not a rendering bug.")
            return not_comparable

        divergence = first_divergence(left, right)
        for name, rating, rate in divergence.profiles:
            known = dictionary.proven("column", name, left.system, left.backend, right.backend)
            note = "" if rating == STABLE else (
                f"  known: {known[0]['slug']}" if known else "  UNEXPLAINED")
            report.lines.append(
                f"  [column] {name:<10} {_fixed(rate * 100, 1):>5}% {rating}{note}")

        if not divergence.columns:
            report.lines.append("  [diverge] no shared signature columns; run both sides with --sig")
        elif divergence.inconclusive:
            report.lines.append("  [diverge] INCONCLUSIVE: no column agrees reliably enough to carry a")
            report.lines.append("            verdict, so no divergence frame is claimed. The two")
            report.lines.append("            emulators sample memory at different instants; see §3.48a.")
        elif divergence.frame is not None:
            first = divergence.frame
            blamed = None
            for f in input_frames:
                if first >= f and first - f <= INPUT_BLAME_WINDOW:
                    blamed = f

            where = ",".join(divergence.differing)
            diverged = Report(DIVERGED, (
                f"DIVERGED-AT {first} ({first - blamed} frames after input@{blamed}) in {where}"
                if blamed is not None else f"DIVERGED-AT {first} in {where}"))
            diverged.lines.extend(report.lines)
            diverged.lines.append(
                f"  [diverge] {len(divergence.columns)} shared column(s), streams aligned at "
                f"offset {_signed(divergence.offset)}, agreeing for {divergence.agreed_frames} frames")
            diverged.lines.append(
                "  An input landed just before this, so the two sides are in different"
                if blamed is not None else
                "  No input precedes this, so the two sides differ deterministically")
            diverged.lines.append(
                "  states rather than rendering the same state differently."
                if blamed is not None else
                "  and this is a real difference worth chasing.")
            _append_screen(diverged, left, right, frame, phase_window)
            return diverged
        else:
            report.lines.append(
                f"  [diverge] none across {divergence.frames_compared} frames of "
                f"{len(divergence.columns)} shared column(s), offset {_signed(divergence.offset)}")

        differing = _append_screen(report, left, right, frame, phase_window)

        # Gate 3 grades an agreement. A frame that plainly disagreed is never
        # vacuous, however uniform either side happens to be - reporting a blank
        # screen as a pass because it carries no information is the exact
        # inversion this gate exists to prevent.
        if differing > 0:
            shown = left.screen(frame)
            percent = 100.0 if shown is None else 100.0 * differing / (shown.width * shown.height)
            differs = Report(DIFFERS,
                             f"DIFFERS: {differing} px ({_fixed(percent, 2)}%) "
                             "and no gate explains it")
            differs.lines.extend(report.lines)
            return differs

        ours = left.screen(frame)
        vacuous = ours is None
        if ours is not None:
            colours, dominant = ours.information()
            vacuous = colours < MINIMUM_COLOURS or dominant > MAXIMUM_DOMINANT_FRACTION
        if not _reference_moves(right, frame, phase_window):
            vacuous = True

        final = Report(
            PASS_VACUOUS if vacuous else PASS,
            "PASS(vacuous): the frames agree, but could not have disagreed" if vacuous else "PASS")
        final.lines.extend(report.lines)
        return final
    finally:
        if close_after:
            dictionary.close()


def _identity_mismatches(a, b):
    if _both(a.board, b.board) and a.board != b.board:
        yield f"board differs: {a.backend}={a.board} {b.backend}={b.board}"
    if (_both(a.region, b.region) and a.region != b.region
            and a.region != "auto" and b.region != "auto"):
        yield f"region differs: {a.backend}={a.region} {b.backend}={b.region}"
    if a.prg_bytes > 0 and b.prg_bytes > 0 and a.prg_bytes != b.prg_bytes:
        yield f"PRG size differs: {a.prg_bytes} vs {b.prg_bytes}"
    if a.chr_bytes > 0 and b.chr_bytes > 0 and a.chr_bytes != b.chr_bytes:
        yield f"CHR size differs: {a.chr_bytes} vs {b.chr_bytes}"
    if _both(a.system, b.system) and a.system != b.system:
        yield f"system differs: {a.system} vs {b.system}"


def _identity_warnings(a, b):
    """Not mismatches: things that lower confidence without invalidating the run."""
    for dump in (a, b):
        if dump.header_trust == "archaic":
            yield (f"{dump.backend} read a {dump.header_trust} header; "
                   "the board it chose is a guess")

    if not _both(a.board, b.board):
        silent = a.backend if len(a.board) == 0 else b.backend
        yield f"{silent} cannot report its board, so the identity gate is one-sided"
    if a.save_loaded != b.save_loaded:
        yield (f"save RAM present on only one side ({a.backend}={_csharp_bool(a.save_loaded)}, "
               f"{b.backend}={_csharp_bool(b.save_loaded)})")


def _both(a, b):
    return len(a) > 0 and len(b) > 0


def _csharp_bool(value):
    """The C# this replaces interpolated a bool, which renders as True/False."""
    return "True" if value else "False"


def first_divergence(a, b):
    """Where the two streams first parted, once aligned and once columns are rated."""
    shared = [c for c in a.columns if c in set(b.columns)]
    if a.screen_format != b.screen_format and "screen" in shared:
        shared.remove("screen")

    if not shared or not a.signature or not b.signature:
        return Divergence(columns=shared)

    # Aligned on whichever column agrees most, because no column is known in
    # advance to be the trustworthy one. A column that never changes agrees at
    # every offset, so alignment uses only columns that move.
    informative = [c for c in shared if a.varies(c) and b.varies(c)]
    if not informative:
        informative = shared

    best_offset, best_rate = 0, -1.0
    for offset in range(-MAX_STREAM_OFFSET, MAX_STREAM_OFFSET + 1):
        rate = max(_rate_of(a, b, c, offset) for c in informative)
        if rate > best_rate:
            best_rate, best_offset = rate, offset

    # Rated over a leading window rather than the whole stream: a real divergence
    # half way through would otherwise drag the column's rate down and disqualify
    # the one column that would have revealed it.
    calibration_end = _calibration_end(a, b, best_offset)
    rates = [(c, _rate_of(a, b, c, best_offset, calibration_end)) for c in shared]
    profiles = [(c, _rate(rate), rate) for c, rate in rates]

    usable = [name for name, rating, _ in profiles
              if rating == STABLE and name not in STRUCTURAL]
    scan = _scan_at(a, b, usable, best_offset)

    return Divergence(frame=scan.frame, columns=shared, differing=scan.differing,
                      frames_compared=scan.frames_compared, offset=best_offset,
                      agreed_frames=scan.agreed_frames, profiles=profiles, usable=usable)


def _rate_of(a, b, column, offset, through_frame=None):
    return dumpdb.agreement_rate(a.db, a.set_id, b.set_id, column, offset, through_frame)


def _rate(rate):
    return STABLE if rate >= STABLE_RATE else (NEVER if rate <= NEVER_RATE else INTERMITTENT)


def _calibration_end(a, b, offset):
    common = sorted(f for f in a.signature if (f + offset) in b.signature)
    if not common:
        return None

    span = int(len(common) * CALIBRATION_SHARE)
    index = min(len(common) - 1, max(MINIMUM_CALIBRATION_FRAMES, span))
    return common[index]


def _scan_at(a, b, shared, offset):
    frames = sorted(f for f in a.signature if (f + offset) in b.signature)

    agreed = 0
    for frame in frames:
        left = a.signature[frame]
        right = b.signature[frame + offset]
        differing = [c for c in shared
                     if c in left and c in right and left[c] != right[c]]
        if differing:
            return Divergence(frame=frame, columns=shared, differing=differing,
                              frames_compared=len(frames), offset=offset,
                              agreed_frames=agreed)
        agreed += 1

    return Divergence(columns=shared, frames_compared=len(frames), offset=offset,
                      agreed_frames=agreed)


def _append_screen(report, left, right, frame, phase_window):
    ours = left.screen(frame)
    if ours is None:
        report.lines.append(f"  [screen] {left.backend} has no screen at frame {frame}")
        return 0

    colours, dominant = ours.information()
    report.lines.append(f"  [screen] {colours} colours, dominant {_fixed(dominant * 100, 1)}%")

    best, best_frame = None, frame
    for f in range(frame - phase_window, frame + phase_window + 1):
        theirs = right.screen(f)
        if theirs is None:
            continue
        differing = ours.differing_pixels(theirs)
        if best is None or differing < best:
            best, best_frame = differing, f

    if best is None:
        report.lines.append(
            f"  [screen] {right.backend} has no screen within +/-{phase_window} of {frame}")
        return 0

    percent = 100.0 * best / (ours.width * ours.height)
    report.lines.append(
        f"  [screen] best {best} px ({_fixed(percent, 2)}%) at {right.backend} "
        f"frame {best_frame}, phase {_signed(best_frame - frame)}")

    if not _reference_moves(right, frame, phase_window):
        report.lines.append("  [screen] the reference is static across the phase window, so a phase")
        report.lines.append("           match here proves nothing about timing")

    if best > 0:
        theirs = right.screen(best_frame)
        if theirs is not None:
            dy = ours.vertical_shift(theirs)
            if dy is not None:
                report.lines.append(
                    f"  [screen] the whole image is offset by {dy:+d} scanline(s), which is a")
                report.lines.append("           timing difference rather than a drawing one")

    return best


def _reference_moves(right, frame, phase_window):
    """A reference that does not change across the window cannot falsify a phase."""
    first = None
    for f in range(frame - phase_window, frame + phase_window + 1):
        image = right.screen(f)
        if image is None:
            continue
        if first is None:
            first = image
            continue
        if first.differing_pixels(image) > 0:
            return True
    return False


def _signed(value):
    """C#'s "+0;-0;0": a sign for either direction, but a bare zero."""
    return f"{value:+d}" if value else "0"


def _fixed(value, digits):
    """Matches .NET's "F" format, which needs nothing doing - see the module header."""
    return f"{value:.{digits}f}"


def render(report):
    return "".join(line + "\n" for line in [report.summary] + report.lines)


def main(argv):
    args = list(argv[1:])

    db_path = dumpdb.DEFAULT_DB
    if "--db" in args:
        at = args.index("--db")
        if at + 1 >= len(args):
            print(__doc__)
            return NOT_COMPARABLE
        db_path = args[at + 1]
        del args[at:at + 2]

    if len(args) < 3:
        print(__doc__)
        return NOT_COMPARABLE

    ours_dir, theirs_dir, frame = args[0], args[1], int(args[2])

    # Kept in the order given rather than sorted: the divergence is blamed on the
    # *last* input within the window, and sorting would change which that is.
    phase_window = 8
    inputs = []
    rest = args[3:]
    i = 0
    while i < len(rest):
        if rest[i] == "--input" and i + 1 < len(rest):
            inputs.append(int(rest[i + 1]))
            i += 2
            continue
        if not rest[i].startswith("-"):
            phase_window = int(rest[i])
        i += 1

    db = dumpdb.open_db(db_path)
    ours = DumpSet.load(db, ours_dir)
    theirs = DumpSet.load(db, theirs_dir)
    if ours is None:
        print(f"[ERROR] no dump set in {ours_dir}")
        return DIFFERS
    if theirs is None:
        print(f"[ERROR] no dump set in {theirs_dir}")
        return DIFFERS

    print(f"{ours.backend} vs {theirs.backend}, frame {frame}")
    for dump in (ours, theirs):
        print(f"  {dump.backend}: board={_show(dump.board)} trust={_show(dump.header_trust)} "
              f"prg={dump.prg_bytes} chr={dump.chr_bytes} screen={_show(dump.screen_format)}")
    print()

    report = compare(ours, theirs, frame, phase_window, inputs)
    sys.stdout.write(render(report))
    return report.verdict


def _show(value):
    return "?" if len(value) == 0 else value


if __name__ == "__main__":
    sys.exit(main(sys.argv))
