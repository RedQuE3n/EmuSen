pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Board 23: Eclipse Packages Soft-Patch Layers for cartridge packages, controller-driven with token colors only.
FocusScope {
    id: root
    objectName: "packageLayersScreen"

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
    readonly property color baseCardColor: Qt.rgba(Kit.Tokens.surface.r * 0.28 + Kit.Tokens.inputSurface.r * 0.72, Kit.Tokens.surface.g * 0.28 + Kit.Tokens.inputSurface.g * 0.72, Kit.Tokens.surface.b * 0.28 + Kit.Tokens.inputSurface.b * 0.72, 1)
    readonly property color inkColor: Kit.Tokens.textStrong
    readonly property color mutedColor: Kit.Tokens.muted
    readonly property color coronaColor: Kit.Tokens.accent
    readonly property color successColor: Kit.Tokens.success
    readonly property color hairlineColor: Qt.rgba(Kit.Tokens.surfaceRaised.r * 0.767 + Kit.Tokens.border.r * 0.233, Kit.Tokens.surfaceRaised.g * 0.767 + Kit.Tokens.border.g * 0.233, Kit.Tokens.surfaceRaised.b * 0.767 + Kit.Tokens.border.b * 0.233, 1)
    readonly property color outlineColor: Qt.rgba(Kit.Tokens.border.r * 0.954 + Kit.Tokens.surface.r * 0.046, Kit.Tokens.border.g * 0.954 + Kit.Tokens.surface.g * 0.046, Kit.Tokens.border.b * 0.954 + Kit.Tokens.surface.b * 0.046, 1)
    readonly property color dimColor: Qt.rgba(Kit.Tokens.muted.r * 0.527 + Kit.Tokens.border.r * 0.473, Kit.Tokens.muted.g * 0.527 + Kit.Tokens.border.g * 0.473, Kit.Tokens.muted.b * 0.545 + Kit.Tokens.border.b * 0.455, 1)
    readonly property color roseColor: Qt.rgba(Kit.Tokens.glowColor.r * 0.656 + Kit.Tokens.textStrong.r * 0.344, Kit.Tokens.glowColor.g * 0.656 + Kit.Tokens.textStrong.g * 0.344, Kit.Tokens.glowColor.b * 0.656 + Kit.Tokens.textStrong.b * 0.344, 1)
    readonly property color helpInkColor: Qt.rgba(Kit.Tokens.textStrong.r * 0.52 + Kit.Tokens.muted.r * 0.48, Kit.Tokens.textStrong.g * 0.52 + Kit.Tokens.muted.g * 0.48, Kit.Tokens.textStrong.b * 0.52 + Kit.Tokens.muted.b * 0.48, 1)

    Kit.Backdrop {
        objectName: "eclipseBackdrop"
        anchors.fill: parent
        ground: root.groundColor
        pixelScale: root.pixelScale
        fields: [
            Kit.BackdropField {
                x: 840 * root.unit
                y: -280 * root.unit
                width: 1180 * root.unit
                height: 880 * root.unit
                color: Kit.Tokens.coronaGlowColor
                blur: 180 * root.unit
                opacity: 0.48
            },
            Kit.BackdropField {
                x: -220 * root.unit
                y: 520 * root.unit
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
        objectName: "layersFrame"
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
                else if (event.key === Qt.Key_PageUp) { root.tool.eclipseNav(5); event.accepted = true }
                else if (event.key === Qt.Key_PageDown) { root.tool.eclipseNav(6); event.accepted = true }
            }
        }
    }

    Item {
        id: topBar
        objectName: "layersTopBar"
        x: 88 * root.unit
        y: 38 * root.unit
        width: root.width - 176 * root.unit
        height: 56 * root.unit

        Item {
            objectName: "layersCrescent"
            x: 0
            y: 12 * root.unit
            width: 32 * root.unit
            height: 32 * root.unit
            Shape {
                id: crescent
                readonly property real box: 140
                width: box
                height: box
                scale: (32 * root.unit) / crescent.box
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
            objectName: "layersTrail"
            x: 52 * root.unit
            anchors.verticalCenter: parent.verticalCenter
            spacing: 12 * root.unit

            CardLine {
                objectName: "trailWords.0"
                unit: root.unit
                boardLine: true
                size: 22
                weight: Font.Normal
                color: root.mutedColor
                text: root.view?.trailTools ?? ""
            }
            CardLine {
                objectName: "trailSlash.0"
                unit: root.unit
                boardLine: true
                size: 22
                weight: Font.Normal
                color: root.dimColor
                text: root.view?.trailSep ?? ""
            }
            CardLine {
                objectName: "trailWords.1"
                unit: root.unit
                boardLine: true
                size: 22
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
                size: 22
                weight: Font.Normal
                color: root.dimColor
                text: root.view?.trailSep ?? ""
            }
            CardLine {
                objectName: "trailWords.2"
                unit: root.unit
                boardLine: true
                size: 22
                weight: Font.DemiBold
                color: root.inkColor
                text: root.view?.trailLayers ?? ""
            }
        }

        Rectangle {
            objectName: "statusPill"
            visible: root.battery >= 0 || root.clockText.length > 0
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            height: 48 * root.unit
            width: statusRow.width + 40 * root.unit
            radius: 24 * root.unit
            color: Qt.alpha(root.surfaceColor, 0.86)
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Row {
                id: statusRow
                anchors.centerIn: parent
                spacing: 16 * root.unit

                Row {
                    visible: root.battery >= 0
                    spacing: 8 * root.unit
                    anchors.verticalCenter: parent.verticalCenter

                    Item {
                        width: 28 * root.unit
                        height: 15 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        Rectangle {
                            x: 0
                            y: 0
                            width: 24 * root.unit
                            height: 15 * root.unit
                            radius: 3.5 * root.unit
                            color: Qt.rgba(0, 0, 0, 0)
                            border.width: 1.5 * root.unit
                            border.color: root.mutedColor
                            Rectangle {
                                x: 3 * root.unit
                                y: 3 * root.unit
                                height: 9 * root.unit
                                width: Math.max(2 * root.unit, 18 * root.unit * Math.max(0, Math.min(100, root.battery)) / 100)
                                radius: 1.5 * root.unit
                                color: root.mutedColor
                            }
                        }
                        Rectangle {
                            x: 25 * root.unit
                            y: 4 * root.unit
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
                        size: 16
                        weight: Font.Normal
                        family: Kit.Tokens.faceCode
                        color: root.mutedColor
                        text: root.battery >= 0 ? Words.percent(root.battery) : ""
                    }
                }

                CardLine {
                    objectName: "clock"
                    visible: root.clockText.length > 0
                    unit: root.unit
                    boardLine: true
                    size: 19
                    weight: Font.DemiBold
                    color: root.inkColor
                    text: root.clockText
                }
            }
        }
    }

    Item {
        id: leftRail
        objectName: "layersLeftRail"
        x: 88 * root.unit
        y: 136 * root.unit
        width: 356 * root.unit
        height: 950 * root.unit

        Item {
            width: parent.width
            height: 24 * root.unit
            CardLine {
                objectName: "layersRailTitle"
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                unit: root.unit
                boardLine: true
                size: 12
                weight: Font.DemiBold
                spacing: 1.8
                capitalization: Font.AllUppercase
                color: root.dimColor
                text: root.view?.layersRailTitle ?? ""
            }
            Rectangle {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: railHintLine.width + 16 * root.unit
                height: 24 * root.unit
                radius: 6 * root.unit
                color: root.surfaceColor
                border.width: 1 * root.unit
                border.color: root.outlineColor
                CardLine {
                    id: railHintLine
                    anchors.centerIn: parent
                    unit: root.unit
                    boardLine: true
                    size: 12
                    weight: Font.Normal
                    family: Kit.Tokens.faceCode
                    color: root.mutedColor
                    text: root.view?.layersRailHint ?? ""
                }
            }
        }

        ListView {
            id: railList
            objectName: "layersRailList"
            y: 36 * root.unit
            width: parent.width
            height: parent.height - 36 * root.unit
            spacing: 8 * root.unit
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: root.view?.rail

            delegate: Rectangle {
                id: railRow
                required property int index
                required property var modelData
                readonly property bool isSelected: railRow.modelData.selected === true

                objectName: "layersRailRow." + railRow.index
                width: railList.width
                height: 78 * root.unit
                radius: 16 * root.unit
                color: railRow.isSelected ? root.raisedColor : root.surfaceColor
                border.width: railRow.isSelected ? 1.5 * root.unit : 1 * root.unit
                border.color: railRow.isSelected ? Qt.alpha(root.coronaColor, 0.52) : root.hairlineColor

                Column {
                    x: 18 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 36 * root.unit
                    spacing: 5 * root.unit

                    Item {
                        width: parent.width
                        height: 24 * root.unit
                        CardLine {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 96 * root.unit
                            elide: Text.ElideRight
                            unit: root.unit
                            boardLine: true
                            size: 19.5
                            weight: Font.DemiBold
                            color: root.inkColor
                            text: railRow.modelData.title ?? ""
                        }
                        CardLine {
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 13.5
                            weight: Font.DemiBold
                            family: Kit.Tokens.faceCode
                            color: railRow.isSelected ? root.coronaColor : root.mutedColor
                            text: railRow.modelData.countText ?? ""
                        }
                    }

                    CardLine {
                        width: parent.width
                        elide: Text.ElideRight
                        unit: root.unit
                        boardLine: true
                        size: 14.5
                        weight: Font.Normal
                        color: root.mutedColor
                        text: railRow.modelData.meta ?? ""
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tool.eclipseActCode(6, railRow.modelData.key)
                }
            }
        }
    }

    Item {
        id: stackCol
        objectName: "layersStackCol"
        x: 480 * root.unit
        y: 136 * root.unit
        width: 756 * root.unit
        height: 950 * root.unit

        Flickable {
            anchors.fill: parent
            contentWidth: width
            contentHeight: stackInner.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds

            Column {
                id: stackInner
                width: stackCol.width
                spacing: 0

                Row {
                    spacing: 10 * root.unit
                    Item {
                        width: 15 * root.unit
                        height: 15 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        Rectangle {
                            anchors.centerIn: parent
                            width: 22 * root.unit
                            height: 22 * root.unit
                            radius: 11 * root.unit
                            color: Qt.alpha(Kit.Tokens.glowColor, 0.3)
                        }
                        Rectangle {
                            anchors.fill: parent
                            radius: 7.5 * root.unit
                            color: root.roseColor
                            border.width: 1 * root.unit
                            border.color: Qt.alpha(root.inkColor, 0.5)
                        }
                    }
                    CardLine {
                        objectName: "layersEyebrow"
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        size: 13
                        weight: Font.DemiBold
                        spacing: 2.08
                        capitalization: Font.AllUppercase
                        color: root.roseColor
                        text: root.view?.header?.eyebrow ?? ""
                    }
                }

                Item { width: 1; height: 10 * root.unit }

                CardLine {
                    objectName: "layersTitle"
                    width: parent.width
                    elide: Text.ElideRight
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 44
                    size: 40
                    weight: Font.DemiBold
                    spacing: -0.6
                    color: root.inkColor
                    text: root.view?.header?.title ?? ""
                }

                Item { width: 1; height: 8 * root.unit }

                CardLine {
                    objectName: "layersSubtitle"
                    width: parent.width
                    wrapMode: Text.Wrap
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 24
                    size: 16.5
                    weight: Font.Light
                    color: root.mutedColor
                    text: root.view?.layersSubtitle ?? ""
                }

                Item { width: 1; height: 18 * root.unit }

                Rectangle {
                    id: outputBanner
                    objectName: "layersOutputBanner"
                    width: parent.width
                    height: 54 * root.unit
                    radius: 16 * root.unit
                    color: Qt.alpha(root.coronaColor, 0.07)
                    border.width: 1 * root.unit
                    border.color: Qt.alpha(root.coronaColor, 0.3)

                    Row {
                        x: 22 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 10 * root.unit

                        Rectangle {
                            width: 18 * root.unit
                            height: 18 * root.unit
                            radius: 9 * root.unit
                            color: Qt.alpha(root.successColor, 0.25)
                            border.width: 1.5 * root.unit
                            border.color: root.successColor
                            anchors.verticalCenter: parent.verticalCenter
                        }

                        CardLine {
                            objectName: "layersOutputLabel"
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 15.5
                            weight: Font.DemiBold
                            color: root.inkColor
                            text: root.view?.header?.outputLabel ?? ""
                        }
                    }

                    CardLine {
                        objectName: "layersOutputMeta"
                        anchors.right: parent.right
                        anchors.rightMargin: 22 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        size: 13.5
                        weight: Font.Normal
                        family: Kit.Tokens.faceCode
                        color: root.coronaColor
                        text: root.view?.header?.outputMeta ?? ""
                    }
                }

                Item { width: 1; height: 16 * root.unit }

                Repeater {
                    model: root.view?.stack
                    Column {
                        id: stackItem
                        required property int index
                        required property var modelData
                        width: stackCol.width

                        Item {
                            id: patchCard
                            objectName: "patchCard." + stackItem.index
                            width: parent.width
                            height: 95 * root.unit

                            Kit.FocusRing {
                                objectName: "patchCardRing." + stackItem.index
                                anchors.fill: parent
                                visible: stackItem.modelData.focused === true
                                radius: 20 * root.unit
                                gap: 3 * root.unit
                                thickness: 2.5 * root.unit
                                bandRadius: 25.5 * root.unit
                                glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.42)
                                glowBlur: 38 * root.unit
                                glowSpread: 7 * root.unit
                                pixelScale: root.pixelScale
                            }

                            Rectangle {
                                anchors.fill: parent
                                radius: 20 * root.unit
                                color: stackItem.modelData.focused === true ? root.raisedColor : root.surfaceColor
                                border.width: 1 * root.unit
                                border.color: root.hairlineColor

                                Column {
                                    x: 24 * root.unit
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: parent.width - 200 * root.unit
                                    spacing: 6 * root.unit

                                    Row {
                                        spacing: 10 * root.unit
                                        Rectangle {
                                            height: 26 * root.unit
                                            width: pillText.width + 20 * root.unit
                                            radius: 13 * root.unit
                                            color: Qt.alpha(root.coronaColor, 0.14)
                                            border.width: 1 * root.unit
                                            border.color: Qt.alpha(root.coronaColor, 0.35)
                                            anchors.verticalCenter: parent.verticalCenter
                                            CardLine {
                                                id: pillText
                                                anchors.centerIn: parent
                                                unit: root.unit
                                                boardLine: true
                                                size: 12.5
                                                weight: Font.Bold
                                                family: Kit.Tokens.faceCode
                                                color: root.coronaColor
                                                text: stackItem.modelData.pill ?? ""
                                            }
                                        }

                                        CardLine {
                                            objectName: "patchName." + stackItem.index
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: 430 * root.unit
                                            elide: Text.ElideRight
                                            unit: root.unit
                                            boardLine: true
                                            size: 21
                                            weight: Font.DemiBold
                                            color: root.inkColor
                                            text: stackItem.modelData.name ?? ""
                                        }
                                    }

                                    CardLine {
                                        objectName: "patchMeta." + stackItem.index
                                        width: parent.width
                                        elide: Text.ElideRight
                                        unit: root.unit
                                        boardLine: true
                                        size: 14.5
                                        weight: Font.Normal
                                        color: root.mutedColor
                                        text: stackItem.modelData.meta ?? ""
                                    }
                                }

                                CardLine {
                                    anchors.right: parent.right
                                    anchors.rightMargin: 24 * root.unit
                                    anchors.verticalCenter: parent.verticalCenter
                                    unit: root.unit
                                    boardLine: true
                                    size: 13.5
                                    weight: Font.DemiBold
                                    color: root.mutedColor
                                    text: stackItem.modelData.role ?? ""
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.tool.eclipseActCode(7, String(stackItem.modelData.stackIndex))
                                }
                            }
                        }

                        Item {
                            width: parent.width
                            height: 34 * root.unit
                            Row {
                                x: 46 * root.unit
                                height: parent.height
                                spacing: 14 * root.unit
                                Rectangle {
                                    width: 2 * root.unit
                                    height: 22 * root.unit
                                    color: root.outlineColor
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                                CardLine {
                                    anchors.verticalCenter: parent.verticalCenter
                                    unit: root.unit
                                    boardLine: true
                                    size: 13
                                    weight: Font.Normal
                                    color: root.dimColor
                                    text: stackItem.modelData.linkText ?? ""
                                }
                            }
                        }
                    }
                }

                Item {
                    id: baseCard
                    objectName: "baseRomCard"
                    visible: root.view?.header?.hasEntry === true
                    width: parent.width
                    height: 99 * root.unit

                    Kit.FocusRing {
                        objectName: "baseRomCardRing"
                        anchors.fill: parent
                        visible: root.view?.header?.baseFocused === true
                        radius: 20 * root.unit
                        gap: 3 * root.unit
                        thickness: 2.5 * root.unit
                        bandRadius: 25.5 * root.unit
                        glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.42)
                        glowBlur: 38 * root.unit
                        glowSpread: 7 * root.unit
                        pixelScale: root.pixelScale
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: 20 * root.unit
                        color: root.baseCardColor
                        border.width: 1.5 * root.unit
                        border.color: root.outlineColor

                        Rectangle {
                            id: romBadge
                            x: 24 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            width: 48 * root.unit
                            height: 48 * root.unit
                            radius: 14 * root.unit
                            color: root.raisedColor
                            border.width: 1 * root.unit
                            border.color: root.hairlineColor
                            Rectangle {
                                anchors.centerIn: parent
                                width: 18 * root.unit
                                height: 22 * root.unit
                                radius: 4 * root.unit
                                color: Qt.rgba(0, 0, 0, 0)
                                border.width: 2 * root.unit
                                border.color: root.coronaColor
                            }
                        }

                        Column {
                            x: 88 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 310 * root.unit
                            spacing: 4 * root.unit

                            CardLine {
                                objectName: "baseCardTitle"
                                width: parent.width
                                elide: Text.ElideRight
                                unit: root.unit
                                boardLine: true
                                size: 19
                                weight: Font.DemiBold
                                color: root.inkColor
                                text: root.view?.baseCardTitle ?? ""
                            }

                            CardLine {
                                objectName: "baseCardPath"
                                width: parent.width
                                elide: Text.ElideMiddle
                                unit: root.unit
                                boardLine: true
                                size: 14
                                weight: Font.Normal
                                family: Kit.Tokens.faceCode
                                color: root.mutedColor
                                text: root.view?.header?.basePath ?? ""
                            }
                        }

                        Column {
                            anchors.right: parent.right
                            anchors.rightMargin: 24 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 6 * root.unit

                            Rectangle {
                                anchors.right: parent.right
                                height: 26 * root.unit
                                width: baseExtLine.width + 20 * root.unit
                                radius: 13 * root.unit
                                color: root.raisedColor
                                border.width: 1 * root.unit
                                border.color: root.hairlineColor
                                CardLine {
                                    id: baseExtLine
                                    anchors.centerIn: parent
                                    unit: root.unit
                                    boardLine: true
                                    size: 13
                                    weight: Font.Normal
                                    family: Kit.Tokens.faceCode
                                    color: root.coronaColor
                                    text: root.view?.header?.baseBadge ?? ""
                                }
                            }

                            CardLine {
                                anchors.right: parent.right
                                unit: root.unit
                                boardLine: true
                                size: 13
                                weight: Font.DemiBold
                                color: root.successColor
                                text: root.view?.header?.baseCheck ?? ""
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.tool.eclipseActCode(7, String(root.view?.header?.baseIndex ?? 0))
                        }
                    }
                }

                Item { width: 1; height: 14 * root.unit }

                CardLine {
                    objectName: "layersFootnote"
                    width: parent.width
                    wrapMode: Text.Wrap
                    unit: root.unit
                    boardLine: true
                    size: 15.5
                    weight: Font.Normal
                    color: root.mutedColor
                    text: root.view?.layersFootnote ?? ""
                }
            }
        }
    }

    Column {
        id: rightCol
        objectName: "layersRightCol"
        x: 1272 * root.unit
        y: 136 * root.unit
        width: 560 * root.unit
        spacing: 16 * root.unit

        Rectangle {
            objectName: "layersNoticeBanner"
            visible: (root.view?.noticeText ?? "").length > 0
            width: parent.width
            height: 52 * root.unit
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
                size: 15
                weight: Font.Medium
                color: root.roseColor
                text: root.view?.noticeText ?? ""
            }
        }

        Rectangle {
            id: detailCard
            objectName: "layersDetailCard"
            width: parent.width
            height: detailCol.height + 48 * root.unit
            radius: 22 * root.unit
            color: root.surfaceColor
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Column {
                id: detailCol
                x: 26 * root.unit
                y: 24 * root.unit
                width: 508 * root.unit
                spacing: 10 * root.unit

                CardLine {
                    objectName: "layersDetailEyebrow"
                    unit: root.unit
                    boardLine: true
                    size: 13
                    weight: Font.DemiBold
                    spacing: 2.08
                    capitalization: Font.AllUppercase
                    color: root.roseColor
                    text: root.view?.detail?.eyebrow ?? ""
                }

                CardLine {
                    objectName: "layersDetailTitle"
                    width: parent.width
                    elide: Text.ElideRight
                    unit: root.unit
                    boardLine: true
                    size: 24
                    weight: Font.DemiBold
                    color: root.inkColor
                    text: root.view?.detail?.title ?? ""
                }

                Item { width: 1; height: 2 * root.unit }

                Repeater {
                    model: root.view?.detail?.kv
                    Item {
                        id: kvRow
                        required property var modelData
                        width: 508 * root.unit
                        height: 44 * root.unit

                        CardLine {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 15
                            weight: Font.Normal
                            color: root.dimColor
                            text: kvRow.modelData.k ?? ""
                        }

                        CardLine {
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            width: 280 * root.unit
                            horizontalAlignment: Text.AlignRight
                            elide: Text.ElideMiddle
                            unit: root.unit
                            boardLine: true
                            size: 15.5
                            weight: Font.Normal
                            family: Kit.Tokens.faceCode
                            color: kvRow.modelData.ok === true ? root.coronaColor : root.inkColor
                            text: kvRow.modelData.v ?? ""
                        }

                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width
                            height: 1 * root.unit
                            color: root.hairlineColor
                        }
                    }
                }
            }
        }

        Rectangle {
            id: patchActionsCard
            objectName: "layersActionsCard"
            width: parent.width
            height: patchActionsCol.height + 44 * root.unit
            radius: 22 * root.unit
            color: root.surfaceColor
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Column {
                id: patchActionsCol
                x: 26 * root.unit
                y: 22 * root.unit
                width: 508 * root.unit
                spacing: 8 * root.unit

                CardLine {
                    objectName: "layersActionsTitle"
                    unit: root.unit
                    boardLine: true
                    size: 13
                    weight: Font.DemiBold
                    spacing: 1.8
                    capitalization: Font.AllUppercase
                    color: root.dimColor
                    text: root.view?.actionsCardEyebrow ?? ""
                }

                Item { width: 1; height: 4 * root.unit }

                Repeater {
                    model: root.view?.layersActions
                    Item {
                        id: actItem
                        required property int index
                        required property var modelData
                        objectName: "layersAction." + actItem.modelData.key
                        width: 508 * root.unit
                        height: 52 * root.unit

                        Kit.FocusRing {
                            anchors.fill: parent
                            visible: actItem.modelData.focused === true
                            radius: 14 * root.unit
                            gap: 3 * root.unit
                            thickness: 2.5 * root.unit
                            bandRadius: 19.5 * root.unit
                            glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.42)
                            glowBlur: 32 * root.unit
                            glowSpread: 6 * root.unit
                            pixelScale: root.pixelScale
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 14 * root.unit
                            color: actItem.modelData.focused === true ? root.raisedColor : root.groundColor
                            border.width: 1 * root.unit
                            border.color: root.hairlineColor

                            CardLine {
                                x: 16 * root.unit
                                anchors.verticalCenter: parent.verticalCenter
                                unit: root.unit
                                boardLine: true
                                size: 17
                                weight: Font.DemiBold
                                color: root.inkColor
                                text: actItem.modelData.label ?? ""
                            }

                            Rectangle {
                                visible: !(actItem.modelData.steps === true)
                                anchors.right: parent.right
                                anchors.rightMargin: 16 * root.unit
                                anchors.verticalCenter: parent.verticalCenter
                                width: chipTxt.width + 18 * root.unit
                                height: 26 * root.unit
                                radius: 8 * root.unit
                                color: root.raisedColor
                                border.width: 1 * root.unit
                                border.color: root.outlineColor
                                CardLine {
                                    id: chipTxt
                                    anchors.centerIn: parent
                                    unit: root.unit
                                    boardLine: true
                                    size: 13
                                    weight: Font.Bold
                                    family: Kit.Tokens.faceCode
                                    color: root.mutedColor
                                    text: actItem.modelData.chip ?? ""
                                }
                            }

                            Row {
                                visible: actItem.modelData.steps === true
                                anchors.right: parent.right
                                anchors.rightMargin: 16 * root.unit
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 10 * root.unit

                                CardLine {
                                    anchors.verticalCenter: parent.verticalCenter
                                    unit: root.unit
                                    boardLine: true
                                    size: 13
                                    color: root.mutedColor
                                    text: root.view?.stepLeftGlyph ?? ""
                                }
                                CardLine {
                                    anchors.verticalCenter: parent.verticalCenter
                                    unit: root.unit
                                    boardLine: true
                                    size: 15.5
                                    weight: Font.DemiBold
                                    color: root.coronaColor
                                    text: actItem.modelData.stepVal ?? ""
                                }
                                CardLine {
                                    anchors.verticalCenter: parent.verticalCenter
                                    unit: root.unit
                                    boardLine: true
                                    size: 13
                                    color: root.mutedColor
                                    text: root.view?.stepRightGlyph ?? ""
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.tool.eclipseActCode(8, actItem.modelData.key)
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            objectName: "layersNoteBox"
            width: parent.width
            height: noteTextLine.height + 36 * root.unit
            radius: 18 * root.unit
            color: Qt.alpha(root.coronaColor, 0.05)
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            CardLine {
                id: noteTextLine
                x: 20 * root.unit
                y: 18 * root.unit
                width: 520 * root.unit
                wrapMode: Text.Wrap
                unit: root.unit
                boardLine: true
                boardLineHeight: 22
                size: 14.5
                weight: Font.Normal
                color: root.mutedColor
                text: root.view?.layersNoteText ?? ""
            }
        }
    }

    Item {
        id: footerBar
        objectName: "layersFooter"
        x: 88 * root.unit
        y: 1122 * root.unit
        width: root.width - 176 * root.unit
        height: 44 * root.unit

        Kit.HelpBar {
            id: helpBar
            objectName: "layersHelpBar"
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
            entrySpacing: 26 * root.unit
            glyphGap: 9 * root.unit
            glyphSize: 32 * root.unit
            snapped: false
            model: RowsModel {
                rows: root.tool ? root.tool.eclipseHelp : ""
            }
            label: Component {
                CardLine {
                    objectName: "helpWords-" + text
                    unit: root.unit
                    boardLine: true
                    size: 18.5
                }
            }
            glyph: Component {
                CardGlyph {
                    unit: root.unit
                    side: 32
                    boxSide: 30
                    boxRadius: 9
                    boxPadding: 10
                    wordSize: 13.5
                    letterSize: 15
                    leadLetterSize: 16
                    leadWeight: Font.Bold
                    padding: 10
                    boxMenu: true
                    boxOptions: true
                    keyPairs: false
                    ink: root.helpInkColor
                    ring: root.mutedColor
                }
            }
        }

        Rectangle {
            objectName: "positionCounterPill"
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            height: 38 * root.unit
            width: posLine.width + 32 * root.unit
            radius: 18 * root.unit
            color: Qt.alpha(root.surfaceColor, 0.82)
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            CardLine {
                id: posLine
                objectName: "positionCounterText"
                anchors.centerIn: parent
                unit: root.unit
                boardLine: true
                size: 17
                weight: Font.Normal
                family: Kit.Tokens.faceCode
                color: root.mutedColor
                text: root.view?.positionText ?? ""
            }
        }
    }
}
