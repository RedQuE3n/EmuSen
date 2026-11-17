import QtQuick

// The colours only the games view draws that no token holds, constants of the design with their board lines; those other screens draw too are Ink's (EmuSen_Qt.md §3.4.10).
QtObject {
    // The empty state's line and a game's description (EclipseEmpty:30, EclipseLibrary:44).
    readonly property color words: "#C9CDDB"
    // A filter chip at rest: its ground, rgba(7, 8, 15, 0.6); its line is Ink.chipLine and its words Ink.tileFacts (EclipseLibrary:56).
    readonly property color chipGround: "#9907080F"
    // The path's slash (EclipseLibrary:27).
    readonly property color slash: "#8E94AB"
    // A resting tile's shadow, rgba(0, 0, 0, 0.4) (EclipseLibrary:66).
    readonly property color tileShadow: "#66000000"
    // The letter rail's track (EclipseLibrary:77).
    readonly property color railTrack: "#3A3F55"
}
