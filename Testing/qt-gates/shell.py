#!/usr/bin/python3 -I
"""A terminal over a published tree's platform library: its shell (dianash_new, every EmuSen module and the library
tool's services) with the runtime's settings service (emusen.settings@1 over emusen_crystal_*, as Mistress's shell
gives it), so that `cores get`, `cores approve` and `cores default` run as the player types them. Each line read from
stdin is submitted as typed at the prompt. See README.md.

Usage: shell.py <tree>/lib/EmuSen < lines
"""
import ctypes as C
import sys
import time

if len(sys.argv) != 2:
    sys.exit(__doc__)
base = sys.argv[1].rstrip("/").encode()
lib = C.CDLL(base.decode() + "/libemusen_platform.so")


class Bytes(C.Structure):
    _fields_ = [("data", C.c_void_p), ("len", C.c_size_t)]


class Config(C.Structure):
    _fields_ = [("size", C.c_uint32), ("flags", C.c_uint32), ("root", Bytes), ("budget_ms", C.c_uint32)]


class Service(C.Structure):
    _fields_ = [("size", C.c_uint32), ("name", Bytes), ("major", C.c_uint32), ("minor", C.c_uint32), ("table", C.c_void_p)]


GET = C.CFUNCTYPE(C.c_int32, C.c_void_p, C.c_uint32, Bytes, Bytes, C.c_void_p, C.c_size_t, C.POINTER(C.c_int64))
SET = C.CFUNCTYPE(C.c_int32, C.c_void_p, C.c_uint32, Bytes, Bytes, Bytes)
ONE = C.CFUNCTYPE(C.c_int32, C.c_void_p, Bytes)
ERR = C.CFUNCTYPE(C.c_int32, C.c_void_p, C.c_void_p, C.c_size_t, C.POINTER(C.c_int64))


class Settings(C.Structure):
    _fields_ = [("size", C.c_uint32), ("ctx", C.c_void_p), ("get", GET), ("set", SET), ("approve", ONE), ("revoke", ONE), ("last_error", ERR)]


OK, TOO_SMALL, ABSENT, REFUSED = 0, -7, -268, -256
CONSOLE, GAME = 0, 1  # the service's scopes, as Mistress's RuntimeShellSettings reads them
CRYSTAL_CONSOLE = 0
lib.emusen_galaxia_init.argtypes = [C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t]
lib.emusen_crystal_new.restype = C.c_void_p
lib.emusen_crystal_new.argtypes = [C.c_void_p]
lib.emusen_crystal_last_error.argtypes = [C.c_void_p, C.c_void_p, C.c_size_t]
lib.emusen_crystal_last_error.restype = C.c_int64
lib.emusen_crystal_core_approve.argtypes = lib.emusen_crystal_core_revoke.argtypes = [C.c_void_p, C.c_char_p, C.c_size_t]
lib.emusen_crystal_settings_get.argtypes = [C.c_void_p, C.c_uint32, C.c_uint32, C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t, C.c_void_p, C.c_size_t]
lib.emusen_crystal_settings_get.restype = C.c_int64
lib.emusen_crystal_settings_set.argtypes = [C.c_void_p, C.c_uint32, C.c_uint32, C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t]
lib.emusen_crystal_game_setting_get.argtypes = [C.c_void_p, C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t, C.c_void_p, C.c_size_t]
lib.emusen_crystal_game_setting_get.restype = C.c_int64
lib.emusen_crystal_game_setting_set.argtypes = [C.c_void_p, C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t, C.c_char_p, C.c_size_t]
lib.dianash_new.restype = C.c_void_p
lib.dianash_new.argtypes = [C.POINTER(Config)]
lib.dianash_set_services.argtypes = [C.c_void_p, C.POINTER(Service), C.c_size_t]
lib.dianash_session_new.argtypes = [C.c_void_p]
lib.dianash_session_new.restype = C.c_int64
lib.dianash_session_attach.argtypes = [C.c_void_p, C.c_uint32, C.c_char_p, C.c_size_t]
lib.dianash_submit.argtypes = [C.c_void_p, C.c_uint32, C.c_char_p, C.c_size_t]
lib.dianash_output_take.argtypes = [C.c_void_p, C.c_void_p, C.c_size_t]
lib.dianash_output_take.restype = C.c_int64
lib.dianash_last_error.argtypes = [C.c_void_p, C.c_size_t]
lib.dianash_last_error.restype = C.c_int64

