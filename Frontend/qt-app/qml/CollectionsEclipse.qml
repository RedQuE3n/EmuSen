pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Eclipse Collections hub (#11): pink/purple corner glows, top nav, 4-column fanned cover cards, + New collection, #14 modals.
Kit.NormalizedCanvas {
    id: canvas
    objectName: "collectionsCanvas"
    required property Screens screens
    required property LibraryModel library
    required property Clock clock
    required property int battery
    required property int padFamily
    required property bool splashShown
    property bool bigScreen: true

    readonly property real boardHeight: 1200
    readonly property real boardUnit: Kit.CanvasUnits.designHeight / canvas.boardHeight
    readonly property var view: JSON.parse(collectionsModel.view)
    readonly property real gridWidth: canvas.width - 192 * canvas.boardUnit
    readonly property real cardWidth: (canvas.gridWidth - 3 * 28 * canvas.boardUnit) / 4
    readonly property int focusedRow: Math.floor(collectionsModel.focused / 4)
    readonly property bool sheetOpen: frame.controller.sheetOpen
    onSheetOpenChanged: {
        if (!canvas.sheetOpen) collectionsModel.dialogSettled()
        if (!canvas.sheetOpen && !collectionsModel.dialogOpen && !collectionsModel.typingOpen && canvas.visible && !canvas.splashShown) mainFocus.forceActiveFocus()
    }

    visible: canvas.bigScreen && canvas.screens.shown === Screens.Collections
    focus: !canvas.splashShown && canvas.visible
    onVisibleChanged: {
        if (canvas.visible && canvas.library) canvas.library.ensureMetadata()
        if (canvas.visible) collectionsModel.refresh()
        if (canvas.visible && !canvas.splashShown) mainFocus.forceActiveFocus()
    }
    onActiveFocusChanged: if (canvas.activeFocus && !collectionsModel.dialogOpen && !collectionsModel.typingOpen) mainFocus.forceActiveFocus()

    CollectionsEclipseModel {
        id: collectionsModel
        objectName: "collectionsEclipseModel"
        library: canvas.library
        screens: canvas.screens
        onTypingOpenChanged: {
            if (collectionsModel.typingOpen) {
                nameTypist.setText(collectionsModel.typingInitial, collectionsModel.typingInitial.length)
                if (!canvas.splashShown) collectionKeyboard.forceActiveFocus()
            } else if (canvas.visible && !canvas.splashShown && !collectionsModel.dialogOpen) {
                mainFocus.forceActiveFocus()
            }
        }
        onDialogOpenChanged: if (!collectionsModel.dialogOpen && !collectionsModel.typingOpen && canvas.visible && !canvas.splashShown) mainFocus.forceActiveFocus()
    }

    Connections {
        target: canvas.library
        enabled: canvas.visible
        function onLoadingChanged() { collectionsModel.refresh() }
    }
    Connections {
        target: canvas.library
        enabled: canvas.visible
        function onCollectionsChanged() { collectionsModel.refresh() }
    }
    Connections {
        target: canvas.library
        enabled: canvas.visible
        function onMetadataChanged() { collectionsModel.refresh() }
    }
    Connections {
        target: canvas.library
        function onCollectionProblem(message) { collectionsModel.showProblem(message) }
    }

    Kit.KeyboardModel {
        id: nameTypist
        objectName: "collectionTypist"
        layouts: collectionsModel.layouts
        onClosed: (kept) => collectionsModel.finishTyping(kept, nameTypist.text)
    }

    Kit.Frame {
        id: frame
        objectName: "collectionsFrame"
        anchors.fill: parent
        focus: true
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        padFamily: canvas.padFamily
        controller.navigationShown: false
        controller.actionLines: collectionsModel.dialogOpen ? "" : collectionsModel.typingOpen ? (canvas.view.typingActionLines ?? "") : (canvas.view.actionLines ?? "")

        Connections {
            target: frame.controller
            enabled: !collectionsModel.typingOpen && !collectionsModel.dialogOpen
            function onBackDeclined() { canvas.screens.back() }
        }
        Connections {
            target: frame.controller
            enabled: !collectionsModel.typingOpen && !collectionsModel.dialogOpen
            function onActionInvoked(key) { collectionsModel.handleAction(key) }
        }
        Connections {
            target: frame.controller
            enabled: collectionsModel.typingOpen
            function onActionInvoked(key) { nameTypist.act(key) }
        }

        Item {
            x: -frame.safeX
            y: -frame.safeY
            width: canvas.width
            height: canvas.height

            Rectangle {
                anchors.fill: parent
                color: Kit.Tokens.surface
            }

            Kit.Backdrop {
                objectName: "collectionsBackdrop"
                anchors.fill: parent
                pixelScale: canvas.unit
                fields: [
                    Kit.BackdropField {
                        x: -200 * canvas.boardUnit
                        y: -300 * canvas.boardUnit
                        width: 1000 * canvas.boardUnit
                        height: Kit.CanvasUnits.designHeight * canvas.boardUnit
                        color: ThemeInk.rose
                        blur: 170 * canvas.boardUnit
                        opacity: 0.30
                    },
                    Kit.BackdropField {
                        x: canvas.boardHeight * canvas.boardUnit
                        y: 600 * canvas.boardUnit
                        width: 1000 * canvas.boardUnit
                        height: Kit.CanvasUnits.designHeight * canvas.boardUnit
                        color: ThemeInk.backdropHalo
                        blur: 170 * canvas.boardUnit
                        opacity: 0.30
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

            FocusScope {
                id: mainFocus
                anchors.fill: parent
                enabled: !collectionsModel.dialogOpen && !collectionsModel.typingOpen
                focus: !collectionsModel.dialogOpen && !collectionsModel.typingOpen
                Keys.onPressed: (event) => event.accepted = collectionsModel.handleKey(event.key)

                Flickable {
                    id: scroller
                    x: 68 * canvas.boardUnit
                    y: 162 * canvas.boardUnit
                    width: canvas.gridWidth + 56 * canvas.boardUnit
                    height: canvas.height - 252 * canvas.boardUnit
                    leftMargin: 28 * canvas.boardUnit
                    rightMargin: 28 * canvas.boardUnit
                    topMargin: 28 * canvas.boardUnit
                    bottomMargin: 28 * canvas.boardUnit
                    contentWidth: canvas.gridWidth
                    contentHeight: gridLayout.height
                    contentY: Math.max(0, canvas.focusedRow >= 2 ? (canvas.focusedRow - 1) * 438 * canvas.boardUnit : 0)
                    clip: true

                    Grid {
                        id: gridLayout
                        objectName: "collectionsGrid"
                        columns: 4
                        columnSpacing: 28 * canvas.boardUnit
                        rowSpacing: 28 * canvas.boardUnit

                        Repeater {
                            model: canvas.view.cards ?? []
                            Item {
                                id: card
                                required property int index
                                required property var modelData
                                readonly property bool focusedCard: !collectionsModel.dialogOpen && !collectionsModel.typingOpen && collectionsModel.focused === card.index
                                readonly property var covers: card.modelData.covers ?? []

                                objectName: card.modelData.objectName ?? ""
                                width: canvas.cardWidth
                                height: 410 * canvas.boardUnit

                                Kit.FocusRing {
                                    objectName: "collectionCardRing." + card.index
                                    anchors.fill: parent
                                    visible: card.focusedCard
                                    radius: 26 * canvas.boardUnit
                                    gap: 4 * canvas.boardUnit
                                    thickness: 3 * canvas.boardUnit
                                    glowColor: ThemeInk.glowMid
                                    glowBlur: 46 * canvas.boardUnit
                                    glowSpread: 10 * canvas.boardUnit
                                    pixelScale: canvas.unit
                                }

                                Rectangle {
                                    objectName: "collectionCardBg." + card.index
                                    anchors.fill: parent
                                    radius: 26 * canvas.boardUnit
                                    color: Kit.Tokens.inputSurface
                                    border.width: 1 * canvas.boardUnit
                                    border.color: ThemeInk.hairline
                                }

                                Item {
                                    x: 26 * canvas.boardUnit
                                    y: 26 * canvas.boardUnit
                                    width: parent.width - 52 * canvas.boardUnit
                                    height: 220 * canvas.boardUnit

                                    EclipseArtPlate {
                                        x: 96 * canvas.boardUnit
                                        y: 20 * canvas.boardUnit
                                        width: 250 * canvas.boardUnit
                                        height: 170 * canvas.boardUnit
                                        rotation: 7
                                        radius: 14
                                        unit: canvas.boardUnit
                                        pixelScale: canvas.unit
                                        cover: card.covers[2]?.cover ?? ""
                                        slot: 2
                                        swatch: card.covers[2]
                                    }

                                    EclipseArtPlate {
                                        x: 50 * canvas.boardUnit
                                        y: 14 * canvas.boardUnit
                                        width: 250 * canvas.boardUnit
                                        height: 170 * canvas.boardUnit
                                        rotation: -4
                                        radius: 14
                                        unit: canvas.boardUnit
                                        pixelScale: canvas.unit
                                        shadowBlur: 20
                                        shadowOffsetY: 8
                                        shadowColor: ThemeInk.shadowMid
                                        cover: card.covers[1]?.cover ?? ""
                                        slot: 1
                                        swatch: card.covers[1]
                                    }

                                    EclipseArtPlate {
                                        x: 14 * canvas.boardUnit
                                        y: 34 * canvas.boardUnit
                                        width: 270 * canvas.boardUnit
                                        height: 180 * canvas.boardUnit
                                        radius: 14
                                        unit: canvas.boardUnit
                                        pixelScale: canvas.unit
                                        shadowBlur: 26
                                        shadowOffsetY: 10
                                        shadowColor: ThemeInk.shadowStrong
                                        cover: card.covers[0]?.cover ?? ""
                                        slot: 0
                                        swatch: card.covers[0]
                                    }
                                }

                                Column {
                                    x: 26 * canvas.boardUnit
                                    anchors.bottom: parent.bottom
                                    anchors.bottomMargin: 26 * canvas.boardUnit
                                    width: parent.width - 52 * canvas.boardUnit
                                    spacing: 8 * canvas.boardUnit

                                    CardLine {
                                        objectName: "collectionCardTitle." + card.index
                                        width: parent.width
                                        unit: canvas.boardUnit
                                        boardLine: true
                                        text: card.modelData.name ?? ""
                                        size: 30
                                        weight: Font.DemiBold
                                        color: Kit.Tokens.text
                                        elide: Text.ElideRight
                                    }

                                    CardLine {
                                        objectName: "collectionCardMeta." + card.index
                                        width: parent.width
                                        unit: canvas.boardUnit
                                        boardLine: true
                                        text: card.modelData.meta ?? ""
                                        size: 19
                                        weight: Font.Normal
                                        color: Kit.Tokens.muted
                                        elide: Text.ElideRight
                                    }
                                }

                                TapHandler {
                                    onTapped: collectionsModel.openSlot(card.index)
                                }
                            }
                        }

                        Item {
                            id: newCard
                            objectName: "newCollectionCard"
                            readonly property int slotIndex: canvas.view.totalCount ?? 0
                            readonly property bool focusedCard: !collectionsModel.dialogOpen && !collectionsModel.typingOpen && collectionsModel.focused === newCard.slotIndex
                            readonly property real cr: 26 * canvas.boardUnit

                            width: canvas.cardWidth
                            height: 410 * canvas.boardUnit

                            Kit.FocusRing {
                                anchors.fill: parent
                                visible: newCard.focusedCard
                                radius: 26 * canvas.boardUnit
                                gap: 4 * canvas.boardUnit
                                thickness: 3 * canvas.boardUnit
                                glowColor: ThemeInk.glowMid
                                glowBlur: 46 * canvas.boardUnit
                                glowSpread: 10 * canvas.boardUnit
                                pixelScale: canvas.unit
                            }

                            Shape {
                                anchors.fill: parent
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    fillColor: "transparent"
                                    strokeColor: ThemeInk.pillBorder
                                    strokeWidth: 2 * canvas.boardUnit
                                    strokeStyle: ShapePath.DashLine
                                    dashPattern: Kit.Tokens.dashPattern
                                    startX: newCard.cr
                                    startY: 0
                                    PathLine { x: newCard.width - newCard.cr; y: 0 }
                                    PathArc { x: newCard.width; y: newCard.cr; radiusX: newCard.cr; radiusY: newCard.cr }
                                    PathLine { x: newCard.width; y: newCard.height - newCard.cr }
                                    PathArc { x: newCard.width - newCard.cr; y: newCard.height; radiusX: newCard.cr; radiusY: newCard.cr }
                                    PathLine { x: newCard.cr; y: newCard.height }
                                    PathArc { x: 0; y: newCard.height - newCard.cr; radiusX: newCard.cr; radiusY: newCard.cr }
                                    PathLine { x: 0; y: newCard.cr }
                                    PathArc { x: newCard.cr; y: 0; radiusX: newCard.cr; radiusY: newCard.cr }
                                }
                            }

                            Column {
                                anchors.centerIn: parent
                                spacing: 18 * canvas.boardUnit

                                Item {
                                    anchors.horizontalCenter: parent.horizontalCenter
                                    width: 56 * canvas.boardUnit
                                    height: 56 * canvas.boardUnit
                                    Shape {
                                        width: 24
                                        height: 24
                                        scale: 56 * canvas.boardUnit / 24
                                        transformOrigin: Item.TopLeft
                                        preferredRendererType: Shape.CurveRenderer
                                        ShapePath {
                                            fillColor: "transparent"
                                            strokeColor: ThemeInk.body
                                            strokeWidth: 1.6
                                            capStyle: ShapePath.RoundCap
                                            PathSvg { path: canvas.view.plusIcon ?? "" }
                                        }
                                    }
                                }

                                CardLine {
                                    objectName: "newCollectionLabel"
                                    anchors.horizontalCenter: parent.horizontalCenter
                                    unit: canvas.boardUnit
                                    boardLine: true
                                    text: canvas.view.newCardLabel ?? ""
                                    size: 26
                                    weight: Font.Normal
                                    color: ThemeInk.body
                                }
                            }

                            TapHandler {
                                onTapped: collectionsModel.openSlot(newCard.slotIndex)
                            }
                        }
                    }
                }
            }

            Item {
                objectName: "collectionsFooter"
                x: 96 * canvas.boardUnit
                y: canvas.height - 78 * canvas.boardUnit
                width: canvas.gridWidth
                height: 38 * canvas.boardUnit
                enabled: !collectionsModel.typingOpen && !collectionsModel.dialogOpen

                EclipseBoardHelpBar {
                    objectName: "collectionsHelp"
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    controller: frame.controller
                    padFamily: canvas.padFamily
                    unit: canvas.boardUnit
                    entriesJson: collectionsModel.footerHelp
                }

                CardLine {
                    objectName: "collectionsFooterSummary"
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    unit: canvas.boardUnit
                    boardLine: true
                    text: canvas.view.footerRight ?? ""
                    size: 21
                    weight: Font.Normal
                    color: ThemeInk.body
                }
            }

            FocusScope {
                id: typingOverlay
                anchors.fill: parent
                visible: collectionsModel.typingOpen
                focus: collectionsModel.typingOpen
                z: 20

                Rectangle {
                    anchors.fill: parent
                    color: ThemeInk.glassStrong
                    MouseArea {
                        anchors.fill: parent
                    }
                }

                Column {
                    anchors.horizontalCenter: parent.horizontalCenter
                    y: 120 * canvas.boardUnit
                    width: 1446 * canvas.boardUnit
                    spacing: 24 * canvas.boardUnit

                    CardLine {
                        objectName: "collectionTypingTitle"
                        unit: canvas.boardUnit
                        boardLine: true
                        text: collectionsModel.typingTitle
                        size: 40
                        weight: Font.DemiBold
                        color: Kit.Tokens.text
                    }

                    CardLine {
                        unit: canvas.boardUnit
                        boardLine: true
                        text: collectionsModel.typingPrompt
                        size: 22
                        weight: Font.Normal
                        color: Kit.Tokens.muted
                    }

                    Item {
                        width: parent.width
                        height: 104 * canvas.boardUnit

                        Kit.FocusRing {
                            anchors.fill: parent
                            radius: 52 * canvas.boardUnit
                            gap: 3 * canvas.boardUnit
                            thickness: 3 * canvas.boardUnit
                            glowColor: ThemeInk.glowRing
                            glowBlur: 46 * canvas.boardUnit
                            glowSpread: 8 * canvas.boardUnit
                            pixelScale: canvas.unit
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 52 * canvas.boardUnit
                            color: Kit.Tokens.inputSurface
                        }

                        CardLine {
                            objectName: "collectionTyped"
                            x: 36 * canvas.boardUnit
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 72 * canvas.boardUnit
                            unit: canvas.boardUnit
                            boardLine: true
                            text: nameTypist.text
                            size: 40
                            weight: Font.DemiBold
                            color: Kit.Tokens.text
                            elide: Text.ElideRight
                        }
                    }

                    EclipseKeyboard {
                        id: collectionKeyboard
                        unit: canvas.boardUnit
                        pixelScale: canvas.unit
                        typist: nameTypist
                        focus: collectionsModel.typingOpen
                        activeFocusOnKeys: collectionsModel.typingOpen && !canvas.splashShown
                        eraseIcon: canvas.view.eraseIcon ?? ""
                        keyPrefix: canvas.view.keyPrefix ?? ""
                        keyRingPrefix: canvas.view.keyRingPrefix ?? ""
                        keyBgPrefix: canvas.view.keyBgPrefix ?? ""
                        keyTextPrefix: canvas.view.keyTextPrefix ?? ""
                    }
                }
            }
        }
    }

    QuestionHost {
        id: dialogHost
        objectName: "collectionsDialog"
        controller: frame.controller
        showing: collectionsModel.dialogOpen
        title: collectionsModel.dialogTitle
        message: collectionsModel.dialogMessage
        choices: collectionsModel.dialogChoices
        backIndex: collectionsModel.dialogBackIndex
        padFamily: canvas.padFamily
        onChosen: (idx) => collectionsModel.dialogChoose(idx)
    }
}
