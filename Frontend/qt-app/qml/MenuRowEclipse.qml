pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// One row of an Eclipse card: Home Menu Continue card, Home Menu icon row, or standard pill row (boards #5, #44-#46).
Kit.Walkable {
    id: root
    required property real unit
    required property real pixelScale
    required property int rowIndex
    required property int padFamily
    property bool menu: false
    property bool continueCard: false
    property string continueTitle
    property string continueSubtitle
    property string continuePicture
    property string label
    property string value
    property string icon
    property bool steps: false
    property bool opens: false
    property bool sectionEnd: false
    property bool current: false
    readonly property real boxHeightPx: root.continueCard ? 118 : (root.menu ? 72 : (root.current ? 72 : 66))
    readonly property real boxH: root.boxHeightPx * root.unit
    readonly property real pillRadius: (root.continueCard ? 22 : (root.menu ? 18 : (root.current ? 36 : 18))) * root.unit
    readonly property bool showPill: root.current || root.continueCard
    readonly property bool showGlow: root.current
    readonly property bool showGlyph: !root.menu && !root.continueCard && root.current
    readonly property bool showSteps: !root.menu && !root.continueCard && root.steps && root.current
    readonly property bool showChevron: !root.menu && !root.continueCard && root.opens
    readonly property color ink: root.current ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
    readonly property color mutedInk: root.current ? ThemeInk.overAccentMuted : Kit.Tokens.muted
    readonly property real labelLeft: (root.menu ? 70 : (root.current ? 76 : 22)) * root.unit
    readonly property real valueRight: root.menu ? root.width - 20 * root.unit : (root.showSteps ? root.width - 58 * root.unit : (root.showChevron ? root.width - 60 * root.unit : root.width - 22 * root.unit))
    readonly property real rightEdge: root.showSteps ? stepPrev.x - 16 * root.unit : (root.value.length > 0 ? valueText.x - (root.menu ? 20 : 16) * root.unit : (root.showChevron ? chevronMark.x - 16 * root.unit : root.width - (root.menu ? 20 : 22) * root.unit))
    implicitHeight: root.boxH + (root.continueCard ? 28 * root.unit : (root.sectionEnd ? 27 * root.unit : 0))
    Accessible.role: Accessible.ListItem
    Accessible.name: root.continueCard ? root.continueTitle : root.label
    Kit.Glow {
        objectName: "rowGlow." + root.rowIndex
        visible: root.showGlow
        x: 0
        y: 0
        width: root.width
        height: root.boxH
        radius: root.pillRadius
        color: Kit.Tokens.glowColor
        blur: 46 * root.unit
        spread: 6 * root.unit
        pixelScale: root.pixelScale
    }
    Rectangle {
        objectName: "rowRing." + root.rowIndex
        visible: root.showGlow
        x: -6 * root.unit
        y: -6 * root.unit
        width: root.width + 12 * root.unit
        height: root.boxH + 12 * root.unit
        radius: root.pillRadius + 6 * root.unit
        color: Qt.alpha(Kit.Tokens.accent, 0.18)
    }
    Rectangle {
        objectName: "rowBg." + root.rowIndex
        visible: root.showPill
        x: 0
        y: 0
        width: root.width
        height: root.boxH
        radius: root.pillRadius
        color: root.current ? Kit.Tokens.accent : ThemeInk.rowSurface
        border.width: !root.current && root.continueCard ? 1 * root.unit : 0
        border.color: ThemeInk.hairline
    }
    Item {
        id: continueGroup
        visible: root.continueCard
        x: 0
        y: 0
        width: root.width
        height: root.boxH
        Rectangle {
            objectName: "continueArt"
            x: 16 * root.unit
            y: 17 * root.unit
            width: 132 * root.unit
            height: 84 * root.unit
            radius: 14 * root.unit
            gradient: Gradient {
                GradientStop { position: 0.0; color: Kit.Tokens.success }
                GradientStop { position: 50 / 84; color: Kit.Tokens.success }
                GradientStop { position: 50 / 84 + 0.0001; color: ThemeInk.rose }
                GradientStop { position: 1.0; color: ThemeInk.rose }
            }
            Kit.RoundPicture {
                anchors.fill: parent
                visible: root.continuePicture.length > 0
                source: root.continuePicture
                radius: 14 * root.unit
                crop: true
                pixelScale: root.pixelScale
            }
        }
        Column {
            x: 168 * root.unit
            y: 16 * root.unit
            width: root.width - 184 * root.unit
            spacing: 4 * root.unit
            CardLine {
                objectName: "continueTag"
                unit: root.unit
                boardLine: true
                width: parent.width
                text: Words.continueEntry
                capitalization: Font.AllUppercase
                color: root.current ? Kit.Tokens.overAccent : Kit.Tokens.muted
                size: 17
                weight: Font.Bold
                spacing: 0.14 * 17
            }
            CardLine {
                objectName: "continueTitle"
                unit: root.unit
                boardLine: true
                width: parent.width
                text: root.continueTitle
                color: root.current ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                size: 28
                weight: Font.DemiBold
                elide: Text.ElideRight
            }
            CardLine {
                objectName: "continueSubtitle"
                unit: root.unit
                boardLine: true
                width: parent.width
                text: root.continueSubtitle
                color: root.current ? Kit.Tokens.overAccent : Kit.Tokens.muted
                size: 18
                weight: Font.Normal
                elide: Text.ElideRight
            }
        }
    }
    CardGlyph {
        objectName: "rowGlyph." + root.rowIndex
        visible: root.showGlyph
        unit: root.unit
        x: 14 * root.unit
        y: (root.boxH - height) / 2
        button: Kit.Pad.Accept
        family: root.padFamily
        lead: true
        side: 46
        leadLetterSize: 21
        leadWeight: Font.DemiBold
        leadFill: Kit.Tokens.overAccent
        leadInk: Kit.Tokens.accent
    }
    Item {
        id: rowIconBox
        objectName: "rowIcon." + root.rowIndex
        visible: root.menu && !root.continueCard && root.icon.length > 0
        readonly property color ink: root.ink
        x: 20 * root.unit
        y: (root.boxH - height) / 2
        width: 30 * root.unit
        height: 30 * root.unit
        Shape {
            width: 24
            height: 24
            scale: parent.width / 24
            transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: "transparent"
                strokeColor: rowIconBox.ink
                strokeWidth: 1.7
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                PathSvg { path: Marks.path(root.icon) }
            }
        }
    }
    FontMetrics {
        id: labelMetrics
        font: labelText.font
    }
    FontMetrics {
        id: valueMetrics
        font: valueText.font
    }
    Text {
        id: labelText
        objectName: "label"
        visible: !root.continueCard
        text: root.label
        color: root.ink
        font.family: Kit.Tokens.faceText
        font.pixelSize: root.menu ? 25 : 24
        font.weight: !root.menu && root.current ? Font.DemiBold : Font.Normal
        elide: Text.ElideRight
        x: root.labelLeft
        y: ((root.boxHeightPx - (Math.round(labelMetrics.ascent) + Math.round(labelMetrics.descent))) / 2 + Math.round(labelMetrics.ascent) - labelText.baselineOffset) * root.unit
        width: Math.max(0, (root.rightEdge - root.labelLeft) / root.unit)
        scale: root.unit
        transformOrigin: Item.TopLeft
    }
    CardChevron {
        id: stepPrev
        objectName: "rowStepPrev." + root.rowIndex
        visible: root.showSteps
        unit: root.unit
        back: true
        side: 20
        ink: root.ink
        x: valueText.x - 36 * root.unit
        y: (root.boxH - height) / 2
    }
    Text {
        id: valueText
        objectName: "value"
        visible: !root.continueCard && root.value.length > 0
        text: root.value
        color: root.mutedInk
        font.family: Kit.Tokens.faceText
        font.pixelSize: 19
        font.weight: Font.Normal
        x: root.valueRight - valueText.implicitWidth * root.unit
        y: ((root.boxHeightPx - (Math.round(valueMetrics.ascent) + Math.round(valueMetrics.descent))) / 2 + Math.round(valueMetrics.ascent) - valueText.baselineOffset) * root.unit
        scale: root.unit
        transformOrigin: Item.TopLeft
    }
    CardChevron {
        id: stepNext
        objectName: "rowStepNext." + root.rowIndex
        visible: root.showSteps
        unit: root.unit
        side: 20
        ink: root.ink
        x: root.width - 42 * root.unit
        y: (root.boxH - height) / 2
    }
    CardChevron {
        id: chevronMark
        objectName: "rowChevron." + root.rowIndex
        visible: root.showChevron
        unit: root.unit
        side: 22
        ink: root.ink
        x: root.width - 44 * root.unit
        y: (root.boxH - height) / 2
    }
    Rectangle {
        objectName: "rowDivider." + root.rowIndex
        visible: !root.continueCard && root.sectionEnd
        x: 22 * root.unit
        y: root.boxH + 16 * root.unit
        width: root.width - 44 * root.unit
        height: 1 * root.unit
        color: ThemeInk.hairline
    }
}
