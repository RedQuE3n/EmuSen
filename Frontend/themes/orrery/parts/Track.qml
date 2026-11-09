import QtQuick
import EmuSen.Theme

// A position track for a list longer than it shows: 2 units of Border at 35%, its thumb Border at 100% and at least 24 long (orrery.md §3).
Item {
    id: track
    property bool horizontal
    // The share shown and where the shown part starts, both of the whole.
    property real shown: 1
    property real at: 0
    readonly property real length: track.horizontal ? track.width : track.height
    readonly property real thumb: Math.max(24, track.length * Math.min(1, track.shown))
    visible: track.shown < 1
    Rectangle {
        anchors.fill: parent
        radius: 1
        color: Tokens.border
        opacity: 0.35
    }
    Rectangle {
        objectName: "trackThumb"
        x: track.horizontal ? (track.length - track.thumb) * Math.min(1, track.at / Math.max(0.0001, 1 - track.shown)) : 0
        y: track.horizontal ? 0 : (track.length - track.thumb) * Math.min(1, track.at / Math.max(0.0001, 1 - track.shown))
        width: track.horizontal ? track.thumb : track.width
        height: track.horizontal ? track.height : track.thumb
        radius: 1
        color: Tokens.border
    }
}
