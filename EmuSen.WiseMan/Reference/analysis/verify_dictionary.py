#!/usr/bin/env python3
"""Re-run every dictionary assertion against a real pair of dump sets.

    verify_dictionary.py <oursDir> <theirsDir> [--db <path>] [--dictionary <dir>]

This is the only path to 'proven'. The schema refuses a definition promoted by
hand; promotion needs a verification row, and a verification row is written here
and nowhere else, after the assertion has actually been re-run and observed. See
schema.sql's triggers and EmuSen_Debugging_Tools_Reference_v5.md §3.49.

Exit code 1 if any assertion failed, 0 otherwise. A skip is not a failure.

Three things this deliberately does not do
------------------------------------------
It does not run an assertion against a pair that cannot decide it. An assertion
demonstrated on one image says nothing about another, and running it anyway is
how a correct claim gets demoted by an irrelevant pair - which is what happened
on this verifier's first run, hence `applies_to`.

It does not treat "cannot decide" as failure. A claim about PAL images says
nothing when handed two NTSC ones, so `evaluate` returns None and the row is
skipped rather than recorded.

It does not delete a claim that stops holding, only demote it. The claim may
still be true elsewhere; what it has stopped being is something the comparator
may act on.
"""
import os
import subprocess
import sys

import compare
import dictionary as dictionary_module
import dumpdb
from dumpset import DumpSet


def verify(ours, theirs, dictionary, commit, out=print):
    """Returns (passed, failed, skipped)."""
    divergence = compare.first_divergence(ours, theirs)
    rates = {name: rate for name, _, rate in divergence.profiles}

    passed = failed = skipped = 0
    for assertion in dictionary.assertions():
        if not applies_to(assertion, ours):
            out(f"  SKIP  {assertion['slug']}: needs fixture '{assertion['fixture']}'")
            skipped += 1
            continue

        holds, observed = evaluate(assertion, ours, theirs, rates)

        if holds is None:
            out(f"  SKIP  {assertion['slug']}: {observed}")
            skipped += 1
            continue

        dictionary.record_verification(assertion, holds, observed, commit)

        if holds:
            promoted, error = dictionary.try_promote(assertion["definition_id"])
            note = "  [promoted to proven]" if promoted else (f"  [{error}]" if error else "")
            out(f"  PASS  {assertion['slug']}: {observed}{note}")
            passed += 1
        else:
            dictionary.demote(assertion["definition_id"])
            out(f"  FAIL  {assertion['slug']}: {observed}  [demoted to provisional]")
            failed += 1

    return passed, failed, skipped


def applies_to(assertion, ours):
    """Whether this pair is one the assertion was ever a claim about."""
    fixture = assertion["fixture"]
    if fixture is None or not fixture.strip():
        return True
    if fixture.startswith("any "):
        return True
    return fixture.lower() in ours.rom.lower()


def evaluate(assertion, ours, theirs, rates):
    """(holds, observed). None means the pair cannot decide it, which is not failure."""
    kind = assertion["kind"]
    subject = assertion["subject"]

    if kind == "column-agreement":
        if subject not in rates:
            return None, f"neither side exposes a '{subject}' column"
        rate = rates[subject]
        return (_holds(rate, assertion["op"], assertion["value"]),
                f"{subject} agreement {rate * 100:.1f}% {assertion['op']} "
                f"{assertion['value'] * 100:.1f}%")

    if kind == "identity-field":
        mine = _field(ours, subject)
        yours = _field(theirs, subject)
        if len(mine) == 0 or len(yours) == 0:
            return None, f"'{subject}' not reported by both sides"
        differs = mine != yours
        return (differs if assertion["op"] == "<>" else not differs,
                f"{subject}: {ours.backend}={mine} {theirs.backend}={yours}")

    if kind == "screen-shift":
        frame = int(assertion["value"])
        a = ours.screen(frame)
        b = theirs.screen(frame)
        if a is None or b is None:
            return None, "no screen at the assertion's frame"
        shift = a.vertical_shift(b)
        return shift is not None, "no whole-image shift found" if shift is None else f"shift {shift:+d}"

    return None, f"unknown assertion kind '{kind}'"


def _holds(observed, op, value):
    if op == "<=":
        return observed <= value
    if op == ">=":
        return observed >= value
    if op == "=":
        return abs(observed - value) < 1e-9
    return abs(observed - value) >= 1e-9


def _field(dump, name):
    return {"region": dump.region, "board": dump.board,
            "headerTrust": dump.header_trust, "system": dump.system}.get(name, "")


def current_commit():
    """What the run was verified against, so a stale claim is visible as stale."""
    try:
        sha = subprocess.run(["git", "rev-parse", "--short", "HEAD"],
                             capture_output=True, text=True, check=False).stdout.strip()
        return sha or None
    except OSError:
        return None


def main(argv):
    args = list(argv[1:])

    db_path = dumpdb.DEFAULT_DB
    dictionary_dir = None
    for flag, setter in (("--db", "db"), ("--dictionary", "dictionary")):
        if flag in args:
            at = args.index(flag)
            if at + 1 >= len(args):
                print(__doc__)
                return 1
            if setter == "db":
                db_path = args[at + 1]
            else:
                dictionary_dir = args[at + 1]
            del args[at:at + 2]

    if len(args) < 2:
        print(__doc__)
        return 1

    db = dumpdb.open_db(db_path)
    ours = DumpSet.load(db, args[0])
    theirs = DumpSet.load(db, args[1])
    if ours is None:
        print(f"[ERROR] no dump set in {args[0]}")
        return 1
    if theirs is None:
        print(f"[ERROR] no dump set in {args[1]}")
        return 1

    with dictionary_module.open_dictionary(dictionary_dir) as dictionary:
        passed, failed, skipped = verify(ours, theirs, dictionary, current_commit())
        total, proven, provisional, retracted = dictionary.counts()

    print()
    print(f"{passed} passed, {failed} failed, {skipped} skipped against this pair")
    print(f"dictionary: {total} definitions - {proven} proven, "
          f"{provisional} provisional, {retracted} retracted")
    return 1 if failed > 0 else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
