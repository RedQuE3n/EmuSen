#!/usr/bin/env python3
"""The 68000 referee: single-step cases run through a referee core's bench (tb68k: fx68k, or Nuked-MD's with
--core nuked, given before the command), its bus timeline and final registers
compared with each suite's expectation and with Beryl's own result. Beryl_M68k.md section 6 is the method.

  referee.py suite <sst|th> <file> [--sample N] [--where address-error|plain] [--out results.jsonl]
      fx68k against one suite's cases (a control, or a class seen from that suite's side).
  referee.py disputes <listing.jsonl> [--class NAME] [--file FILE] [--sample N] [--out results.jsonl]
      each listed case through the core: does it agree with its suite, with Beryl's result in the listing, both or
      neither. The listing is singlestep.rs's: TomHarte's disputed cases (the default) or, with --suite sst,
      SingleStepTests' misses.
  referee.py divzero
      DIVU and DIVS by zero from each condition code, the stacked status word's flags.

Times are clocks from the instruction's start; a bus cycle is four clocks with DTACK at once; an abandoned
(address-error) cycle is silent on the bus, so a suite's `re`/`we` counts as idle time.
"""

import argparse
import json
import os
import random
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

CORPUS = os.environ.get("EMUSEN_BERYL_CORPUS", os.path.expanduser("~/.cache/emusen/probe/nephrite/singlestep"))
WORKS = {"fx68k": os.path.expanduser("~/.cache/emusen/probe/fx68k"), "nuked": os.path.expanduser("~/.cache/emusen/probe/nuked68k")}
WORK = WORKS["fx68k"]
CORE = "fx68k"
COMPLEMENT = False
SUITES = {"sst": ("m68000/v1", 4), "th": ("680x0/68000/v1", 0)}
LIB = os.path.expanduser("~/.cache/emusen/toolchains/verilator/usr/lib64")

_files = {}


def cases_of(suite, name):
    key = (suite, name)
    if key not in _files:
        with open(os.path.join(CORPUS, SUITES[suite][0], name)) as f:
            _files[key] = json.load(f)
    return _files[key]


def block(cid, case, offset, probe=None, window=None):
    i = case["initial"]
    regs = [i[f"d{n}"] for n in range(8)] + [i[f"a{n}"] for n in range(7)]
    regs += [i["usp"], i["ssp"], i["sr"], (i["pc"] - offset) & 0xFFFFFFFF] + list(i["prefetch"])
    ram = " ".join(f"{a & 0xFFFFFF} {v}" for a, v in i["ram"])
    lines = [f"case {cid}", "regs " + " ".join(map(str, regs)), "ram " + ram, f"window {window or case['length']}"]
    if probe is not None:
        lines.append(f"probe {probe & 0xFFFFFF}")
    lines.append("end")
    return "\n".join(lines) + "\n"


