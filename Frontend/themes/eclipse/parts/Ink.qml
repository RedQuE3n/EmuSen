import QtQuick

// The boards' colours that no kit token carries, each with the board line that draws it (EmuSen_Qt.md §3.4.8): constants of the design.
QtObject {
    // The ground of Home and the games (Main:17, EclipseLibrary:17); Consoles' (EclipseConsoles:17) is Tokens.surface, read from the token.
    readonly property color ground: "#0A1024"
    // The ground of an empty library (EclipseEmpty:17).
    readonly property color emptyGround: "#05060B"
    // Home's eyebrow (Main:54).
    readonly property color rose: "#F3C9D2"
    // The help line's words (Main:99, EclipseConsoles:99, EclipseLibrary:82).
    readonly property color help: "#D8DBE6"
    // The status and the help line's glass, rgba(7, 8, 15, 0.78) (Main:40, EclipseLibrary:32, 88).
    readonly property color glass: "#C707080F"
    // A tile's title and its facts (Main:85, 86).
    readonly property color tileTitle: "#FFFFFF"
    readonly property color tileFacts: "#E3E6F0"
    // The shade under a tile's words, rgba(4, 5, 10, 0.92) to clear (Main:84).
    readonly property color shade: "#EB04050A"
    readonly property color shadeClear: "#0004050A"
}
