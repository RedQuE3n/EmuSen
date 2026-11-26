pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Board 69: Eclipse Package Page with facts on left and actions on right, token colors only.
FocusScope {
    id: root
    objectName: "packagePageScreen"

    property ToolModel tool: null
    property int padFamily: 0
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""

    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    readonly property var controller: frame.controller
    readonly property string rawCores: root.tool ? root.tool.eclipseCores : ""
    readonly property var view: root.rawCores.length > 0 ? JSON.parse(root.rawCores) : null
    readonly property string emptyText: ""

    readonly property color groundColor: Kit.Tokens.surface
    readonly property color surfaceColor: Kit.Tokens.inputSurface
    readonly property color raisedColor: Kit.Tokens.surfaceRaised
    readonly property color inkColor: Kit.Tokens.textStrong
    readonly property color mutedColor: Kit.Tokens.muted
    readonly property color coronaColor: Kit.Tokens.accent
    readonly property color hairlineColor: Qt.rgba(Kit.Tokens.surfaceRaised.r * 0.767 + Kit.Tokens.border.r * 0.233, Kit.Tokens.surfaceRaised.g * 0.767 + Kit.Tokens.border.g * 0.233, Kit.Tokens.surfaceRaised.b * 0.767 + Kit.Tokens.border.b * 0.233, 1)
    readonly property color dimColor: Qt.rgba(Kit.Tokens.muted.r * 0.527 + Kit.Tokens.border.r * 0.473, Kit.Tokens.muted.g * 0.527 + Kit.Tokens.border.g * 0.473, Kit.Tokens.muted.b * 0.545 + Kit.Tokens.border.b * 0.455, 1)
    readonly property color roseColor: Qt.rgba(Kit.Tokens.glowColor.r * 0.656 + Kit.Tokens.textStrong.r * 0.344, Kit.Tokens.glowColor.g * 0.656 + Kit.Tokens.textStrong.g * 0.344, Kit.Tokens.glowColor.b * 0.656 + Kit.Tokens.textStrong.b * 0.344, 1)
    readonly property color subtitleInkColor: Qt.rgba(Kit.Tokens.textStrong.r * 0.28 + Kit.Tokens.muted.r * 0.72, Kit.Tokens.textStrong.g * 0.28 + Kit.Tokens.muted.g * 0.72, Kit.Tokens.textStrong.b * 0.28 + Kit.Tokens.muted.b * 0.72, 1)
    readonly property color helpInkColor: Qt.rgba(Kit.Tokens.textStrong.r * 0.52 + Kit.Tokens.muted.r * 0.48, Kit.Tokens.textStrong.g * 0.52 + Kit.Tokens.muted.g * 0.48, Kit.Tokens.textStrong.b * 0.52 + Kit.Tokens.muted.b * 0.48, 1)

    Kit.Backdrop {
        objectName: "eclipseBackdrop"
        anchors.fill: parent
        ground: root.groundColor
        pixelScale: root.pixelScale
        fields: [
            Kit.BackdropField {
                x: 840 * root.unit
                y: -260 * root.unit
                width: 1180 * root.unit
                height: 880 * root.unit
                color: Kit.Tokens.coronaGlowColor
                blur: 180 * root.unit
                opacity: 0.48
            },
            Kit.BackdropField {
                x: -220 * root.unit
                y: 540 * root.unit
                width: 1060 * root.unit
                height: 820 * root.unit
                color: Kit.Tokens.coronaHaloColor
                blur: 190 * root.unit
                opacity: 0.44
            }
        ]
    }

    Kit.Frame {
        id: frame
        objectName: "packagePageFrame"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        focus: true
        controller.navigationShown: false
        controller.actionLines: root.tool ? root.tool.actionLines : ""

        Connections {
            target: frame.controller
            function onBackDeclined() { root.tool.eclipseNav(7) }
            function onActionInvoked(key) { root.tool.eclipseActionKey(key) }
        }

        FocusScope {
            id: bodyFocus
            anchors.fill: parent
            focus: true
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_Up) { root.tool.eclipseNav(0); event.accepted = true }
                else if (event.key === Qt.Key_Down) { root.tool.eclipseNav(1); event.accepted = true }
                else if (event.key === Qt.Key_Left) { root.tool.eclipseNav(2); event.accepted = true }
                else if (event.key === Qt.Key_Right) { root.tool.eclipseNav(3); event.accepted = true }
                else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) { root.tool.eclipseNav(4); event.accepted = true }
            }
        }
    }

    Item {
        id: topBar
        objectName: "packagePageHeader"
        x: 96 * root.unit
        y: 48 * root.unit
        width: root.width - 192 * root.unit
        height: 64 * root.unit

        Item {
            objectName: "packagePageCrescent"
            x: 0
            y: 15 * root.unit
            width: 34 * root.unit
            height: 34 * root.unit
            Shape {
                id: crescent
                readonly property real box: 140
                width: box
                height: box
                scale: (34 * root.unit) / crescent.box
                transformOrigin: Item.TopLeft
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: root.coronaColor
                    strokeColor: Qt.rgba(0, 0, 0, 0)
                    fillRule: ShapePath.OddEvenFill
                    PathSvg { path: Marks.menuCrescent }
                }
            }
        }

        Row {
            objectName: "packagePageTrail"
            x: 62 * root.unit
            anchors.verticalCenter: parent.verticalCenter
            spacing: 14 * root.unit

            CardLine {
                objectName: "trailWords.0"
                unit: root.unit
                boardLine: true
                size: 24
                weight: Font.Normal
                color: root.mutedColor
                text: root.view?.trailTools ?? ""
            }
            CardLine {
                objectName: "trailSlash.0"
                unit: root.unit
                boardLine: true
                size: 24
                weight: Font.Normal
                color: root.dimColor
                text: root.view?.trailSep ?? ""
            }
            CardLine {
                objectName: "trailWords.1"
                unit: root.unit
                boardLine: true
                size: 24
                weight: Font.Normal
                color: root.mutedColor
                text: root.view?.trailPackages ?? ""
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tool.eclipseNav(7)
                }
            }
            CardLine {
                objectName: "trailSlash.1"
                unit: root.unit
                boardLine: true
                size: 24
                weight: Font.Normal
                color: root.dimColor
                text: root.view?.trailSep ?? ""
            }
            CardLine {
                objectName: "trailWords.2"
                unit: root.unit
                boardLine: true
                size: 24
                weight: Font.DemiBold
                color: root.inkColor
                text: root.view?.trailPage ?? ""
            }
        }

        Rectangle {
            objectName: "statusPill"
            visible: root.battery >= 0 || root.clockText.length > 0
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            height: 56 * root.unit
            width: statusRow.width + 44 * root.unit
            radius: 28 * root.unit
            color: Qt.alpha(root.surfaceColor, 0.86)
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Row {
                id: statusRow
                anchors.centerIn: parent
                spacing: 18 * root.unit

                Row {
                    visible: root.battery >= 0
                    spacing: 9 * root.unit
                    anchors.verticalCenter: parent.verticalCenter

                    Item {
                        width: 31 * root.unit
                        height: 16 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        Rectangle {
                            x: 0
                            y: 0
                            width: 26 * root.unit
                            height: 16 * root.unit
                            radius: 3.5 * root.unit
                            color: Qt.rgba(0, 0, 0, 0)
                            border.width: 1.5 * root.unit
                            border.color: root.mutedColor
                            Rectangle {
                                x: 3 * root.unit
                                y: 3 * root.unit
                                height: 10 * root.unit
                                width: Math.max(2 * root.unit, 20 * root.unit * Math.max(0, Math.min(100, root.battery)) / 100)
                                radius: 1.5 * root.unit
                                color: root.mutedColor
                            }
                        }
                        Rectangle {
                            x: 27.5 * root.unit
                            y: 4.5 * root.unit
                            width: 2.5 * root.unit
                            height: 7 * root.unit
                            radius: 1 * root.unit
                            color: root.mutedColor
                        }
                    }

                    CardLine {
                        objectName: "battery"
                        unit: root.unit
                        boardLine: true
                        size: 22
                        weight: Font.Normal
                        color: root.inkColor
                        text: root.battery >= 0 ? Words.percent(root.battery) : ""
                    }
                }

                CardLine {
                    objectName: "clock"
                    visible: root.clockText.length > 0
                    unit: root.unit
                    boardLine: true
                    size: 22
                    weight: Font.DemiBold
                    color: root.inkColor
                    text: root.clockText
                }
            }
        }
    }

    Column {
        id: headingCol
        objectName: "packagePageHeading"
        x: 96 * root.unit
        y: 150 * root.unit
        width: 1100 * root.unit
        spacing: 10 * root.unit

        CardLine {
            objectName: "packagePageTitle"
            width: parent.width
            elide: Text.ElideRight
            unit: root.unit
            boardLine: true
            boardLineHeight: 59.36
            size: 56
            weight: Font.DemiBold
            spacing: -0.56
            color: root.inkColor
            text: root.view?.pageTitle ?? ""
        }

        CardLine {
            objectName: "packagePageSubtitle"
            width: parent.width
            wrapMode: Text.Wrap
            unit: root.unit
            boardLine: true
            boardLineHeight: 31.9
            size: 22
            weight: Font.Light
            color: root.subtitleInkColor
            text: root.view?.pageSubtitle ?? ""
        }
    }

    Rectangle {
        objectName: "pageNoticeBanner"
        visible: (root.view?.noticeText ?? "").length > 0
        x: 1240 * root.unit
        y: 170 * root.unit
        width: 584 * root.unit
        height: 54 * root.unit
        radius: 16 * root.unit
        color: Qt.alpha(root.roseColor, 0.12)
        border.width: 1.5 * root.unit
        border.color: root.roseColor
        CardLine {
            x: 20 * root.unit
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 40 * root.unit
            elide: Text.ElideRight
            unit: root.unit
            boardLine: true
            size: 15.5
            weight: Font.Medium
            color: root.roseColor
            text: root.view?.noticeText ?? ""
        }
    }

    Column {
        id: factsColumn
        objectName: "packageFactsCol"
        x: 96 * root.unit
        y: 300 * root.unit
        width: 840 * root.unit
        spacing: 10 * root.unit

        Repeater {
            model: root.view?.facts
            Rectangle {
                id: factRow
                required property int index
                required property var modelData
                objectName: "packageFactRow." + factRow.index
                width: 840 * root.unit
                height: 62 * root.unit
                radius: 16 * root.unit
                color: root.surfaceColor
                border.width: 1 * root.unit
                border.color: root.hairlineColor

                CardLine {
                    objectName: "packageFactLabel." + factRow.index
                    x: 22 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    unit: root.unit
                    boardLine: true
                    size: 22
                    weight: Font.DemiBold
                    color: root.inkColor
                    text: factRow.modelData.label ?? ""
                }

                CardLine {
                    objectName: "packageFactValue." + factRow.index
                    anchors.right: parent.right
                    anchors.rightMargin: 22 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    width: 460 * root.unit
                    horizontalAlignment: Text.AlignRight
                    elide: Text.ElideRight
                    unit: root.unit
                    boardLine: true
                    size: 21
                    weight: Font.Normal
                    color: factRow.modelData.attention === true ? root.roseColor : root.mutedColor
                    text: factRow.modelData.value ?? ""
                }
            }
        }
    }

    ListView {
        id: actionsList
        objectName: "packageActionsCol"
        x: 984 * root.unit
        y: 300 * root.unit
        width: 840 * root.unit
        height: 790 * root.unit
        spacing: 10 * root.unit
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        currentIndex: root.view?.focusedActionIndex ?? -1
        highlightFollowsCurrentItem: true
        highlightMoveDuration: 0
        preferredHighlightBegin: 62 * root.unit
        preferredHighlightEnd: height - 62 * root.unit
        highlightRangeMode: ListView.ApplyRange
        model: root.view?.actions

        delegate: Item {
            id: actionRow
            required property int index
            required property var modelData
            readonly property bool isFocused: actionRow.modelData.focused === true

            objectName: "packageActionRow." + actionRow.index
            width: 840 * root.unit
            height: 62 * root.unit

            Kit.FocusRing {
                objectName: "packageActionRing." + actionRow.index
                anchors.fill: parent
                visible: actionRow.isFocused
                radius: 16 * root.unit
                gap: 4 * root.unit
                thickness: 3 * root.unit
                bandRadius: 23 * root.unit
                glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.5)
                glowBlur: 40 * root.unit
                glowSpread: 10 * root.unit
                pixelScale: root.pixelScale
            }

            Rectangle {
                anchors.fill: parent
                radius: 16 * root.unit
                color: actionRow.isFocused ? root.raisedColor : root.surfaceColor
                border.width: 1 * root.unit
                border.color: root.hairlineColor

                CardLine {
                    objectName: "packageActionLabel." + actionRow.index
                    x: 22 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    width: 480 * root.unit
                    elide: Text.ElideRight
                    unit: root.unit
                    boardLine: true
                    size: 22
                    weight: Font.DemiBold
                    color: root.inkColor
                    text: actionRow.modelData.label ?? ""
                }

                CardLine {
                    objectName: "packageActionValue." + actionRow.index
                    anchors.right: parent.right
                    anchors.rightMargin: 22 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    width: 300 * root.unit
                    horizontalAlignment: Text.AlignRight
                    elide: Text.ElideRight
                    unit: root.unit
                    boardLine: true
                    size: 21
                    weight: Font.Normal
                    color: root.mutedColor
                    text: actionRow.modelData.value ?? ""
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tool.eclipseActCode(10, actionRow.modelData.key)
                }
            }
        }
    }

    Item {
        id: footerBar
        objectName: "packagePageFooter"
        x: 96 * root.unit
        y: 1122 * root.unit
        width: root.width - 192 * root.unit
        height: 38 * root.unit

        Kit.HelpBar {
            id: helpBar
            objectName: "packagePageHelpBar"
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            controller: frame.controller
            family: root.padFamily
            fitted: true
            ruled: false
            color: Qt.rgba(0, 0, 0, 0)
            labelColor: root.helpInkColor
            iconColor: root.mutedColor
            leadStyle: Kit.GlyphStyles.Filled
            leadColor: root.coronaColor
            entrySpacing: 28 * root.unit
            glyphGap: 10 * root.unit
            glyphSize: 34 * root.unit
            snapped: false
            model: RowsModel {
                rows: root.tool ? root.tool.eclipseHelp : ""
            }
            label: Component {
                CardLine {
                    objectName: "helpWords-" + text
                    unit: root.unit
                    boardLine: true
                    size: 21
                }
            }
            glyph: Component {
                CardGlyph {
                    unit: root.unit
                    side: 34
                    boxSide: 30
                    boxRadius: 9
                    boxPadding: 10
                    wordSize: 14
                    letterSize: 16
                    leadLetterSize: 17
                    leadWeight: Font.Bold
                    padding: 10
                    boxMenu: true
                    boxOptions: false
                    keyPairs: false
                    ink: root.helpInkColor
                    ring: root.mutedColor
                }
            }
        }

        CardLine {
            objectName: "positionCounterText"
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            unit: root.unit
            boardLine: true
            size: 20
            weight: Font.Normal
            family: Kit.Tokens.faceCode
            color: root.mutedColor
            text: root.view?.positionText ?? ""
        }
    }
}