def bench(text):
    env = dict(os.environ, LD_LIBRARY_PATH=LIB)
    out = subprocess.run([os.path.join(WORK, "tb68k")], input=text, capture_output=True, text=True, cwd=WORK, env=env, timeout=3600)
    if out.returncode:
        sys.exit(f"tb68k failed: {out.stderr[-2000:]}")
    res, cur = {}, None
    for line in out.stdout.splitlines():
        w = line.split()
        if w[0] == "case":
            cur = {"tx": [], "status": "ok"}
            res[w[1]] = cur
        elif w[0] == "tx":
            rel, k, fc, addr, size, val = int(w[1]), w[2], int(w[3]), int(w[4]), w[5], int(w[6])
            ab = k.endswith("e")
            cur["tx"].append((rel // 2, k, fc, addr & ~1 if ab else addr, None if ab else size == "w", None if ab else val))
        elif w[0] == "next":
            cur["next"] = int(w[1]) // 2
        elif w[0] == "dump":
            cur["dump"] = list(map(int, w[1:]))
        elif w[0] in ("halted", "nodump", "skipped", "unbooted"):
            cur["status"] = w[0]
    return res


def run_bench(blocks, threads=4):
    chunks = [blocks[i::threads] for i in range(threads)]
    merged = {}
    with ThreadPoolExecutor(threads) as ex:
        for r in ex.map(lambda c: bench("".join(c)), [c for c in chunks if c]):
            merged.update(r)
    return merged


def suite_timeline(transactions):
    """A suite's transaction list as (start clock, kind, fc, address, word, value) cycles, idle and abandoned cycles
    advancing the clock only. TomHarte's `t` is TAS's read, two idle clocks and its write."""
    t, out = 0, []
    for x in transactions:
        k = x[0]
        if k == "n":
            t += x[1]
            continue
        clocks, fc, addr, word, val = x[1], x[2], x[3] & 0xFFFFFF, x[4] == ".w", x[5]
        if not word and len(x) >= 8:
            if (x[6], x[7]) == (0, 1):
                addr |= 1
            elif (x[6], x[7]) == (1, 0):
                val >>= 8
        if k in ("re", "we"):
            out.append((t, k, fc, addr & ~1, None, None))
            t += clocks
        elif k == "t":
            out.append((t, "r", fc, addr, word, None))
            out.append((t + 6, "w", fc, addr, word, val))
            t += clocks
        else:
            out.append((t, k, fc, addr, word, val))
            t += clocks
    return out


def beryl_timeline(transactions):
    t, out = 0, []
    for x in transactions:
        if x[0] == "n":
            t += x[1]
            continue
        k, fc, addr, size, val = x
        if k in ("re", "we"):
            out.append((t, k, fc, addr & ~1, None, None))
            t += 4
            continue
        kind = "r" if k == "tr" else k
        out.append((t, kind, fc, addr, size == "w", None if k == "tr" else val))
        t += 4
    return out


def same_cycle(x, y):
    if x[:4] != y[:4] or not (x[4] is None or y[4] is None or x[4] == y[4]):
        return False
    if x[5] is None or y[5] is None or x[5] == y[5]:
        return True
    # Nuked-MD stacks a group 0 frame's access address complemented (section 6.2); with --complement that is accepted.
    return COMPLEMENT and x[1] == "w" and x[4] and x[5] == (~y[5] & 0xFFFF)


def same_tx(a, b):
    return len(a) == len(b) and all(same_cycle(x, y) for x, y in zip(a, b))


def first_diff(a, b):
    for n, (x, y) in enumerate(zip(a, b)):
        if not same_cycle(x, y):
            return f"#{n} {CORE} {fmt(x)} ref {fmt(y)}"
    if len(a) != len(b):
        return f"#{min(len(a), len(b))} {CORE} {fmt(a[len(b)]) if len(a) > len(b) else '-'} ref {fmt(b[len(a)]) if len(b) > len(a) else '-'}"
    return ""


def fmt(x):
    t, k, fc, addr, word, val = x
    return f"{t}:{k}{fc}:{addr:06X}.{'?' if word is None else 'w' if word else 'b'}={'?' if val is None else format(val, 'X')}"


def regs_of(final):
    s = final["sr"]
    a7 = final["ssp"] if s & 0x2000 else final["usp"]
    return [final[f"d{n}"] for n in range(8)] + [final[f"a{n}"] for n in range(7)] + [a7, s]


NAMES = [f"d{n}" for n in range(8)] + [f"a{n}" for n in range(8)] + ["sr"]


def regs_diff(dump, want):
    return [f"{NAMES[i]} {CORE} {dump[i]:X} ref {want[i]:X}" for i in range(17) if dump[i] != want[i]]


def unprobed(c):
    """A case compared on the bus only: a trace follows it (an address error clears T), or it is STOP, which waits."""
    faults = any(t[0] in ("re", "we") for t in c["transactions"])
    traced = (c["initial"]["sr"] | c["final"]["sr"]) & 0x8000 and not faults
    return bool(traced) or c["initial"]["prefetch"][0] == 0x4E72


def fx_timeline(r, window):
    return [x for x in r["tx"] if x[0] < window]


def cmd_suite(a):
    suite, offset = a.suite, SUITES[a.suite][1]
    cases = cases_of(suite, a.file)
    idx = list(range(len(cases)))
    if a.where == "address-error":
        idx = [i for i in idx if any(t[0] in ("re", "we") for t in cases[i]["transactions"])] if suite == "sst" else idx
    elif a.where == "plain":
        idx = [i for i in idx if not any(t[0] in ("re", "we") for t in cases[i]["transactions"])]
    random.seed(a.seed)
    if a.sample and 0 < a.sample < len(idx):
        idx = sorted(random.sample(idx, a.sample))
    blocks = []
    for i in idx:
        c = cases[i]
        blocks.append(block(f"t{i}", c, offset))
        blocks.append(block(f"p{i}", c, offset, (c["final"]["pc"] - offset) & 0xFFFFFF))
    res = run_bench(blocks)
    tally = {"tx": 0, "regs": 0, "both": 0, "unrun": 0}
    shown = 0
    for i in idx:
        c = cases[i]
        r, p = res[f"t{i}"], res[f"p{i}"]
        if r["status"] in ("skipped", "unbooted"):
            tally["unrun"] += 1
            continue
        fx, ref = fx_timeline(r, c["length"]), suite_timeline(c["transactions"])
        traced = unprobed(c)
        ok_tx = same_tx(fx, ref) and (bool(traced) or p.get("next") in (None, c["length"]))
        tally["timed"] = tally.get("timed", 0) + (not traced and p.get("next") is not None)
        ok_regs = bool(traced) or ("dump" in p and not regs_diff(p["dump"], regs_of(c["final"])))
        tally["traced"] = tally.get("traced", 0) + bool(traced)
        tally["tx"] += ok_tx
        tally["regs"] += ok_regs and not traced
        tally["both"] += ok_tx and ok_regs
        if a.out:
            a.out.write(json.dumps({"file": a.file, "index": i, "name": c["name"], "tx": ok_tx, "regs": ok_regs, "core": CORE,
                "fx": fx, "dump": p.get("dump"), "traced": bool(traced),
                "tx_diff": "" if ok_tx else first_diff(fx, ref) or f"length {CORE} {p.get('next')} ref {c['length']}", "next": p.get("next"), "length": c["length"],
                "regs_diff": [] if ok_regs else regs_diff(p["dump"], regs_of(c["final"])) if "dump" in p else [p["status"]]}) + "\n")
        if not (ok_tx and ok_regs) and shown < a.show:
            shown += 1
            print(f"== {c['name']}")
            if not ok_tx:
                print("   tx  ", first_diff(fx, ref))
            if not ok_regs:
                print("   regs", regs_diff(p["dump"], regs_of(c["final"])) if "dump" in p else p["status"])
    n = len(idx) - tally["unrun"]
    tr, timed = tally.get("traced", 0), tally.get("timed", 0)
    print(f"{suite} {a.file}: {n} cases run ({tally['unrun']} not runnable); {CORE} agrees on transactions {tally['tx']} of {n} (length measured on {timed}), registers {tally['regs']} of {n - tr} untraced, both {tally['both']}")


def cmd_disputes(a):
    rows = []
    with open(a.listing) as f:
        for line in f:
            d = json.loads(line)
            if a.cls and d["class"] != a.cls:
                continue
            if a.file and d["file"] != a.file:
                continue
            rows.append(d)
    random.seed(a.seed)
    if a.sample and 0 < a.sample < len(rows):
        rows = random.sample(rows, a.sample)
    rows.sort(key=lambda d: (d["file"], d["index"]))
    blocks = []
    suite, offset = a.suite, SUITES[a.suite][1]
    for n, d in enumerate(rows):
        c = cases_of(suite, d["file"])[d["index"]]
        w = max(c["length"], beryl_length(d))
        final_pc = (c["final"]["pc"] - offset) & 0xFFFFFFFF
        blocks.append(block(f"t{n}", c, offset, None, w))
        blocks.append(block(f"p{n}", c, offset, final_pc, w))
        if d["final"]["pc"] != final_pc:
            blocks.append(block(f"q{n}", c, offset, d["final"]["pc"], w))
    res = run_bench(blocks)
    verdicts = {}
    shown = 0
    for n, d in enumerate(rows):
        c = cases_of(suite, d["file"])[d["index"]]
        r = res[f"t{n}"]
        if r["status"] in ("skipped", "unbooted"):
            verdicts["unrun"] = verdicts.get("unrun", 0) + 1
            continue
        fx = fx_timeline(r, max(c["length"], beryl_length(d)))
        th, be = suite_timeline(c["transactions"]), beryl_timeline(d["transactions"])
        fx_th, fx_be = [x for x in fx if x[0] < c["length"]], [x for x in fx if x[0] < beryl_length(d)]
        p = res[f"p{n}"]
        q = res.get(f"q{n}", p)
        traced = unprobed(c)
        len_th = bool(traced) or p.get("next") in (None, c["length"])
        len_be = bool(traced) or q.get("next") in (None, beryl_length(d))
        tx_th, tx_be = same_tx(fx_th, th) and len_th, same_tx(fx_be, be) and len_be
        rg_th = "dump" in p and not regs_diff(p["dump"], regs_of(c["final"]))
        rg_be = "dump" in q and not regs_diff(q["dump"], regs_of(d["final"]))
        key = f"tx:{side(tx_th, tx_be)} regs:{'traced' if traced else side(rg_th, rg_be)}"
        verdicts[key] = verdicts.get(key, 0) + 1
        if traced:
            rg_th = rg_be = True
        if a.out:
            a.out.write(json.dumps({"file": d["file"], "index": d["index"], "name": d["name"], "verdict": key, "core": CORE,
                "fx": fx, "dump": p.get("dump"), "dump_be": q.get("dump"), "traced": bool(traced),
                "tx_th": "" if tx_th else first_diff(fx_th, th) or f"length {CORE} {p.get('next')} ref {c['length']}",
                "tx_be": "" if tx_be else first_diff(fx_be, be) or f"length {CORE} {q.get('next')} ref {beryl_length(d)}",
                "regs_th": [] if rg_th else regs_diff(p["dump"], regs_of(c["final"])) if "dump" in p else [p["status"]],
                "regs_be": [] if rg_be else regs_diff(q["dump"], regs_of(d["final"])) if "dump" in q else [q["status"]]}) + "\n")
        if shown < a.show and (not tx_be or not rg_be or not tx_th or not rg_th):
            shown += 1
            print(f"== {d['file']}[{d['index']}] {d['name']} ({key})")
            if not tx_th:
                print("   vs suite    tx  ", first_diff(fx_th, th) or f"length {CORE} {p.get('next')} ref {c['length']}")
            if not tx_be:
                print("   vs Beryl    tx  ", first_diff(fx_be, be) or f"length {CORE} {q.get('next')} ref {beryl_length(d)}")
            if not rg_th:
                print("   vs suite    regs", regs_diff(p["dump"], regs_of(c["final"])) if "dump" in p else p["status"])
            if not rg_be:
                print("   vs Beryl    regs", regs_diff(q["dump"], regs_of(d["final"])) if "dump" in q else q["status"])
    print(f"{len(rows)} disputed cases{' of ' + a.cls if a.cls else ''}{' in ' + a.file if a.file else ''}:")
    for k in sorted(verdicts):
        print(f"  {verdicts[k]:6} {k}")


def cmd_divzero(a):
    """DIVU and DIVS by a zero register from each of the 32 condition codes: the status word the exception stacks."""
    out = {}
    blocks = []
    for op, name in ((0x80C1, "DIVU D1,D0"), (0x81C1, "DIVS D1,D0")):
        for ccr in range(32):
            for dividend in (0x00000000, 0x00001234, 0x00008000, 0x00012345, 0x7FFF0000, 0x80000000, 0x80008000, 0xFFFFFFFF):
                regs = {f"d{n}": 0 for n in range(8)} | {f"a{n}": 0 for n in range(7)}
                regs.update(d0=dividend, d1=0x12340000, usp=0x4000, ssp=0x2000, sr=0x2700 | ccr, pc=0x1000, prefetch=[op, 0x4E71])
                regs["ram"] = [[0x1000, op >> 8], [0x1001, op & 0xFF], [0x1002, 0x4E], [0x1003, 0x71], [0x1004, 0x4E], [0x1005, 0x71],
                               [0x14, 0], [0x15, 0], [0x16, 0x30], [0x17, 0]]
                cid = f"{op:x}_{ccr}_{dividend:x}"
                blocks.append(block(cid, {"initial": regs, "length": 60}, 0))
                out[cid] = (name, ccr, dividend)
    res = run_bench(blocks)
    table = {}
    for cid, (name, ccr, dividend) in out.items():
        stacked = [x for x in res[cid]["tx"] if x[1] == "w" and x[3] == 0x1FFA]
        table.setdefault((name, dividend), {}).setdefault(stacked[0][5] & 0x1F if stacked else None, []).append(ccr)
    for (name, dividend), got in sorted(table.items()):
        x_kept = all(g is not None and all((g ^ c) & 0x10 == 0 for c in cs) for g, cs in got.items())
        nzvc = {g & 0xF for g in got if g is not None}
        rule = f"X kept, NZVC {format(nzvc.pop(), '04b')} from every ccr" if x_kept and len(nzvc) == 1 else repr(got)
        print(f"{CORE} {name} dividend {dividend:08X}: {rule}")


def beryl_length(d):
    t = 0
    for x in d["transactions"]:
        t += x[1] if x[0] == "n" else 4
    return t


def side(th, be):
    return {(True, True): "both", (True, False): "suite", (False, True): "Beryl", (False, False): "neither"}[(th, be)]


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("suite")
    s.add_argument("suite", choices=SUITES)
    s.add_argument("file")
    s.add_argument("--where", choices=["address-error", "plain"])
    d = sub.add_parser("disputes")
    d.add_argument("listing")
    d.add_argument("--class", dest="cls")
    d.add_argument("--file")
    d.add_argument("--suite", choices=SUITES, default="th", help="the suite the listing's cases come from")
    sub.add_parser("divzero")
    for p in (s, d):
        p.add_argument("--sample", type=int, default=200)
        p.add_argument("--seed", type=int, default=68000)
        p.add_argument("--show", type=int, default=4)
        p.add_argument("--out", type=argparse.FileType("a"), help="each case's result appended as a JSON line")
    ap.add_argument("--core", choices=WORKS, default="fx68k")
    ap.add_argument("--complement", action="store_true", help="accept a written word that is the complement of the reference's")
    a = ap.parse_args()
    global WORK, CORE, COMPLEMENT
    WORK, CORE, COMPLEMENT = WORKS[a.core], a.core, a.complement
    {"suite": cmd_suite, "disputes": cmd_disputes, "divzero": cmd_divzero}[a.cmd](a)


if __name__ == "__main__":
    main()
