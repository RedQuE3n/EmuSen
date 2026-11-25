pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Eclipse Search screen (#8): purple top glow, pill search bar, horizontal game cards, 4-row keyboard, centered footer.
Item {
    id: root
    required property Screens screens
    required property GameModel games
    required property Kit.KeyboardModel typist
    property LibraryModel library: null
    required property int padFamily
    required property bool splashShown
    readonly property real boardHeight: 1200
    property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    property real pixelScale: 1
    readonly property bool onKeys: root.screens.shown === Screens.Search
    readonly property string layouts: searchModel.layouts
    readonly property var view: JSON.parse(searchModel.view)

    SearchEclipseModel {
        id: searchModel
        objectName: "searchEclipseModel"
        library: root.library
        games: root.games
        screens: root.screens
        query: root.typist.text
        onKeys: root.onKeys
    }

    Connections {
        target: root.games
        function onRevisionChanged() { searchModel.refresh() }
    }
    Connections {
        target: root.library
        function onLoadingChanged() { searchModel.refresh() }
    }
    Connections {
        target: root.library
        function onMetadataChanged() { searchModel.refresh() }
    }

    onVisibleChanged: {
        if (root.visible && root.library) root.library.ensureMetadata()
        if (root.visible && !root.splashShown) (root.onKeys ? keyboard : resultsScope).forceActiveFocus()
    }
    onFocusChanged: if (root.focus && root.visible && !root.splashShown) (root.onKeys ? keyboard : resultsScope).forceActiveFocus()
    onOnKeysChanged: if (root.visible && !root.splashShown) (root.onKeys ? keyboard : resultsScope).forceActiveFocus()

    Kit.Frame {
        id: frame
        objectName: "searchEclipseFrame"
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
            function onActionInvoked(key) { root.typist.act(key) }
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { searchModel.handleAction(key) }
        }

        FocusScope {
            id: rootFocus
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            focus: true

            Rectangle {
                anchors.fill: parent
                color: "#07080F"
            }

            Kit.Backdrop {
                objectName: "searchBackdrop"
                anchors.fill: parent
                pixelScale: root.pixelScale
                fields: [
                    Kit.BackdropField {
                        x: 500 * root.unit
                        y: -420 * root.unit
                        width: 1000 * root.unit
                        height: Kit.CanvasUnits.designHeight * root.unit
                        color: "#5B3FD6"
                        blur: 150 * root.unit
                        opacity: 0.45
                    }
                ]
            }

            Item {
                id: inputPill
                objectName: "searchInput"
                x: 96 * root.unit
                y: 64 * root.unit
                width: rootFocus.width - 192 * root.unit
                height: 104 * root.unit

                Kit.FocusRing {
                    objectName: "searchInputRing"
                    anchors.fill: parent
                    visible: root.onKeys
                    radius: 52 * root.unit
                    gap: 3 * root.unit
                    thickness: 3 * root.unit
                    glowColor: "#59F0B4C4"
                    glowBlur: 46 * root.unit
                    glowSpread: 8 * root.unit
                    pixelScale: root.pixelScale
                }

                Rectangle {
                    objectName: "searchInputBg"
                    anchors.fill: parent
                    radius: 52 * root.unit
                    color: "#11131C"
                }

                Item {
                    id: magnifierBox
                    x: 36 * root.unit
                    y: (parent.height - height) / 2
                    width: 36 * root.unit
                    height: 36 * root.unit
                    Shape {
                        width: 24
                        height: 24
                        scale: 36 * root.unit / 24
                        transformOrigin: Item.TopLeft
                        preferredRendererType: Shape.CurveRenderer
                        ShapePath {
                            fillColor: "transparent"
                            strokeColor: "#F6E7D8"
                            strokeWidth: 2
                            capStyle: ShapePath.RoundCap
                            PathSvg { path: root.view.magnifierIcon ?? "" }
                        }
                    }
                }

                CardLine {
                    id: queryText
                    objectName: "searchQueryText"
                    x: 94 * root.unit
                    y: (parent.height - height) / 2
                    width: Math.max(0, countLabel.x - x - 22 * root.unit)
                    unit: root.unit
                    boardLine: true
                    text: root.typist.text.length > 0 ? root.typist.text : (root.view.placeholder ?? "")
                    size: 44
                    weight: Font.DemiBold
                    color: root.typist.text.length > 0 ? "#F4F1EC" : "#B9BED0"
                    elide: Text.ElideRight
                }

                CardLine {
                    id: countLabel
                    objectName: "searchCount"
                    anchors.right: parent.right
                    anchors.rightMargin: 36 * root.unit
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.view.countLine ?? ""
                    size: 22
                    weight: Font.Normal
                    color: "#B9BED0"
                }

                TapHandler {
                    onTapped: if (!root.onKeys) root.screens.keysFocused(true)
                }
            }

            Item {
                id: resultsSection
                x: 96 * root.unit
                y: 214 * root.unit
                width: rootFocus.width - 192 * root.unit
                height: 280 * root.unit

                CardLine {
                    id: resultsHeading
                    objectName: "searchHeading"
                    unit: root.unit
                    boardLine: true
                    text: root.view.gamesHeading ?? ""
                    size: 24
                    weight: Font.DemiBold
                    color: "#D8DBE6"
                }

                FocusScope {
                    id: resultsScope
                    focus: !root.onKeys
                    y: resultsHeading.height + 18 * root.unit
                    width: parent.width
                    height: 124 * root.unit
                    Keys.onPressed: (event) => event.accepted = searchModel.handleResultKey(event.key)

                    ListView {
                        id: resultsList
                        objectName: "searchResults"
                        x: -24 * root.unit
                        y: 0
                        width: parent.width + 48 * root.unit
                        height: 124 * root.unit
                        leftMargin: 24 * root.unit
                        rightMargin: 24 * root.unit
                        orientation: ListView.Horizontal
                        spacing: 24 * root.unit
                        interactive: resultsList.contentWidth > resultsList.width
                        model: root.view.cards ?? []
                        currentIndex: searchModel.focusedResult

                        delegate: Item {
                            id: card
                            required property int index
                            required property var modelData
                            readonly property bool focusedCard: !root.onKeys && searchModel.focusedResult === card.index

                            objectName: card.modelData.objectName ?? ""
                            width: 400 * root.unit
                            height: 124 * root.unit

                            Kit.FocusRing {
                                objectName: "searchResultRing." + card.index
                                anchors.fill: parent
                                visible: card.focusedCard
                                radius: 20 * root.unit
                                gap: 3 * root.unit
                                thickness: 3 * root.unit
                                glowColor: "#73F0B4C4"
                                glowBlur: 34 * root.unit
                                glowSpread: 8 * root.unit
                                pixelScale: root.pixelScale
                            }

                            Rectangle {
                                objectName: "searchResultBg." + card.index
                                anchors.fill: parent
                                radius: 20 * root.unit
                                color: "#11131C"
                                border.width: 1 * root.unit
                                border.color: "#262A3A"
                            }

                            EclipseArtPlate {
                                objectName: "searchResultArt." + card.index
                                x: 14 * root.unit
                                y: 14 * root.unit
                                width: 150 * root.unit
                                height: 96 * root.unit
                                radius: 12
                                unit: root.unit
                                pixelScale: root.pixelScale
                                cover: card.modelData.cover ?? ""
                                bgColor: card.modelData.bg ?? "#12806F"
                                dotColor: card.modelData.dot ?? "#B8FFE9"
                                waveColor: card.modelData.wave ?? "#6A4CF0"
                            }

                            Column {
                                x: 182 * root.unit
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 196 * root.unit
                                spacing: 6 * root.unit

                                CardLine {
                                    objectName: "searchResultTitle." + card.index
                                    width: parent.width
                                    unit: root.unit
                                    boardLine: true
                                    text: card.modelData.title ?? ""
                                    size: 23
                                    weight: Font.DemiBold
                                    color: "#F4F1EC"
                                    elide: Text.ElideRight
                                }

                                CardLine {
                                    objectName: "searchResultMeta." + card.index
                                    width: parent.width
                                    unit: root.unit
                                    boardLine: true
                                    text: card.modelData.subtitle ?? ""
                                    size: 17
                                    weight: Font.Normal
                                    color: "#B9BED0"
                                    elide: Text.ElideRight
                                }
                            }

                            TapHandler {
                                onTapped: {
                                    searchModel.focusedResult = card.index
                                    root.screens.openGame(card.modelData.path ?? "")
                                }
                            }
                        }
                    }

                    CardLine {
                        objectName: "searchEmptyGames"
                        visible: resultsList.count === 0
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.unit
                        boardLine: true
                        text: root.view.emptyLine ?? ""
                        size: 22
                        weight: Font.Normal
                        color: "#B9BED0"
                    }
                }

                CardLine {
                    objectName: "searchNote"
                    visible: (root.view.note ?? "").length > 0
                    y: resultsScope.y + resultsScope.height + 24 * root.unit
                    width: parent.width
                    unit: root.unit
                    boardLine: true
                    text: root.view.note ?? ""
                    size: 20
                    weight: Font.Normal
                    color: "#B9BED0"
                    elide: Text.ElideRight
                }
            }

            Rectangle {
                objectName: "searchDivider"
                x: 0
                y: 540 * root.unit
                width: rootFocus.width
                height: 2 * root.unit
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0.0; color: "transparent" }
                    GradientStop { position: 0.30; color: "#8CF6E7D8" }
                    GradientStop { position: 0.70; color: "#8CF6E7D8" }
                    GradientStop { position: 1.0; color: "transparent" }
                }
            }

            EclipseKeyboard {
                id: keyboard
                x: (rootFocus.width - width) / 2
                y: 590 * root.unit
                unit: root.unit
                pixelScale: root.pixelScale
                typist: root.typist
                focus: root.onKeys
                activeFocusOnKeys: root.onKeys && !root.splashShown
                eraseIcon: root.view.eraseIcon ?? ""
                keyPrefix: root.view.keyPrefix ?? ""
                keyRingPrefix: root.view.keyRingPrefix ?? ""
                keyBgPrefix: root.view.keyBgPrefix ?? ""
                keyTextPrefix: root.view.keyTextPrefix ?? ""
            }

            Item {
                objectName: "searchFooter"
                x: 96 * root.unit
                y: rootFocus.height - 72 * root.unit
                width: rootFocus.width - 192 * root.unit
                height: 38 * root.unit

                EclipseBoardHelpBar {
                    objectName: "searchHelp"
                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.verticalCenter: parent.verticalCenter
                    controller: frame.controller
                    padFamily: root.padFamily
                    unit: root.unit
                    entriesJson: searchModel.footerHelp
                }
            }
        }
    }
}
