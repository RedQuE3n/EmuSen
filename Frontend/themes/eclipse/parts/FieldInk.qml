import QtQuick

// A board colour field's colour for a game (Main:19-23): the board colour's saturation, lightness and alpha with the game's hue
// in degrees; the board colour itself where no hue is given (EmuSen_Qt.md §3.4.8).
QtObject {
    required property color board
    // The hue in degrees, any value, wrapped; below none, NaN, the board's own colour.
    property real hue: NaN
    readonly property color color: isNaN(hue) ? board : Qt.hsla((((hue % 360) + 360) % 360) / 360, board.hslSaturation, board.hslLightness, board.a)
}
