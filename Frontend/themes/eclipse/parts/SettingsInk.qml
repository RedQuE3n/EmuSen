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
    // The picture's glow, rgba(120, 140, 255, 0.25) (G:146).
    readonly property color pictureGlow: "#40788CFF"
}
