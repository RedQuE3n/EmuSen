#!/usr/bin/env python3
"""Clone detection between a core under audit and a reference emulator's sources, across C++, C# and Rust.

See VenusRT_Native.md §4 for the method, its calibration and what it cannot see. It is gate G8 of
VenusRT_Plan.md §7, and the audit owed for every core.

    clonecheck.py <audited dir> <reference dir> [--min-shared N] [--k N] [--window N] [--vocabulary PATH] [--show] [--json PATH]

Three signals, each reported on its own:

- **Structure.** Both sides are reduced to one token vocabulary: identifiers become ID, types and casts
  disappear, parentheses and statement ends are dropped, the control keywords of the three languages are
  mapped onto one set, and numbers keep their value. Fingerprints are winnowed k-grams (MOSS's method).
  A pair is reported when it shares at least --min-shared fingerprints; its score, the share of the smaller
  file's fingerprints it holds, is printed beside it.
- **Tables.** Runs of eight or more numeric literals in a row, by value, with at least four distinct values. A table
  a document also prints
  is expected; the report lists it for a reader to check against the document.
- **Names.** Identifiers of the reference, in a spelling-free form (case and underscores folded), that
  the audited code also uses, less those in the vocabulary files given with --vocabulary (the hardware
  documents, other cores).

Only the audited side is ever printed (--show). The reference side appears as file names and scores, so
that a writer bound by a clean-room protocol can run the tool against sources they may not read.
"""
import argparse
import hashlib
import json
import os
import re
import sys
from collections import defaultdict

EXTENSIONS = {'.rs': 'rust', '.cpp': 'cpp', '.cc': 'cpp', '.h': 'cpp', '.hpp': 'cpp', '.c': 'cpp', '.cs': 'cs'}

# Control words of all three languages, onto one vocabulary; every other keyword and type is dropped.
CONTROL = {
    'if': 'IF', 'else': 'ELSE', 'for': 'LOOP', 'while': 'LOOP', 'loop': 'LOOP', 'do': 'LOOP', 'foreach': 'LOOP',
    'return': 'RET', 'break': 'BRK', 'continue': 'CONT', 'switch': 'SW', 'match': 'SW', 'case': 'CASE',
    'default': 'CASE', 'true': 'TRUE', 'false': 'FALSE', 'goto': 'GOTO',
}
DROPPED = set('''
fn let mut const static pub struct enum impl trait use mod crate Self super where as ref move unsafe extern dyn type
auto void bool char short int long signed unsigned float double inline virtual override template typename class public
private protected namespace using typedef friend explicit volatile mutable constexpr noexcept nullptr new delete
operator sizeof final sealed readonly partial internal abstract get set var out in is object string decimal byte
sbyte ushort uint ulong nint nuint checked unchecked fixed stackalloc params lock yield async await event delegate
u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 str String Vec Box Option Some None Ok Err Result
uint8_t uint16_t uint32_t uint64_t int8_t int16_t int32_t int64_t size_t
'''.split())

TOKEN = re.compile(r'''
    (?P<ws>\s+)
  | (?P<lc>//[^\n]*)
  | (?P<bc>/\*.*?\*/)
  | (?P<str>r\#*"(?:[^"]|"(?!\#))*"\#*|@?"(?:\\.|[^"\\])*")
  | (?P<chr>'(?:\\.|[^'\\])'|b'(?:\\.|[^'\\])')
  | (?P<life>'[A-Za-z_]\w*)
  | (?P<num>0[xX][0-9A-Fa-f_']+|0[bB][01_']+|\d[\d_']*(?:\.\d+)?(?:[eE][+-]?\d+)?)(?P<suffix>[A-Za-z_]\w*)?
  | (?P<id>[A-Za-z_]\w*)
  | (?P<pp>\#[^\n]*)
  | (?P<op>>>=|<<=|->|::|=>|&&|\|\||<<|>>|[-+*/%&|^!=<>]=|\+\+|--|[-+*/%&|^!~=<>?:.,;{}\[\]()])
  | (?P<other>.)
''', re.S | re.X)


def number(text):
    t = text.replace('_', '').replace("'", '')
    try:
        if t[:2] in ('0x', '0X'):
            return int(t[2:], 16)
        if t[:2] in ('0b', '0B'):
            return int(t[2:], 2)
        if re.fullmatch(r'\d+', t):
            return int(t)
        return float(t)
    except ValueError:
        return t


