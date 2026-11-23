import QtQuick
import EmuSen.Theme

// A line's box as CSS's "normal" line height draws it, the face's ascent and descent each rounded to a whole board pixel, where a Text's own height rounds their sum up (EmuSen_Qt.md §3.4.11).
FontMetrics {
    // The words' size and weight, in board pixels.
    property real size: 16
    property int weight: Font.Normal
    readonly property real box: Math.round(ascent) + Math.round(descent)
    font.family: Tokens.faceText
    font.pixelSize: size
    font.weight: weight
}
