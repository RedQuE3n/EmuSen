pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Top-centered Eclipse notice banner with warning/ok icon, heading, body, and optional X action (board #14, EclipseDialog:23-30).
Item {
    id: root
    required property Kit.NoticeModel model
    required property NoticePresenter presenter
    property int padFamily: Kit.Families.Xbox
    property real pixelScale: 1
    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    Connections {
        target: root.model
        function onRaised() { hold.restart() }
    }
    Timer {
        id: hold
        interval: root.model.holdMs
        onTriggered: root.model.expire()
    }
    Kit.Glow {
        objectName: "noticeShadow"
        visible: card.visible
        opacity: card.opacity
        x: card.x
        y: card.y
        width: card.width
        height: card.height
        radius: card.radius
        color: Qt.rgba(0, 0, 0, 0.6)
        blur: 50 * root.unit
        spread: 0
        offsetX: 0
        offsetY: 18 * root.unit
        pixelScale: root.pixelScale
    }
    Rectangle {
        id: card
        objectName: "notice"
        property real slide: root.model.shown ? 0 : -16 * root.unit
        readonly property bool twoLines: root.presenter.body.length > 0
        x: (root.width - width) / 2
        y: 56 * root.unit + card.slide
        width: Math.min(960 * root.unit, root.width - 48 * root.unit)
        height: Math.max(textCol.implicitHeight, actionPill.visible ? actionPill.height : 0, iconBox.height) + 44 * root.unit
        radius: 22 * root.unit
        color: Kit.Tokens.surfaceRaised
        border.width: 2 * root.unit
        border.color: Kit.Tokens.textStrong
        opacity: root.model.shown ? 1 : 0
        visible: card.opacity > 0
        Accessible.role: Accessible.AlertMessage
        Accessible.name: root.model.words
        Behavior on opacity {
            NumberAnimation {
                duration: root.model.shown ? Kit.Tokens.enterFade : Kit.Tokens.exitFade
                easing.type: root.model.shown ? Kit.Tokens.enterCurve : Kit.Tokens.exitCurve
            }
        }
        Behavior on slide {
            NumberAnimation {
                duration: root.model.shown ? Kit.Tokens.enter : Kit.Tokens.exit
                easing.type: root.model.shown ? Kit.Tokens.enterCurve : Kit.Tokens.exitCurve
            }
        }
        Item {
            id: iconBox
            objectName: "noticeIcon"
            readonly property color ink: Kit.Tokens.textStrong
            x: 26 * root.unit
            y: card.twoLines ? 24 * root.unit : (card.height - height) / 2
            width: 34 * root.unit
            height: 34 * root.unit
            Shape {
                width: 24
                height: 24
                scale: parent.width / 24
                transformOrigin: Item.TopLeft
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: "transparent"
                    strokeColor: iconBox.ink
                    strokeWidth: 2
                    capStyle: ShapePath.RoundCap
                    joinStyle: ShapePath.RoundJoin
                    PathSvg { path: Marks.path(root.presenter.icon) }
                }
            }
        }
        Column {
            id: textCol
            objectName: "noticeTextCol"
            x: 78 * root.unit
            y: card.twoLines ? 22 * root.unit : (card.height - height) / 2
            width: (actionPill.visible ? actionPill.x - 18 * root.unit : card.width - 26 * root.unit) - x
            spacing: 6 * root.unit
            CardLine {
                objectName: "noticeHeading"
                unit: root.unit
                boardLine: true
                width: textCol.width
                text: root.presenter.heading
                color: Kit.Tokens.textStrong
                size: 24
                weight: Font.DemiBold
                wrapMode: Text.WordWrap
            }
            CardLine {
                objectName: "noticeBody"
                visible: card.twoLines
                unit: root.unit
                boardLine: true
                boardLineHeight: 27.55
                width: textCol.width
                text: root.presenter.body
                color: "#D8DBE6"
                size: 19
                weight: Font.Normal
                wrapMode: Text.WordWrap
                lineHeight: 27.55
                lineHeightMode: Text.FixedHeight
            }
        }
        Item {
            id: actionPill
            objectName: "noticeAction"
            visible: root.presenter.hasAction
            x: card.width - 26 * root.unit - width
            y: card.twoLines ? 22 * root.unit : (card.height - height) / 2
            width: actionGlyph.width + 10 * root.unit + actionText.implicitWidth
            height: Math.max(actionGlyph.height, actionText.implicitHeight)
            CardGlyph {
                id: actionGlyph
                objectName: "noticeActionGlyph"
                unit: root.unit
                x: 0
                y: (actionPill.height - height) / 2
                button: Kit.Pad.Screensaver
                family: root.padFamily
                lead: false
                side: 32
                letterSize: 15
                ring: Kit.Tokens.muted
                ink: "#D8DBE6"
            }
            CardLine {
                id: actionText
                objectName: "noticeActionLabel"
                unit: root.unit
                boardLine: true
                x: actionGlyph.width + 10 * root.unit
                y: (actionPill.height - height) / 2
                text: root.presenter.actionLabel
                color: "#D8DBE6"
                size: 18
                weight: Font.Normal
            }
            TapHandler {
                onTapped: root.presenter.invokeAction()
            }
        }
    }
}
