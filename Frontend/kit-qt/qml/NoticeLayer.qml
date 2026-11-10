import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import EmuSen.Kit as Kit

// The notice at the top right inside the safe area: a raised card with a status glyph and the words, entering over Enter, held, leaving over Exit; one at a time (EmuSen_Interface.md §18.6, EmuSen_Kit.md §7.23).
Item {
    id: root
    required property Kit.NoticeModel model
    property font labelFont: Qt.font({ pixelSize: Kit.Tokens.fontBody })
    // The glyph's box in design units.
    property real glyphSize: Kit.Tokens.base * 5
    // The shadow's darkness and blur, §5.4's raised shadow from the tokens.
    property real shadowOpacity: Kit.Tokens.shadowOpacity
    property real shadowRadius: Kit.Tokens.shadowBlur
    Connections {
        target: root.model
        function onRaised() { hold.restart() }
    }
    Timer {
        id: hold
        interval: root.model.holdMs
        onTriggered: root.model.expire()
    }
    // The raised card's one shadow, offset 0,8, blur 24, black at 50% (EmuSen_Interface.md §5.4, §18.6), cast by a copy
    // of its shape as the cards cast theirs; the software backend draws no MultiEffect (EmuSen_Kit.md §7.23).
    Rectangle {
        id: caster
        x: card.x
        y: card.y
        width: card.width
        height: card.height
        radius: card.radius
        color: card.color
        visible: false
    }
    MultiEffect {
        objectName: "noticeShadow"
        source: caster
        anchors.fill: caster
        visible: card.visible
        opacity: card.opacity
        shadowEnabled: true
        shadowColor: Kit.Tokens.shadowColor
        shadowOpacity: root.shadowOpacity
        shadowHorizontalOffset: Kit.Tokens.shadowOffsetX
        shadowVerticalOffset: Kit.Tokens.shadowOffsetY
        blurMax: root.shadowRadius
        shadowBlur: 1
    }
    Rectangle {
        id: card
        objectName: "notice"
        property real slide: root.model.shown ? 0 : -Kit.Tokens.base * 4
        x: root.width - Kit.Tokens.safeAreaX - width
        y: Kit.Tokens.safeAreaY + card.slide
        width: row.width + 2 * Kit.Tokens.paneGutter
        height: row.height + 2 * (Kit.Tokens.base * 3.5)
        radius: Kit.Tokens.tileRadius
        color: Kit.Tokens.surfaceRaised
        border.width: Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
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
        Row {
            id: row
            anchors.centerIn: parent
            spacing: Kit.Tokens.base * 3
            Shape {
                objectName: "noticeGlyph"
                anchors.verticalCenter: parent.verticalCenter
                width: root.glyphSize
                height: root.glyphSize
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeWidth: root.model.attention ? -1 : root.glyphSize / 8
                    strokeColor: root.model.attention ? "transparent" : Kit.Tokens.success
                    fillColor: root.model.attention ? Kit.Tokens.attention : "transparent"
                    capStyle: ShapePath.RoundCap
                    joinStyle: ShapePath.RoundJoin
                    startX: root.model.attention ? root.glyphSize / 2 : root.glyphSize * 0.12
                    startY: root.model.attention ? root.glyphSize * 0.05 : root.glyphSize * 0.55
                    PathLine {
                        x: root.model.attention ? root.glyphSize : root.glyphSize * 0.4
                        y: root.model.attention ? root.glyphSize * 0.95 : root.glyphSize * 0.82
                    }
                    PathLine {
                        x: root.model.attention ? 0 : root.glyphSize * 0.9
                        y: root.model.attention ? root.glyphSize * 0.95 : root.glyphSize * 0.18
                    }
                }
            }
            Text {
                objectName: "noticeWords"
                anchors.verticalCenter: parent.verticalCenter
                text: root.model.words
                color: Kit.Tokens.textStrong
                font: root.labelFont
            }
        }
    }
}
