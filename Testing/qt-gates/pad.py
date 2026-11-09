#!/usr/bin/python3 -I
"""The gate pad's script: steps wait:<s>, settle:<grabs.tsv>:<s>, tap:<button>, hold:<button>:<s>, written as presses
("Right 1", "Right 0") down the FIFO that emusen-qt, built with its gate-pad feature, reads into an SDL virtual pad of its
own process (EmuSen_Qt.md §9.3.6). No input device is made: nothing outside that one program sees a press. Buttons: a b
x y l r select start guide up down left right. "ready" is printed once the program holds the FIFO open. See README.md.

Usage: pad.py <fifo> <step>...
"""
import sys
import time

# The pad's buttons by their place, as kit-core's PadControl names them: a is the bottom face button.
CONTROLS = {"a": "South", "b": "East", "x": "West", "y": "North", "l": "LeftShoulder", "r": "RightShoulder",
            "select": "Select", "start": "Start", "guide": "Guide", "up": "Up", "down": "Down", "left": "Left", "right": "Right"}


def settle(tsv, still):
    """Waits until the window is lit (over 1% of the screen) and its lit share has not changed for `still` seconds in the
    polled grabs of run.sh (grabs.tsv), or 15 s; prints the epoch time it stopped waiting."""
    give_up = time.time() + 15
    while time.time() < give_up:
        try:
            rows = [line.split() for line in open(tsv) if line.split() and line.split()[1] != "-"]
        except OSError:
            rows = []
        lit = [(float(r[0]), float(r[1])) for r in rows if float(r[1]) > 0.01]
        if lit:
            changed = lit[0][0]
            for (_, a), (t, b) in zip(lit, lit[1:]):
                if abs(a - b) > 0.0005:
                    changed = t
            if lit[-1][0] - changed >= still:
                break
        time.sleep(0.02)
    print(f"{time.time():.3f} {step} (stopped waiting)", flush=True)


fifo = open(sys.argv[1], "w", buffering=1)  # blocks until the program's pad thread opens it
print("ready", flush=True)


def send(button, down):
    fifo.write(f"{CONTROLS[button]} {int(down)}\n")


for step in sys.argv[2:]:
    word, *rest = step.split(":")
    if word == "wait":
        time.sleep(float(rest[0]))
    elif word == "settle":
        settle(rest[0], float(rest[1]))
    elif word in ("tap", "hold"):
        send(rest[0], True)
        time.sleep(float(rest[1]) if word == "hold" else 0.1)
        send(rest[0], False)
        print(f"{time.time():.3f} {step}", flush=True)
fifo.close()
