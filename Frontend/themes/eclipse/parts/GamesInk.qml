import QtQuick

// The colours the games view and its empty state draw that no token holds, constants of the design with their board lines (EmuSen_Qt.md §3.4.7.2).
QtObject {
    // A game's description and the empty state's line (EclipseLibrary:44, EclipseEmpty:30).
    readonly property color words: "#C9CDDB"
    // The empty state's ground (EclipseEmpty:17).
    readonly property color emptyGround: "#05060B"
    // A filter chip at rest: its line and its ground, rgba(7, 8, 15, 0.6) (EclipseLibrary:59).
    readonly property color chipLine: "#4A5068"
    readonly property color chipGround: "#9907080F"
    // The letter rail's track (EclipseLibrary:78).
    readonly property color railTrack: "#3A3F55"
    // Drawn by Home's board too, so Home's shared Ink holds them once its commit is merged: a chip's words (EclipseLibrary:59, Main),
    // the pill's words (EclipseLibrary:62, Main) and its ground, rgba(7, 8, 15, 0.78) (EclipseLibrary:62, Main), the eyebrow's rose (EclipseLibrary:38, Main:54).
    readonly property color chipWords: "#E3E6F0"
    readonly property color pillWords: "#D8DBE6"
    readonly property color pillGround: "#C707080F"
    readonly property color rose: "#F3C9D2"
}