def tokens(text):
    """(token, line, raw identifier or None) triples in the shared vocabulary."""
    out = []
    line = 1
    for m in TOKEN.finditer(text):
        kind, s = m.lastgroup, m.group()
        if kind == 'suffix':
            kind = 'num'
        start_line = line
        line += s.count('\n')
        if kind in ('ws', 'lc', 'bc', 'pp', 'life', 'other'):
            continue
        if kind == 'str':
            out.append(('STR', start_line, None))
        elif kind == 'chr':
            out.append(('CHR', start_line, None))
        elif kind == 'num':
            out.append((f'N{number(m.group("num"))}', start_line, None))
        elif kind == 'id':
            if s in CONTROL:
                out.append((CONTROL[s], start_line, None))
            elif s not in DROPPED:
                out.append(('ID', start_line, s))
        else:
            if s in ('(', ')', ';', ','):
                continue
            out.append(({'->': '.', '::': '.', '=>': 'CASE'}.get(s, s), start_line, None))
    return out


# Tokens that differ between the languages only in spelling: C's truthiness, case labels and breaks, implicit returns,
# type ascriptions and the receiver parameter carry no structure of their own.
SILENT = {'CASE', 'BRK', 'RET', ':'}


def normalize(toks):
    """The syntax the three languages spell differently removed; each member chain `a.b.c` read as one ID."""
    out = []
    i = 0
    while i < len(toks):
        t = toks[i]
        nxt = toks[i + 1] if i + 1 < len(toks) else ('', 0, None)
        if t[2] in ('self', 'this'):
            if out and out[-1][0] == '&':
                out.pop()
            i += 2 if nxt[0] == '.' else 1
            continue
        if t[0] in SILENT:
            i += 1
            continue
        if t[0] == '!=' and nxt[0] == 'N0':
            i += 2
            continue
        if t[0] in ('++', '--'):
            out.append(('+=' if t[0] == '++' else '-=', t[1], None))
            out.append(('N1', t[1], None))
            i += 1
            continue
        if t[0] == '~':
            t = ('!', t[1], None)
        if t[0] == 'ID':
            while i + 2 < len(toks) and toks[i + 1][0] == '.' and toks[i + 2][0] == 'ID':
                i += 2
        out.append(t)
        i += 1
    return out


def winnow(toks, k, window):
    """Fingerprints as {hash: [start token index]}, by winnowing the k-gram hashes."""
    hashes = []
    for i in range(len(toks) - k + 1):
        h = hashlib.blake2b(' '.join(t for t, _, _ in toks[i:i + k]).encode(), digest_size=8).digest()
        hashes.append((int.from_bytes(h, 'little'), i))
    prints = defaultdict(list)
    last = None
    for i in range(max(0, len(hashes) - window + 1)):
        best = min(hashes[i:i + window], key=lambda p: (p[0], -p[1]))
        if best != last:
            prints[best[0]].append(best[1])
            last = best
    return prints


def tables(toks, least=8, distinct=4):
    """Runs of at least `least` numeric literals in a row with `distinct` values among them, with their first line."""
    runs, cur = [], []
    for t, line, _ in toks + [('', 0, None)]:
        if t.startswith('N'):
            cur.append((t, line))
            continue
        if len(cur) >= least and len({v for v, _ in cur}) >= distinct:
            runs.append((tuple(v for v, _ in cur), cur[0][1]))
        cur = []
    return runs


def fold(name):
    return name.replace('_', '').lower()


def load(root):
    files = {}
    for dirpath, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d not in ('target', 'bin', 'obj', '.git')]
        for n in sorted(names):
            ext = os.path.splitext(n)[1]
            if ext in EXTENSIONS:
                path = os.path.join(dirpath, n)
                with open(path, encoding='utf-8', errors='replace') as f:
                    files[os.path.relpath(path, root)] = tokens(f.read())
    return files


def vocabulary(paths):
    words = set()
    for p in paths:
        if os.path.isdir(p):
            for dirpath, _, names in os.walk(p):
                for n in names:
                    with open(os.path.join(dirpath, n), encoding='utf-8', errors='replace') as f:
                        words |= {fold(w) for w in re.findall(r'[A-Za-z_]\w+', f.read())}
        elif os.path.exists(p):
            with open(p, encoding='utf-8', errors='replace') as f:
                words |= {fold(w) for w in re.findall(r'[A-Za-z_]\w+', f.read())}
    return words


