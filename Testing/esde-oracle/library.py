#!/usr/bin/python3 -I
"""The oracle's synthetic library: empty ROM files, ES-DE gamelists and a downloaded_media tree of flat-hue labelled
PNGs, written under <out> and listed as plain data (library.tsv, media.tsv). The games, metadata, picture sizes and
colours are those of the WiseMan fixture SyntheticLibrary, so ES-DE, the C# painter and the Qt painter read one library;
the SNES system adds a folder with a folder link and one game with hidden metadata (EmuSen_BigPicture.md 31.1). The
media viewer's game, the NES's first, has every type the viewer shows: a video (a still picture held for four seconds,
encoded by ffmpeg), a back cover, fan art and a custom picture besides the five types every game has.
Nothing is read from a real library. Existing files are kept, so a second run changes nothing. See README.md.

Usage: library.py <out>
  <out>/roms/<system>/...                     empty ROM files
  <out>/gamelists/<system>/gamelist.xml       ES-DE's gamelists, copied into each run's home
  <out>/downloaded_media/<system>/<type>/...  the pictures
  <out>/library.tsv, <out>/media.tsv          what was written, one row each
"""
import os
import subprocess
import sys
from datetime import datetime
from xml.sax.saxutils import escape

from PIL import Image, ImageDraw, ImageFont

SYSTEMS = [("nes", ".nes"), ("snes", ".sfc"), ("n64", ".z64"), ("gb", ".gb"), ("gbc", ".gbc")]
TITLES = ["Aurora Drift", "Brass Lantern", "Cobalt Harbor", "Dune Relay", "Ember Circuit", "Fable of Tiles", "Granite Choir",
          "Hollow Comet", "Ivory Signal", "Juniper Vault", "Kestrel Run", "Lumen Garden"]
GENRES = ["Platform", "Puzzle", "Racing", "Role playing", "Shooter", "Sports"]
MEDIA = {"covers": (600, 800), "screenshots": (512, 448), "titlescreens": (512, 448), "marquees": (800, 300), "miximages": (640, 480)}
# The types only the media viewer's game has (USERGUIDE.md, "Game media viewer"), kept off the SNES the watched states show.
VIEWER_GAME, VIEWER_MEDIA = ("nes", 0), {"videos": (512, 448), "backcovers": (600, 800), "fanart": (1280, 720), "custom": (640, 480)}
FOLDER, FOLDER_GAMES, HIDDEN = "Tower Set", ["Quartz Spire", "Rune Gate"], "Lumen Garden"
DESCRIPTION = ("{t} is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a "
               "flat colour with its name written on it. This second sentence is here so that the description is long enough to "
               "wrap over several lines and to be cut off where its box ends.")


def colour(game, folder):
    return 40 + 17 * game % 180, 60 + len(folder) * 23 % 160, 90 + 31 * game % 150


def rating(i):
    return None if i % 4 == 3 else f"{0.2 + 0.2 * (i % 5) - 0.1:.2f}".rstrip("0").rstrip(".")


def game(i, title, ext, system):
    last = None if i % 3 == 0 else datetime(2026, 9, 24 - i, 10, 0, 0)
    return {
        "system": system, "path": f"{title} (Synthetic){ext}", "name": title, "desc": DESCRIPTION.format(t=title),
        "developer": f"Studio {chr(ord('A') + i % 5)}", "publisher": f"Publisher {chr(ord('P') + i % 3)}",
        "genre": GENRES[i % len(GENRES)], "players": str(1 + i % 4),
        "releasedate": datetime(1990 + i, 1 + i % 12, 1 + i).strftime("%Y%m%dT%H%M%S"), "playcount": str(i),
        "rating": rating(i), "lastplayed": last.strftime("%Y%m%dT%H%M%S") if last else None,
        "favorite": "true" if i % 2 == 0 else None, "completed": "true" if i % 3 == 1 else None,
        "hidemetadata": "true" if system == "snes" and title == HIDDEN else None, "index": i,
    }


