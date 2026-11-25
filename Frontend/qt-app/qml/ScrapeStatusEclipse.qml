pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Eclipse Scrape Status (#67): 460 px progress ring, live status summary, and Pause/Cancel/Hide actions.
FocusScope {
    id: root
    objectName: "scrapeStatusEclipse"
    required property ToolModel tool
    required property int padFamily
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""
    readonly property alias controller: scaffold.controller
    readonly property bool sheetOpen: scaffold.sheetOpen

    ToolTableEclipse {
        id: scaffold
        anchors.fill: parent
        focus: true
        tool: root.tool
        padFamily: root.padFamily
        pixelScale: root.pixelScale
        battery: root.battery
        clockText: root.clockText
        tableShown: false
        screenCode: 3
        bodySlot: statusBody
        onBacked: root.tool.eclipseNav(7)
    }

    Component {
        id: statusBody
        Item {
            anchors.fill: parent
            Item {
                id: ringBox
                objectName: "statusRing"
                x: 150 * scaffold.unit
                y: 300 * scaffold.unit
                width: 460 * scaffold.unit
                height: 460 * scaffold.unit
                Kit.Glow {
                    objectName: "statusRingGlow"
                    anchors.fill: parent
                    radius: 230 * scaffold.unit
                    color: "#4DF0B4C4"
                    blur: 60 * scaffold.unit
                    spread: 10 * scaffold.unit
                    pixelScale: root.pixelScale
                }
                Shape {
                    anchors.fill: parent
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        strokeWidth: 14 * scaffold.unit
                        strokeColor: "#262A3A"
                        fillColor: "transparent"
                        PathAngleArc {
                            centerX: 230 * scaffold.unit
                            centerY: 230 * scaffold.unit
                            radiusX: 223 * scaffold.unit
                            radiusY: 223 * scaffold.unit
                            startAngle: -90
                            sweepAngle: 360
                        }
                    }
                    ShapePath {
                        strokeWidth: 14 * scaffold.unit
                        strokeColor: Kit.Tokens.accent
                        fillColor: "transparent"
                        capStyle: ShapePath.FlatCap
                        PathAngleArc {
                            centerX: 230 * scaffold.unit
                            centerY: 230 * scaffold.unit
                            radiusX: 223 * scaffold.unit
                            radiusY: 223 * scaffold.unit
                            startAngle: -90
                            sweepAngle: (scaffold.view.fraction ?? 0) * 360
                        }
                    }
                }
                Rectangle {
                    objectName: "statusRingInner"
                    x: 14 * scaffold.unit
                    y: 14 * scaffold.unit
                    width: 432 * scaffold.unit
                    height: 432 * scaffold.unit
                    radius: 216 * scaffold.unit
                    color: Kit.Tokens.surface
                    Column {
                        x: (parent.width - width) / 2
                        y: (parent.height - height) / 2
                        spacing: 6 * scaffold.unit
                        CardLine {
                            objectName: "statusDone"
                            x: (parent.width - width) / 2
                            unit: scaffold.unit
                            boardLine: true
                            boardLineHeight: 104
                            text: scaffold.view.done ?? ""
                            size: 104
                            weight: Font.DemiBold
                            color: Kit.Tokens.textStrong
                            horizontalAlignment: Text.AlignHCenter
                        }
                        CardLine {
                            objectName: "statusRingSub"
                            x: (parent.width - width) / 2
                            unit: scaffold.unit
                            boardLine: true
                            text: scaffold.view.ringSubtitle ?? ""
                            size: 26
                            weight: Font.Normal
                            color: "#B9BED0"
                            horizontalAlignment: Text.AlignHCenter
                        }
                    }
                }
            }

            Column {
                id: summaryCol
                objectName: "statusSummaryCol"
                x: 720 * scaffold.unit
                y: 290 * scaffold.unit
                width: 1100 * scaffold.unit
                spacing: 18 * scaffold.unit
                CardLine {
                    objectName: "statusCurrentTitle"
                    width: summaryCol.width
                    unit: scaffold.unit
                    boardLine: true
                    boardLineHeight: 36
                    lineHeight: 36
                    lineHeightMode: Text.FixedHeight
                    text: scaffold.view.currentTitle ?? ""
                    size: 30
                    weight: Font.DemiBold
                    color: Kit.Tokens.textStrong
                    elide: Text.ElideRight
                }
                CardLine {
                    objectName: "statusCurrentStage"
                    width: summaryCol.width
                    unit: scaffold.unit
                    boardLine: true
                    boardLineHeight: 29
                    text: scaffold.view.currentStage ?? ""
                    size: 20
                    weight: Font.Normal
                    color: "#D8DBE6"
                    elide: Text.ElideRight
                }
                CardLine {
                    objectName: "statusPaceLine"
                    width: summaryCol.width
                    unit: scaffold.unit
                    boardLine: true
                    boardLineHeight: 29
                    text: scaffold.view.paceLine ?? ""
                    size: 20
                    weight: Font.Normal
                    color: "#B9BED0"
                    elide: Text.ElideRight
                }
                Item {
                    width: summaryCol.width
                    height: 17 * scaffold.unit
                    Rectangle {
                        objectName: "statusDivider.0"
                        y: 8 * scaffold.unit
                        width: parent.width
                        height: 1 * scaffold.unit
                        color: "#262A3A"
                    }
                }
                CardLine {
                    objectName: "statusCountsLine"
                    width: summaryCol.width
                    unit: scaffold.unit
                    boardLine: true
                    boardLineHeight: 29
                    text: scaffold.view.countsLine ?? ""
                    size: 20
                    weight: Font.Normal
                    color: "#D8DBE6"
                    elide: Text.ElideRight
                }
                CardLine {
                    objectName: "statusAccountLine"
                    width: summaryCol.width
                    unit: scaffold.unit
                    boardLine: true
                    boardLineHeight: 29
                    text: scaffold.view.accountLine ?? ""
                    size: 20
                    weight: Font.Normal
                    color: "#B9BED0"
                    elide: Text.ElideRight
                }
                CardLine {
                    objectName: "statusQuotaLine"
                    width: summaryCol.width
                    unit: scaffold.unit
                    boardLine: true
                    boardLineHeight: 29
                    text: scaffold.view.quotaLine ?? ""
                    size: 20
                    weight: Font.Normal
                    color: "#B9BED0"
                    elide: Text.ElideRight
                }
                Item {
                    width: summaryCol.width
                    height: 17 * scaffold.unit
                    Rectangle {
                        objectName: "statusDivider.1"
                        y: 8 * scaffold.unit
                        width: parent.width
                        height: 1 * scaffold.unit
                        color: "#262A3A"
                    }
                }
                Repeater {
                    model: scaffold.view.recentLines ?? []
                    CardLine {
                        id: recentLine
                        required property int index
                        required property string modelData
                        objectName: "statusRecent." + recentLine.index
                        width: summaryCol.width
                        unit: scaffold.unit
                        boardLine: true
                        boardLineHeight: 29
                        text: recentLine.modelData
                        size: 20
                        weight: Font.Normal
                        color: "#D8DBE6"
                        elide: Text.ElideRight
                    }
                }
            }

            Row {
                objectName: "statusActions"
                x: 150 * scaffold.unit
                y: 820 * scaffold.unit
                spacing: 16 * scaffold.unit
                Repeater {
                    model: scaffold.view.actions ?? []
                    Item {
                        id: actPill
                        required property int index
                        required property var modelData
                        objectName: actPill.modelData.objectName ?? ""
                        height: 68 * scaffold.unit
                        width: (actPill.modelData.focused ? actGlyph.width : 0) + actText.implicitWidth + 56 * scaffold.unit
                        opacity: actPill.modelData.enabled === false ? 0.42 : 1.0
                        EclipsePillHighlight {
                            visible: actPill.modelData.focused === true
                            anchors.fill: actBg
                            unit: scaffold.unit
                            pixelScale: root.pixelScale
                            radius: 34 * scaffold.unit
                            rowIndex: actPill.index
                        }
                        Rectangle {
                            id: actBg
                            anchors.fill: parent
                            radius: 34 * scaffold.unit
                            color: actPill.modelData.focused ? Kit.Tokens.accent : "transparent"
                            border.width: actPill.modelData.focused ? 0 : 2 * scaffold.unit
                            border.color: "#5A6076"
                        }
                        Row {
                            id: actRow
                            x: (actPill.modelData.focused ? 12 : 28) * scaffold.unit
                            y: 0
                            height: parent.height
                            spacing: 14 * scaffold.unit
                            CardGlyph {
                                id: actGlyph
                                objectName: actPill.modelData.glyphObjectName ?? ""
                                visible: actPill.modelData.focused === true
                                y: (actRow.height - height) / 2
                                unit: scaffold.unit
                                family: root.padFamily
                                button: Kit.Pad.Accept
                                lead: true
                                leads: true
                                keyPairs: false
                                side: 44
                                leadLetterSize: 20
                                leadWeight: Font.DemiBold
                                leadFill: "#07080F"
                                leadInk: "#F6E7D8"
                            }
                            CardLine {
                                id: actText
                                objectName: actPill.modelData.labelObjectName ?? ""
                                y: (actRow.height - height) / 2
                                unit: scaffold.unit
                                boardLine: true
                                text: actPill.modelData.label ?? ""
                                size: 24
                                weight: actPill.modelData.focused ? Font.DemiBold : Font.Normal
                                color: actPill.modelData.focused ? Kit.Tokens.surface : Kit.Tokens.textStrong
                            }
                        }
                        MouseArea {
                            enabled: actPill.modelData.enabled !== false
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(5, actPill.modelData.key ?? "")
                        }
                    }
                }
            }
        }
    }
}
