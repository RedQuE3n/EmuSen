#!/usr/bin/env python3
"""Runs a PS-EXE on a PlayStation referee as a black box and prints its outputs only: RAM words, or the TTY text.

See Diamond_Referees.md §2 for what each referee is, how it is driven and why, and §3 for how their outputs are
weighed. Nothing here reads a referee's source, and the BIOS is only ever passed to a referee, never read.

    psxreferee.py fetch
    psxreferee.py ram REFEREE EXE --bios FILE [--for N] [--pal] ADDRESS:WORDS ...
    psxreferee.py tty REFEREE EXE --bios FILE [--for N] [--pal] --heading TEXT
    psxreferee.py wav REFEREE EXE --bios FILE [--for N] [--pal] [--option KEY=VALUE ...] OUT.wav
    psxreferee.py pictures REFEREE DISC --bios FILE [--for N] [--every K] [--pal] [PLAY ...] OUTDIR

REFEREE is `duckstation`, `pcsx-redux`, or a libretro core pinned in libretro-cores.sha256 (`mednafen_psx`,
`swanstation`, `pcsx_rearmed`), which the reference probe drives. --bios names the player's own BIOS file in the
probe's firmware folder; DuckStation and the libretro cores are given its folder, PCSX-Redux the file. --for is
frames for PCSX-Redux and the libretro cores (counted from power-on, the BIOS's boot included; PCSX-Redux is stopped
after 60 + N/20 seconds if a program stops its frames) and wall seconds for DuckStation, which runs unthrottled. --pal makes the referee a PAL console. ADDRESS is hexadecimal, WORDS decimal.
Every run lives in a user scope of its own, stopped when it returns; DuckStation's runs take a lock and a free GDB port
(Diamond_Referees.md §2.5).

`wav` records a libretro referee's mixed audio through the probe's `--wav` into OUT.wav, as the core gives it to the
frontend (Diamond_SPU.md §10); the other referees give no audio here. `--option` passes a core option on.

`pictures` runs a libretro referee on a disc image (a sheet, a lone track or an `.iso`) and keeps its screen every K
frames (30 unless given) to frame N as OUTDIR/fFRAME_WxH.rgba, the form Diamond's run tool writes, so that a disc's
boot is compared by the screen alone (Diamond_Referees.md §4.14). The spaces the probe dumps beside each screen are
removed unread: on a player's BIOS they hold its RAM. PLAY drives the game through the probe's input callback:
`--press F:BTN[:DUR]` and `--presson P:F:BTN[:DUR]` (RetroPad names, B the Cross, A the Circle, Y the Square, X the
Triangle), `--stick F:AXIS:VALUE[:DUR]` (AXIS 0-3 the left stick's x and y, then the right's, VALUE -1 to 1),
`--device P=NAME`, `--option KEY=VALUE`, `--card-in FILE` (the first card, as the core's save RAM), `--card-out FILE`
(that card as it stands at the last picture) and `--wav FILE` (the mixed audio) (Diamond_Players.md §13).

`fetch` downloads the two standalone releases pinned in psx-referees.sha256 into the cache, refuses any whose
SHA-256 differs, unpacks them and prepares them to run headless (§2). Exit codes: 0 done, 1 a refused or failed
run, 2 a usage error.
"""
import array
import fcntl
import hashlib
import json
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
PROBE = Path(os.environ.get("EMUSEN_PSX_PROBE", Path.home() / ".cache/emusen/probe/diamond/probe"))
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


def free_port():
    """A port no one listens on now, so that a run never reads another DuckStation's RAM (Diamond_Referees.md §2.5)."""
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def duck_settings(sysdir, pal, port):
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
    wanted = {**DUCK_SETTINGS, "BIOS": {**DUCK_SETTINGS["BIOS"], "SearchDirectory": str(sysdir)}, "Console": {"Region": "PAL" if pal else "Auto"},
              "Debug": {**DUCK_SETTINGS["Debug"], "GDBServerPort": str(port)}}
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


