import QtQuick

// The colours only the Settings boards draw that no kit token or Ink holds, constants of the design with their gen_settings.py lines (EmuSen_Qt.md §3.4.11).
QtObject {
    // The glow behind a section's page, by its page key, as the boards give each section its own (G:163, 178, 195, 299, 329).
    readonly property var pageGlows: ({ "display": "#3B4AA8", "sound": "#7A3FA8", "controllers": "#2A6FD6", "library": "#1FA890", "emusen": "#B0487A" })
    // The limb's fill, its 2 px rim at rgba(246, 231, 216, 0.75) and its glow at rgba(246, 214, 200, 0.35) (G:91).
    readonly property color limb: "#0B0D16"
    readonly property color limbRim: "#BFF6E7D8"
    readonly property color limbGlow: "#F6D6C8"
    readonly property real limbGlowOpacity: 0.35
    // A row at rest and its inset line, 1 px of #22263A (G:62-63).
    readonly property color row: "#0F111A"
    readonly property color rowLine: "#22263A"
    // Each section's icon by its key, a path in a 24-unit box (G:7-14).
    readonly property var sectionIcons: ({
        "controllers": "M6.5 8h11a4.5 4.5 0 0 1 0 9c-1.6 0-2.4-1.5-3.5-1.5h-4c-1.1 0-1.9 1.5-3.5 1.5a4.5 4.5 0 0 1 0-9zM8 11v3M6.5 12.5h3M16 12.5h.01",
        "display": "M4 5h16v11H4zM9 20h6M12 16v4",
        "sound": "M5 10v4h3l5 4V6L8 10zM16.5 9a4 4 0 0 1 0 6M19 6.5a8 8 0 0 1 0 11",
        "consoles": "M4 8h16v9H4zM8 17v3M16 17v3M8 12h3M15 12h.01",
        "library": "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "emusen": "M17 4.5a8.5 8.5 0 1 0 0 15 6.5 6.5 0 1 1 0-15z"
    })
    // The picture's glow, rgba(120, 140, 255, 0.25) (G:146).
    readonly property color pictureGlow: "#40788CFF"
}