def games(system, ext):
    listed = [game(i, t, ext, system) for i, t in enumerate(TITLES)]
    if system == "snes":
        for k, t in enumerate(FOLDER_GAMES):
            g = game(len(TITLES) + k, t, ext, system)
            g["path"] = f"{FOLDER}/{g['path']}"
            listed.append(g)
    return listed


def video(path, size, rgb, text):
    """A still labelled picture held for four seconds as H.264, so a capture of the playing video is the same at any time."""
    still = os.path.splitext(path)[0] + ".still.png"
    label(still, size, rgb, text)
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-loop", "1", "-i", still, "-t", "4", "-r", "30", "-c:v", "libx264", "-pix_fmt",
                    "yuv420p", "-threads", "1", "-an", path], check=True)
    os.remove(still)


def label(path, size, rgb, text):
    w, h = size
    image = Image.new("RGB", size, rgb)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, w - 1, h - 1), outline=(255, 255, 255), width=6)
    font = ImageFont.load_default(size=max(12, h // 12))
    draw.multiline_text((w / 2, h / 2), text, fill=(255, 255, 255), font=font, anchor="mm", align="center")
    image.save(path)


GAME_KEYS = ["path", "name", "desc", "developer", "publisher", "genre", "players", "releasedate", "playcount", "rating",
             "lastplayed", "favorite", "completed", "hidemetadata"]


def gamelist(system, listed):
    rows = ['<?xml version="1.0" encoding="utf-8"?>', "<gameList>"]
    if system == "snes":
        link = next(g["path"] for g in listed if g["path"].startswith(FOLDER + "/"))
        rows += ["  <folder>", f"    <path>./{escape(FOLDER)}</path>", f"    <name>{escape(FOLDER)}</name>",
                 f"    <folderlink>./{escape(link)}</folderlink>", "  </folder>"]
    for g in listed:
        rows.append("  <game>")
        rows += [f"    <{k}>{escape(('./' if k == 'path' else '') + g[k])}</{k}>" for k in GAME_KEYS if g[k] is not None]
        rows.append("  </game>")
    rows.append("</gameList>")
    return "\n".join(rows) + "\n"


def main(out):
    lib_rows = ["system\tpath\tname\tfavorite\tcompleted\thidemetadata"]
    media_rows = ["system\ttype\tfile\twidth\theight\tr\tg\tb"]
    for system, ext in SYSTEMS:
        listed = games(system, ext)
        for g in listed:
            rom = os.path.join(out, "roms", system, g["path"])
            os.makedirs(os.path.dirname(rom), exist_ok=True)
            if not os.path.exists(rom):
                open(rom, "wb").close()
            lib_rows.append("\t".join([system, g["path"], g["name"]] + [g[k] or "false" for k in ("favorite", "completed", "hidemetadata")]))
            types = dict(MEDIA, **(VIEWER_MEDIA if (system, g["index"]) == VIEWER_GAME else {}))
            for folder, size in types.items():
                stem = os.path.splitext(g["path"])[0]
                picture = os.path.join(out, "downloaded_media", system, folder, stem + (".mp4" if folder == "videos" else ".png"))
                rgb = colour(g["index"], folder)
                media_rows.append("\t".join([system, folder, os.path.relpath(picture, out), *map(str, size), *map(str, rgb)]))
                if not os.path.exists(picture):
                    os.makedirs(os.path.dirname(picture), exist_ok=True)
                    (video if folder == "videos" else label)(picture, size, rgb, f"{system} {folder}\n{g['name']}\n{size[0]}x{size[1]}")
        listing = os.path.join(out, "gamelists", system, "gamelist.xml")
        os.makedirs(os.path.dirname(listing), exist_ok=True)
        with open(listing, "w", encoding="utf-8") as f:
            f.write(gamelist(system, listed))
    for name, rows in (("library.tsv", lib_rows), ("media.tsv", media_rows)):
        with open(os.path.join(out, name), "w", encoding="utf-8") as f:
            f.write("\n".join(rows) + "\n")
    print(f"{len(lib_rows) - 1} games, {len(media_rows) - 1} pictures under {out}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
