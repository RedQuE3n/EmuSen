#!/usr/bin/env python3
"""Runs a PS-EXE on a PlayStation referee as a black box and prints its outputs only: RAM words, or the TTY text.

See Diamond_Referees.md §2 for what each referee is, how it is driven and why, and §3 for how their outputs are
weighed. Nothing here reads a referee's source, and the BIOS is only ever passed to a referee, never read.

    psxreferee.py fetch
    psxreferee.py ram REFEREE EXE --bios FILE [--for N] [--pal] ADDRESS:WORDS ...
    psxreferee.py tty REFEREE EXE --bios FILE [--for N] [--pal] --heading TEXT

REFEREE is `duckstation`, `pcsx-redux`, or a libretro core pinned in libretro-cores.sha256 (`mednafen_psx`,
`swanstation`, `pcsx_rearmed`), which the reference probe drives. --bios names the player's own BIOS file in the
probe's firmware folder; DuckStation and the libretro cores are given its folder, PCSX-Redux the file. --for is
frames for PCSX-Redux and the libretro cores (counted from power-on, the BIOS's boot included; PCSX-Redux is stopped
after 60 + N/20 seconds if a program stops its frames) and wall seconds for DuckStation, which runs unthrottled. --pal makes the referee a PAL console. ADDRESS is hexadecimal, WORDS decimal.

`fetch` downloads the two standalone releases pinned in psx-referees.sha256 into the cache, refuses any whose
SHA-256 differs, unpacks them and prepares them to run headless (§2). Exit codes: 0 done, 1 a refused or failed
run, 2 a usage error.
"""
import hashlib
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
PINS = HERE.parent / "psx-referees.sha256"
CACHE = Path.home() / ".cache/emusen/probe/diamond/referees"
PROBE = Path.home() / ".cache/emusen/probe/diamond/probe"
CORES = Path.home() / ".cache/emusen/probe/libretro/cores"
LIBRETRO = ("mednafen_psx", "swanstation", "pcsx_rearmed")
GDB_PORT = 2345

# DuckStation's settings for a headless black-box run (Diamond_Referees.md §2.1): interpreter, software renderer,
# unthrottled, no audio, no updater, TTY logged and nothing else, the GDB server on.
DUCK_SETTINGS = {
    "Main": {"ConfirmPowerOff": "false", "SaveStateOnExit": "false", "SetupWizardIncomplete": "false", "EmulationSpeed": "0"},
    "CPU": {"ExecutionMode": "Interpreter"},
    "GPU": {"Renderer": "Software", "UseThread": "false"},
    "Audio": {"Backend": "Null"},
    "Display": {"ShowOSDMessages": "false"},
    "BIOS": {"TTYLogging": "true", "PatchFastBoot": "false"},
    "Debug": {"EnableGDBServer": "true", "GDBServerPort": str(GDB_PORT)},
    "AutoUpdater": {"CheckAtStartup": "false"},
}


def die(message, code=1):
    print(message, file=sys.stderr)
    sys.exit(code)


def pins():
    out = {}
    for line in PINS.read_text().splitlines():
        if line.strip() and not line.startswith("#"):
            name, sha, url = line.split()[:3]
            out[name] = (sha, url)
    return out


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def fetch():
    CACHE.mkdir(parents=True, exist_ok=True)
    for name, (sha, url) in pins().items():
        path = CACHE / name
        if not path.exists():
            print(f"fetching {url}")
            urllib.request.urlretrieve(url, path)
        if sha256(path) != sha:
            die(f"{name}: SHA-256 is not the pinned one; re-pinning is a deliberate edit of {PINS.name}")
    redux = CACHE / "PCSX-Redux-HEAD-x86_64.AppImage"
    if not redux.exists():
        shutil.unpack_archive(str(next(CACHE.glob("PCSX-Redux-*-linux-x86_64.zip"))), str(CACHE))
    for appimage, folder in ((next(CACHE.glob("DuckStation-*.AppImage")), "duckstation"), (redux, "pcsx-redux")):
        if not (CACHE / folder / "squashfs-root").exists():
            appimage.chmod(0o755)
            (CACHE / folder).mkdir(exist_ok=True)
            subprocess.run([str(appimage), "--appimage-extract"], cwd=CACHE / folder, check=True, stdout=subprocess.DEVNULL)
    prepare_duckstation()
    print("ready")


def prepare_duckstation():
    root = CACHE / "duckstation/squashfs-root/usr"
    for lib in root.glob("lib/libQt6*.so.6"):
        system = Path("/usr/lib64") / lib.name
        if not lib.is_symlink() and system.exists():
            lib.unlink()
            lib.symlink_to(system)
    (root / "bin/portable.txt").touch()


