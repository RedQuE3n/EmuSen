pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Eclipse Tools hub (#9): blue/magenta corner glows, top nav, "Tools" heading, 3x2 tool cards, footer.
Kit.NormalizedCanvas {
    id: canvas
    objectName: "toolsCanvas"
    required property Screens screens
    required property LibraryModel library
    required property SettingsModel settings
    required property Clock clock
    required property int battery
    required property int padFamily
    required property bool splashShown

    readonly property real boardHeight: 1200
    readonly property real boardUnit: Kit.CanvasUnits.designHeight / canvas.boardHeight
    readonly property var view: JSON.parse(toolsModel.view)
    readonly property real gridWidth: canvas.width - 192 * canvas.boardUnit
    readonly property real cardWidth: (canvas.gridWidth - 2 * 28 * canvas.boardUnit) / 3

    visible: canvas.screens.shown === Screens.Tools
    focus: !canvas.splashShown && canvas.visible
    onVisibleChanged: {
        if (canvas.visible && canvas.library) canvas.library.ensureMetadata()
        if (canvas.visible) toolsModel.refresh()
        if (canvas.visible && !canvas.splashShown) mainFocus.forceActiveFocus()
    }
    onActiveFocusChanged: if (canvas.activeFocus) mainFocus.forceActiveFocus()

    ToolsHubModel {
        id: toolsModel
        objectName: "toolsHubModel"
        library: canvas.library
        settings: canvas.settings
        screens: canvas.screens
    }

    Connections {
        target: canvas.library
        function onLoadingChanged() { toolsModel.refresh() }
    }
    Connections {
        target: canvas.library
        function onToolSummariesChanged() { toolsModel.refresh() }
    }
    Connections {
        target: canvas.library
        function onMetadataChanged() { toolsModel.refresh() }
    }
    Connections {
        target: canvas.settings
        function onReadyChanged() { toolsModel.refresh() }
    }
    Connections {
        target: canvas.settings
        function onSaved() { toolsModel.refresh() }
    }

    Kit.Frame {
        id: frame
        objectName: "toolsFrame"
        anchors.fill: parent
        focus: true
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        padFamily: canvas.padFamily
        controller.navigationShown: false
        controller.actionLines: canvas.view.actionLines ?? ""

        Connections {
            target: frame.controller
            function onBackDeclined() { canvas.screens.back() }
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { toolsModel.handleAction(key) }
        }

        FocusScope {
            id: mainFocus
            x: -frame.safeX
            y: -frame.safeY
            width: canvas.width
            height: canvas.height
            focus: true
            Keys.onPressed: (event) => event.accepted = toolsModel.handleKey(event.key)

            Rectangle {
                anchors.fill: parent
                color: "#07080F"
            }

            Kit.Backdrop {
                objectName: "toolsBackdrop"
                anchors.fill: parent
                pixelScale: canvas.unit
                fields: [
                    Kit.BackdropField {
                        x: 1100 * canvas.boardUnit
                        y: -300 * canvas.boardUnit
                        width: 1100 * canvas.boardUnit
                        height: 900 * canvas.boardUnit
                        color: "#2A6FD6"
                        blur: 160 * canvas.boardUnit
                        opacity: 0.35
                    },
                    Kit.BackdropField {
                        x: -300 * canvas.boardUnit
                        y: 700 * canvas.boardUnit
                        width: 900 * canvas.boardUnit
                        height: Kit.CanvasUnits.designHeight * canvas.boardUnit
                        color: "#B0487A"
                        blur: 170 * canvas.boardUnit
                        opacity: 0.25
                    }
                ]
            }

            EclipseSectionHeader {
                unit: canvas.boardUnit
                navSections: canvas.view.navSections ?? []
                battery: canvas.battery
                clockText: canvas.clock.shortTime
                onSectionSelected: (key) => canvas.screens.openSection(key)
            }

            Column {
                x: 96 * canvas.boardUnit
                y: 170 * canvas.boardUnit
                width: canvas.gridWidth
                spacing: 8 * canvas.boardUnit

                CardLine {
                    objectName: "toolsHeading"
                    unit: canvas.boardUnit
                    boardLine: true
                    text: canvas.view.title ?? ""
                    size: 56
                    weight: Font.DemiBold
                    spacing: -56 * 0.01
                    color: "#F4F1EC"
                }

                CardLine {
                    objectName: "toolsSubtitle"
                    unit: canvas.boardUnit
                    boardLine: true
                    text: canvas.view.subtitle ?? ""
                    size: 22
                    weight: Font.Light
                    color: "#B9BED0"
                }
            }

            Grid {
                objectName: "toolsGrid"
                x: 96 * canvas.boardUnit
                y: 330 * canvas.boardUnit
                columns: 3
                columnSpacing: 28 * canvas.boardUnit
                rowSpacing: 28 * canvas.boardUnit

                Repeater {
                    model: canvas.view.cards ?? []
                    Item {
                        id: card
                        required property int index
                        required property var modelData
                        readonly property bool focusedCard: toolsModel.focused === card.index

                        objectName: card.modelData.objectName ?? ""
                        width: canvas.cardWidth
                        height: 312 * canvas.boardUnit

                        Kit.FocusRing {
                            objectName: "toolCardRing." + card.index
                            anchors.fill: parent
                            visible: card.focusedCard
                            radius: 26 * canvas.boardUnit
                            gap: 4 * canvas.boardUnit
                            thickness: 3 * canvas.boardUnit
                            glowColor: "#73F0B4C4"
                            glowBlur: 46 * canvas.boardUnit
                            glowSpread: 10 * canvas.boardUnit
                            pixelScale: canvas.unit
                        }

                        Rectangle {
                            objectName: "toolCardBg." + card.index
                            anchors.fill: parent
                            radius: 26 * canvas.boardUnit
                            color: "#11131C"
                            border.width: 1 * canvas.boardUnit
                            border.color: "#262A3A"
                        }

                        Item {
                            anchors.fill: parent
                            anchors.margins: 32 * canvas.boardUnit

                            Item {
                                objectName: "toolIconDisc." + card.index
                                width: 84 * canvas.boardUnit
                                height: 84 * canvas.boardUnit

                                Kit.FocusRing {
                                    anchors.fill: parent
                                    radius: 42 * canvas.boardUnit
                                    gap: 0
                                    thickness: 2 * canvas.boardUnit
                                    glowColor: "#66F6D6C8"
                                    glowBlur: 26 * canvas.boardUnit
                                    glowSpread: 4 * canvas.boardUnit
                                    pixelScale: canvas.unit
                                }

                                Rectangle {
                                    objectName: "toolIconCircle." + card.index
                                    anchors.fill: parent
                                    radius: 42 * canvas.boardUnit
                                    color: "#07080F"
                                }

                                Item {
                                    anchors.centerIn: parent
                                    width: 38 * canvas.boardUnit
                                    height: 38 * canvas.boardUnit
                                    Shape {
                                        width: 24
                                        height: 24
                                        scale: 38 * canvas.boardUnit / 24
                                        transformOrigin: Item.TopLeft
                                        preferredRendererType: Shape.CurveRenderer
                                        ShapePath {
                                            fillColor: "transparent"
                                            strokeColor: "#F6E7D8"
                                            strokeWidth: 1.6
                                            capStyle: ShapePath.RoundCap
                                            joinStyle: ShapePath.RoundJoin
                                            PathSvg { path: card.modelData.icon ?? "" }
                                        }
                                    }
                                }
                            }

                            Rectangle {
                                visible: (card.modelData.badge ?? "").length > 0
                                anchors.right: parent.right
                                anchors.top: parent.top
                                width: badgeLabel.width + 32 * canvas.boardUnit
                                height: 38 * canvas.boardUnit
                                radius: 19 * canvas.boardUnit
                                color: "#F3C9D2"

                                CardLine {
                                    id: badgeLabel
                                    objectName: card.modelData.badgeName ?? ""
                                    anchors.centerIn: parent
                                    unit: canvas.boardUnit
                                    boardLine: true
                                    text: card.modelData.badge ?? ""
                                    size: 17
                                    weight: Font.Bold
                                    color: "#07080F"
                                }
                            }

                            Column {
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.bottom: parent.bottom
                                spacing: 10 * canvas.boardUnit

                                CardLine {
                                    objectName: "toolCardTitle." + card.index
                                    width: parent.width
                                    unit: canvas.boardUnit
                                    boardLine: true
                                    text: card.modelData.name ?? ""
                                    size: 34
                                    weight: Font.DemiBold
                                    color: "#F4F1EC"
                                    elide: Text.ElideRight
                                }

                                CardLine {
                                    objectName: "toolCardLine." + card.index
                                    width: parent.width
                                    unit: canvas.boardUnit
                                    text: card.modelData.line ?? ""
                                    size: 20
                                    weight: Font.Normal
                                    color: "#C9CDDB"
                                    lineHeight: 1.4
                                    lineHeightMode: Text.ProportionalHeight
                                    wrapMode: Text.WordWrap
                                    maximumLineCount: 2
                                    elide: Text.ElideRight
                                }
                            }
                        }

                        TapHandler {
                            onTapped: toolsModel.openTool(card.index)
                        }
                    }
                }
            }

            Item {
                objectName: "toolsFooter"
                x: 96 * canvas.boardUnit
                y: canvas.height - 78 * canvas.boardUnit
                width: canvas.gridWidth
                height: 38 * canvas.boardUnit

                EclipseBoardHelpBar {
                    objectName: "toolsHelp"
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    controller: frame.controller
                    padFamily: canvas.padFamily
                    unit: canvas.boardUnit
                    entriesJson: toolsModel.footerHelp
                }
            }
        }
    }
}
