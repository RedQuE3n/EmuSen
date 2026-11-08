#!/usr/bin/python3 -I
"""The desk's own compositor in place of a virtual one (run.sh with DESK): a KWin script, loaded into the session's KWin
for the run only and unloaded at its end, that makes the program's windows fullscreen on one output and activates
them. KWin reports back over D-Bus: every window added and every activation, one line each in <run-dir>/desk.log
(wall-clock time, then the words), and <run-dir>/active is written when a matching window first holds the focus. The
session's settings, rules and windows are left as they were. See README.md.

Usage: desk.py <run-dir> <output> <class-regex> <seconds>
"""
import os
import signal
import sys
import time

import dbus
import dbus.service
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib, GLibUnix

run, output, match, seconds = sys.argv[1], sys.argv[2], sys.argv[3], float(sys.argv[4])
name, plugin = f"org.emusen.gate.p{os.getpid()}", f"emusen-gate-{os.getpid()}"
SCRIPT = """
const target = workspace.screens.find(s => s.name == %(output)s);
const match = new RegExp(%(match)s, "i");
function note(words) { callDBus(%(name)s, "/gate", "org.emusen.Gate", "Note", words); }
function mine(w) { return w.normalWindow && match.test(w.resourceClass + " " + w.resourceName); }
function describe(w) {
    const g = w.frameGeometry;
    return w.resourceClass + " pid " + w.pid + " on " + (w.output ? w.output.name : "?") + " " + g.width + "x" + g.height
        + " fullscreen " + w.fullScreen + " active " + w.active;
}
note("outputs " + workspace.screens.map(s => s.name).join(",") + "; target " + (target ? target.name : "missing"));
workspace.windowAdded.connect(w => {
    if (!mine(w)) { note("added, not the program's: " + w.resourceClass); return; }
    if (target) workspace.sendClientToScreen(w, target);
    w.fullScreen = true;
    workspace.activeWindow = w;
    note("taken: " + describe(w));
    w.fullScreenChanged.connect(() => note("fullscreen changed: " + describe(w)));
    w.outputChanged.connect(() => note("output changed: " + describe(w)));
});
workspace.windowActivated.connect(w => note(w ? (mine(w) ? "active: " : "active, not the program's: ") + describe(w) : "active: none"));
"""


class Gate(dbus.service.Object):
    def __init__(self, bus, log):
        super().__init__(bus, "/gate")
        self.log = log

    @dbus.service.method("org.emusen.Gate", in_signature="s")
    def Note(self, words):
        print(f"{time.time():.3f} {words}", file=self.log, flush=True)
        if words.startswith("active: ") and not os.path.exists(f"{run}/active"):
            open(f"{run}/active", "w").write(f"{time.time():.3f}\n")


DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()
held = dbus.service.BusName(name, bus)
log = open(f"{run}/desk.log", "a")
gate = Gate(bus, log)
quote = lambda s: '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'
path = f"{run}/desk.js"
open(path, "w").write(SCRIPT % {"output": quote(output), "match": quote(match), "name": quote(name)})
scripting = dbus.Interface(bus.get_object("org.kde.KWin", "/Scripting"), "org.kde.kwin.Scripting")
loop = GLib.MainLoop()


def end(*_):
    loop.quit()
    return False


for sig in (signal.SIGTERM, signal.SIGINT):
    GLibUnix.signal_add(GLib.PRIORITY_DEFAULT, sig, end)
GLib.timeout_add(int(seconds * 1000), end)
try:
    number = scripting.loadScript(path, plugin, signature="ss")
    dbus.Interface(bus.get_object("org.kde.KWin", f"/Scripting/Script{number}"), "org.kde.kwin.Script").run()
    open(f"{run}/desk-ready", "w").write(f"{time.time():.3f}\n")
    loop.run()
finally:
    print(f"{time.time():.3f} unloaded: {bool(scripting.unloadScript(plugin))}", file=log, flush=True)
