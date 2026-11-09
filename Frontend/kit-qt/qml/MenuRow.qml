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
    // The row stands on a raised surface, a question's card: clear at rest, Surface when focused (EmuSen_Kit.md §7.24).
    property bool overRaised: false
    // The marks' sizes as Mistress's render draws them: a step's ◂ and ▸, and the larger ▸ of a row that opens a page.
    readonly property size stepMark: Qt.size(8, 11)
    readonly property size openMark: Qt.size(9, 13)
    readonly property size afterMark: root.steps ? root.stepMark : root.openMark
    readonly property real markGap: Kit.Tokens.base * 3
    implicitHeight: Kit.Tokens.rowHeight
    Accessible.role: Accessible.ListItem
    Accessible.name: root.label
    Rectangle {
        anchors.fill: parent
        radius: Kit.Tokens.rowRadius
        color: root.activeFocus || root.current ? (root.overRaised ? Kit.Tokens.surface : Kit.Tokens.surfaceRaised) : (root.overRaised ? "transparent" : Kit.Tokens.surface)
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
        // The software backend paints a Shape wholly past its clip unclipped, so there it is not drawn while wholly past (EmuSen_Kit.md §7.34, §7.40).
        visible: root.steps && (GraphicsInfo.api !== GraphicsInfo.Software || beforeShown.inside)
        width: root.stepMark.width
        height: root.stepMark.height
        anchors.right: valueText.left
        anchors.rightMargin: root.markGap
        anchors.verticalCenter: parent.verticalCenter
        preferredRendererType: Shape.CurveRenderer
        Kit.InsideClips { id: beforeShown; anchors.fill: parent }
        ShapePath {
            strokeWidth: -1
            fillColor: Kit.Tokens.muted
            startX: root.stepMark.width
            startY: 0
            PathLine { x: 0; y: root.stepMark.height / 2 }
            PathLine { x: root.stepMark.width; y: root.stepMark.height }
        }
    }
    Text {
        id: valueText
        objectName: "value"
        text: root.value
        color: Kit.Tokens.muted
        font: root.labelFont
        // At most half the row, a long value (a folder's path) cut at its start so its end shows (EmuSen_Kit.md §7.46).
        width: Math.min(valueText.implicitWidth, root.width / 2)
        elide: Text.ElideLeft
        anchors.right: after.visible ? after.left : parent.right
        anchors.rightMargin: after.visible ? root.markGap : root.inset
        anchors.verticalCenter: parent.verticalCenter
    }
    Shape {
        id: after
        objectName: "after"
        visible: (root.steps || root.opens) && (GraphicsInfo.api !== GraphicsInfo.Software || afterShown.inside)
        width: root.afterMark.width
        height: root.afterMark.height
        anchors.right: parent.right
        anchors.rightMargin: root.inset
        anchors.verticalCenter: parent.verticalCenter
        preferredRendererType: Shape.CurveRenderer
        Kit.InsideClips { id: afterShown; anchors.fill: parent }
        ShapePath {
            strokeWidth: -1
            fillColor: Kit.Tokens.muted
            startX: 0
            startY: 0
            PathLine { x: root.afterMark.width; y: root.afterMark.height / 2 }
            PathLine { x: 0; y: root.afterMark.height }
        }
    }
}
