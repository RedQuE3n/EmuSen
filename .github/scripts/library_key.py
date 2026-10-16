#!/usr/bin/env python3
"""The input key of a library rust-cores.yml builds, and the run's plan of which to build and which to reuse.

  library_key.py key  DIR NAME TARGET   print the key of crate DIR's library NAME for TARGET (the inputs to stderr)
  library_key.py plan                   from CRATES, PLATFORMS and FULL in the environment, write the job outputs

See EmuSen_RustPlatform.md §28.
"""
import functools
import hashlib
import io
import json
import os
import re
import subprocess
import sys
import zipfile

# What a crate's build or tests read from outside its path dependencies, by the folder that reads it.
EXTRA = {
    "EmuSen/Cores/Nintendo/MarsRT - N64": ["EmuSen/Cores/Nintendo/Mars - N64/Rdp/Gpu/Shaders"],
    "EmuSen/Cores/Nintendo/VenusRT - SNES": [
        "EmuSen.WiseMan/Reference/analysis/dsp1_tables.py",
        "EmuSen.WiseMan/Reference/analysis/st010_tables.py",
    ],
    "EmuSen/Cores/Sony/Diamond - PlayStation": ["EmuSen.WiseMan/Reference/analysis/psxplay"],
    "EmuSen/Cores/Sony/Diamond - PlayStation/kernel-harness": [
        "EmuSen.DianaSh/DianaSh/Etc/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md",
    ],
    "Platform/serenity": ["EmuSen.Serenity/Slang/spirv-cache-schema.sql"],
    "Platform/dianash": ["EmuSen.WiseMan/DianaSh/Golden", "EmuSen.Crystal/include/emusen_session.h"],
}
# The sidecar's writer, which every library job runs, and the files that say how a library is built.
TOOLS = ["EmuSen/Cores/Shared/emusen-core-conform"]
FILES = [".github/workflows/rust-cores.yml", ".github/scripts/library_key.py"]
KEY_VERSION = "1"

PATH_DEP = re.compile(r'\bpath\s*=\s*"([^"]+)"')
UP_LITERAL = re.compile(r'"/?((?:\.\./)+[^"]*)"')


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def norm(path):
    return os.path.normpath(path).replace("\\", "/")


def inside(path, folder):
    return path == folder or path.startswith(folder + "/")


def closure(start):
    """The crate folders a crate's path dependencies reach, itself and the tools included."""
    seen, todo = set(), [norm(start), *TOOLS]
    while todo:
        d = todo.pop()
        if d in seen or not os.path.isfile(os.path.join(d, "Cargo.toml")):
            continue
        seen.add(d)
        for manifest in git("ls-files", "-z", "--", f"{d}/Cargo.toml", f"{d}/**/Cargo.toml").split("\0"):
            if manifest:
                with open(manifest, encoding="utf-8") as f:
                    todo += [norm(os.path.join(os.path.dirname(manifest), rel)) for rel in PATH_DEP.findall(f.read())]
    return seen


def outermost(paths):
    return sorted(p for p in paths if not any(q != p and inside(p, q) for q in paths))


def undeclared(folders, declared):
    """Paths a crate's sources name with ../ that exist outside every declared input."""
    found = set()
    files = git("ls-files", "-z", "--", *folders).split("\0")
    for f in (f for f in files if f.endswith(".rs")):
        with open(f, encoding="utf-8", errors="replace") as h:
            text = h.read()
        manifest = os.path.dirname(f)
        while manifest and not os.path.isfile(os.path.join(manifest, "Cargo.toml")):
            manifest = os.path.dirname(manifest)
        for rel in UP_LITERAL.findall(text):
            for base in {os.path.dirname(f), manifest}:
                p = norm(os.path.join(base, rel))
                if os.path.exists(p) and not any(inside(p, d) for d in declared):
                    found.add(f"{f} names {p}")
    return sorted(found)


def toolchain(folder, name, target):
    """The toolchain the PGO step chooses, as rustc names itself."""
    manifest = os.path.join(folder, "pgo", f"{name}.{target}.pgo")
    spec = "stable"
    if os.path.isfile(manifest.replace(".pgo", ".profdata")):
        with open(manifest, encoding="utf-8") as f:
            spec = next((l.split("=", 1)[1].strip() for l in f if l.startswith("toolchain =")), "stable")
    return version(spec)


@functools.cache
def version(spec):
    try:
        subprocess.run(["rustup", "toolchain", "install", spec, "--profile", "minimal", "--no-self-update"], check=True, capture_output=True)
    except FileNotFoundError:
        return subprocess.run(["rustc", "-V"], check=True, capture_output=True, text=True).stdout.strip()
    return subprocess.run(["rustc", f"+{spec}", "-V"], check=True, capture_output=True, text=True).stdout.strip()


