pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Circular 900x900 CoronaDisc Power dialog (board #15, EclipsePower.dc.html).
FocusScope {
    id: root
    required property Kit.FrameController controller
    required property var rows
    property string title
    property string subtitle
    property int current: 0
    property HelpModel help
    property int padFamily: Kit.Families.Xbox
    property real pixelScale: 1
    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    signal chosen(int index)
    signal focused(int index)
    onCurrentChanged: if (powerList.currentIndex !== root.current) powerList.currentIndex = root.current
    Rectangle {
        objectName: "powerScrim"
        anchors.fill: parent
        color: Qt.rgba(4 / 255, 5 / 255, 10 / 255, 0.74)
    }
    Kit.CoronaDisc {
        id: disc
        objectName: "powerDisc"
        x: (root.width - width) / 2
        y: 150 * root.unit
        width: 900 * root.unit
        height: 900 * root.unit
        fill: Kit.Tokens.surface
        pixelScale: root.pixelScale
    }
    Column {
        id: section
        objectName: "powerSection"
        x: (root.width - width) / 2
        y: root.subtitle.length > 0 ? 300 * root.unit : 150 * root.unit + (900 * root.unit - height) / 2
        width: 600 * root.unit
        spacing: 26 * root.unit
        Column {
            id: header
            objectName: "powerHeader"
            width: section.width
            spacing: 8 * root.unit
            CardLine {
                objectName: "powerTitle"
                unit: root.unit
                boardLine: true
                width: header.width
                text: root.title
                color: Kit.Tokens.textStrong
                size: 46
                weight: Font.DemiBold
                horizontalAlignment: Text.AlignHCenter
            }
            CardLine {
                objectName: "powerSubtitle"
                visible: root.subtitle.length > 0
                unit: root.unit
                boardLine: true
                width: header.width
                text: root.subtitle
                color: "#C9CDDB"
                size: 20
                weight: Font.Normal
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
            }
        }
        ListView {
            id: powerList
            objectName: "powerList"
            width: section.width
            height: count * 84 * root.unit + Math.max(0, count - 1) * spacing
            spacing: 10 * root.unit
            model: root.rows
            focus: true
            interactive: false
            keyNavigationEnabled: true
            keyNavigationWraps: false
            highlightMoveDuration: Kit.Tokens.instant
            highlightResizeDuration: Kit.Tokens.instant
            Accessible.role: Accessible.List
            onCurrentIndexChanged: if (powerList.currentIndex !== root.current) root.focused(powerList.currentIndex)
            delegate: Kit.Walkable {
                id: row
                required property int index
                required property var model
                readonly property bool current: ListView.isCurrentItem
                readonly property color ink: row.current ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                objectName: "powerRow." + row.index
                controller: root.controller
                width: powerList.width
                height: 84 * root.unit
                focus: row.current
                Accessible.role: Accessible.ListItem
                Accessible.name: row.model.label
                onActivated: root.chosen(row.index)
                onActiveFocusChanged: powerList.currentIndex = row.activeFocus ? row.index : powerList.currentIndex
                EclipsePillHighlight {
                    visible: row.current
                    anchors.fill: parent
                    unit: root.unit
                    pixelScale: root.pixelScale
                    radius: 42 * root.unit
                    rowIndex: row.index
                }
                Rectangle {
                    objectName: "powerRowBg." + row.index
                    anchors.fill: parent
                    radius: 42 * root.unit
                    color: row.current ? Kit.Tokens.accent : "transparent"
                    border.width: row.current ? 0 : 2 * root.unit
                    border.color: "#3A3F55"
                }
                Item {
                    id: iconBox
                    objectName: "powerIcon." + row.index
                    readonly property color ink: row.ink
                    x: 26 * root.unit
                    y: (row.height - height) / 2
                    width: 32 * root.unit
                    height: 32 * root.unit
                    Shape {
                        width: 24
                        height: 24
                        scale: parent.width / 24
                        transformOrigin: Item.TopLeft
                        preferredRendererType: Shape.CurveRenderer
                        ShapePath {
                            fillColor: "transparent"
                            strokeColor: iconBox.ink
                            strokeWidth: 1.8
                            capStyle: ShapePath.RoundCap
                            joinStyle: ShapePath.RoundJoin
                            PathSvg { path: Marks.path(row.model.icon) }
                        }
                    }
                }
                CardLine {
                    id: hintLine
                    objectName: "powerHint." + row.index
                    visible: row.model.value.length > 0
                    unit: root.unit
                    boardLine: true
                    x: row.width - 26 * root.unit - width
                    y: (row.height - height) / 2
                    text: row.model.value
                    color: row.current ? "#2A2E40" : Kit.Tokens.muted
                    size: 18
                    weight: Font.Normal
                }
                CardLine {
                    objectName: "powerLabel." + row.index
                    unit: root.unit
                    boardLine: true
                    x: 78 * root.unit
                    y: (row.height - height) / 2
                    width: (hintLine.visible ? hintLine.x - 20 * root.unit : row.width - 26 * root.unit) - x
                    text: row.model.label
                    color: row.ink
                    size: 25
                    weight: row.current ? Font.DemiBold : Font.Normal
                    elide: Text.ElideRight
                }
            }
        }
    }
    EclipseFooter {
        objectName: "powerHelp"
        unit: root.unit
        x: (root.width - width) / 2
        y: root.height - 40 * root.unit - height
        controller: root.controller
        model: root.help
        family: root.padFamily
    }
}
