import QtQuick

// The boards' colours that no kit token carries, each with the board line that draws it (EmuSen_Qt.md §3.4.8): constants of the design.
QtObject {
    // The ground of Home and the games (Main:17, EclipseLibrary:17); Consoles' (EclipseConsoles:17) is Tokens.surface, read from the token.
    readonly property color ground: "#0A1024"
    // Home's five colour fields (Main:19-23): their saturation, lightness and alpha are the board's; a game's art gives the hue.
    readonly property color field0: "#1FC9A8"
    readonly property color field1: "#6A4CF0"
    readonly property color field2: "#1D6BF2"
    readonly property color field3: "#B8FFE9"
    readonly property color field4: "#D9C8FF"
    // The splash's ground and its moon, its stars, its sun and the counts under its name (EclipseSplashPlay:31, 41, 51, 55, 71).
    readonly property color splashGround: "#05060B"
    readonly property color star: "#E9ECF7"
    readonly property color sun: "#FFF4E6"
    readonly property color splashCounts: "#C9CDDB"
    // The orbit's line, the sun's glow and the corona's three, rgba as the board writes them (EclipseSplashPlay:45, 51, 54).
    readonly property color orbit: Qt.rgba(246 / 255, 231 / 255, 216 / 255, 0.3)
    readonly property color sunGlow: Qt.rgba(255 / 255, 240 / 255, 225 / 255, 0.6)
    readonly property color coronaNear: Qt.rgba(255 / 255, 244 / 255, 230 / 255, 0.85)
    readonly property color coronaMiddle: Qt.rgba(246 / 255, 214 / 255, 200 / 255, 0.5)
    readonly property color coronaFar: Qt.rgba(236 / 255, 160 / 255, 186 / 255, 0.26)
    // The flare's point and its two glows, and its two bars' clear ends and middles (EclipseSplashPlay:59-61).
    readonly property color flare: "#FFFFFF"
    readonly property color flareNear: Qt.rgba(255 / 255, 250 / 255, 240 / 255, 0.9)
    readonly property color flareFar: Qt.rgba(255 / 255, 230 / 255, 215 / 255, 0.55)
    readonly property color flareClear: Qt.rgba(1, 1, 1, 0)
    readonly property color flareAcross: Qt.rgba(1, 1, 1, 0.95)
    readonly property color flareDown: Qt.rgba(1, 1, 1, 0.9)
    // The line under the splash's name, clear at its ends (EclipseSplashPlay:70).
    readonly property color rule: Qt.rgba(246 / 255, 231 / 255, 216 / 255, 0.7)
    readonly property color ruleClear: Qt.rgba(246 / 255, 231 / 255, 216 / 255, 0)
    // The ground of an empty library (EclipseEmpty:17).
    readonly property color emptyGround: "#05060B"
    // The words under an empty library's headline (EclipseEmpty:30).
    readonly property color emptyAbout: "#C9CDDB"
    // Home's eyebrow (Main:54).
    readonly property color rose: "#F3C9D2"
    // The help line's words (Main:99, EclipseConsoles:99, EclipseLibrary:82).
    readonly property color help: "#D8DBE6"
    // The status and the help line's glass, rgba(7, 8, 15, 0.78) (Main:40, EclipseLibrary:32, 88).
    readonly property color glass: "#C707080F"
    // The resume point's picture's inset rim, rgba(246, 231, 216, 0.5) (Main:58).
    readonly property color resumeRim: "#80F6E7D8"
    // An outlined pill's line (Main:65, 68).
    readonly property color pillLine: "#5A6076"
    // A chip's line, unpressed (EclipseConsoles:44, 45).
    readonly property color chipLine: "#4A5068"
    // A console plate's line where it holds no game (EclipseConsoles:129).
    readonly property color emptyLine: "#8E94AB"
    // A console plate's inset edge and its played share's track (EclipseConsoles:64, 66).
    readonly property color plateEdge: "#262A3A"
    readonly property color track: "#2C3044"
    // The focus ring's glow round a tile or a plate (Main:113, EclipseConsoles:64).
    readonly property color ringGlow: "#80F0B4C4"
    // The section shown's words in the header's nav (Main:34, EclipseConsoles:31).
    readonly property color current: "#FFFFFF"
    // A tile's title and its facts (Main:85, 86).
    readonly property color tileTitle: "#FFFFFF"
    readonly property color tileFacts: "#E3E6F0"
    // The shade under a tile's words, rgba(4, 5, 10, 0.92) to clear (Main:84).
    readonly property color shade: "#EB04050A"
    readonly property color shadeClear: "#0004050A"
}
