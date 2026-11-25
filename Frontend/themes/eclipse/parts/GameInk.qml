import QtQuick

// The colours only the game page draws that no token or Ink entry holds, constants of the design with their board lines (EmuSen_Qt.md §3.4.12).
QtObject {
    // The horizon's line: rgba(246, 231, 216, 0.55) between 30% and 70% of the width, clear at its ends (EclipseGame:24).
    readonly property color horizon: "#8CF6E7D8"
    readonly property color horizonClear: "#00F6E7D8"
    // A state card at rest: its shadow, 0 8px 24px rgba(0, 0, 0, 0.45) (EclipseGame:105).
    readonly property color cardShadow: "#73000000"
    // The empty card where the game has no state: its dashed line and its words (EclipseGameNoState:64-65).
    readonly property color restLine: "#3A3F55"
    readonly property color restWords: "#8E94AB"
    // Crop preview frame, edge shades and inactive edge rule (EclipseGameCrop:56, 62-63).
    readonly property color cropFrame: "#12205A"
    readonly property color cropShade: "#D107080F"
    readonly property color cropLineDim: "#73F6E7D8"
    // Scrape This Game dialog shadow and focused match halo/glow (EclipseGameScrape:43, 68).
    readonly property color dialogShadow: "#80000000"
    readonly property color matchRing: "#2EF6E7D8"
    readonly property color matchGlow: "#66F0B4C4"
}
