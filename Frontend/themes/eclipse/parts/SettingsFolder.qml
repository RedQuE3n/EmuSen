pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Library's ROM folder count card: big grouped game count, "games under" label, and monospace path in a 28 px padded panel (EclipseSettingsLibrary:67-71; EmuSen_Qt.md §3.4.11).
Rectangle {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    property string countText
    property string underText
    property string pathText
    objectName: "asideFolder"
    width: parent ? parent.width : 0
    height: content.height + 56 * root.unit
    radius: 22 * root.unit
    color: own.facesGround
    // Its 1 px line inside its edge (EclipseSettingsLibrary:67).
    border.width: root.unit
    border.color: own.facesLine
    SettingsInk {
        id: own
    }
    Column {
        id: content
        x: 28 * root.unit
        y: 28 * root.unit
        width: root.width - 56 * root.unit
        spacing: 10 * root.unit
        Line {
            objectName: "asideFolderCount"
            unit: root.unit
            boardLine: true
            width: parent.width
            text: root.countText
            color: Tokens.text
            size: 64
            spacing: -0.02 * 64
            weight: Tokens.weightTitle
        }
        Line {
            objectName: "asideFolderUnder"
            unit: root.unit
            boardLine: true
            width: parent.width
            text: root.underText
            color: Tokens.muted
            size: 20
        }
        Line {
            objectName: "asideFolderPath"
            unit: root.unit
            boardLine: lineCount <= 1
            width: parent.width
            text: root.pathText
            color: Tokens.accent
            family: Tokens.faceCode
            wrapMode: Text.WrapAnywhere
            lineHeightMode: Text.FixedHeight
            lineHeight: 26
            size: 20
        }
    }
}
