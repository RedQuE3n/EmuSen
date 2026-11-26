pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Board 22: Eclipse Packages Library over Packages and RomDirectory, controller-driven with token colors only.
FocusScope {
    id: root
    objectName: "packagesLibraryScreen"

    property ToolModel tool: null
    property int padFamily: 0
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""
    signal backed()

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
    readonly property color borderColor: Kit.Tokens.border
    readonly property color coronaColor: Kit.Tokens.accent
    readonly property color gapColor: Kit.Tokens.ringGapColor
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
                x: 860 * root.unit
                y: -260 * root.unit
                width: 1180 * root.unit
                height: 880 * root.unit
                color: Kit.Tokens.coronaGlowColor
                blur: 180 * root.unit
                opacity: 0.5
            },
            Kit.BackdropField {
                x: -220 * root.unit
                y: 520 * root.unit
                width: 1060 * root.unit
                height: 820 * root.unit
                color: Kit.Tokens.coronaHaloColor
                blur: 190 * root.unit
                opacity: 0.46
            }
        ]
    }

    Kit.Frame {
        id: frame
        objectName: "packagesFrame"
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
            function onBackDeclined() { root.backed() }
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
        objectName: "packagesTopBar"
        x: 88 * root.unit
        y: 38 * root.unit
        width: root.width - 176 * root.unit
        height: 56 * root.unit

        Item {
            objectName: "packagesCrescent"
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
            objectName: "packagesTrail"
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
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.backed()
                }
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
                weight: Font.DemiBold
                color: root.inkColor
                text: root.view?.trailPackages ?? ""
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

    Rectangle {
        id: storageCard
        objectName: "storageCard"
        x: 88 * root.unit
        y: 112 * root.unit
        width: root.width - 176 * root.unit
        height: 82 * root.unit
        radius: 20 * root.unit
        color: root.surfaceColor
        border.width: 1 * root.unit
        border.color: root.hairlineColor

        Row {
            objectName: "storageBar"
            x: 27 * root.unit
            y: 19.5 * root.unit
            width: parent.width - 54 * root.unit
            height: 11 * root.unit
            spacing: 6 * root.unit

            readonly property real trackWidth: width - 12 * root.unit

            Rectangle {
                objectName: "segPkg"
                height: parent.height
                width: Math.max(0, parent.trackWidth * (root.view?.segPkgRatio ?? 0))
                radius: 5.5 * root.unit
                color: root.coronaColor
            }
            Rectangle {
                objectName: "segRom"
                height: parent.height
                width: Math.max(0, parent.trackWidth * (root.view?.segRomRatio ?? 0))
                radius: 5.5 * root.unit
                color: root.outlineColor
            }
            Rectangle {
                objectName: "segFree"
                height: parent.height
                width: Math.max(0, parent.trackWidth * (root.view?.segFreeRatio ?? 1))
                radius: 5.5 * root.unit
                color: root.hairlineColor
            }
        }

        Item {
            objectName: "storageLegend"
            x: 27 * root.unit
            y: 41.5 * root.unit
            width: parent.width - 54 * root.unit
            height: 21 * root.unit

            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: 24 * root.unit

                Row {
                    spacing: 8 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    Rectangle {
                        width: 9 * root.unit
                        height: 9 * root.unit
                        radius: 2.5 * root.unit
                        color: root.coronaColor
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    CardLine {
                        objectName: "storagePkgAmount"
                        unit: root.unit
                        boardLine: true
                        size: 15.5
                        weight: Font.Bold
                        family: Kit.Tokens.faceCode
                        color: root.inkColor
                        text: root.view?.storagePkgAmount ?? ""
                    }
                    CardLine {
                        objectName: "storagePkgLabel"
                        unit: root.unit
                        boardLine: true
                        size: 15.5
                        weight: Font.Normal
                        color: root.mutedColor
                        text: root.view?.storagePkgLabel ?? ""
                    }
                }

                Row {
                    spacing: 8 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    Rectangle {
                        width: 9 * root.unit
                        height: 9 * root.unit
                        radius: 2.5 * root.unit
                        color: root.outlineColor
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    CardLine {
                        objectName: "storageRomAmount"
                        unit: root.unit
                        boardLine: true
                        size: 15.5
                        weight: Font.Bold
                        family: Kit.Tokens.faceCode
                        color: root.inkColor
                        text: root.view?.storageRomAmount ?? ""
                    }
                    CardLine {
                        objectName: "storageRomLabel"
                        unit: root.unit
                        boardLine: true
                        size: 15.5
                        weight: Font.Normal
                        color: root.mutedColor
                        text: root.view?.storageRomLabel ?? ""
                    }
                }

                Row {
                    spacing: 8 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    Rectangle {
                        width: 9 * root.unit
                        height: 9 * root.unit
                        radius: 2.5 * root.unit
                        color: root.hairlineColor
                        border.width: 1 * root.unit
                        border.color: root.outlineColor
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    CardLine {
                        objectName: "storageFreeAmount"
                        unit: root.unit
                        boardLine: true
                        size: 15.5
                        weight: Font.Bold
                        family: Kit.Tokens.faceCode
                        color: root.inkColor
                        text: root.view?.storageFreeAmount ?? ""
                    }
                    CardLine {
                        objectName: "storageFreeLabel"
                        unit: root.unit
                        boardLine: true
                        size: 15.5
                        weight: Font.Normal
                        color: root.mutedColor
                        text: root.view?.storageFreeLabel ?? ""
                    }
                }
            }

            CardLine {
                objectName: "storageNote"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                unit: root.unit
                boardLine: true
                size: 15
                weight: Font.Normal
                color: root.dimColor
                text: root.view?.storageNote ?? ""
            }
        }
    }

    Item {
        id: leftRail
        objectName: "leftRail"
        x: 88 * root.unit
        y: 218 * root.unit
        width: 304 * root.unit
        height: 882 * root.unit

        Column {
            width: parent.width
            spacing: 6 * root.unit

            CardLine {
                objectName: "filterEyebrow"
                unit: root.unit
                boardLine: true
                size: 12
                weight: Font.DemiBold
                spacing: 1.8
                capitalization: Font.AllUppercase
                color: root.dimColor
                text: root.view?.filterEyebrow ?? ""
            }

            Item { width: 1; height: 6 * root.unit }

            Repeater {
                model: root.view?.filters
                Item {
                    id: filterItem
                    required property int index
                    required property var modelData
                    readonly property bool selected: modelData.selected === true
                    readonly property bool zoneFocused: (root.view.focusZone ?? 1) === 0 && selected

                    objectName: "filterRow." + filterItem.modelData.key
                    width: 304 * root.unit
                    height: 52 * root.unit

                    Kit.FocusRing {
                        anchors.fill: parent
                        visible: filterItem.zoneFocused
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
                        color: filterItem.selected ? root.raisedColor : Qt.rgba(0, 0, 0, 0)
                        border.width: filterItem.selected ? 1 * root.unit : 0
                        border.color: filterItem.selected ? Qt.alpha(root.coronaColor, 0.48) : Qt.rgba(0, 0, 0, 0)

                        CardLine {
                            x: 16 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 17
                            weight: filterItem.selected ? Font.DemiBold : Font.Normal
                            color: filterItem.selected ? root.inkColor : root.mutedColor
                            text: filterItem.modelData.label ?? ""
                        }

                        CardLine {
                            anchors.right: parent.right
                            anchors.rightMargin: 16 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 14
                            weight: Font.Normal
                            family: Kit.Tokens.faceCode
                            color: filterItem.selected ? root.coronaColor : root.dimColor
                            text: filterItem.modelData.count ?? ""
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.tool.eclipseChooseFilter(filterItem.modelData.key)
                    }
                }
            }

            Item { width: 1; height: 8 * root.unit }

            Rectangle {
                id: consoleScopePill
                objectName: "consoleScopePill"
                width: 304 * root.unit
                height: 50 * root.unit
                radius: 25 * root.unit
                color: root.surfaceColor
                border.width: 1 * root.unit
                border.color: root.outlineColor

                CardLine {
                    objectName: "consoleScopeLabel"
                    x: 18 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 96 * root.unit
                    elide: Text.ElideRight
                    unit: root.unit
                    boardLine: true
                    size: 16
                    weight: Font.Medium
                    color: root.inkColor
                    text: root.view?.consoleScope ?? ""
                }

                Rectangle {
                    anchors.right: parent.right
                    anchors.rightMargin: 12 * root.unit
                    anchors.verticalCenter: parent.verticalCenter
                    width: consoleHintLine.width + 16 * root.unit
                    height: 24 * root.unit
                    radius: 6 * root.unit
                    color: root.raisedColor
                    border.width: 1 * root.unit
                    border.color: root.outlineColor
                    CardLine {
                        id: consoleHintLine
                        anchors.centerIn: parent
                        unit: root.unit
                        boardLine: true
                        size: 12
                        weight: Font.Normal
                        family: Kit.Tokens.faceCode
                        color: root.mutedColor
                        text: root.view?.consoleHint ?? ""
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tool.eclipseActCode(4, root.emptyText)
                }
            }
        }

        Rectangle {
            id: batchCard
            objectName: "batchCard"
            anchors.bottom: parent.bottom
            width: 304 * root.unit
            height: 158 * root.unit
            radius: 20 * root.unit
            color: root.raisedColor
            border.width: 1 * root.unit
            border.color: Qt.alpha(root.roseColor, 0.42)

            Column {
                x: 20 * root.unit
                y: 18 * root.unit
                width: 264 * root.unit
                spacing: 10 * root.unit

                Item {
                    width: parent.width
                    height: 24 * root.unit
                    CardLine {
                        objectName: "batchTitle"
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        size: 16.5
                        weight: Font.DemiBold
                        color: root.inkColor
                        text: root.view?.batchTitle ?? ""
                    }
                    Rectangle {
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        height: 24 * root.unit
                        width: batchChipLine.width + 18 * root.unit
                        radius: 12 * root.unit
                        color: Qt.alpha(root.roseColor, 0.14)
                        border.width: 1 * root.unit
                        border.color: Qt.alpha(root.roseColor, 0.45)
                        CardLine {
                            id: batchChipLine
                            objectName: "batchChip"
                            anchors.centerIn: parent
                            unit: root.unit
                            boardLine: true
                            size: 12
                            weight: Font.Bold
                            family: Kit.Tokens.faceCode
                            color: root.roseColor
                            text: root.view?.batchChip ?? ""
                        }
                    }
                }

                CardLine {
                    objectName: "batchSummary"
                    width: parent.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 19
                    size: 14
                    weight: Font.Normal
                    color: root.mutedColor
                    elide: Text.ElideRight
                    text: root.view?.batchSummary ?? ""
                }

                Rectangle {
                    objectName: "batchReviewButton"
                    width: parent.width
                    height: 44 * root.unit
                    radius: 22 * root.unit
                    color: root.surfaceColor
                    border.width: 1.5 * root.unit
                    border.color: (root.view?.batchCount ?? 0) > 0 ? root.coronaColor : root.outlineColor

                    Row {
                        anchors.centerIn: parent
                        spacing: 10 * root.unit

                        Rectangle {
                            width: menuBadgeLine.width + 14 * root.unit
                            height: 22 * root.unit
                            radius: 6 * root.unit
                            color: root.raisedColor
                            border.width: 1 * root.unit
                            border.color: root.outlineColor
                            anchors.verticalCenter: parent.verticalCenter
                            CardLine {
                                id: menuBadgeLine
                                anchors.centerIn: parent
                                unit: root.unit
                                boardLine: true
                                size: 11.5
                                weight: Font.Bold
                                family: Kit.Tokens.faceCode
                                color: root.mutedColor
                                text: root.view?.batchBtnKey ?? ""
                            }
                        }

                        CardLine {
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 15
                            weight: Font.DemiBold
                            color: root.inkColor
                            text: root.view?.batchBtnText ?? ""
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.tool.eclipseActCode(2, root.emptyText)
                    }
                }
            }
        }
    }

    Item {
        id: centerColumn
        objectName: "centerColumn"
        x: 420 * root.unit
        y: 218 * root.unit
        width: 744 * root.unit
        height: 882 * root.unit

        Item {
            id: centerHeader
            width: parent.width
            height: 21 * root.unit
            CardLine {
                objectName: "centerEyebrow"
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                unit: root.unit
                boardLine: true
                size: 12
                weight: Font.DemiBold
                spacing: 1.8
                capitalization: Font.AllUppercase
                color: root.dimColor
                text: root.view?.centerEyebrow ?? ""
            }
            CardLine {
                objectName: "sortEyebrow"
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                unit: root.unit
                boardLine: true
                size: 12
                weight: Font.DemiBold
                spacing: 1.4
                capitalization: Font.AllUppercase
                color: root.dimColor
                text: root.view?.sortEyebrow ?? ""
            }
        }

        CardLine {
            objectName: "emptyStateLabel"
            visible: !(root.view?.rows && root.view.rows.length > 0)
            anchors.centerIn: parent
            unit: root.unit
            boardLine: true
            size: 20
            weight: Font.Normal
            color: root.mutedColor
            text: root.view?.emptyText ?? ""
        }

        ListView {
            id: gamesList
            objectName: "packagesRowsList"
            y: 34 * root.unit
            width: parent.width
            height: parent.height - 34 * root.unit
            spacing: 10 * root.unit
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            currentIndex: root.view?.focusedIndex ?? -1
            highlightFollowsCurrentItem: true
            highlightMoveDuration: 0
            preferredHighlightBegin: 84 * root.unit
            preferredHighlightEnd: height - 84 * root.unit
            highlightRangeMode: ListView.ApplyRange
            model: root.view?.rows

            delegate: Item {
                id: gameRow
                required property int index
                required property var modelData
                readonly property bool isFocused: gameRow.modelData.focused === true && (root.view?.focusZone ?? 1) === 1
                readonly property int badgeKind: gameRow.modelData.badgeKind ?? 0

                objectName: "packageRow." + gameRow.index
                width: gamesList.width
                height: 84 * root.unit

                Kit.FocusRing {
                    objectName: "packageRowRing." + gameRow.index
                    anchors.fill: parent
                    visible: gameRow.isFocused
                    radius: 18 * root.unit
                    gap: 3 * root.unit
                    thickness: 2.5 * root.unit
                    bandRadius: 23.5 * root.unit
                    glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.42)
                    glowBlur: 38 * root.unit
                    glowSpread: 7 * root.unit
                    pixelScale: root.pixelScale
                }

                Rectangle {
                    anchors.fill: parent
                    radius: 18 * root.unit
                    color: gameRow.modelData.focused === true ? root.raisedColor : root.surfaceColor
                    border.width: 1 * root.unit
                    border.color: root.hairlineColor

                    Column {
                        x: 22 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 310 * root.unit
                        spacing: 4 * root.unit

                        CardLine {
                            objectName: "packageRowTitle." + gameRow.index
                            width: parent.width
                            elide: Text.ElideRight
                            unit: root.unit
                            boardLine: true
                            size: 21
                            weight: Font.DemiBold
                            color: root.inkColor
                            text: gameRow.modelData.title ?? ""
                        }

                        CardLine {
                            objectName: "packageRowSub." + gameRow.index
                            width: parent.width
                            elide: Text.ElideRight
                            unit: root.unit
                            boardLine: true
                            size: 15
                            weight: Font.Normal
                            color: root.mutedColor
                            text: gameRow.modelData.subtitle ?? ""
                        }
                    }

                    Row {
                        anchors.right: parent.right
                        anchors.rightMargin: 22 * root.unit
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 16 * root.unit

                        Rectangle {
                            objectName: "packageRowBadge." + gameRow.index
                            anchors.verticalCenter: parent.verticalCenter
                            height: gameRow.badgeKind === 1 || gameRow.badgeKind === 2 ? 30 * root.unit : 28 * root.unit
                            width: badgeLabel.width + 26 * root.unit
                            radius: height / 2
                            color: gameRow.badgeKind === 1 ? root.roseColor : (gameRow.badgeKind === 2 ? Qt.alpha(root.roseColor, 0.1) : Qt.alpha(root.coronaColor, 0.08))
                            border.width: gameRow.badgeKind === 2 ? 1.5 * root.unit : (gameRow.badgeKind === 1 ? 0 : 1 * root.unit)
                            border.color: gameRow.badgeKind === 2 ? root.roseColor : root.hairlineColor

                            CardLine {
                                id: badgeLabel
                                anchors.centerIn: parent
                                unit: root.unit
                                boardLine: true
                                size: gameRow.badgeKind === 0 ? 13 : 13.5
                                weight: gameRow.badgeKind === 1 ? Font.Bold : Font.DemiBold
                                color: gameRow.badgeKind === 1 ? root.groundColor : (gameRow.badgeKind === 2 ? root.roseColor : root.mutedColor)
                                text: gameRow.modelData.badgeText ?? ""
                            }
                        }

                        CardLine {
                            objectName: "packageRowSize." + gameRow.index
                            width: 92 * root.unit
                            horizontalAlignment: Text.AlignRight
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 16
                            weight: Font.Normal
                            family: Kit.Tokens.faceCode
                            color: root.mutedColor
                            text: gameRow.modelData.sizeText ?? ""
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.tool.eclipseActCode(3, gameRow.modelData.key)
                        onDoubleClicked: root.tool.eclipseActCode(0, gameRow.modelData.key)
                    }
                }
            }
        }
    }

    Column {
        id: rightColumn
        objectName: "rightInspector"
        x: 1196 * root.unit
        y: 218 * root.unit
        width: 636 * root.unit
        spacing: 16 * root.unit

        Rectangle {
            objectName: "noticeBanner"
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
            id: inspectorCard
            objectName: "inspectorCard"
            width: parent.width
            height: inspectorCol.height + 48 * root.unit
            radius: 22 * root.unit
            color: root.surfaceColor
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Column {
                id: inspectorCol
                x: 26 * root.unit
                y: 24 * root.unit
                width: 584 * root.unit
                spacing: 14 * root.unit

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
                        objectName: "inspectorEyebrow"
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        size: 13
                        weight: Font.DemiBold
                        spacing: 2.08
                        capitalization: Font.AllUppercase
                        color: root.roseColor
                        text: root.view?.inspector?.eyebrow ?? ""
                    }
                }

                CardLine {
                    objectName: "inspectorTitle"
                    width: parent.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 38.88
                    size: 36
                    weight: Font.DemiBold
                    elide: Text.ElideRight
                    color: root.inkColor
                    text: root.view?.inspector?.title ?? ""
                }

                CardLine {
                    objectName: "inspectorPath"
                    width: parent.width
                    unit: root.unit
                    boardLine: true
                    size: 14
                    weight: Font.Normal
                    family: Kit.Tokens.faceCode
                    elide: Text.ElideMiddle
                    color: root.mutedColor
                    text: root.view?.inspector?.path ?? ""
                }

                Grid {
                    id: kvGrid
                    columns: 2
                    columnSpacing: 10 * root.unit
                    rowSpacing: 10 * root.unit
                    width: parent.width

                    Repeater {
                        model: root.view?.inspector?.kv
                        Rectangle {
                            id: kvCard
                            required property var modelData
                            width: 287 * root.unit
                            height: 72 * root.unit
                            radius: 14 * root.unit
                            color: root.groundColor
                            border.width: 1 * root.unit
                            border.color: root.hairlineColor

                            Column {
                                x: 15 * root.unit
                                y: 13 * root.unit
                                width: 257 * root.unit
                                spacing: 4 * root.unit
                                CardLine {
                                    unit: root.unit
                                    boardLine: true
                                    size: 13.5
                                    weight: Font.Normal
                                    color: root.dimColor
                                    text: kvCard.modelData.k ?? ""
                                }
                                CardLine {
                                    width: parent.width
                                    elide: Text.ElideRight
                                    unit: root.unit
                                    boardLine: true
                                    size: 17.5
                                    weight: Font.DemiBold
                                    color: kvCard.modelData.warn === true ? root.roseColor : root.inkColor
                                    text: kvCard.modelData.v ?? ""
                                }
                            }
                        }
                    }
                }

                Item {
                    id: ctaRow
                    objectName: "openPackagePageCta"
                    width: parent.width
                    height: 56 * root.unit
                    visible: root.view?.inspector?.hasEntry === true

                    Kit.FocusRing {
                        anchors.fill: parent
                        visible: root.view?.ctaFocused === true
                        radius: 28 * root.unit
                        gap: 3 * root.unit
                        thickness: 2.5 * root.unit
                        bandRadius: 33.5 * root.unit
                        glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.42)
                        glowBlur: 36 * root.unit
                        glowSpread: 7 * root.unit
                        pixelScale: root.pixelScale
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: 28 * root.unit
                        color: root.raisedColor
                        border.width: 1.5 * root.unit
                        border.color: Qt.alpha(root.coronaColor, 0.5)

                        Row {
                            x: 12 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 12 * root.unit

                            Rectangle {
                                width: 32 * root.unit
                                height: 32 * root.unit
                                radius: 16 * root.unit
                                color: root.coronaColor
                                CardLine {
                                    anchors.centerIn: parent
                                    unit: root.unit
                                    boardLine: true
                                    size: 15
                                    weight: Font.Bold
                                    color: root.groundColor
                                    text: root.view?.ctaKey ?? ""
                                }
                            }

                            CardLine {
                                anchors.verticalCenter: parent.verticalCenter
                                unit: root.unit
                                boardLine: true
                                size: 17.5
                                weight: Font.DemiBold
                                color: root.inkColor
                                text: root.view?.ctaLabel ?? ""
                            }
                        }

                        CardLine {
                            anchors.right: parent.right
                            anchors.rightMargin: 18 * root.unit
                            anchors.verticalCenter: parent.verticalCenter
                            unit: root.unit
                            boardLine: true
                            size: 14.5
                            weight: Font.Normal
                            color: root.mutedColor
                            text: root.view?.ctaHint ?? ""
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.tool.eclipseActCode(0, root.emptyText)
                        }
                    }
                }
            }
        }

        Rectangle {
            id: actionsCard
            objectName: "actionsCard"
            width: parent.width
            height: actionsCol.height + 44 * root.unit
            radius: 22 * root.unit
            color: root.surfaceColor
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Column {
                id: actionsCol
                x: 26 * root.unit
                y: 22 * root.unit
                width: 584 * root.unit
                spacing: 14 * root.unit

                Item {
                    width: parent.width
                    height: 24 * root.unit
                    CardLine {
                        objectName: "actionsTitle"
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        size: 18
                        weight: Font.DemiBold
                        color: root.inkColor
                        text: root.view?.actionsTitle ?? ""
                    }
                    CardLine {
                        objectName: "actionsHint"
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        size: 14
                        weight: Font.Normal
                        color: root.dimColor
                        text: root.view?.actionsHint ?? ""
                    }
                }

                Grid {
                    columns: 2
                    columnSpacing: 10 * root.unit
                    rowSpacing: 10 * root.unit
                    width: parent.width

                    Repeater {
                        model: root.view?.inspector?.tiles
                        Item {
                            id: tileItem
                            required property int index
                            required property var modelData
                            readonly property bool queued: tileItem.modelData.queued === true
                            readonly property bool enabledTile: tileItem.modelData.enabled !== false

                            objectName: "actionTile." + tileItem.modelData.key
                            width: 287 * root.unit
                            height: 68 * root.unit
                            opacity: tileItem.enabledTile ? 1.0 : 0.48

                            Kit.FocusRing {
                                anchors.fill: parent
                                visible: tileItem.modelData.focused === true
                                radius: 15 * root.unit
                                gap: 3 * root.unit
                                thickness: 2.5 * root.unit
                                bandRadius: 20.5 * root.unit
                                glowColor: Qt.alpha(Kit.Tokens.glowColor, 0.42)
                                glowBlur: 32 * root.unit
                                glowSpread: 6 * root.unit
                                pixelScale: root.pixelScale
                            }

                            Rectangle {
                                anchors.fill: parent
                                radius: 15 * root.unit
                                color: tileItem.queued ? Qt.alpha(root.roseColor, 0.09) : root.groundColor
                                border.width: tileItem.queued ? 1.5 * root.unit : 1 * root.unit
                                border.color: tileItem.queued ? root.roseColor : root.hairlineColor

                                Column {
                                    x: 16 * root.unit
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: parent.width - 32 * root.unit
                                    spacing: 3 * root.unit

                                    Item {
                                        width: parent.width
                                        height: 20 * root.unit
                                        CardLine {
                                            anchors.left: parent.left
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: parent.width - (tileItem.queued ? 68 * root.unit : 0)
                                            elide: Text.ElideRight
                                            unit: root.unit
                                            boardLine: true
                                            size: 16
                                            weight: Font.DemiBold
                                            color: root.inkColor
                                            text: tileItem.modelData.title ?? ""
                                        }
                                        CardLine {
                                            visible: tileItem.queued
                                            anchors.right: parent.right
                                            anchors.verticalCenter: parent.verticalCenter
                                            unit: root.unit
                                            boardLine: true
                                            size: 11.5
                                            weight: Font.Bold
                                            family: Kit.Tokens.faceCode
                                            color: root.roseColor
                                            text: tileItem.modelData.queuedTag ?? ""
                                        }
                                    }

                                    CardLine {
                                        width: parent.width
                                        elide: Text.ElideRight
                                        unit: root.unit
                                        boardLine: true
                                        size: 13
                                        weight: Font.Normal
                                        color: root.mutedColor
                                        text: tileItem.modelData.sub ?? ""
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    enabled: tileItem.enabledTile
                                    cursorShape: tileItem.enabledTile ? Qt.PointingHandCursor : Qt.ArrowCursor
                                    onClicked: root.tool.eclipseActCode(5, tileItem.modelData.key)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Item {
        id: footerBar
        objectName: "packagesFooter"
        x: 88 * root.unit
        y: 1122 * root.unit
        width: root.width - 176 * root.unit
        height: 44 * root.unit

        Kit.HelpBar {
            id: helpBar
            objectName: "packagesHelpBar"
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
