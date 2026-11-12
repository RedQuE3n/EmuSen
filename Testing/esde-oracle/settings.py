#!/usr/bin/python3 -I
"""Writes ES-DE's es_settings.xml for one oracle state: the oracle's fixed settings (no updater, no scraper account, no
sounds, no screensaver, no device notices, help and clock on), then the state's own as Name=value pairs. Only the names
in the table below are accepted, each with ES-DE's type; ES-DE gives every setting not written its own default.
See README.md.

Usage: settings.py <es_settings.xml> Name=value...
"""
import sys
from xml.sax.saxutils import quoteattr

TYPES = {
    "ApplicationUpdaterFrequency": "string", "ScraperUseAccountScreenScraper": "bool", "ScraperUsernameScreenScraper": "string",
    "ScraperPasswordScreenScraper": "string", "NavigationSounds": "bool", "ScreensaverTimer": "int",
    "InputDeviceNotifications": "bool", "ShowHelpPrompts": "bool", "DisplayClock": "bool", "ROMDirectory": "string",
    "MediaDirectory": "string", "Theme": "string", "ThemeVariant": "string", "ThemeColorScheme": "string",
    "ThemeFontSize": "string", "ThemeAspectRatio": "string", "ThemeTransitions": "string", "StartupSystem": "string",
    "StartupView": "string", "MediaViewerHelpPrompts": "string", "MediaViewerShowTypes": "bool",
    "MediaViewerKeepVideoRunning": "bool", "MediaViewerStretchVideos": "bool", "MediaViewerVideoAudio": "bool",
    "MediaViewerVideoBlur": "bool", "MediaViewerVideoScanlines": "bool", "MediaViewerScreenshotScanlines": "bool",
}
FIXED = {
    "ApplicationUpdaterFrequency": "never", "ScraperUseAccountScreenScraper": "false", "ScraperUsernameScreenScraper": "",
    "ScraperPasswordScreenScraper": "", "NavigationSounds": "false", "ScreensaverTimer": "0",
    "InputDeviceNotifications": "false", "ShowHelpPrompts": "true", "DisplayClock": "true",
}
LOCKED = {"ApplicationUpdaterFrequency", "ScraperUseAccountScreenScraper", "ScraperUsernameScreenScraper", "ScraperPasswordScreenScraper"}


def settings(pairs):
    values = dict(FIXED)
    for pair in pairs:
        name, sep, value = pair.partition("=")
        if not sep or name not in TYPES:
            raise SystemExit(f"settings.py: not a setting the oracle writes: {pair}")
        if name in LOCKED and value != FIXED[name]:
            raise SystemExit(f"settings.py: {name} stays {FIXED[name]!r}")
        values[name] = value
    rows = ['<?xml version="1.0"?>'] + [f"<{TYPES[n]} name={quoteattr(n)} value={quoteattr(v)} />" for n, v in sorted(values.items())]
    return "\n".join(rows) + "\n"


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    text = settings(sys.argv[2:])
    with open(sys.argv[1], "w", encoding="utf-8") as f:
        f.write(text)
