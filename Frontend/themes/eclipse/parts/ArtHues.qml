import QtQuick
import EmuSen.Theme

// A game's hues for a board's colour fields (Main:19-23, EclipseLibrary:17-18): its art's three most prominent, spread by the board's own offsets where the art has fewer, else its console's hue the same way, else none (EmuSen_Qt.md §3.4.8).
QtObject {
    id: root
    // The game's key and its console's hue in degrees; an empty key for no game.
    property string key
    property real consoleHue: NaN
    readonly property var art: Library.artHuesRead >= 0 && root.key.length > 0 ? Library.artHues(root.key, 3) : []
    readonly property var hues: {
        const art = root.art.length > 0 ? root.art : (root.key.length > 0 ? [root.consoleHue] : [])
        if (art.length === 0)
            return []
        // The board's three hues, 168, 251 and 218 degrees: the second 83 past the first, the third 50 past it.
        const spread = (from, to) => from + ((((to - from) % 360) + 540) % 360 - 180) * 50 / 83
        if (art.length === 1)
            return [art[0], art[0] + 83, art[0] + 50]
        if (art.length === 2)
            return [art[0], art[1], spread(art[0], art[1])]
        return art
    }
}