def check(audited, reference, k=16, window=8, min_shared=12, vocab=()):
    a_files, r_files = load(audited), load(reference)
    a_norm = {f: normalize(t) for f, t in a_files.items()}
    a_prints = {f: winnow(t, k, window) for f, t in a_norm.items()}
    r_prints = {f: winnow(normalize(t), k, window) for f, t in r_files.items()}
    index = defaultdict(set)
    for f, prints in r_prints.items():
        for h in prints:
            index[h].add(f)

    pairs = []
    for f, prints in a_prints.items():
        shared = defaultdict(set)
        for h in prints:
            for g in index.get(h, ()):
                shared[g].add(h)
        for g, hs in shared.items():
            smaller = min(len(prints), len(r_prints[g])) or 1
            score = len(hs) / smaller
            if len(hs) >= min_shared:
                lines = sorted({a_norm[f][i][1] for h in hs for i in prints[h]})
                pairs.append({'audited': f, 'reference': g, 'score': round(score, 3), 'shared': len(hs), 'lines': lines})
    pairs.sort(key=lambda p: -p['score'])

    r_tables = defaultdict(list)
    for g, t in r_files.items():
        for values, line in tables(t):
            for i in range(len(values) - 7):
                if len(set(values[i:i + 8])) >= 4:
                    r_tables[values[i:i + 8]].append(g)
    table_hits = []
    for f, t in a_files.items():
        for values, line in tables(t):
            hit = {g for i in range(len(values) - 7) for g in r_tables.get(values[i:i + 8], ())}
            if hit:
                table_hits.append({'audited': f, 'line': line, 'length': len(values), 'reference': sorted(hit)})

    known = vocabulary(vocab)
    r_names = {fold(s) for t in r_files.values() for _, _, s in t if s and len(s) >= 6}
    a_names = defaultdict(set)
    for f, t in a_files.items():
        for _, _, s in t:
            if s and len(s) >= 6:
                a_names[fold(s)].add(f)
    names = sorted(n for n in a_names if n in r_names and n not in known)
    return {
        'audited_files': len(a_files), 'reference_files': len(r_files), 'k': k, 'window': window, 'min_shared': min_shared,
        'pairs': pairs, 'tables': table_hits, 'names': [{'name': n, 'files': sorted(a_names[n])} for n in names],
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('audited')
    ap.add_argument('reference')
    ap.add_argument('--k', type=int, default=16)
    ap.add_argument('--window', type=int, default=8)
    ap.add_argument('--min-shared', type=int, default=12, help='fingerprints a pair must share to be reported (VenusRT_Native.md §4.2)')
    ap.add_argument('--vocabulary', action='append', default=[], help='a file or folder of words that are not findings')
    ap.add_argument('--show', action='store_true', help='print the audited lines of each pair')
    ap.add_argument('--json')
    a = ap.parse_args()
    r = check(a.audited, a.reference, a.k, a.window, a.min_shared, a.vocabulary)
    if a.json:
        with open(a.json, 'w') as f:
            json.dump(r, f, indent=1)
    print(f"{r['audited_files']} audited files, {r['reference_files']} reference files; k={a.k} window={a.window} min-shared={a.min_shared}")
    print(f"structure: {len(r['pairs'])} pairs sharing {a.min_shared} or more fingerprints")
    for p in r['pairs'][:40]:
        print(f"  {p['score']:.3f}  {p['shared']:4d} shared  {p['audited']}  ~  {p['reference']}")
        if a.show:
            with open(os.path.join(a.audited, p['audited']), encoding='utf-8', errors='replace') as f:
                text = f.read().split('\n')
            for n in p['lines'][:20]:
                print(f"        {n:5d}  {text[n - 1].rstrip()}")
    print(f"tables: {len(r['tables'])} runs of 8+ literals shared")
    for t in r['tables'][:40]:
        print(f"  {t['audited']}:{t['line']} ({t['length']} values) ~ {', '.join(t['reference'][:3])}")
    print(f"names: {len(r['names'])} reference identifiers used, less the vocabulary")
    for n in r['names'][:60]:
        print(f"  {n['name']}  ({', '.join(n['files'][:3])})")
    return 0


if __name__ == '__main__':
    sys.exit(main())
