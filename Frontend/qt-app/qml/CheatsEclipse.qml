pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Eclipse Cheats screen (board 21) with zero hardcoded hex colors (R-20261125-18).
FocusScope {
    id: root
    required property ToolModel tool
    required property int padFamily
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""
    signal backed()

    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    readonly property var view: JSON.parse(root.tool.eclipseCores)
    readonly property var dbCard: root.view.dbCard ?? ({})
    readonly property var inspector: root.view.inspector ?? ({})
    readonly property var addCard: root.view.addCard ?? ({})
    readonly property bool overlayOpen: root.view.overlayOpen === true
    readonly property bool browseOpen: root.view.browseOpen === true
    readonly property bool editorOpen: root.view.editorOpen === true
    readonly property bool editorTyping: root.view.editorTyping === true
    readonly property int editorField: root.view.editorField ?? 0
    readonly property string activeFieldText: root.editorField === 1 ? (root.view.editorCode ?? "") : (root.view.editorDesc ?? "")
    readonly property int focusedRowY: root.view.focusedRowY ?? 0
    readonly property alias controller: frame.controller
    readonly property bool sheetOpen: frame.controller.sheetOpen
    onSheetOpenChanged: if (!root.sheetOpen && !root.tool.cardOpen && !root.tool.asking && root.visible) frame.forceActiveFocus()

    onFocusedRowYChanged: {
        const top = root.focusedRowY * root.unit
        const bottom = top + 70 * root.unit
        if (top < cheatsFlick.contentY) cheatsFlick.contentY = Math.max(0, top - 8 * root.unit)
        else if (bottom > cheatsFlick.contentY + cheatsFlick.height) cheatsFlick.contentY = Math.max(0, bottom - cheatsFlick.height + 8 * root.unit)
    }

    readonly property color groundColor: Kit.Tokens.surface
    readonly property color cardColor: Kit.Tokens.inputSurface
    readonly property color raisedColor: Kit.Tokens.surfaceRaised
    readonly property color inkColor: Kit.Tokens.textStrong
    readonly property color mutedColor: Kit.Tokens.muted
    readonly property color coronaColor: Kit.Tokens.coronaColor
    readonly property color overAccentColor: Kit.Tokens.overAccent
    readonly property color hairlineColor: Qt.rgba(raisedColor.r * 0.764 + Kit.Tokens.border.r * 0.236, raisedColor.g * 0.764 + Kit.Tokens.border.g * 0.236, raisedColor.b * 0.764 + Kit.Tokens.border.b * 0.236, 1)
    readonly property color outlineColor: Qt.rgba(Kit.Tokens.border.r * 0.954 + Kit.Tokens.surface.r * 0.046, Kit.Tokens.border.g * 0.954 + Kit.Tokens.surface.g * 0.046, Kit.Tokens.border.b * 0.954 + Kit.Tokens.surface.b * 0.046, 1)
    readonly property color dimColor: Qt.rgba(mutedColor.r * 0.528 + Kit.Tokens.border.r * 0.472, mutedColor.g * 0.528 + Kit.Tokens.border.g * 0.472, mutedColor.b * 0.565 + Kit.Tokens.border.b * 0.435, 1)
    readonly property color roseColor: Qt.rgba(inkColor.r * 0.35 + Kit.Tokens.glowColor.r * 0.65, inkColor.g * 0.35 + Kit.Tokens.glowColor.g * 0.65, inkColor.b * 0.35 + Kit.Tokens.glowColor.b * 0.65, 1)
    readonly property color transparentColor: Qt.rgba(0, 0, 0, 0)

    Kit.Backdrop {
        objectName: "cheatsBackdrop"
        anchors.fill: parent
        ground: root.groundColor
        pixelScale: root.pixelScale
        fields: [
            Kit.BackdropField { x: 1180 * root.unit; y: -280 * root.unit; width: 920 * root.unit; height: 680 * root.unit; color: Kit.Tokens.success; blur: 165 * root.unit; opacity: 0.17 },
            Kit.BackdropField { x: 1320 * root.unit; y: 520 * root.unit; width: 860 * root.unit; height: 760 * root.unit; color: Kit.Tokens.coronaHaloColor; blur: 175 * root.unit; opacity: 0.20 }
        ]
    }

    Kit.Frame {
        id: frame
        objectName: "cheatsEclipseFrame"
        anchors.fill: parent
        focus: true
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        controller.navigationShown: false
        controller.actionLines: root.view.actionLines ?? ""
        Connections {
            target: frame.controller
            function onBackDeclined() { root.backed() }
            function onActionInvoked(key) { root.tool.eclipseActionKey(key) }
        }
        FocusScope {
            id: cheatsNavScope
            anchors.fill: parent
            focus: !root.editorTyping
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_Up) { root.tool.eclipseNav(0); event.accepted = true }
                else if (event.key === Qt.Key_Down) { root.tool.eclipseNav(1); event.accepted = true }
                else if (event.key === Qt.Key_Left) { root.tool.eclipseNav(2); event.accepted = true }
                else if (event.key === Qt.Key_Right) { root.tool.eclipseNav(3); event.accepted = true }
                else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) { root.tool.eclipseNav(4); event.accepted = true }
                else if (event.key === Qt.Key_PageUp) { root.tool.eclipseNav(5); event.accepted = true }
                else if (event.key === Qt.Key_PageDown) { root.tool.eclipseNav(6); event.accepted = true }
                else if (event.key === Qt.Key_BracketLeft) { root.tool.eclipseNav(7); event.accepted = true }
                else if (event.key === Qt.Key_BracketRight) { root.tool.eclipseNav(8); event.accepted = true }
            }
        }
    }

    // Top bar (y: 38, h: 56).
    Item {
        objectName: "cheatsTopBar"
        x: 88 * root.unit
        y: 38 * root.unit
        width: parent.width - 176 * root.unit
        height: 56 * root.unit

        Row {
            id: topTrailRow
            x: 0
            height: 32 * root.unit
            y: (parent.height - height) / 2
            spacing: 20 * root.unit

            Item {
                objectName: "cheatsMark"
                width: 32 * root.unit
                height: 32 * root.unit
                Shape {
                    width: 34
                    height: 34
                    scale: 32 * root.unit / 34
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        fillColor: root.coronaColor
                        strokeColor: root.transparentColor
                        fillRule: ShapePath.OddEvenFill
                        PathSvg { path: Marks.menuCrescent }
                    }
                }
            }

            Row {
                objectName: "cheatsTrail"
                height: topTrailRow.height
                spacing: 12 * root.unit
                Repeater {
                    model: root.view.trail ?? []
                    Row {
                        id: crumb
                        required property var modelData
                        height: topTrailRow.height
                        spacing: 12 * root.unit
                        CardLine {
                            objectName: crumb.modelData.objectName ?? ""
                            y: (crumb.height - height) / 2
                            unit: root.unit
                            boardLine: true
                            text: crumb.modelData.label ?? ""
                            size: 22
                            weight: crumb.modelData.current ? Font.DemiBold : Font.Normal
                            color: crumb.modelData.current ? root.inkColor : root.mutedColor
                        }
                        CardLine {
                            objectName: crumb.modelData.slashObjectName ?? ""
                            visible: (crumb.modelData.slash ?? "").length > 0
                            y: (crumb.height - height) / 2
                            unit: root.unit
                            boardLine: true
                            text: crumb.modelData.slash ?? ""
                            size: 22
                            weight: Font.Normal
                            color: root.dimColor
                        }
                    }
                }
            }
        }

        Rectangle {
            objectName: "cheatsStatusPill"
            visible: root.battery >= 0 || root.clockText.length > 0
            anchors.right: parent.right
            y: (parent.height - height) / 2
            height: 48 * root.unit
            width: statusPillRow.implicitWidth + 40 * root.unit
            radius: 24 * root.unit
            color: Qt.alpha(root.cardColor, 0.86)
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Row {
                id: statusPillRow
                x: 20 * root.unit
                height: parent.height
                spacing: 18 * root.unit

                CardLine {
                    objectName: "cheatsBattery"
                    visible: root.battery >= 0
                    y: (statusPillRow.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.battery + (root.view.percentSuffix ?? "")
                    size: 19
                    weight: Font.Medium
                    color: root.inkColor
                }

                CardLine {
                    objectName: "cheatsClock"
                    visible: root.clockText.length > 0
                    y: (statusPillRow.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.clockText
                    size: 19
                    weight: Font.Medium
                    color: root.inkColor
                }
            }
        }
    }

    // Game context strip (x: 88, y: 112, w: 1744, h: 108).
    Rectangle {
        id: gameStrip
        objectName: "gameStrip"
        x: 88 * root.unit
        y: 112 * root.unit
        width: parent.width - 176 * root.unit
        height: 108 * root.unit
        radius: 22 * root.unit
        color: root.cardColor
        border.width: 1 * root.unit
        border.color: root.hairlineColor

        Rectangle {
            x: 28 * root.unit
            y: 0
            width: 340 * root.unit
            height: 1
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: root.transparentColor }
                GradientStop { position: 0.5; color: Qt.alpha(root.inkColor, 0.20) }
                GradientStop { position: 1.0; color: root.transparentColor }
            }
        }

        Item {
            id: coverThumb
            objectName: "gameCover"
            x: 18 * root.unit
            y: (parent.height - height) / 2
            width: 124 * root.unit
            height: 78 * root.unit

            Rectangle {
                anchors.fill: parent
                radius: 13 * root.unit
                color: root.raisedColor
                border.width: 1 * root.unit
                border.color: root.hairlineColor

                CardPlanet {
                    visible: (root.view.cover ?? "").length === 0
                    anchors.centerIn: parent
                    unit: root.unit
                    planet: root.view.planet ?? 0
                    side: 36
                    pixelScale: root.pixelScale
                }

                Image {
                    visible: (root.view.cover ?? "").length > 0
                    anchors.fill: parent
                    anchors.margins: 1
                    source: root.view.cover ?? ""
                    fillMode: Image.PreserveAspectCrop
                    clip: true
                }
            }
        }

        Column {
            x: coverThumb.x + coverThumb.width + 20 * root.unit
            y: (parent.height - height) / 2
            width: Math.max(0, masterRow.x - x - 20 * root.unit)
            spacing: 5 * root.unit

            Row {
                spacing: 14 * root.unit
                CardLine {
                    objectName: "gameTitle"
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.gameTitle ?? ""
                    size: 34
                    weight: Font.DemiBold
                    spacing: -0.51
                    color: root.inkColor
                    elide: Text.ElideRight
                }
                Rectangle {
                    objectName: "systemTag"
                    visible: (root.view.systemBadge ?? "").length > 0
                    y: (parent.height - height) / 2
                    width: sysTagLine.implicitWidth + 20 * root.unit
                    height: 24 * root.unit
                    radius: 12 * root.unit
                    color: Qt.alpha(root.roseColor, 0.10)
                    border.width: 1 * root.unit
                    border.color: Qt.alpha(root.roseColor, 0.28)
                    CardLine {
                        id: sysTagLine
                        anchors.centerIn: parent
                        unit: root.unit
                        boardLine: true
                        text: root.view.systemBadge ?? ""
                        size: 12
                        weight: Font.DemiBold
                        spacing: 1.68
                        color: root.roseColor
                    }
                }
            }

            Row {
                objectName: "gameFacts"
                spacing: 14 * root.unit
                CardPlanet {
                    visible: (root.view.planet ?? -3) > -3
                    y: (parent.height - height) / 2
                    unit: root.unit
                    planet: root.view.planet ?? 0
                    side: 16
                    pixelScale: root.pixelScale
                }
                CardLine {
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.coreText ?? ""
                    size: 16
                    color: root.mutedColor
                }
                CardLine {
                    visible: (root.view.listPathText ?? "").length > 0
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.dotSep ?? ""
                    size: 16
                    color: root.mutedColor
                }
                CardLine {
                    visible: (root.view.listPathText ?? "").length > 0
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.listPathText ?? ""
                    family: Kit.Tokens.faceCode
                    size: 14.5
                    color: root.mutedColor
                }
                CardLine {
                    visible: (root.view.dbMatchText ?? "").length > 0
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.dotSep ?? ""
                    size: 16
                    color: root.mutedColor
                }
                CardLine {
                    visible: (root.view.dbMatchText ?? "").length > 0
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.dbMatchText ?? ""
                    family: Kit.Tokens.faceCode
                    size: 14.5
                    color: root.mutedColor
                }
            }
        }

        Row {
            id: masterRow
            anchors.right: parent.right
            anchors.rightMargin: 26 * root.unit
            y: (parent.height - height) / 2
            spacing: 20 * root.unit

            Column {
                y: (parent.height - height) / 2
                spacing: 2 * root.unit
                CardLine {
                    objectName: "activeSummary"
                    anchors.right: parent.right
                    unit: root.unit
                    boardLine: true
                    text: root.view.activeSummary ?? ""
                    size: 17
                    weight: Font.DemiBold
                    color: root.coronaColor
                }
                CardLine {
                    objectName: "appliedSub"
                    anchors.right: parent.right
                    unit: root.unit
                    boardLine: true
                    text: root.view.appliedSub ?? ""
                    size: 14
                    color: root.dimColor
                }
            }

            Item {
                id: masterPillBox
                objectName: "masterPill"
                width: masterContent.implicitWidth + 30 * root.unit
                height: 58 * root.unit
                y: (parent.height - height) / 2

                Kit.FocusRing {
                    anchors.fill: parent
                    visible: root.view.masterFocused === true
                    radius: 29 * root.unit
                    gap: 3 * root.unit
                    thickness: 2.5 * root.unit
                    glowColor: Kit.Tokens.glowColor
                    glowBlur: 38 * root.unit
                    glowSpread: 7 * root.unit
                    pixelScale: root.pixelScale
                }

                Rectangle {
                    anchors.fill: parent
                    radius: 29 * root.unit
                    color: root.raisedColor
                    border.width: 1 * root.unit
                    border.color: root.view.masterFocused === true ? root.coronaColor : root.outlineColor
                }

                Row {
                    id: masterContent
                    x: 18 * root.unit
                    y: (parent.height - height) / 2
                    spacing: 14 * root.unit

                    Rectangle {
                        y: (parent.height - height) / 2
                        width: menuCapLine.implicitWidth + 18 * root.unit
                        height: menuCapLine.implicitHeight + 10 * root.unit
                        radius: 9 * root.unit
                        color: Qt.alpha(root.inkColor, 0.08)
                        border.width: 1 * root.unit
                        border.color: root.outlineColor
                        CardLine {
                            id: menuCapLine
                            anchors.centerIn: parent
                            unit: root.unit
                            boardLine: true
                            text: root.view.masterHint ?? ""
                            family: Kit.Tokens.faceCode
                            size: 12
                            weight: Font.Bold
                            spacing: 0.48
                            color: root.inkColor
                        }
                    }

                    CardLine {
                        objectName: "masterLabel"
                        y: (parent.height - height) / 2
                        unit: root.unit
                        boardLine: true
                        text: root.view.masterLabel ?? ""
                        size: 18
                        weight: Font.DemiBold
                        color: root.inkColor
                    }

                    Rectangle {
                        id: masterSwitchTrack
                        y: (parent.height - height) / 2
                        width: 62 * root.unit
                        height: 34 * root.unit
                        radius: 17 * root.unit
                        color: root.view.masterOn === true ? root.coronaColor : root.hairlineColor
                        border.width: root.view.masterOn === true ? 0 : 1 * root.unit
                        border.color: root.outlineColor

                        Rectangle {
                            x: root.view.masterOn === true ? 32 * root.unit : 3 * root.unit
                            y: (parent.height - height) / 2
                            width: 26 * root.unit
                            height: 26 * root.unit
                            radius: 13 * root.unit
                            color: root.view.masterOn === true ? root.overAccentColor : root.dimColor
                        }
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    onClicked: root.tool.eclipseActCode(1, "")
                }
            }
        }
    }

    // Left column: filter bar and grouped cheat rows (x: 88, y: 242, w: 1068, h: 874).
    Item {
        objectName: "leftColumn"
        x: 88 * root.unit
        y: 242 * root.unit
        width: 1068 * root.unit
        height: parent.height - 242 * root.unit - 94 * root.unit

        Item {
            id: filterBar
            objectName: "filterBar"
            x: 0
            y: 0
            width: parent.width
            height: 48 * root.unit

            Row {
                anchors.left: parent.left
                y: (parent.height - height) / 2
                spacing: 10 * root.unit

                Repeater {
                    model: root.view.filters ?? []
                    Item {
                        id: pillItem
                        required property var modelData
                        objectName: pillItem.modelData.objectName ?? ""
                        width: pillRow.implicitWidth + 36 * root.unit
                        height: 42 * root.unit

                        Kit.FocusRing {
                            anchors.fill: parent
                            visible: pillItem.modelData.focused === true
                            radius: 21 * root.unit
                            gap: 3 * root.unit
                            thickness: 2 * root.unit
                            glowColor: Kit.Tokens.glowColor
                            glowBlur: 24 * root.unit
                            glowSpread: 4 * root.unit
                            pixelScale: root.pixelScale
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 21 * root.unit
                            color: pillItem.modelData.active ? root.raisedColor : Qt.alpha(root.cardColor, 0.85)
                            border.width: pillItem.modelData.active ? 1.5 * root.unit : 1 * root.unit
                            border.color: pillItem.modelData.active ? Qt.alpha(root.coronaColor, 0.55) : root.hairlineColor
                        }

                        Row {
                            id: pillRow
                            x: 18 * root.unit
                            y: (parent.height - height) / 2
                            spacing: 8 * root.unit

                            CardLine {
                                objectName: pillItem.modelData.labelObjectName ?? ""
                                y: (pillRow.height - height) / 2
                                unit: root.unit
                                boardLine: true
                                text: pillItem.modelData.label ?? ""
                                size: 16.5
                                weight: pillItem.modelData.active ? Font.DemiBold : Font.Normal
                                color: pillItem.modelData.active ? root.inkColor : root.mutedColor
                            }
                            CardLine {
                                y: (pillRow.height - height) / 2
                                unit: root.unit
                                boardLine: true
                                text: pillItem.modelData.count ?? ""
                                family: Kit.Tokens.faceCode
                                size: 14.5
                                weight: Font.DemiBold
                                color: pillItem.modelData.active ? root.coronaColor : root.dimColor
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(6, pillItem.modelData.key ?? "")
                        }
                    }
                }
            }

            CardLine {
                objectName: "sortCaption"
                anchors.right: parent.right
                y: (parent.height - height) / 2
                unit: root.unit
                boardLine: true
                text: root.view.sortCaption ?? ""
                size: 16
                color: root.dimColor
            }
        }

        Flickable {
            id: cheatsFlick
            objectName: "cheatsFlick"
            x: -8 * root.unit
            y: 64 * root.unit
            width: parent.width + 16 * root.unit
            height: parent.height - 64 * root.unit
            contentWidth: width
            contentHeight: groupsCol.height + 16 * root.unit
            clip: true

            Column {
                id: groupsCol
                x: 8 * root.unit
                y: 8 * root.unit
                width: cheatsFlick.width - 16 * root.unit
                spacing: 18 * root.unit

                Rectangle {
                    visible: (root.view.groups ?? []).length === 0
                    width: parent.width
                    height: 112 * root.unit
                    radius: 16 * root.unit
                    color: root.cardColor
                    border.width: 1 * root.unit
                    border.color: root.hairlineColor

                    CardLine {
                        anchors.centerIn: parent
                        unit: root.unit
                        boardLine: true
                        text: root.view.emptyStateWords ?? ""
                        size: 18
                        color: root.mutedColor
                    }
                }

                Repeater {
                    model: root.view.groups ?? []
                    Column {
                        id: groupBlock
                        required property var modelData
                        width: groupsCol.width
                        spacing: 8 * root.unit

                        Item {
                            width: parent.width
                            height: 20 * root.unit
                            CardLine {
                                objectName: groupBlock.modelData.objectName ?? ""
                                x: 6 * root.unit
                                y: 0
                                unit: root.unit
                                boardLine: true
                                text: groupBlock.modelData.title ?? ""
                                size: 13
                                weight: Font.DemiBold
                                spacing: 2.08
                                capitalization: Font.AllUppercase
                                color: root.roseColor
                            }
                        }

                        Column {
                            width: parent.width
                            spacing: 9 * root.unit

                            Repeater {
                                model: groupBlock.modelData.rows ?? []
                                Item {
                                    id: cheatRowItem
                                    required property var modelData
                                    objectName: cheatRowItem.modelData.objectName ?? ""
                                    width: groupsCol.width
                                    height: 70 * root.unit

                                    Kit.FocusRing {
                                        objectName: cheatRowItem.modelData.ringObjectName ?? ""
                                        anchors.fill: parent
                                        visible: cheatRowItem.modelData.focused === true
                                        radius: 16 * root.unit
                                        gap: 3 * root.unit
                                        thickness: 2.5 * root.unit
                                        glowColor: Kit.Tokens.glowColor
                                        glowBlur: 38 * root.unit
                                        glowSpread: 7 * root.unit
                                        pixelScale: root.pixelScale
                                    }

                                    Rectangle {
                                        objectName: cheatRowItem.modelData.bgObjectName ?? ""
                                        anchors.fill: parent
                                        radius: 16 * root.unit
                                        color: cheatRowItem.modelData.focused === true ? root.raisedColor : root.cardColor
                                        border.width: cheatRowItem.modelData.focused === true ? 0 : 1 * root.unit
                                        border.color: root.hairlineColor
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: root.tool.eclipseFocusCore(cheatRowItem.modelData.key ?? "")
                                    }

                                    Item {
                                        x: 22 * root.unit
                                        anchors.top: parent.top
                                        anchors.bottom: parent.bottom
                                        width: parent.width - 44 * root.unit

                                        Item {
                                            id: rowSwitchBox
                                            x: 0
                                            y: (parent.height - height) / 2
                                            width: 62 * root.unit
                                            height: 34 * root.unit

                                            Rectangle {
                                                anchors.fill: parent
                                                radius: 17 * root.unit
                                                color: cheatRowItem.modelData.on === true ? root.coronaColor : root.hairlineColor
                                                border.width: cheatRowItem.modelData.on === true ? 0 : 1 * root.unit
                                                border.color: root.outlineColor

                                                Rectangle {
                                                    x: cheatRowItem.modelData.on === true ? 32 * root.unit : 3 * root.unit
                                                    y: (parent.height - height) / 2
                                                    width: 26 * root.unit
                                                    height: 26 * root.unit
                                                    radius: 13 * root.unit
                                                    color: cheatRowItem.modelData.on === true ? root.overAccentColor : root.dimColor
                                                }
                                            }

                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: root.tool.eclipseActCode(0, cheatRowItem.modelData.key ?? "")
                                            }
                                        }

                                        CardLine {
                                            x: rowSwitchBox.width + 18 * root.unit
                                            y: (parent.height - height) / 2
                                            width: 270 * root.unit
                                            unit: root.unit
                                            boardLine: true
                                            text: cheatRowItem.modelData.name ?? ""
                                            size: 20
                                            weight: Font.DemiBold
                                            color: root.inkColor
                                            elide: Text.ElideRight
                                        }

                                        CardLine {
                                            id: rowDesc
                                            x: rowSwitchBox.width + 18 * root.unit + 270 * root.unit + 18 * root.unit
                                            y: (parent.height - height) / 2
                                            width: Math.max(0, rowBadgeBox.x - x - 18 * root.unit)
                                            unit: root.unit
                                            boardLine: true
                                            text: cheatRowItem.modelData.desc ?? ""
                                            size: 15.5
                                            color: root.mutedColor
                                            elide: Text.ElideRight
                                        }

                                        Rectangle {
                                            id: rowBadgeBox
                                            x: rowCodeLine.x - width - 18 * root.unit
                                            y: (parent.height - height) / 2
                                            width: rowBadgeText.implicitWidth + 22 * root.unit
                                            height: 28 * root.unit
                                            radius: 14 * root.unit
                                            color: cheatRowItem.modelData.isLiveRam === true ? Qt.alpha(root.coronaColor, 0.12) : Qt.alpha(root.roseColor, 0.10)
                                            border.width: 1 * root.unit
                                            border.color: cheatRowItem.modelData.isLiveRam === true ? Qt.alpha(root.coronaColor, 0.36) : Qt.alpha(root.roseColor, 0.28)

                                            CardLine {
                                                id: rowBadgeText
                                                anchors.centerIn: parent
                                                unit: root.unit
                                                boardLine: true
                                                text: cheatRowItem.modelData.badge ?? ""
                                                size: 13
                                                weight: Font.DemiBold
                                                spacing: 0.26
                                                color: cheatRowItem.modelData.isLiveRam === true ? root.coronaColor : root.roseColor
                                            }
                                        }

                                        CardLine {
                                            id: rowCodeLine
                                            anchors.right: parent.right
                                            y: (parent.height - height) / 2
                                            width: 148 * root.unit
                                            unit: root.unit
                                            boardLine: true
                                            text: cheatRowItem.modelData.code ?? ""
                                            family: Kit.Tokens.faceCode
                                            size: 17
                                            weight: Font.Bold
                                            spacing: 0.68
                                            horizontalAlignment: Text.AlignRight
                                            elide: Text.ElideRight
                                            color: root.coronaColor
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Right column: Inspector, Local DB card, Add Cheat card (x: 1192, y: 242, w: 640).
    Column {
        objectName: "rightColumn"
        x: 1192 * root.unit
        y: 242 * root.unit
        width: parent.width - 1192 * root.unit - 88 * root.unit
        spacing: 16 * root.unit

        // Card 1: Focused Cheat Inspector.
        Rectangle {
            objectName: "inspectorCard"
            width: parent.width
            height: inspectorCol.implicitHeight + 48 * root.unit
            radius: 22 * root.unit
            color: root.cardColor
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Column {
                id: inspectorCol
                x: 26 * root.unit
                y: 24 * root.unit
                width: parent.width - 52 * root.unit
                spacing: 14 * root.unit

                Item {
                    width: parent.width
                    height: inspectorHeadLeft.implicitHeight

                    Column {
                        id: inspectorHeadLeft
                        anchors.left: parent.left
                        width: Math.max(0, stateTagBox.visible ? stateTagBox.x - 14 * root.unit : parent.width)
                        spacing: 6 * root.unit

                        CardLine {
                            unit: root.unit
                            boardLine: true
                            text: root.inspector.eyebrow ?? ""
                            size: 12.5
                            weight: Font.DemiBold
                            spacing: 1.875
                            capitalization: Font.AllUppercase
                            color: root.dimColor
                        }
                        CardLine {
                            objectName: "inspectorName"
                            width: parent.width
                            unit: root.unit
                            boardLine: true
                            text: root.inspector.name ?? ""
                            size: 27
                            weight: Font.DemiBold
                            spacing: -0.27
                            color: root.inkColor
                            elide: Text.ElideRight
                        }
                    }

                    Rectangle {
                        id: stateTagBox
                        objectName: "inspectorStateTag"
                        visible: (root.inspector.stateTag ?? "").length > 0
                        anchors.right: parent.right
                        y: (parent.height - height) / 2
                        width: stateTagLine.implicitWidth + 24 * root.unit
                        height: 30 * root.unit
                        radius: 15 * root.unit
                        color: root.inspector.stateActive === true ? Qt.alpha(root.coronaColor, 0.12) : root.raisedColor
                        border.width: 1 * root.unit
                        border.color: root.inspector.stateActive === true ? Qt.alpha(root.coronaColor, 0.45) : root.outlineColor

                        CardLine {
                            id: stateTagLine
                            anchors.centerIn: parent
                            unit: root.unit
                            boardLine: true
                            text: root.inspector.stateTag ?? ""
                            size: 12.5
                            weight: Font.DemiBold
                            spacing: 1.5
                            color: root.inspector.stateActive === true ? root.coronaColor : root.mutedColor
                        }
                    }
                }

                Rectangle {
                    objectName: "inspectorCodeBox"
                    width: parent.width
                    height: codeBoxCol.implicitHeight + 30 * root.unit
                    radius: 14 * root.unit
                    color: root.groundColor
                    border.width: 1 * root.unit
                    border.color: root.hairlineColor

                    Column {
                        id: codeBoxCol
                        x: 20 * root.unit
                        y: 15 * root.unit
                        width: parent.width - 40 * root.unit
                        spacing: 5 * root.unit

                        CardLine {
                            objectName: "inspectorCode"
                            width: parent.width
                            unit: root.unit
                            boardLine: true
                            text: root.inspector.code ?? ""
                            family: Kit.Tokens.faceCode
                            size: 31
                            weight: Font.Bold
                            spacing: 2.48
                            color: root.coronaColor
                            elide: Text.ElideRight
                        }

                        CardLine {
                            objectName: "inspectorDecoded"
                            visible: (root.inspector.decoded ?? "").length > 0
                            width: parent.width
                            unit: root.unit
                            boardLine: true
                            text: root.inspector.decoded ?? ""
                            family: Kit.Tokens.faceCode
                            size: 14
                            color: root.mutedColor
                            elide: Text.ElideRight
                        }
                    }
                }

                CardLine {
                    objectName: "inspectorCopy"
                    visible: (root.inspector.sentence ?? "").length > 0
                    width: parent.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 24
                    text: root.inspector.sentence ?? ""
                    size: 16
                    weight: Font.Light
                    color: root.mutedColor
                    wrapMode: Text.WordWrap
                }

                Column {
                    objectName: "inspectorFacts"
                    width: parent.width
                    spacing: 10 * root.unit

                    Rectangle {
                        width: parent.width
                        height: 1
                        color: root.hairlineColor
                    }

                    Repeater {
                        model: root.inspector.facts ?? []
                        Item {
                            id: factRow
                            required property var modelData
                            width: parent.width
                            height: Math.max(factLabelLine.height, factValRow.height)

                            CardLine {
                                id: factLabelLine
                                x: 0
                                width: 142 * root.unit
                                unit: root.unit
                                boardLine: true
                                text: factRow.modelData.label ?? ""
                                size: 15.5
                                color: root.dimColor
                            }

                            Row {
                                id: factValRow
                                x: 156 * root.unit
                                width: Math.max(0, parent.width - 156 * root.unit)
                                spacing: 8 * root.unit

                                CardPlanet {
                                    visible: (factRow.modelData.planet ?? -3) > -3
                                    y: (parent.height - height) / 2
                                    unit: root.unit
                                    planet: factRow.modelData.planet ?? 0
                                    side: 16
                                    pixelScale: root.pixelScale
                                }

                                CardLine {
                                    width: Math.max(0, factValRow.width - ((factRow.modelData.planet ?? -3) > -3 ? 24 * root.unit : 0))
                                    unit: root.unit
                                    boardLine: true
                                    text: factRow.modelData.value ?? ""
                                    family: factRow.modelData.mono ? Kit.Tokens.faceCode : Kit.Tokens.faceText
                                    size: 15.5
                                    color: root.inkColor
                                    elide: Text.ElideRight
                                }
                            }
                        }
                    }
                }
            }
        }

        // Card 2: Local Cheat Database.
        Rectangle {
            objectName: "dbCard"
            width: parent.width
            height: dbCol.implicitHeight + 44 * root.unit
            radius: 22 * root.unit
            color: root.cardColor
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            Column {
                id: dbCol
                x: 24 * root.unit
                y: 22 * root.unit
                width: parent.width - 48 * root.unit
                spacing: 12 * root.unit

                Item {
                    width: parent.width
                    height: Math.max(dbTitleLine.height, licenseTagBox.height)

                    CardLine {
                        id: dbTitleLine
                        anchors.left: parent.left
                        y: (parent.height - height) / 2
                        unit: root.unit
                        boardLine: true
                        text: root.dbCard.title ?? ""
                        size: 21
                        weight: Font.DemiBold
                        color: root.inkColor
                    }

                    Rectangle {
                        id: licenseTagBox
                        anchors.right: parent.right
                        y: (parent.height - height) / 2
                        width: licenseTagLine.implicitWidth + 20 * root.unit
                        height: licenseTagLine.implicitHeight + 8 * root.unit
                        radius: 11 * root.unit
                        color: root.raisedColor
                        border.width: 1 * root.unit
                        border.color: root.hairlineColor

                        CardLine {
                            id: licenseTagLine
                            anchors.centerIn: parent
                            unit: root.unit
                            boardLine: true
                            text: root.dbCard.attributionTag ?? ""
                            family: Kit.Tokens.faceCode
                            size: 12.5
                            color: root.mutedColor
                        }
                    }
                }

                Rectangle {
                    width: parent.width
                    height: 8 * root.unit
                    radius: 4 * root.unit
                    color: root.hairlineColor
                    Rectangle {
                        width: parent.width * Math.max(0, Math.min(1, root.dbCard.meterPct ?? 0))
                        height: parent.height
                        radius: 4 * root.unit
                        color: root.coronaColor
                    }
                }

                CardLine {
                    objectName: "dbCopy"
                    width: parent.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 22.5
                    text: root.dbCard.copy ?? ""
                    size: 15.5
                    color: root.mutedColor
                    wrapMode: Text.WordWrap
                }

                Row {
                    width: parent.width
                    spacing: 10 * root.unit

                    Item {
                        objectName: "dbBrowseButton"
                        visible: root.dbCard.showBrowse === true
                        width: (parent.width - 10 * root.unit) / 2
                        height: 46 * root.unit

                        Kit.FocusRing {
                            anchors.fill: parent
                            visible: root.dbCard.browseFocused === true
                            radius: 23 * root.unit
                            gap: 3 * root.unit
                            thickness: 2 * root.unit
                            glowColor: Kit.Tokens.glowColor
                            glowBlur: 24 * root.unit
                            glowSpread: 4 * root.unit
                            pixelScale: root.pixelScale
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 23 * root.unit
                            color: root.raisedColor
                            border.width: 1.5 * root.unit
                            border.color: root.dbCard.browseFocused === true ? root.coronaColor : root.outlineColor

                            Row {
                                anchors.centerIn: parent
                                spacing: 10 * root.unit
                                CardGlyph {
                                    y: (parent.height - height) / 2
                                    unit: root.unit
                                    button: Kit.Pad.Search
                                    family: root.padFamily
                                    side: 22
                                    letterSize: 13
                                    ink: root.inkColor
                                    ring: root.outlineColor
                                }
                                CardLine {
                                    y: (parent.height - height) / 2
                                    unit: root.unit
                                    boardLine: true
                                    text: root.dbCard.browseLabel ?? ""
                                    size: 15.5
                                    weight: Font.DemiBold
                                    color: root.inkColor
                                }
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(3, "")
                        }
                    }

                    Item {
                        objectName: "dbUpdateButton"
                        width: root.dbCard.showBrowse === true ? (parent.width - 10 * root.unit) / 2 : parent.width
                        height: 46 * root.unit

                        Kit.FocusRing {
                            anchors.fill: parent
                            visible: root.dbCard.updateFocused === true
                            radius: 23 * root.unit
                            gap: 3 * root.unit
                            thickness: 2 * root.unit
                            glowColor: Kit.Tokens.glowColor
                            glowBlur: 24 * root.unit
                            glowSpread: 4 * root.unit
                            pixelScale: root.pixelScale
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 23 * root.unit
                            color: root.raisedColor
                            border.width: 1.5 * root.unit
                            border.color: root.dbCard.updateFocused === true ? root.coronaColor : root.outlineColor

                            CardLine {
                                anchors.centerIn: parent
                                unit: root.unit
                                boardLine: true
                                text: root.dbCard.updateLabel ?? ""
                                size: 15.5
                                weight: Font.DemiBold
                                color: root.inkColor
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(5, "")
                        }
                    }
                }
            }
        }

        // Card 3: Add a cheat code (dashed outline card).
        Item {
            id: addCheatBox
            objectName: "addCheatCard"
            width: parent.width
            height: addCol.implicitHeight + 40 * root.unit

            Kit.FocusRing {
                anchors.fill: parent
                visible: root.addCard.focused === true
                radius: 22 * root.unit
                gap: 3 * root.unit
                thickness: 2.5 * root.unit
                glowColor: Kit.Tokens.glowColor
                glowBlur: 38 * root.unit
                glowSpread: 7 * root.unit
                pixelScale: root.pixelScale
            }

            Rectangle {
                anchors.fill: parent
                radius: 22 * root.unit
                color: root.addCard.focused === true ? root.raisedColor : root.cardColor
            }

            Shape {
                anchors.fill: parent
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    readonly property real cr: 22 * root.unit
                    fillColor: root.transparentColor
                    strokeColor: root.addCard.focused === true ? root.coronaColor : root.outlineColor
                    strokeWidth: 1.5 * root.unit
                    strokeStyle: ShapePath.DashLine
                    dashPattern: Kit.Tokens.dashPattern
                    startX: cr
                    startY: 0
                    PathLine { x: addCheatBox.width - (22 * root.unit); y: 0 }
                    PathArc { x: addCheatBox.width; y: 22 * root.unit; radiusX: 22 * root.unit; radiusY: 22 * root.unit }
                    PathLine { x: addCheatBox.width; y: addCheatBox.height - (22 * root.unit) }
                    PathArc { x: addCheatBox.width - (22 * root.unit); y: addCheatBox.height; radiusX: 22 * root.unit; radiusY: 22 * root.unit }
                    PathLine { x: 22 * root.unit; y: addCheatBox.height }
                    PathArc { x: 0; y: addCheatBox.height - (22 * root.unit); radiusX: 22 * root.unit; radiusY: 22 * root.unit }
                    PathLine { x: 0; y: 22 * root.unit }
                    PathArc { x: 22 * root.unit; y: 0; radiusX: 22 * root.unit; radiusY: 22 * root.unit }
                }
            }

            Row {
                x: 24 * root.unit
                y: 20 * root.unit
                width: parent.width - 48 * root.unit
                spacing: 18 * root.unit

                Rectangle {
                    y: (parent.height - height) / 2
                    width: viewPillLine.implicitWidth + 24 * root.unit
                    height: 34 * root.unit
                    radius: 17 * root.unit
                    color: root.transparentColor
                    border.width: 1.5 * root.unit
                    border.color: root.mutedColor

                    CardLine {
                        id: viewPillLine
                        anchors.centerIn: parent
                        unit: root.unit
                        boardLine: true
                        text: root.addCard.pill ?? ""
                        size: 13
                        weight: Font.DemiBold
                        spacing: 0.78
                        color: root.inkColor
                    }
                }

                Column {
                    id: addCol
                    width: parent.width - viewPillLine.implicitWidth - 42 * root.unit
                    spacing: 4 * root.unit
                    CardLine {
                        unit: root.unit
                        boardLine: true
                        text: root.addCard.title ?? ""
                        size: 19
                        weight: Font.DemiBold
                        color: root.inkColor
                    }
                    CardLine {
                        width: parent.width
                        unit: root.unit
                        boardLine: true
                        text: root.addCard.copy ?? ""
                        size: 15
                        color: root.mutedColor
                        elide: Text.ElideRight
                    }
                }
            }

            MouseArea {
                anchors.fill: parent
                onClicked: root.tool.eclipseActCode(4, "")
            }
        }
    }

    // Bottom footer help bar and position counter (bottom: 34, h: 44).
    Item {
        objectName: "cheatsFooter"
        x: 88 * root.unit
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 34 * root.unit
        width: parent.width - 176 * root.unit
        height: 44 * root.unit

        EclipseBoardHelpBar {
            objectName: "cheatsHelp"
            x: 0
            y: (parent.height - height) / 2
            unit: root.unit
            padFamily: root.padFamily
            entriesJson: root.tool.eclipseHelp
        }

        Rectangle {
            objectName: "cheatsPosCounter"
            visible: (root.view.positionCounter ?? "").length > 0
            anchors.right: parent.right
            y: (parent.height - height) / 2
            width: posLine.implicitWidth + 32 * root.unit
            height: 38 * root.unit
            radius: 18 * root.unit
            color: Qt.alpha(root.cardColor, 0.82)
            border.width: 1 * root.unit
            border.color: root.hairlineColor

            CardLine {
                id: posLine
                anchors.centerIn: parent
                unit: root.unit
                boardLine: true
                text: root.view.positionCounter ?? ""
                family: Kit.Tokens.faceCode
                size: 17
                color: root.mutedColor
            }
        }
    }

    // Browse .cht file overlay modal.
    Item {
        objectName: "cheatsBrowseOverlay"
        visible: root.browseOpen
        anchors.fill: parent
        z: 10

        Rectangle {
            anchors.fill: parent
            color: Qt.alpha(root.groundColor, 0.86)
            MouseArea { anchors.fill: parent }
        }

        Rectangle {
            anchors.centerIn: parent
            width: 1040 * root.unit
            height: 760 * root.unit
            radius: 24 * root.unit
            color: root.cardColor
            border.width: 1 * root.unit
            border.color: root.outlineColor

            Column {
                x: 32 * root.unit
                y: 28 * root.unit
                width: parent.width - 64 * root.unit
                height: parent.height - 56 * root.unit
                spacing: 16 * root.unit

                Item {
                    width: parent.width
                    height: 52 * root.unit
                    Column {
                        anchors.left: parent.left
                        spacing: 4 * root.unit
                        CardLine {
                            unit: root.unit
                            boardLine: true
                            text: root.view.browseTitle ?? ""
                            size: 26
                            weight: Font.DemiBold
                            color: root.inkColor
                        }
                        CardLine {
                            unit: root.unit
                            boardLine: true
                            text: root.view.browseSubtitle ?? ""
                            size: 16
                            color: root.mutedColor
                        }
                    }
                    Rectangle {
                        objectName: "browseCloseButton"
                        anchors.right: parent.right
                        y: (parent.height - height) / 2
                        width: 40 * root.unit
                        height: 40 * root.unit
                        radius: 20 * root.unit
                        color: root.raisedColor
                        border.width: 1 * root.unit
                        border.color: root.outlineColor
                        CardGlyph {
                            anchors.centerIn: parent
                            unit: root.unit
                            button: Kit.Pad.Back
                            family: root.padFamily
                            side: 22
                            letterSize: 13
                            ink: root.inkColor
                            ring: root.outlineColor
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(10, "")
                        }
                    }
                }

                Flickable {
                    width: parent.width
                    height: parent.height - 68 * root.unit
                    contentWidth: width
                    contentHeight: browseListCol.height + 12 * root.unit
                    clip: true

                    Column {
                        id: browseListCol
                        width: parent.width
                        spacing: 9 * root.unit

                        Repeater {
                            model: root.view.browseRows ?? []
                            Item {
                                id: chtRow
                                required property var modelData
                                width: browseListCol.width
                                height: 64 * root.unit

                                Kit.FocusRing {
                                    anchors.fill: parent
                                    visible: chtRow.modelData.focused === true
                                    radius: 14 * root.unit
                                    gap: 2 * root.unit
                                    thickness: 2 * root.unit
                                    glowColor: Kit.Tokens.glowColor
                                    glowBlur: 24 * root.unit
                                    glowSpread: 4 * root.unit
                                    pixelScale: root.pixelScale
                                }

                                Rectangle {
                                    anchors.fill: parent
                                    radius: 14 * root.unit
                                    color: chtRow.modelData.focused === true ? root.raisedColor : root.groundColor
                                    border.width: 1 * root.unit
                                    border.color: chtRow.modelData.focused === true ? root.coronaColor : root.hairlineColor
                                }

                                CardLine {
                                    x: 20 * root.unit
                                    y: (parent.height - height) / 2
                                    width: 480 * root.unit
                                    unit: root.unit
                                    boardLine: true
                                    text: chtRow.modelData.name ?? ""
                                    size: 18
                                    weight: Font.DemiBold
                                    color: root.inkColor
                                    elide: Text.ElideRight
                                }

                                CardLine {
                                    x: 520 * root.unit
                                    y: (parent.height - height) / 2
                                    width: 200 * root.unit
                                    unit: root.unit
                                    boardLine: true
                                    text: chtRow.modelData.code ?? ""
                                    family: Kit.Tokens.faceCode
                                    size: 16
                                    weight: Font.Bold
                                    color: root.coronaColor
                                    elide: Text.ElideRight
                                }

                                Rectangle {
                                    anchors.right: parent.right
                                    anchors.rightMargin: 20 * root.unit
                                    y: (parent.height - height) / 2
                                    width: statusBadgeLine.implicitWidth + 24 * root.unit
                                    height: 32 * root.unit
                                    radius: 16 * root.unit
                                    color: chtRow.modelData.added ? Qt.alpha(root.coronaColor, 0.16) : root.raisedColor
                                    border.width: 1 * root.unit
                                    border.color: chtRow.modelData.added ? root.coronaColor : root.outlineColor
                                    CardLine {
                                        id: statusBadgeLine
                                        anchors.centerIn: parent
                                        unit: root.unit
                                        boardLine: true
                                        text: chtRow.modelData.statusWords ?? ""
                                        size: 14
                                        weight: Font.DemiBold
                                        color: chtRow.modelData.added ? root.coronaColor : root.inkColor
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: root.tool.eclipseActCode(0, chtRow.modelData.indexKey ?? "")
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Add / Edit cheat modal with docked EclipseKeyboard.
    Kit.KeyboardModel {
        id: cheatTypist
        onTextChanged: root.tool.eclipseSetField(root.editorField, cheatTypist.text)
        onClosed: root.tool.eclipseActCode(11, "")
    }

    Connections {
        target: root
        function onEditorTypingChanged() { cheatTypist.setText(root.activeFieldText, root.activeFieldText.length) }
    }

    Item {
        objectName: "cheatsEditorOverlay"
        visible: root.editorOpen
        anchors.fill: parent
        z: 15

        Rectangle {
            anchors.fill: parent
            color: Qt.alpha(root.groundColor, 0.88)
            MouseArea { anchors.fill: parent }
        }

        Column {
            anchors.horizontalCenter: parent.horizontalCenter
            y: 90 * root.unit
            width: 1446 * root.unit
            spacing: 20 * root.unit

            CardLine {
                objectName: "cheatEditorTitle"
                unit: root.unit
                boardLine: true
                text: root.view.editorTitle ?? ""
                size: 36
                weight: Font.DemiBold
                color: root.inkColor
            }

            CardLine {
                unit: root.unit
                boardLine: true
                text: root.view.editorSubtitle ?? ""
                size: 20
                color: root.mutedColor
            }

            Row {
                width: parent.width
                spacing: 20 * root.unit

                Column {
                    width: (parent.width - 20 * root.unit) / 2
                    spacing: 8 * root.unit
                    CardLine {
                        unit: root.unit
                        boardLine: true
                        text: root.view.editorDescLabel ?? ""
                        size: 15
                        weight: Font.DemiBold
                        color: root.roseColor
                    }
                    Item {
                        objectName: "cheatEditorDesc"
                        width: parent.width
                        height: 68 * root.unit
                        Kit.FocusRing {
                            anchors.fill: parent
                            visible: root.editorField === 0
                            radius: 16 * root.unit
                            gap: 3 * root.unit
                            thickness: 2.5 * root.unit
                            glowColor: Kit.Tokens.glowColor
                            glowBlur: 28 * root.unit
                            glowSpread: 5 * root.unit
                            pixelScale: root.pixelScale
                        }
                        Rectangle {
                            anchors.fill: parent
                            radius: 16 * root.unit
                            color: root.cardColor
                            border.width: 1.5 * root.unit
                            border.color: root.editorField === 0 ? root.coronaColor : root.hairlineColor
                            CardLine {
                                x: 20 * root.unit
                                y: (parent.height - height) / 2
                                width: parent.width - 40 * root.unit
                                unit: root.unit
                                boardLine: true
                                text: (root.view.editorDesc ?? "").length > 0 ? (root.view.editorDesc ?? "") : (root.view.editorDescHint ?? "")
                                size: 22
                                color: (root.view.editorDesc ?? "").length > 0 ? root.inkColor : root.dimColor
                                elide: Text.ElideRight
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(12, "")
                        }
                    }
                }

                Column {
                    width: (parent.width - 20 * root.unit) / 2
                    spacing: 8 * root.unit
                    CardLine {
                        unit: root.unit
                        boardLine: true
                        text: root.view.editorCodeLabel ?? ""
                        size: 15
                        weight: Font.DemiBold
                        color: root.roseColor
                    }
                    Item {
                        objectName: "cheatEditorCode"
                        width: parent.width
                        height: 68 * root.unit
                        Kit.FocusRing {
                            anchors.fill: parent
                            visible: root.editorField === 1
                            radius: 16 * root.unit
                            gap: 3 * root.unit
                            thickness: 2.5 * root.unit
                            glowColor: Kit.Tokens.glowColor
                            glowBlur: 28 * root.unit
                            glowSpread: 5 * root.unit
                            pixelScale: root.pixelScale
                        }
                        Rectangle {
                            anchors.fill: parent
                            radius: 16 * root.unit
                            color: root.cardColor
                            border.width: 1.5 * root.unit
                            border.color: root.editorField === 1 ? root.coronaColor : root.hairlineColor
                            CardLine {
                                x: 20 * root.unit
                                y: (parent.height - height) / 2
                                width: parent.width - 40 * root.unit
                                unit: root.unit
                                boardLine: true
                                text: (root.view.editorCode ?? "").length > 0 ? (root.view.editorCode ?? "") : (root.view.editorCodeHint ?? "")
                                family: Kit.Tokens.faceCode
                                size: 22
                                weight: Font.Bold
                                color: (root.view.editorCode ?? "").length > 0 ? root.coronaColor : root.dimColor
                                elide: Text.ElideRight
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseActCode(13, "")
                        }
                    }
                }
            }

            CardLine {
                objectName: "cheatEditorError"
                visible: (root.view.editorError ?? "").length > 0
                width: parent.width
                unit: root.unit
                boardLine: true
                text: root.view.editorError ?? ""
                size: 17
                weight: Font.DemiBold
                color: Kit.Tokens.error
            }

            Row {
                spacing: 16 * root.unit
                Item {
                    objectName: "cheatEditorSave"
                    width: saveLabelLine.implicitWidth + 48 * root.unit
                    height: 54 * root.unit
                    Kit.FocusRing {
                        anchors.fill: parent
                        visible: root.editorField === 2
                        radius: 27 * root.unit
                        gap: 3 * root.unit
                        thickness: 2 * root.unit
                        glowColor: Kit.Tokens.glowColor
                        glowBlur: 24 * root.unit
                        glowSpread: 4 * root.unit
                        pixelScale: root.pixelScale
                    }
                    Rectangle {
                        anchors.fill: parent
                        radius: 27 * root.unit
                        color: root.editorField === 2 ? root.coronaColor : root.raisedColor
                        border.width: root.editorField === 2 ? 0 : 1.5 * root.unit
                        border.color: root.outlineColor
                        CardLine {
                            id: saveLabelLine
                            anchors.centerIn: parent
                            unit: root.unit
                            boardLine: true
                            text: root.view.editorSaveLabel ?? ""
                            size: 18
                            weight: Font.DemiBold
                            color: root.editorField === 2 ? root.overAccentColor : root.inkColor
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: root.tool.eclipseActCode(8, "")
                    }
                }

                Item {
                    objectName: "cheatEditorCancel"
                    width: cancelLabelLine.implicitWidth + 48 * root.unit
                    height: 54 * root.unit
                    Kit.FocusRing {
                        anchors.fill: parent
                        visible: root.editorField === 3
                        radius: 27 * root.unit
                        gap: 3 * root.unit
                        thickness: 2 * root.unit
                        glowColor: Kit.Tokens.glowColor
                        glowBlur: 24 * root.unit
                        glowSpread: 4 * root.unit
                        pixelScale: root.pixelScale
                    }
                    Rectangle {
                        anchors.fill: parent
                        radius: 27 * root.unit
                        color: root.raisedColor
                        border.width: 1.5 * root.unit
                        border.color: root.editorField === 3 ? root.coronaColor : root.outlineColor
                        CardLine {
                            id: cancelLabelLine
                            anchors.centerIn: parent
                            unit: root.unit
                            boardLine: true
                            text: root.view.editorCancelLabel ?? ""
                            size: 18
                            weight: Font.DemiBold
                            color: root.inkColor
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: root.tool.eclipseActCode(9, "")
                    }
                }
            }

            EclipseKeyboard {
                id: cheatKeyboard
                visible: root.editorTyping
                unit: root.unit
                pixelScale: root.pixelScale
                typist: cheatTypist
                focus: root.editorOpen && root.editorTyping
                activeFocusOnKeys: root.editorOpen && root.editorTyping
                eraseIcon: root.view.eraseIcon ?? ""
            }
        }
    }
}
