import QtQuick

// A core's planet on the splash's orbit by its place: the board's angle and its three colours for its first eight, the next eight halfway between two of its angles, the colours in turn; no more places than that (EclipseSplashPlay:160-169; R-20261123-25).
QtObject {
    required property int place
    // The board's eight places, and every place a planet may take: the eight and the eight halfway, constants of the design.
    readonly property int boardPlaces: 8
    readonly property int places: 2 * boardPlaces
    readonly property int k: place % boardPlaces
    readonly property int next: (place + 1) % boardPlaces
    // The board's eight angles in degrees, the place's and the next's.
    readonly property real own: k === 0 ? 205 : k === 1 ? 232 : k === 2 ? 262 : k === 3 ? 300 : k === 4 ? 335 : k === 5 ? 20 : k === 6 ? 150 : 172
    readonly property real after: next === 0 ? 205 : next === 1 ? 232 : next === 2 ? 262 : next === 3 ? 300 : next === 4 ? 335 : next === 5 ? 20 : next === 6 ? 150 : 172
    readonly property real angle: place < boardPlaces ? own : own + ((((after - own) % 360) + 540) % 360 - 180) / 2
    // The light, the middle at 52% and the shade of its radial gradient.
    readonly property color light: k === 0 ? "#F4F1EC" : k === 1 ? "#FFEBB5" : k === 2 ? "#FFC2A8" : k === 3 ? "#D6E6FF" : k === 4 ? "#C8F0DA" : k === 5 ? "#D9F5C4" : k === 6 ? "#FFFFFF" : "#CFE0FF"
    readonly property color middle: k === 0 ? "#C9CCD8" : k === 1 ? "#E8B34B" : k === 2 ? "#D4553F" : k === 3 ? "#7FA6D9" : k === 4 ? "#4FA87A" : k === 5 ? "#6FAE5A" : k === 6 ? "#BFE3F5" : "#3F6FE0"
    readonly property color shade: k === 0 ? "#5E6272" : k === 1 ? "#7A5416" : k === 2 ? "#6B1E16" : k === 3 ? "#2C4670" : k === 4 ? "#1C4A36" : k === 5 ? "#274A1E" : k === 6 ? "#4A7890" : "#162C6B"
}