def confined(cmd, name):
    """The command in a user scope of its own, which `reap` stops with all it started (Diamond_Referees.md §2.5)."""
    unit = f"emusen-referee-{os.getpid()}-{name}"
    return ["systemd-run", "--user", "--scope", "--quiet", "--collect", f"--unit={unit}", "--"] + cmd, unit


def reap(unit):
    subprocess.run(["systemctl", "--user", "stop", f"{unit}.scope"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def rsp_words(address, words, port=GDB_PORT):
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
            s = socket.create_connection(("127.0.0.1", port), timeout=10)
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
    CACHE.mkdir(parents=True, exist_ok=True)
    with open(CACHE / "duckstation.lock", "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return duckstation_locked(exe, sysdir, seconds, pal, dumps)


def duckstation_locked(exe, sysdir, seconds, pal, dumps):
    """One run at a time: its settings file is shared by every run (Diamond_Referees.md §2.5)."""
    port = free_port()
    duck_settings(sysdir, pal, port)
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
            r.write('%x %s\\n' % (a, ' '.join('%08X' % w for w in rsp_words(a, n, {port}))))
    p.terminate(); p.wait()
""")
        env = {k: v for k, v in os.environ.items() if k not in ("DISPLAY", "WAYLAND_DISPLAY")}
        env["XDG_CONFIG_HOME"] = str(CACHE / "kwin-config")
        cmd, unit = confined(["timeout", str(seconds + 90), "dbus-run-session", "--", "kwin_wayland", "--virtual", "--no-lockscreen",
                              "--no-global-shortcuts", "--no-kactivities", "--socket", f"emusen-referee-{os.getpid()}",
                              "--exit-with-session", f"{sys.executable} {inner}"], "duckstation")
        try:
            subprocess.run(cmd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        finally:
            reap(unit)
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
    cmd, unit = confined([str(root / "squashfs-root/usr/bin/pcsx-redux"), "-portable", ".", "-cli", "-run", "-interpreter", "-bios", str(bios),
                          "-exe", str(exe), "-exec", lua], "pcsx-redux")
    p = subprocess.Popen(cmd, cwd=root / "portable", env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
    try:
        stdout, _ = p.communicate(timeout=60 + frames / 20)
    except subprocess.TimeoutExpired:
        reap(unit)
        stdout, _ = p.communicate()
    finally:
        reap(unit)
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


def libretro_command(core, exe, sysdir, frames, pal, tmp):
    regions = {"mednafen_psx": "beetle_psx_region=pal", "swanstation": "swanstation_Console_Region=PAL", "pcsx_rearmed": "pcsx_rearmed_region=PAL"}
    cmd = [str(PROBE), str(exe), tmp, str(frames), str(frames), "--core", str(CORES / f"{core}_libretro.so")]
    if core != "pcsx_rearmed":
        cmd += ["--sysdir", str(sysdir)]
    if pal:
        cmd += ["--option", regions[core]]
    if core == "swanstation":
        cmd += ["--option", "swanstation_CPU_ExecutionMode=Interpreter", "--option", "swanstation_GPU_Renderer=Software"]
    if core == "pcsx_rearmed":
        cmd += ["--option", "pcsx_rearmed_drc=disabled"]
    return cmd


def run_libretro_wav(core, exe, sysdir, frames, pal, options, out):
    """The probe's recording of the core's audio, whatever its RAM."""
    with tempfile.TemporaryDirectory(dir=CACHE) as tmp:
        cmd = libretro_command(core, exe, sysdir, frames, pal, tmp) + [x for o in options for x in ("--option", o)] + ["--wav", str(out)]
        cmd, unit = confined(cmd, core)
        try:
            p = subprocess.run(cmd, capture_output=True)
        finally:
            reap(unit)
        if p.returncode == 4:
            die(f"{core}: the probe refused the run (no firmware of the player's own)")
        if not Path(out).exists():
            die(f"{core}: no audio recorded")


def rgba(raw, pixel_format):
    """A screen of the probe's as RGBA bytes."""
    if pixel_format == "Rgb565":
        table = [bytes(((v >> 11) << 3 | v >> 13, (v >> 5 & 63) << 2 | (v >> 9 & 3), (v & 31) << 3 | (v >> 2 & 7), 255)) for v in range(65536)]
        return b"".join(table[v] for v in array.array("H", raw))
    if pixel_format == "Xrgb8888":
        out = bytearray(raw)
        out[0::4], out[2::4], out[3::4] = raw[2::4], raw[0::4], b"\xff" * (len(raw) // 4)
        return bytes(out)
    die(f"a screen format the driver does not know: {pixel_format}")


def run_libretro_pictures(core, disc, sysdir, frames, every, pal, out, play=(), card_out=None):
    """The core's screen every `every` frames, with the presses, devices, options and card of `play`; of the rest the
    probe wrote only the card is kept, when `card_out` asks for it."""
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=CACHE) as tmp:
        cmd = libretro_command(core, disc, sysdir, frames, pal, tmp)
        cmd[3:5] = [str(every), str(frames), str(every)]
        cmd += list(play)
        cmd, unit = confined(cmd, core)
        try:
            p = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        finally:
            reap(unit)
        if p.returncode == 4:
            die(f"{core}: the probe refused the run (no firmware of the player's own)")
        cards = sorted(Path(tmp).glob("*_sram_f*.bin"), key=lambda c: int(c.stem.rsplit("_f", 1)[1]))
        if card_out and cards:
            shutil.copyfile(cards[-1], card_out)
            print(f"{core}: the card at frame {cards[-1].stem.rsplit('_f', 1)[1]} in {card_out}")
        for space in Path(tmp).glob("*_f*.bin"):
            if "_screen_f" not in space.name:
                space.unlink()
        kept = 0
        for manifest in sorted(Path(tmp).glob("*_manifest_f*.json")):
            m = json.loads(manifest.read_text())
            screen = m["screen"]
            if screen is None:
                continue
            raw = (Path(tmp) / screen["file"]).read_bytes()
            (out / f"f{m['frame']:06d}_{screen['width']}x{screen['height']}.rgba").write_bytes(rgba(raw, screen["format"]))
            kept += 1
        if kept == 0:
            die(f"{core}: the core drew no screen at any frame asked for")
        print(f"{core}: {kept} screens in {out}")


def run_libretro(core, exe, sysdir, frames, pal, dumps):
    with tempfile.TemporaryDirectory(dir=CACHE) as tmp:
        cmd = libretro_command(core, exe, sysdir, frames, pal, tmp)
        cmd, unit = confined(cmd, core)
        try:
            p = subprocess.run(cmd, capture_output=True)
        finally:
            reap(unit)
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
    if len(argv) < 3 or argv[0] not in ("ram", "tty", "wav", "pictures"):
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
    if mode == "pictures":
        every = int(option("--every", "30"))
        play = []
        for flag, probe_flag in (("--press", "--press"), ("--presson", "--presson"), ("--stick", "--stick"), ("--device", "--device"), ("--option", "--option"), ("--card-in", "--sram"), ("--wav", "--wav")):
            while flag in rest:
                value = option(flag)
                play += [probe_flag, str(Path(value).resolve()) if flag in ("--card-in", "--wav") else value]
        card_out = option("--card-out")
        if referee not in LIBRETRO or len(rest) != 1:
            die(__doc__, 2)
        return run_libretro_pictures(referee, exe, bios.parent, count, every, pal, Path(rest[0]).resolve(), play, card_out and Path(card_out).resolve())
    if mode == "wav":
        options = []
        while "--option" in rest:
            options.append(option("--option"))
        if referee not in LIBRETRO or len(rest) != 1:
            die(__doc__, 2)
        return run_libretro_wav(referee, exe, bios.parent, count, pal, options, Path(rest[0]).resolve())
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
