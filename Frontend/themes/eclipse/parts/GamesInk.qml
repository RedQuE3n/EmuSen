import QtQuick

// The colours only the games view draws that no token holds, constants of the design with their board lines; those other screens draw too are Ink's (EmuSen_Qt.md §3.4.10).
QtObject {
    // A game's description and the empty state's line (EclipseLibrary:44, EclipseEmpty:30).
    readonly property color words: "#C9CDDB"
    // A filter chip at rest: its line and its ground, rgba(7, 8, 15, 0.6); its words are Ink.tileFacts (EclipseLibrary:59).
    readonly property color chipLine: "#4A5068"
    readonly property color chipGround: "#9907080F"
    // The letter rail's track (EclipseLibrary:78).
    readonly property color railTrack: "#3A3F55"
}