def duck_settings(sysdir, pal):
    path = CACHE / "duckstation/squashfs-root/usr/bin/settings.ini"
    sections = {}
    order = []
    if path.exists():
        current = None
        for line in path.read_text().splitlines():
            m = re.match(r"\[(.+)\]$", line)
            if m:
                current = m.group(1)
                order.append(current)
                sections.setdefault(current, {})
            elif current and " = " in line:
                k, v = line.split(" = ", 1)
                sections[current][k] = v
    wanted = {**DUCK_SETTINGS, "BIOS": {**DUCK_SETTINGS["BIOS"], "SearchDirectory": str(sysdir)}, "Console": {"Region": "PAL" if pal else "Auto"}}
    for section, values in wanted.items():
        if section not in sections:
            sections[section] = {}
            order.append(section)
        sections[section].update(values)
    logging = sections.setdefault("Logging", {})
    if "Logging" not in order:
        order.append("Logging")
    for k in list(logging):
        if logging[k] in ("true", "false") and k not in ("LogToConsole",):
            logging[k] = "true" if k in ("TTY", "System") else "false"
    logging.update({"LogToConsole": "true", "LogLevel": "Info"})
    path.write_text("".join(f"[{s}]\n" + "".join(f"{k} = {v}\n" for k, v in sections[s].items()) + "\n" for s in order))


def rsp_words(address, words):
    """RAM words over DuckStation's GDB server: the target halted by a status query, then read."""
    def packet(s, body):
        data = body.encode()
        s.sendall(b"$" + data + b"#" + b"%02x" % (sum(data) & 0xFF))

    def reply(s):
        buf = b""
        while True:
            chunk = s.recv(65536)
            if not chunk:
                raise EOFError
            buf += chunk
            i = buf.find(b"$")
            j = buf.find(b"#", i)
            if i >= 0 and j >= 0 and len(buf) >= j + 3:
                s.sendall(b"+")
                return buf[i + 1:j].decode()

    for _ in range(100):
        try:
            s = socket.create_connection(("127.0.0.1", GDB_PORT), timeout=10)
            break
        except OSError:
            time.sleep(0.2)
    else:
        raise ConnectionError("no GDB server")
    packet(s, "?")
    reply(s)
    hexs = ""
    for at in range(address, address + 4 * words, 1024):
        packet(s, "m%x,%x" % (at, min(1024, address + 4 * words - at)))
        hexs += reply(s)
    packet(s, "D")
    s.close()
    return [int.from_bytes(bytes.fromhex(hexs[i:i + 8]), "little") for i in range(0, len(hexs), 8)]


