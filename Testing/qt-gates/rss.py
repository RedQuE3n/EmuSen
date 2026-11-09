#!/usr/bin/python3 -I
"""The memory of a program started by run.sh, sampled every 0.5 s: the process named <comm> at or below <pid> (a wrapper
such as bwrap may stand between, or env may have become it), one line each: wall-clock time, VmRSS and RssAnon in KiB. See README.md.

Usage: rss.py <out.tsv> <pid> <comm>
"""
import os
import sys
import time

out, root, comm = sys.argv[1], int(sys.argv[2]), sys.argv[3][:15]


def children():
    table = {}
    for name in os.listdir("/proc"):
        if name.isdigit():
            try:
                stat = open(f"/proc/{name}/stat").read()
                table.setdefault(int(stat[stat.rindex(")") + 2:].split()[1]), []).append((int(name), stat[stat.index("(") + 1:stat.rindex(")")]))
            except OSError:
                pass
    return table


def find():
    try:
        if open(f"/proc/{root}/comm").read().strip() == comm:
            return root
    except OSError:
        pass
    table, todo = children(), [root]
    while todo:
        pid = todo.pop()
        for child, name in table.get(pid, []):
            if name == comm:
                return child
            todo.append(child)
    return None


def kib(pid, key):
    for line in open(f"/proc/{pid}/status"):
        if line.startswith(key):
            return int(line.split()[1])


with open(out, "w") as sink:
    pid = None
    while True:
        pid = pid or find()
        if pid:
            try:
                print(f"{time.time():.3f} {kib(pid, 'VmRSS:')} {kib(pid, 'RssAnon:')}", file=sink, flush=True)
            except OSError:
                pid = None
        time.sleep(0.5)