@functools.cache
def sources(folder):
    """The folders and files a crate's library is built and tested from, by git's id; and any it names undeclared."""
    crates = closure(folder)
    extra = [x for d, xs in EXTRA.items() if any(inside(d, c) for c in crates) for x in xs]
    declared = outermost(crates | set(extra))
    ids = tuple(f"{git('rev-parse', 'HEAD:' + p).strip()} {p}" for p in sorted(set(declared) | set(FILES)))
    return ids, tuple(undeclared(outermost(crates), declared))


def inputs(folder, name, target):
    """Every input of the library, one line each, and the paths its sources name outside them."""
    ids, missing = sources(folder)
    lines = [f"key {KEY_VERSION}", f"library {name}", f"target {target}", f"toolchain {toolchain(folder, name, target)}"]
    return lines + list(ids), list(missing)


def key_of(lines):
    return hashlib.sha256("\n".join(lines).encode()).hexdigest()[:24]


def artifact_name(name, rid, key):
    return f"{name}-{rid}-key-{key}"


def gh_api(path, binary=False):
    out = subprocess.run(["gh", "api", "-H", "Accept: application/vnd.github+json", path], check=True, capture_output=True)
    return out.stdout if binary else json.loads(out.stdout)


def reusable(repo, name, library):
    """The newest unexpired artifact of that name built in this repository, if it holds the library; else None."""
    listing = gh_api(f"/repos/{repo}/actions/artifacts?name={name}&per_page=20")
    for a in sorted(listing.get("artifacts", []), key=lambda a: a["created_at"], reverse=True):
        run = a.get("workflow_run") or {}
        if a["expired"] or run.get("head_repository_id") != run.get("repository_id"):
            continue
        try:
            z = zipfile.ZipFile(io.BytesIO(gh_api(f"/repos/{repo}/actions/artifacts/{a['id']}/zip", binary=True)))
            names = {i.filename: i.file_size for i in z.infolist()}
            if names.get(library, 0) == 0 or not set(names) <= {library, library + ".core.json"}:
                continue
            if library + ".core.json" in names:
                json.loads(z.read(library + ".core.json"))
            z.testzip()
        except (subprocess.CalledProcessError, zipfile.BadZipFile, ValueError):
            continue
        return {"id": a["id"], "run": run.get("id")}
    return None


def plan():
    crates, platforms = json.loads(os.environ["CRATES"]), json.loads(os.environ["PLATFORMS"])
    full, repo = os.environ.get("FULL") == "true", os.environ["GITHUB_REPOSITORY"]
    build, reuse, rows = [], [], []
    for c in crates:
        for p in platforms:
            library = f"{p['prefix']}{c['name']}{p['suffix']}"
            try:
                lines, missing = inputs(c["dir"], c["name"], p["target"])
                key = key_of(lines)
            except (OSError, subprocess.CalledProcessError) as e:
                print(f"::warning title=no input key::{c['name']} {p['rid']}: {e}; it is built")
                lines, missing, key = [], ["no key"], "none"
            for m in missing:
                print(f"::warning title=undeclared input::{m}; {c['name']} is built (add it to EXTRA in library_key.py)")
            found = None
            if not full and not missing:
                try:
                    found = reusable(repo, artifact_name(c["name"], p["rid"], key), library)
                except (OSError, ValueError, subprocess.CalledProcessError) as e:
                    print(f"::warning title=no lookup::{c['name']} {p['rid']}: {e}; it is built")
            if found:
                reuse.append({"name": c["name"], "rid": p["rid"], "artifact": found["id"], "library": library})
                rows.append(f"| {c['name']} | {p['rid']} | reused from run {found['run']} | `{key}` |")
            else:
                build.append({"crate": c, "platform": p})
                why = "a full build" if full else "inputs unknown" if missing else "no build of these inputs"
                rows.append(f"| {c['name']} | {p['rid']} | built ({why}) | `{key}` |")
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as f:
        f.write(f"build={json.dumps(build)}\nreuse={json.dumps(reuse)}\n")
    with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as f:
        f.write("| Library | Platform | This run | Input key |\n|---|---|---|---|\n" + "\n".join(rows) + "\n")


def main():
    if sys.argv[1:2] == ["key"] and len(sys.argv) == 5:
        lines, missing = inputs(*sys.argv[2:5])
        print("\n".join(lines + [f"undeclared {m}" for m in missing]), file=sys.stderr)
        print(key_of(lines) if not missing else "undeclared")
    elif sys.argv[1:2] == ["plan"]:
        plan()
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
