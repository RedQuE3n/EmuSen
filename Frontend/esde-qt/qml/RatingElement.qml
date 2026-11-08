pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// Five stars across the box: the filled ones to the rating's fraction, over the unfilled or beside them.
ElementBox {
    id: root
    required property real value
    required property url filledSource
    required property string filledMark
    required property url unfilledSource
    required property string unfilledMark
    required property color tint
    required property bool overlay
    readonly property real star: root.width / 5
    Item {
        x: root.overlay ? 0 : root.width * root.value
        width: root.overlay ? root.width : root.width * (1 - root.value)
        height: root.height
        clip: true
        Row {
            x: -parent.x
            Repeater {
                model: 5
                Icon {
                    width: root.star
                    height: root.height
                    source: root.unfilledSource
                    mark: root.unfilledMark
                    tint: root.tint
                }
            }
        }
    }
    Item {
        width: root.width * root.value
        height: root.height
        clip: true
        Row {
            Repeater {
                model: 5
                Icon {
                    width: root.star
                    height: root.height
                    source: root.filledSource
                    mark: root.filledMark
                    tint: root.tint
                }
            }
        }
    }
}
