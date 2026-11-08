import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// A menu's row: a label and a value, the focus bar at its left when focused, the hover border; walkable, so a click is A there; a stepped value between ◂ and ▸, a row that opens a page with ▸, a rule at its foot (EmuSen_Kit.md §7.8, §7.21).
Kit.Walkable {
    id: root
    property string label
    property string value
    property bool current: false
    // Left and Right step the value: drawn between two marks.
    property bool steps: false
    // The row opens a page of its own: a mark at its right.
    property bool opens: false
    // A rule along its foot, drawn in Border where the row ends a section.
    property bool ruled: false
    property bool sectionEnd: false
    // The words' distance from the row's edges, and their face.
    property real inset: Kit.Tokens.base * 4
    property font labelFont: Qt.font({ pixelSize: Kit.Tokens.fontRow })
    readonly property real markWidth: 7
    readonly property real markGap: Kit.Tokens.base * 3
    implicitHeight: Kit.Tokens.rowHeight
    Accessible.role: Accessible.ListItem
    Accessible.name: root.label
    Rectangle {
        anchors.fill: parent
        radius: Kit.Tokens.rowRadius
        color: root.activeFocus || root.current ? Kit.Tokens.surfaceRaised : Kit.Tokens.surface
        border.width: root.hovered ? Kit.Tokens.hoverBorder : 0
        border.color: Kit.Tokens.border
        Rectangle {
            width: Kit.Tokens.barWidth
            height: parent.height
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.accent
            visible: root.activeFocus
        }
    }
    Rectangle {
        objectName: "rule"
        visible: root.ruled || root.sectionEnd
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: Kit.Tokens.hoverBorder
        color: root.sectionEnd ? Kit.Tokens.border : Kit.Tokens.rule
    }
    Text {
        objectName: "label"
        text: root.label
        color: root.activeFocus ? Kit.Tokens.textStrong : Kit.Tokens.text
        font: root.labelFont
        elide: Text.ElideRight
        anchors.left: parent.left
        anchors.leftMargin: root.inset
        anchors.right: valueText.left
        anchors.rightMargin: Kit.Tokens.base * 2
        anchors.verticalCenter: parent.verticalCenter
    }
    Shape {
        objectName: "before"
        visible: root.steps
        width: root.markWidth
        height: root.markWidth * 10 / 7
        anchors.right: valueText.left
        anchors.rightMargin: root.markGap
        anchors.verticalCenter: parent.verticalCenter
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillColor: Kit.Tokens.muted
            PathPolyline { path: [Qt.point(root.markWidth, 0), Qt.point(0, root.markWidth * 5 / 7), Qt.point(root.markWidth, root.markWidth * 10 / 7)] }
        }
    }
    Text {
        id: valueText
        objectName: "value"
        text: root.value
        color: Kit.Tokens.muted
        font: root.labelFont
        anchors.right: after.visible ? after.left : parent.right
        anchors.rightMargin: after.visible ? root.markGap : root.inset
        anchors.verticalCenter: parent.verticalCenter
    }
    Shape {
        id: after
        objectName: "after"
        visible: root.steps || root.opens
        width: root.markWidth
        height: root.markWidth * 10 / 7
        anchors.right: parent.right
        anchors.rightMargin: root.inset
        anchors.verticalCenter: parent.verticalCenter
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillColor: Kit.Tokens.muted
            PathPolyline { path: [Qt.point(0, 0), Qt.point(root.markWidth, root.markWidth * 5 / 7), Qt.point(0, root.markWidth * 10 / 7)] }
        }
    }
}