if lib.emusen_galaxia_init(base, len(base), None, 0) != 0:
    sys.exit("the tree was already named")
rt = lib.emusen_crystal_new(None)
if not rt:
    sys.exit("no runtime")
said = [b""]


def text(b):
    return C.string_at(b.data, b.len) if b.data else b""


def answer(value, out, cap, needed):
    if needed:
        needed[0] = len(value)
    if not out or len(value) > cap:
        return TOO_SMALL
    C.memmove(out, value, len(value))
    return OK


def crystal_error():
    buf = C.create_string_buffer(4096)
    n = lib.emusen_crystal_last_error(rt, buf, 4096)
    said[0] = buf.raw[: max(n, 0)]
    return REFUSED


def get(_, scope, target, key, out, cap, needed):
    t, k, buf = text(target), text(key), C.create_string_buffer(65536)
    if scope == GAME:
        n = lib.emusen_crystal_game_setting_get(rt, t, len(t), k, len(k), buf, 65536)
    else:
        n = lib.emusen_crystal_settings_get(rt, 0, CRYSTAL_CONSOLE, t, len(t), k, len(k), buf, 65536)
    return answer(buf.raw[:n], out, cap, needed) if n >= 0 else ABSENT


def put(_, scope, target, key, value):
    t, k, v = text(target), text(key), (text(value) if value.data else None)
    if scope == GAME:
        code = lib.emusen_crystal_game_setting_set(rt, t, len(t), k, len(k), v, len(v or b""))
    else:
        code = lib.emusen_crystal_settings_set(rt, 0, CRYSTAL_CONSOLE, t, len(t), k, len(k), v, len(v or b""))
    return OK if code == 0 else crystal_error()


def approve(_, sha):
    s = text(sha)
    return OK if lib.emusen_crystal_core_approve(rt, s, len(s)) == 0 else crystal_error()


def revoke(_, sha):
    s = text(sha)
    return OK if lib.emusen_crystal_core_revoke(rt, s, len(s)) == 0 else crystal_error()


def last_error(_, out, cap, needed):
    return answer(said[0], out, cap, needed)


calls = (GET(get), SET(put), ONE(approve), ONE(revoke), ERR(last_error))
table = Settings(C.sizeof(Settings), None, *calls)
name = b"emusen.settings"
service = Service(C.sizeof(Service), Bytes(C.cast(C.c_char_p(name), C.c_void_p), len(name)), 1, 0, C.cast(C.pointer(table), C.c_void_p))
INTERACTIVE = 1 << 1
sh = lib.dianash_new(C.byref(Config(C.sizeof(Config), INTERACTIVE, Bytes(None, 0), 0)))
if not sh or lib.dianash_set_services(sh, C.byref(service), 1) != 0:
    sys.exit("no shell")
session = 0  # one session, the current one
out = C.create_string_buffer(1 << 20)
for line in sys.stdin:
    line = line.rstrip("\n").encode()
    if not line:
        continue
    print(f"$ {line.decode()}", flush=True)
    started = time.monotonic()
    code = lib.dianash_submit(sh, session, line, len(line))
    while (n := lib.dianash_output_take(sh, out, len(out))) > 0:
        sys.stdout.write(out.raw[:n].decode(errors="replace"))
    print(f"[status {code}, {time.monotonic() - started:.2f} s]", flush=True)
    if code == 2:
        break
