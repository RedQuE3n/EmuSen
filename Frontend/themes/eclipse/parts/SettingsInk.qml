import QtQuick
import EmuSen.Theme

// The colours only the Settings boards draw that no kit token or Ink holds, constants of the design with their gen_settings.py lines (EmuSen_Qt.md §3.4.11).
QtObject {
    // The section shown, by its place in the program's sections, which the boards order as gen_settings.py:7-14 does.
    property int section: Settings.section
    // The glow behind a section's page and every page under it, each section its own (G:163, 178, 195, 299, 329); Consoles' board draws none.
    readonly property color pageGlow: section === 0 ? "#2A6FD6" : section === 1 ? "#3B4AA8" : section === 2 ? "#7A3FA8" : section === 4 ? "#1FA890" : section === 5 ? "#B0487A" : "transparent"
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
    // Sound's ring: the fill's end, the track and its glow, rgba(240, 180, 196, 0.22) (G:166).
    readonly property color ringEnd: "#F3C9D2"
    readonly property color ringTrack: "#262A3A"
    readonly property color ringGlow: "#38F0B4C4"
    // Controllers' picture's panel and its 1 px inset line (G:183).
    readonly property color facesGround: "#11131C"
    readonly property color facesLine: "#262A3A"
}