def run_duckstation(exe, sysdir, seconds, pal, dumps):
    """In a private, virtual KWin with its own message bus: never the person's desktop (Diamond_Referees.md §2.1)."""
    duck_settings(sysdir, pal)
    root = CACHE / "duckstation/squashfs-root"
    with tempfile.TemporaryDirectory(dir=CACHE) as tmp:
        log, ram = Path(tmp) / "log", Path(tmp) / "ram"
        inner = Path(tmp) / "inner.py"
        inner.write_text(f"""import os, subprocess, sys, time
sys.path.insert(0, {str(HERE)!r})
from psxreferee import rsp_words
env = dict(os.environ, QT_QPA_PLATFORM='wayland', QT_PLUGIN_PATH='/usr/lib64/qt6/plugins', LD_PRELOAD={str(root / 'usr/lib/libpng16.so.16')!r})
with open({str(log)!r}, 'w') as out:
    p = subprocess.Popen(['./usr/bin/duckstation-qt', '-batch', '-nogui', '--', {str(exe)!r}], cwd={str(root)!r}, env=env, stdout=out, stderr=subprocess.STDOUT)
    time.sleep({seconds})
    with open({str(ram)!r}, 'w') as r:
        for a, n in {dumps!r}:
            r.write('%x %s\\n' % (a, ' '.join('%08X' % w for w in rsp_words(a, n))))
    p.terminate(); p.wait()
""")
        env = {k: v for k, v in os.environ.items() if k not in ("DISPLAY", "WAYLAND_DISPLAY")}
        env["XDG_CONFIG_HOME"] = str(CACHE / "kwin-config")
        subprocess.run(["timeout", str(seconds + 90), "dbus-run-session", "--", "kwin_wayland", "--virtual", "--no-lockscreen",
                        "--no-global-shortcuts", "--no-kactivities", "--socket", f"emusen-referee-{os.getpid()}",
                        "--exit-with-session", f"{sys.executable} {inner}"], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        text = re.sub(r"\x1b\[[0-9;]*m", "", log.read_text(errors="replace")) if log.exists() else ""
        if "Using BIOS:" not in text:
            die("DuckStation started no BIOS from the firmware folder: refused")
        tty = "\n".join(line[7:] for line in text.splitlines() if line.startswith("I/TTY: "))
        words = {}
        if ram.exists():
            for line in ram.read_text().split("\n"):
                if line:
                    a, *ws = line.split()
                    words[int(a, 16)] = [int(w, 16) for w in ws]
        return tty, words


def run_redux(exe, bios, frames, dumps):
    root = CACHE / "pcsx-redux"
    (root / "portable").mkdir(exist_ok=True)
    lua = "local n=0; L=PCSX.Events.createEventListener('GPU::Vsync', function() n=n+1; if n==%d then local m=PCSX.getMemPtr(); " % frames
    for a, count in dumps:
        lua += "local s='== RAM %08X'; for i=0,%d do local o=%d+4*i; s=s..string.format(' %%08X', m[o]+m[o+1]*256+m[o+2]*65536+m[o+3]*16777216) end; print(s); " % (a, count - 1, a & 0x1FFFFF)
    lua += "PCSX.quit(0) end end)"
    env = {k: v for k, v in os.environ.items() if k not in ("DISPLAY", "WAYLAND_DISPLAY")}
    p = subprocess.Popen([str(root / "squashfs-root/usr/bin/pcsx-redux"), "-portable", ".", "-cli", "-run", "-interpreter", "-bios", str(bios),
                          "-exe", str(exe), "-exec", lua], cwd=root / "portable", env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
    try:
        stdout, _ = p.communicate(timeout=60 + frames / 20)
    except subprocess.TimeoutExpired:
        p.kill()
        stdout, _ = p.communicate()
    if "Loaded BIOS:" not in stdout:
        die("PCSX-Redux loaded no BIOS from the file given: refused")
    words = {}
    lines = []
    for line in stdout.splitlines():
        if line.startswith("== RAM "):
            a, *ws = line[7:].split()
            words[int(a, 16)] = [int(w, 16) for w in ws]
        elif "from unknown address" not in line:
            lines.append(line)
    return "\n".join(lines), words


def run_libretro(core, exe, sysdir, frames, pal, dumps):
    options = {"mednafen_psx": "beetle_psx_region=pal", "swanstation": "swanstation_Console_Region=PAL", "pcsx_rearmed": "pcsx_rearmed_region=PAL"}
    with tempfile.TemporaryDirectory(dir=CACHE) as tmp:
        cmd = [str(PROBE), str(exe), tmp, str(frames), str(frames), "--core", str(CORES / f"{core}_libretro.so")]
        if core != "pcsx_rearmed":
            cmd += ["--sysdir", str(sysdir)]
        if pal:
            cmd += ["--option", options[core]]
        if core == "swanstation":
            cmd += ["--option", "swanstation_CPU_ExecutionMode=Interpreter", "--option", "swanstation_GPU_Renderer=Software"]
        if core == "pcsx_rearmed":
            cmd += ["--option", "pcsx_rearmed_drc=disabled"]
        p = subprocess.run(cmd, capture_output=True)
        if p.returncode == 4:
            die(f"{core}: the probe refused the run (no firmware of the player's own)")
        ram = next(Path(tmp).glob("*_ram_f*.bin"), None)
        if ram is None:
            die(f"{core}: no RAM dumped")
        data = ram.read_bytes()
        return "", {a: [int.from_bytes(data[(a & 0x1FFFFF) + 4 * i:(a & 0x1FFFFF) + 4 * i + 4], "little") for i in range(n)] for a, n in dumps}


def main(argv):
    if argv[:1] == ["fetch"]:
        return fetch()
    if len(argv) < 3 or argv[0] not in ("ram", "tty"):
        die(__doc__, 2)
    mode, referee, exe = argv[0], argv[1], Path(argv[2]).resolve()
    rest = argv[3:]

    def option(name, default=None):
        if name in rest:
            i = rest.index(name)
            value = rest[i + 1]
            del rest[i:i + 2]
            return value
        return default

    bios = Path(option("--bios") or die("--bios names the player's own BIOS file", 2)).resolve()
    count = int(option("--for", "600"))
    heading = option("--heading", "")
    pal = "--pal" in rest
    if pal:
        rest.remove("--pal")
    dumps = [(int(a, 16), int(n)) for a, n in (d.split(":") for d in rest)]
    if referee == "duckstation":
        tty, words = run_duckstation(exe, bios.parent, count, pal, dumps)
    elif referee == "pcsx-redux":
        tty, words = run_redux(exe, bios, count, dumps)
    elif referee in LIBRETRO:
        tty, words = run_libretro(referee, exe, bios.parent, count, pal, dumps)
    else:
        die(f"unknown referee {referee}", 2)
    if mode == "tty":
        lines = tty.splitlines()
        at = next((i for i, line in enumerate(lines) if line.startswith(heading)), len(lines))
        print("\n".join(lines[at:]))
    for a, n in dumps:
        ws = words.get(a, [])
        print(f"== RAM {a:08X} {n}")
        for i in range(0, len(ws), 8):
            print(" ".join("%08X" % w for w in ws[i:i + 8]))


if __name__ == "__main__":
    main(sys.argv[1:])
