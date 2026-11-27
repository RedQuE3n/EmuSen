pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's game page and all six inner pages as their boards draw them (EclipseGame*.dc.html; EmuSen_Qt.md §3.4.12).
ThemeScreen {
    id: screen
    objectName: "gameScreen"
    role: ThemeScreen.Game
    // All six tabs and all six deeper pages; questions and typing keyboard stay the program's (1.19).
    pageSections: (1 << Page.Overview) | (1 << Page.States) | (1 << Page.Media) | (1 << Page.Details) | (1 << Page.Options) | (1 << Page.Cheats)
    pageSubpages: (1 << Page.Package) | (1 << Page.Edit) | (1 << Page.Cover) | (1 << Page.Scrape) | (1 << Page.Viewer) | (1 << Page.Crop)
    // Where the focus starts: the deeper page's focused item, the tab's content (cards, media or rows), else the first button.
    focusItems: [
        Page.subpage === Page.Package
            ? ((boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(Page.focusedContent) as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem ?? null)
            : Page.subpage === Page.Edit
                ? ((editRows.itemAt(Page.focusedContent) as EditRowSlot)?.rowItem ?? (editRows.itemAt(0) as EditRowSlot)?.rowItem ?? null)
                : Page.subpage === Page.Cover
                    ? ((coverGrid.currentItem as CoverSlot)?.walk ?? (coverGrid.itemAtIndex(Page.focusedContent) as CoverSlot)?.walk ?? (coverGrid.itemAtIndex(0) as CoverSlot)?.walk ?? null)
                    : Page.subpage === Page.Scrape
                        ? ((scrapeList.currentItem as ScrapeSlot)?.rowItem ?? (scrapeList.itemAtIndex(0) as ScrapeSlot)?.rowItem ?? null)
                        : Page.subpage === Page.Viewer
                            ? viewerScope
                            : Page.subpage === Page.Crop
                                ? ((cropRows.currentItem as CropRowSlot)?.rowItem ?? (cropRows.itemAtIndex(Page.focusedContent) as CropRowSlot)?.rowItem ?? (cropRows.itemAtIndex(0) as CropRowSlot)?.rowItem ?? null)
                                : Page.tab === Page.States
                                    ? ((cards.currentItem as CardSlot)?.walk ?? (cards.itemAtIndex(0) as CardSlot)?.walk ?? (pills.itemAt(0) as PillSlot)?.walk ?? null)
                                    : Page.tab === Page.Media
                                        ? ((mediaList.currentItem as MediaSlot)?.walk ?? (mediaList.itemAtIndex(0) as MediaSlot)?.walk ?? (pills.itemAt(0) as PillSlot)?.walk ?? null)
                                        : (Page.tab === Page.Details || Page.tab === Page.Options || Page.tab === Page.Cheats)
                                            ? ((boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem ?? (pills.itemAt(0) as PillSlot)?.walk ?? null)
                                            : ((pills.itemAt(0) as PillSlot)?.walk ?? null)
    ]
    NormalizedCanvas {
        id: canvas
        objectName: "gameCanvas"
        FocusScope {
            id: root
            objectName: "game"
            anchors.fill: parent
            focus: true
            // A board pixel in canvas units, Board's.
            readonly property real px: design.unit
            // The board's left margin, a constant of the design (EclipseGame:26, 42, 55, 64, 86).
            readonly property real margin: 96 * root.px
            readonly property int shownSubpage: Page.subpage
            readonly property bool inPackage: Page.subpage === Page.Package
            readonly property bool inEdit: Page.subpage === Page.Edit
            readonly property bool inCover: Page.subpage === Page.Cover
            readonly property bool inScrape: Page.subpage === Page.Scrape
            readonly property bool inViewer: Page.subpage === Page.Viewer
            readonly property bool inCrop: Page.subpage === Page.Crop
            readonly property bool inSubpage: Page.subpage !== Page.None
            readonly property bool inHeaderSubpage: root.inSubpage && !root.inViewer
            readonly property int shownTab: Page.tab
            // The help in the boards' words where they draw it, else the program's for the focus.
            readonly property int helpKind: Page.boardHelp >= 0 ? Page.boardHelp : Page.help
            readonly property var helpEntries: root.helpKind === Help.PageOption
                ? Help.entries(root.helpKind, { defaultable: Page.defaultable })
                : root.helpKind === Help.PageMedia
                    ? Help.entries(root.helpKind, { defaultable: Page.coverable })
                    : Help.entries(root.helpKind)
            onShownSubpageChanged: {
                if (root.inPackage) {
                    const at = Math.min(Math.max(Page.focusedContent, 0), Math.max(boardRows.count - 1, 0))
                    boardRows.positionViewAtIndex(at, ListView.Contain)
                    boardRows.currentIndex = at
                    const item = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(at) as PageRowSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (root.inEdit) {
                    const at = Math.min(Math.max(Page.focusedContent, 0), Math.max(editRows.count - 1, 0))
                    const item = (editRows.itemAt(at) as EditRowSlot)?.rowItem ?? (editRows.itemAt(0) as EditRowSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (root.inCover) {
                    const at = Math.min(Math.max(Page.focusedContent, 0), Math.max(coverGrid.count - 1, 0))
                    coverGrid.positionViewAtIndex(at, GridView.Contain)
                    coverGrid.currentIndex = at
                    const item = (coverGrid.currentItem as CoverSlot)?.walk ?? (coverGrid.itemAtIndex(at) as CoverSlot)?.walk
                    item?.forceActiveFocus()
                } else if (root.inScrape) {
                    scrapeList.positionViewAtIndex(0, ListView.Contain)
                    scrapeList.currentIndex = 0
                    const item = (scrapeList.currentItem as ScrapeSlot)?.rowItem ?? (scrapeList.itemAtIndex(0) as ScrapeSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (root.inViewer) {
                    viewerScope.forceActiveFocus()
                } else if (root.inCrop) {
                    const at = Math.min(Math.max(Page.focusedContent, 0), Math.max(cropRows.count - 1, 0))
                    cropRows.positionViewAtIndex(at, ListView.Contain)
                    cropRows.currentIndex = at
                    const item = (cropRows.currentItem as CropRowSlot)?.rowItem ?? (cropRows.itemAtIndex(at) as CropRowSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (rowsBox.visible) {
                    boardRows.positionViewAtIndex(0, ListView.Contain)
                    boardRows.currentIndex = 0
                    const item = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (Page.tab === Page.Media) {
                    if (mediaList.count > 0) {
                        const at = Math.min(Math.max(mediaList.currentIndex, 0), mediaList.count - 1)
                        mediaList.positionViewAtIndex(at, ListView.Contain)
                        mediaList.currentIndex = at
                        const item = (mediaList.currentItem as MediaSlot)?.walk ?? (mediaList.itemAtIndex(at) as MediaSlot)?.walk
                        item?.forceActiveFocus()
                    } else {
                        (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus()
                    }
                }
            }
            onShownTabChanged: {
                if (root.inSubpage)
                    return
                if (rowsBox.visible && boardRows.count > 0) {
                    boardRows.positionViewAtIndex(0, ListView.Contain)
                    boardRows.currentIndex = 0
                    const item = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (Page.tab === Page.Media) {
                    if (mediaList.count > 0) {
                        mediaList.positionViewAtIndex(0, ListView.Contain)
                        mediaList.currentIndex = 0
                        const item = (mediaList.currentItem as MediaSlot)?.walk ?? (mediaList.itemAtIndex(0) as MediaSlot)?.walk
                        item?.forceActiveFocus()
                    } else {
                        (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus()
                    }
                }
            }
            Board {
                id: design
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            GamesInk {
                id: gamesInk
            }
            GameInk {
                id: own
            }
            // The game's hues for the board's fields and the disc (parts/ArtHues.qml; R-20261115-27, -28).
            ArtHues {
                id: hues
                key: Page.key
            }
            FieldInk { id: field0; board: ink.field0; hue: hues.hues.length > 0 ? hues.hues[0] : NaN }
            FieldInk { id: field1; board: ink.field1; hue: hues.hues.length > 0 ? hues.hues[1] : NaN }
            FieldInk { id: field3; board: ink.field3; hue: hues.hues.length > 0 ? hues.hues[0] : NaN }
            // The ground and the board's three blurred fields, each at its place, size, turn, blur and opacity (EclipseGame:17-21).
            Backdrop {
                objectName: "gameBackdrop"
                visible: !root.inSubpage
                anchors.fill: parent
                ground: ink.ground
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: 980 * root.px; y: -160 * root.px; width: 980 * root.px; height: 800 * root.px; color: field0.color; blur: 90 * root.px; opacity: 0.85 },
                    BackdropField { x: 1340 * root.px; y: 260 * root.px; width: 760 * root.px; height: 700 * root.px; color: field1.color; blur: 110 * root.px; opacity: 0.9 },
                    BackdropField { x: 1180 * root.px; y: 210 * root.px; width: 640 * root.px; height: 46 * root.px; corner: 40 * root.px; color: field3.color; blur: 22 * root.px; opacity: 0.55; rotation: -18 }
                ]
            }
            // The deeper pages' dark ground and two top-right fields (EclipseGamePackage/Edit/Cover/Scrape/Crop:17-19).
            Backdrop {
                objectName: "packageBackdrop"
                visible: root.inHeaderSubpage
                anchors.fill: parent
                ground: Tokens.surface
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: 1240 * root.px; y: -420 * root.px; width: 900 * root.px; height: 720 * root.px; color: field0.color; blur: 110 * root.px; opacity: 0.5 },
                    BackdropField { x: 1500 * root.px; y: -200 * root.px; width: 700 * root.px; height: 600 * root.px; color: field1.color; blur: 120 * root.px; opacity: 0.55 }
                ]
            }
            // Media Viewer's pure black ground; transparent while the host plays a clip behind it (EclipseGameViewer:17).
            Rectangle {
                objectName: "viewerBackdrop"
                visible: root.inViewer && Page.viewKind !== 1
                anchors.fill: parent
                color: Tokens.voidFill
            }
            CoronaDisc {
                id: disc
                objectName: "gameDisc"
                visible: !root.inSubpage
                x: -760 * root.px
                y: -1500 * root.px
                width: 2900 * root.px
                height: 2900 * root.px
                // The game's first hue at the look's rim's saturation and lightness, its glows at the board's strengths (EclipseGame:22).
                corona: hues.hues.length > 0 ? Qt.hsla((((hues.hues[0] % 360) + 360) % 360) / 360, Tokens.coronaColor.hslSaturation, Tokens.coronaColor.hslLightness, 1) : Tokens.coronaColor
                rim: 2 * root.px
                pixelScale: canvas.unit
                glows: [
                    CoronaGlow {
                        blur: 44 * root.px
                        spread: 8 * root.px
                        color: Qt.rgba(disc.corona.r, disc.corona.g, disc.corona.b, 0.55)
                    },
                    CoronaGlow {
                        blur: 170 * root.px
                        spread: 46 * root.px
                        color: Qt.rgba(disc.corona.r, disc.corona.g, disc.corona.b, 0.25)
                    }
                ]
            }
            Horizon {
                visible: !root.inSubpage
                anchors.fill: parent
                unit: root.px
                ground: Tokens.surface
                line: own.horizon
                clear: own.horizonClear
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                // The help line below draws the entries; the frame keeps their actions, X's and Y's and Start's.
                helpShown: false
                focus: true
                controller.navigationShown: false
                help: root.helpEntries
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.pageDeclared(key) }
                    function onBackDeclined() { Actions.pageBack() }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    // The header: the mark, the trail, and the status on its glass (EclipseGame:26-40, EclipseGamePackage:20-36).
                    Brand {
                        id: brand
                        objectName: "gameBrand"
                        visible: !root.inViewer
                        unit: root.px
                        named: false
                        x: root.margin
                        y: 80 * root.px - height / 2
                    }
                    Trail {
                        id: trail
                        visible: !root.inViewer
                        unit: root.px
                        controller: frame.controller
                        words: Page.trail
                        slash: gamesInk.slash
                        x: brand.x + brand.width + 28 * root.px
                        y: 80 * root.px - height / 2
                        onChosen: (index) => index === 0 ? Actions.openSection(Page.trailSection) : index === 1 ? Actions.openConsole(Page.console) : Actions.pageBack()
                        onDownward: {
                            if (root.inPackage) {
                                boardRows.positionViewAtIndex(0, ListView.Contain)
                                boardRows.currentIndex = 0
                                const first = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem
                                first?.forceActiveFocus(Qt.TabFocusReason)
                            } else if (root.inEdit) {
                                const first = (editRows.itemAt(0) as EditRowSlot)?.rowItem
                                first?.forceActiveFocus(Qt.TabFocusReason)
                            } else if (root.inCover) {
                                coverGrid.positionViewAtIndex(0, GridView.Contain)
                                coverGrid.currentIndex = 0
                                const first = (coverGrid.currentItem as CoverSlot)?.walk ?? (coverGrid.itemAtIndex(0) as CoverSlot)?.walk
                                first?.forceActiveFocus(Qt.TabFocusReason)
                            } else if (root.inScrape) {
                                scrapeList.positionViewAtIndex(0, ListView.Contain)
                                scrapeList.currentIndex = 0
                                const first = (scrapeList.currentItem as ScrapeSlot)?.rowItem ?? (scrapeList.itemAtIndex(0) as ScrapeSlot)?.rowItem
                                first?.forceActiveFocus(Qt.TabFocusReason)
                            } else if (root.inCrop) {
                                cropRows.positionViewAtIndex(0, ListView.Contain)
                                cropRows.currentIndex = 0
                                const first = (cropRows.currentItem as CropRowSlot)?.rowItem ?? (cropRows.itemAtIndex(0) as CropRowSlot)?.rowItem
                                first?.forceActiveFocus(Qt.TabFocusReason)
                            } else {
                                (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason)
                            }
                        }
                    }
                    Status {
                        objectName: "gameStatus"
                        visible: !root.inViewer
                        unit: root.px
                        controller: frame.controller
                        searchable: false
                        battery: true
                        glassColor: ink.glass
                        pixelScale: canvas.unit
                        x: root.width - root.margin - width
                        y: 80 * root.px - height / 2
                    }
                    // The deeper pages' heading and optional subtitle (EclipseGamePackage/Edit/Cover/Scrape/Crop:39-42).
                    Item {
                        id: pkgTitleLine
                        objectName: "packageTitleLine"
                        visible: root.inHeaderSubpage
                        x: root.margin
                        y: 150 * root.px
                        width: 1100 * root.px
                        height: pkgTitle.lineCount * pkgTitle.lineHeight * root.px
                    }
                    Line {
                        id: pkgTitle
                        objectName: "packageTitle"
                        visible: root.inHeaderSubpage
                        unit: root.px
                        x: root.margin
                        y: pkgTitleLine.y + (pkgTitleLine.height - height) / 2
                        width: 1100 * root.px
                        text: Page.subpageTitle
                        color: Tokens.textStrong
                        elide: Text.ElideRight
                        lineHeightMode: Text.FixedHeight
                        lineHeight: 1.06 * 56
                        size: 56
                        weight: Tokens.weightTitle
                        spacing: -0.01 * 56
                    }
                    Line {
                        id: pkgSubtitle
                        objectName: "packageSubtitle"
                        visible: root.inHeaderSubpage && Page.subpageSubtitle.length > 0
                        unit: root.px
                        x: root.margin
                        y: pkgTitleLine.y + pkgTitleLine.height + 10 * root.px
                        width: 1100 * root.px
                        text: Page.subpageSubtitle
                        color: ink.emptyAbout
                        wrapMode: Text.WordWrap
                        lineHeightMode: Text.FixedHeight
                        lineHeight: 1.45 * 22
                        size: 22
                        weight: Tokens.weightSentence
                    }
                    // The game's column: its kicker, its name, its line and the board's buttons (EclipseGame:42-53), 22 apart.
                    Line {
                        id: kicker
                        objectName: "gameKicker"
                        visible: !root.inSubpage
                        unit: root.px
                        x: root.margin
                        y: 190 * root.px
                        width: 900 * root.px
                        text: Page.kicker
                        boardLine: true
                        capitalization: Font.AllUppercase
                        color: ink.rose
                        elide: Text.ElideRight
                        size: 20
                        weight: Font.DemiBold
                        spacing: 0.22 * 20
                    }
                    // The title's line box, the board's line height of 1.02 (EclipseGame:44): the next line follows it.
                    Item {
                        id: titleLine
                        objectName: "gameTitleLine"
                        visible: !root.inSubpage
                        x: root.margin
                        y: kicker.y + kicker.height + 22 * root.px
                        width: 900 * root.px
                        height: title.lineCount * title.lineHeight * root.px
                    }
                    Line {
                        id: title
                        objectName: "gameTitle"
                        visible: !root.inSubpage
                        unit: root.px
                        x: root.margin
                        // The face's own height about the line box's middle, as CSS spreads a line's leading equally above and below.
                        y: titleLine.y + (titleLine.height - height) / 2
                        width: 900 * root.px
                        text: Page.title
                        color: Tokens.textStrong
                        elide: Text.ElideRight
                        lineHeightMode: Text.FixedHeight
                        lineHeight: 1.02 * 104
                        size: 104
                        weight: Tokens.weightTitle
                        spacing: -0.02 * 104
                    }
                    Line {
                        id: facts
                        objectName: "gameLine"
                        visible: !root.inSubpage
                        unit: root.px
                        x: root.margin
                        y: titleLine.y + titleLine.height + 22 * root.px
                        width: 900 * root.px
                        text: Page.line
                        boardLine: true
                        color: Tokens.muted
                        elide: Text.ElideRight
                        size: 24
                        weight: Tokens.weightSentence
                    }
                    Row {
                        objectName: "gamePills"
                        visible: !root.inSubpage
                        x: root.margin
                        // The column's gap and the buttons' own margin above them (EclipseGame:42, 46).
                        y: facts.y + facts.height + (22 + 10) * root.px
                        spacing: 16 * root.px
                        Repeater {
                            id: pills
                            model: Page.pills
                            delegate: PillSlot {}
                        }
                    }
                    // Upper-right Cover & Facts Card (visible: !root.inSubpage, x: 1148, y: 150, w: 676).
                    Rectangle {
                        id: heroCard
                        objectName: "gameHeroCard"
                        visible: !root.inSubpage
                        x: 1148 * root.px
                        y: 150 * root.px
                        width: 676 * root.px
                        readonly property string coverSource: Page.cover.length > 0 ? Page.cover : Page.picture
                        readonly property real sidePad: 24 * root.px
                        readonly property real topPad: 22 * root.px
                        height: Math.max((heroArtBox.visible ? 244 : 108) * root.px + 2 * heroCard.topPad, heroFacts.height + 2 * heroCard.topPad)
                        radius: 22 * root.px
                        color: Tokens.inputSurface
                        border.width: 1 * root.px
                        border.color: ink.plateEdge
                        Item {
                            id: heroArtBox
                            objectName: "gameHeroCover"
                            visible: heroCard.coverSource.length > 0
                            x: heroCard.sidePad
                            y: heroCard.topPad
                            width: 196 * root.px
                            height: 244 * root.px
                            RoundPicture {
                                id: heroArt
                                objectName: "gameHeroPicture"
                                anchors.fill: parent
                                source: heroCard.coverSource
                                radius: 16 * root.px
                                crop: true
                                pixelScale: canvas.unit
                            }
                        }
                        Column {
                            id: heroFacts
                            objectName: "gameHeroFacts"
                            x: heroArtBox.visible ? heroArtBox.x + heroArtBox.width + 22 * root.px : heroCard.sidePad
                            y: (heroCard.height - height) / 2
                            width: heroCard.width - x - heroCard.sidePad
                            spacing: 8 * root.px
                            Line {
                                objectName: "gameHeroKicker"
                                unit: root.px
                                width: parent.width
                                visible: Page.kicker.length > 0
                                text: Page.kicker
                                boardLine: true
                                capitalization: Font.AllUppercase
                                color: ink.rose
                                size: 13
                                weight: Font.DemiBold
                                spacing: 0.16 * 13
                                elide: Text.ElideRight
                            }
                            Line {
                                objectName: "gameHeroTitle"
                                unit: root.px
                                width: parent.width
                                visible: Page.title.length > 0
                                text: Page.title
                                color: Tokens.textStrong
                                size: 28
                                weight: Tokens.weightTitle
                                maximumLineCount: 2
                                wrapMode: Text.WordWrap
                                lineHeightMode: Text.FixedHeight
                                lineHeight: 32
                                elide: Text.ElideRight
                            }
                            Line {
                                objectName: "gameHeroMeta"
                                unit: root.px
                                width: parent.width
                                visible: Page.facts.length > 0
                                text: Page.facts
                                color: Tokens.muted
                                size: 18
                                weight: Tokens.weightSentence
                                maximumLineCount: 4
                                wrapMode: Text.WordWrap
                                lineHeightMode: Text.FixedHeight
                                lineHeight: 25
                                elide: Text.ElideRight
                            }
                        }
                    }
                    Tabs {
                        visible: !root.inSubpage
                        unit: root.px
                        controller: frame.controller
                        tabs: root.inSubpage ? [] : Page.tabs
                        current: Page.tab
                        currentInk: ink.current
                        x: root.margin
                        y: 656 * root.px
                        onChosen: (index) => Actions.pageTab(index)
                    }
                    // Overview: description and screenshot (EclipseGameOverview:64-67).
                    Item {
                        id: overviewBox
                        objectName: "gameOverview"
                        visible: !root.inSubpage && Page.tab === Page.Overview
                        x: root.margin
                        y: 748 * root.px
                        width: (900 + 64 + 560) * root.px
                        height: 315 * root.px
                        onVisibleChanged: if (overviewBox.visible) (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus()
                        Line {
                            objectName: "gameOverviewWords"
                            unit: root.px
                            width: 900 * root.px
                            text: Page.words
                            color: Page.hasDescription ? ink.help : Tokens.muted
                            wrapMode: Text.WordWrap
                            maximumLineCount: 10
                            elide: Text.ElideRight
                            lineHeightMode: Text.FixedHeight
                            lineHeight: 1.5 * 24
                            size: 24
                            weight: Tokens.weightSentence
                        }
                        Item {
                            objectName: "gameOverviewArt"
                            visible: Page.picture.length > 0
                            x: (900 + 64) * root.px
                            width: 560 * root.px
                            height: 315 * root.px
                            Glow {
                                objectName: "gameOverviewShadow"
                                anchors.fill: parent
                                radius: 18 * root.px
                                color: own.cardShadow
                                blur: 24 * root.px
                                offsetY: 8 * root.px
                                pixelScale: canvas.unit
                            }
                            RoundPicture {
                                objectName: "gameOverviewPicture"
                                anchors.fill: parent
                                source: Page.picture
                                radius: 18 * root.px
                                crop: true
                                pixelScale: canvas.unit
                            }
                        }
                    }
                    // The cards at the board's size between the margins, the row following the focus sideways; a card the row has carried past either end is not drawn (R-20261123-33).
                    ListView {
                        id: cards
                        objectName: "gameCards"
                        visible: !root.inSubpage && Page.tab === Page.States
                        x: root.margin
                        y: 748 * root.px
                        width: root.width - 2 * root.margin
                        height: 265 * root.px
                        orientation: ListView.Horizontal
                        spacing: 24 * root.px
                        interactive: false
                        keyNavigationEnabled: false
                        model: Page.cards
                        delegate: CardSlot {}
                        onVisibleChanged: if (cards.visible) ((cards.currentItem as CardSlot)?.walk ?? (cards.itemAtIndex(0) as CardSlot)?.walk ?? (pills.itemAt(0) as PillSlot)?.walk)?.forceActiveFocus()
                    }
                    // The words beside the empty card where the game has no state (EclipseGameNoState:69): 60 down, the row's gap and its own 24 after the card.
                    Line {
                        objectName: "gameNote"
                        unit: root.px
                        visible: !root.inSubpage && Page.tab === Page.States && Page.note.length > 0
                        x: cards.x + (316 + 24 + 24) * root.px
                        y: cards.y + 60 * root.px
                        width: 620 * root.px
                        text: Page.note
                        color: Tokens.muted
                        wrapMode: Text.WordWrap
                        lineHeightMode: Text.FixedHeight
                        lineHeight: 1.45 * 22
                        size: 22
                        weight: Tokens.weightSentence
                    }
                    // Media: horizontal tiles with top-right position count, or the muted note when empty (EclipseGameMedia:64-72).
                    Line {
                        objectName: "gameMediaPlace"
                        unit: root.px
                        visible: mediaList.visible && mediaList.activeFocus && Page.rowPlace.length > 0
                        boardLine: true
                        x: root.width - root.margin - width
                        y: 662 * root.px
                        text: Page.rowPlace
                        color: Tokens.muted
                        size: 20
                        weight: Font.Normal
                    }
                    ListView {
                        id: mediaList
                        objectName: "gameMedia"
                        visible: !root.inSubpage && Page.tab === Page.Media && Page.mediaNote.length === 0
                        x: root.margin
                        y: 748 * root.px
                        width: root.width - 2 * root.margin
                        height: 239 * root.px
                        orientation: ListView.Horizontal
                        spacing: 24 * root.px
                        interactive: false
                        keyNavigationEnabled: false
                        model: Page.mediaTiles
                        delegate: MediaSlot {}
                        onVisibleChanged: if (mediaList.visible) ((mediaList.currentItem as MediaSlot)?.walk ?? (mediaList.itemAtIndex(0) as MediaSlot)?.walk ?? (pills.itemAt(0) as PillSlot)?.walk)?.forceActiveFocus()
                    }
                    Line {
                        objectName: "gameMediaNote"
                        unit: root.px
                        visible: !root.inSubpage && Page.tab === Page.Media && Page.mediaNote.length > 0
                        x: root.margin
                        y: 748 * root.px
                        width: 900 * root.px
                        text: Page.mediaNote
                        color: Tokens.muted
                        wrapMode: Text.WordWrap
                        lineHeightMode: Text.FixedHeight
                        lineHeight: 1.5 * 24
                        size: 24
                        weight: Tokens.weightSentence
                    }
                    // Details, Options, Cheats and Package rows (EclipseGameDetails:64, EclipseGameOptions:64, EclipseGameCheats:64, EclipseGamePackage:41).
                    FocusScope {
                        id: rowsBox
                        objectName: "gameRows"
                        visible: root.inPackage || (!root.inSubpage && (Page.tab === Page.Details || Page.tab === Page.Options || Page.tab === Page.Cheats))
                        x: root.margin
                        y: (root.inPackage ? 300 : 748) * root.px
                        width: 900 * root.px
                        height: Math.min(boardRows.contentHeight, 350 * root.px)
                        onVisibleChanged: {
                            if (!rowsBox.visible)
                                return
                            const at = root.inPackage ? Math.min(Math.max(Page.focusedContent, 0), Math.max(boardRows.count - 1, 0)) : 0
                            boardRows.positionViewAtIndex(at, ListView.Contain)
                            boardRows.currentIndex = at
                            const item = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(at) as PageRowSlot)?.rowItem ?? (pills.itemAt(0) as PillSlot)?.walk
                            item?.forceActiveFocus()
                        }
                        ListView {
                            id: boardRows
                            anchors.fill: parent
                            orientation: ListView.Vertical
                            spacing: 10 * root.px
                            interactive: false
                            keyNavigationEnabled: false
                            boundsBehavior: Flickable.StopAtBounds
                            model: rowsBox.visible ? Page.boardRows : null
                            delegate: PageRowSlot {}
                        }
                    }
                    Column {
                        objectName: "gameAside"
                        visible: !root.inSubpage && rowsBox.visible && rowsBox.activeFocus && Page.rowPlace.length > 0
                        x: 1060 * root.px
                        y: 752 * root.px
                        width: 764 * root.px
                        spacing: 14 * root.px
                        Line {
                            objectName: "gameAsideWords"
                            unit: root.px
                            width: parent.width
                            text: Page.rowSentence
                            color: ink.emptyAbout
                            wrapMode: Text.WordWrap
                            lineHeightMode: Text.FixedHeight
                            lineHeight: 1.45 * 22
                            size: 22
                            weight: Tokens.weightSentence
                        }
                        Line {
                            objectName: "gameAsidePlace"
                            unit: root.px
                            boardLine: true
                            text: Page.rowPlace
                            color: Tokens.muted
                            size: 20
                            weight: Font.Normal
                        }
                    }
                    // Edit Details: two 840px columns of 9 rows each (EclipseGameEdit:43-68, R-20261116-32).
                    FocusScope {
                        id: editBox
                        objectName: "editRows"
                        visible: root.inEdit
                        x: root.margin
                        y: 268 * root.px
                        width: 1728 * root.px
                        height: 638 * root.px
                        onVisibleChanged: if (editBox.visible) ((editRows.itemAt(Page.focusedContent) as EditRowSlot)?.rowItem ?? (editRows.itemAt(0) as EditRowSlot)?.rowItem)?.forceActiveFocus()
                        Repeater {
                            id: editRows
                            model: root.inEdit ? Page.boardRows : null
                            delegate: EditRowSlot {}
                        }
                    }
                    // Cover Art: 4-column grid on left and 584px preview aside on right (EclipseGameCover:43-57).
                    FocusScope {
                        id: coverBox
                        objectName: "coverBox"
                        visible: root.inCover
                        anchors.fill: parent
                        onVisibleChanged: if (coverBox.visible) ((coverGrid.currentItem as CoverSlot)?.walk ?? (coverGrid.itemAtIndex(0) as CoverSlot)?.walk)?.forceActiveFocus()
                        GridView {
                            id: coverGrid
                            objectName: "coverGrid"
                            x: root.margin
                            y: 280 * root.px
                            width: 1080 * root.px
                            height: 731 * root.px
                            cellWidth: 270 * root.px
                            cellHeight: 365.4 * root.px
                            interactive: false
                            keyNavigationEnabled: false
                            boundsBehavior: Flickable.StopAtBounds
                            model: root.inCover ? Page.rows : null
                            delegate: CoverSlot {}
                        }
                        Rectangle {
                            id: coverAside
                            objectName: "coverAside"
                            x: 1240 * root.px
                            y: 268 * root.px
                            width: 584 * root.px
                            height: coverAsideCol.height + 72 * root.px
                            radius: 28 * root.px
                            color: Tokens.inputSurface
                            border.width: root.px
                            border.color: ink.plateEdge
                            Column {
                                id: coverAsideCol
                                x: 36 * root.px
                                y: 36 * root.px
                                width: 512 * root.px
                                spacing: 22 * root.px
                                Item {
                                    objectName: "coverPreviewBox"
                                    width: 512 * root.px
                                    height: 440 * root.px
                                    Rectangle {
                                        anchors.fill: parent
                                        radius: 18 * root.px
                                        color: Tokens.surfaceRaised
                                    }
                                    RoundPicture {
                                        objectName: "coverPreviewPicture"
                                        anchors.fill: parent
                                        visible: ((coverGrid.currentItem as CoverSlot)?.picture ?? "").length > 0
                                        source: (coverGrid.currentItem as CoverSlot)?.picture ?? ""
                                        radius: 18 * root.px
                                        crop: true
                                        pixelScale: canvas.unit
                                    }
                                }
                                Line {
                                    objectName: "coverPreviewTitle"
                                    unit: root.px
                                    width: 512 * root.px
                                    boardLine: true
                                    text: (coverGrid.currentItem as CoverSlot)?.label ?? ""
                                    color: Tokens.textStrong
                                    size: 30
                                    weight: Tokens.weightTitle
                                    wrapMode: Text.WordWrap
                                }
                                Line {
                                    objectName: "coverPreviewSubtitle"
                                    unit: root.px
                                    width: 512 * root.px
                                    boardLine: true
                                    text: Page.rowSentence
                                    color: Tokens.muted
                                    size: 20
                                    weight: Font.Normal
                                    elide: Text.ElideRight
                                }
                            }
                        }
                    }
                    // Scrape This Game: centered 960px dialog card with intro/status and match/action pills (EclipseGameScrape:43-53).
                    FocusScope {
                        id: scrapeBox
                        objectName: "scrapeBox"
                        visible: root.inScrape
                        anchors.fill: parent
                        onVisibleChanged: if (scrapeBox.visible) ((scrapeList.currentItem as ScrapeSlot)?.rowItem ?? (scrapeList.itemAtIndex(0) as ScrapeSlot)?.rowItem)?.forceActiveFocus()
                        Glow {
                            objectName: "scrapeShadow"
                            x: scrapeCard.x
                            y: scrapeCard.y
                            width: scrapeCard.width
                            height: scrapeCard.height
                            radius: 32 * root.px
                            color: own.dialogShadow
                            blur: 90 * root.px
                            offsetY: 30 * root.px
                            pixelScale: canvas.unit
                        }
                        Rectangle {
                            id: scrapeCard
                            objectName: "scrapeCard"
                            x: 480 * root.px
                            y: 300 * root.px
                            width: 960 * root.px
                            height: Math.min(760 * root.px, scrapeIntro.height + 22 * root.px + scrapeList.contentHeight + 88 * root.px)
                            radius: 32 * root.px
                            color: Tokens.inputSurface
                            border.width: root.px
                            border.color: ink.plateEdge
                            Line {
                                id: scrapeIntro
                                objectName: "scrapeIntro"
                                unit: root.px
                                x: 44 * root.px
                                y: 44 * root.px
                                width: 872 * root.px
                                text: Page.rowSentence
                                color: ink.help
                                wrapMode: Text.WordWrap
                                lineHeightMode: Text.FixedHeight
                                lineHeight: 1.45 * 24
                                size: 24
                                weight: Font.Normal
                            }
                            ListView {
                                id: scrapeList
                                objectName: "scrapeList"
                                x: 44 * root.px
                                y: scrapeIntro.y + scrapeIntro.height + 22 * root.px
                                width: 872 * root.px
                                height: Math.max(0, scrapeCard.height - y - 44 * root.px)
                                spacing: 6 * root.px
                                interactive: false
                                keyNavigationEnabled: false
                                boundsBehavior: Flickable.StopAtBounds
                                model: root.inScrape ? Page.boardRows : null
                                delegate: ScrapeSlot {}
                                onCountChanged: if (root.inScrape && count > 0 && !trail.activeFocus) {
                                    scrapeList.positionViewAtIndex(0, ListView.Contain)
                                    scrapeList.currentIndex = 0
                                    const item = (scrapeList.currentItem as ScrapeSlot)?.rowItem ?? (scrapeList.itemAtIndex(0) as ScrapeSlot)?.rowItem
                                    item?.forceActiveFocus()
                                }
                            }
                        }
                    }
                    // Media Viewer: picture, clip or manual on black ground with bottom caption bar (EclipseGameViewer:20-37).
                    FocusScope {
                        id: viewerScope
                        objectName: "viewerScope"
                        visible: root.inViewer
                        anchors.fill: parent
                        onVisibleChanged: if (viewerScope.visible) viewerScope.forceActiveFocus()
                        Keys.onPressed: (event) => event.accepted = Actions.pageViewKey(event.key, event.modifiers)
                        Item {
                            id: viewerArea
                            objectName: "viewerArea"
                            x: 0
                            y: 34 * root.px
                            width: root.width
                            height: 990 * root.px
                            Image {
                                objectName: "viewerPicture"
                                anchors.fill: parent
                                visible: Page.viewKind === 0
                                source: root.inViewer && Page.viewKind === 0 ? Page.viewSource : ""
                                fillMode: Image.PreserveAspectFit
                                asynchronous: true
                            }
                            Loader {
                                id: viewerManualLoader
                                anchors.fill: parent
                                active: root.inViewer && Page.viewKind === 2
                                sourceComponent: PdfPage {
                                    id: viewerManual
                                    objectName: "viewerManual"
                                    readonly property real fit: pageSize.width > 0 && pageSize.height > 0 ? Math.min(viewerManualLoader.width / pageSize.width, viewerManualLoader.height / pageSize.height) : 0
                                    readonly property real zoom: 1 + Page.viewZoom * 0.5
                                    source: Page.viewManual
                                    page: Page.viewPage
                                    width: pageSize.width * fit * zoom
                                    height: pageSize.height * fit * zoom
                                    x: (viewerManualLoader.width - width) / 2 - Page.viewPanX * viewerManualLoader.width / 8
                                    y: (viewerManualLoader.height - height) / 2 - Page.viewPanY * viewerManualLoader.width / 8
                                    onPagesChanged: Actions.pageViewPages(viewerManual.pages)
                                }
                            }
                        }
                        Rectangle {
                            id: viewerBar
                            objectName: "viewerBar"
                            x: 0
                            y: 1068 * root.px
                            width: root.width
                            height: 132 * root.px
                            color: Tokens.surface
                            Rectangle {
                                objectName: "viewerRule"
                                x: 0
                                y: 0
                                width: parent.width
                                height: 2 * root.px
                                color: Tokens.rule
                            }
                        }
                        Line {
                            objectName: "viewerCaption"
                            unit: root.px
                            x: root.margin
                            y: 1112 * root.px - height
                            width: root.width - 2 * root.margin
                            boardLine: true
                            text: Page.viewCaption
                            color: Tokens.textStrong
                            size: 22
                            weight: Font.Normal
                            elide: Text.ElideRight
                        }
                    }
                    // This game's crop: 5 rows on left (700px) and 904x678 live shaded preview on right (EclipseGameCrop:43-65).
                    FocusScope {
                        id: cropBox
                        objectName: "cropBox"
                        visible: root.inCrop
                        anchors.fill: parent
                        onVisibleChanged: if (cropBox.visible) ((cropRows.currentItem as CropRowSlot)?.rowItem ?? (cropRows.itemAtIndex(0) as CropRowSlot)?.rowItem)?.forceActiveFocus()
                        ListView {
                            id: cropRows
                            objectName: "cropRows"
                            x: root.margin
                            y: 330 * root.px
                            width: 700 * root.px
                            height: 350 * root.px
                            orientation: ListView.Vertical
                            spacing: 10 * root.px
                            interactive: false
                            keyNavigationEnabled: false
                            boundsBehavior: Flickable.StopAtBounds
                            model: root.inCrop ? Page.boardRows : null
                            delegate: CropRowSlot {}
                        }
                        Item {
                            id: cropPreview
                            objectName: "cropPreview"
                            x: 920 * root.px
                            y: 268 * root.px
                            width: 904 * root.px
                            height: 678 * root.px
                            Rectangle {
                                objectName: "cropFrame"
                                anchors.fill: parent
                                radius: 18 * root.px
                                color: own.cropFrame
                            }
                            RoundPicture {
                                objectName: "cropPicture"
                                anchors.fill: parent
                                visible: Page.cropPicture.length > 0
                                source: Page.cropPicture
                                radius: 18 * root.px
                                crop: true
                                pixelScale: canvas.unit
                            }
                            Item {
                                objectName: "cropShadeTop"
                                visible: Page.cropTop > 0
                                x: 0
                                y: 0
                                width: parent.width
                                height: parent.height * Page.cropTop
                                clip: true
                                Rectangle {
                                    width: parent.width
                                    height: Math.max(parent.height + 18 * root.px, 36 * root.px)
                                    radius: 18 * root.px
                                    color: own.cropShade
                                }
                            }
                            Rectangle {
                                objectName: "cropLineTop"
                                visible: Page.cropTop > 0
                                x: 0
                                y: parent.height * Page.cropTop
                                width: parent.width
                                height: 2 * root.px
                                color: Page.cropEdge === 2 ? Tokens.accent : own.cropLineDim
                            }
                            Item {
                                objectName: "cropShadeBottom"
                                visible: Page.cropBottom > 0
                                x: 0
                                y: parent.height - height
                                width: parent.width
                                height: parent.height * Page.cropBottom
                                clip: true
                                Rectangle {
                                    y: parent.height - height
                                    width: parent.width
                                    height: Math.max(parent.height + 18 * root.px, 36 * root.px)
                                    radius: 18 * root.px
                                    color: own.cropShade
                                }
                            }
                            Rectangle {
                                objectName: "cropLineBottom"
                                visible: Page.cropBottom > 0
                                x: 0
                                y: parent.height - parent.height * Page.cropBottom - height
                                width: parent.width
                                height: 2 * root.px
                                color: Page.cropEdge === 3 ? Tokens.accent : own.cropLineDim
                            }
                            Item {
                                objectName: "cropShadeLeft"
                                visible: Page.cropLeft > 0
                                x: 0
                                y: 0
                                width: parent.width * Page.cropLeft
                                height: parent.height
                                clip: true
                                Rectangle {
                                    width: Math.max(parent.width + 18 * root.px, 36 * root.px)
                                    height: parent.height
                                    radius: 18 * root.px
                                    color: own.cropShade
                                }
                            }
                            Rectangle {
                                objectName: "cropLineLeft"
                                visible: Page.cropLeft > 0
                                x: parent.width * Page.cropLeft
                                y: 0
                                width: 2 * root.px
                                height: parent.height
                                color: Page.cropEdge === 0 ? Tokens.accent : own.cropLineDim
                            }
                            Item {
                                objectName: "cropShadeRight"
                                visible: Page.cropRight > 0
                                x: parent.width - width
                                y: 0
                                width: parent.width * Page.cropRight
                                height: parent.height
                                clip: true
                                Rectangle {
                                    x: parent.width - width
                                    width: Math.max(parent.width + 18 * root.px, 36 * root.px)
                                    height: parent.height
                                    radius: 18 * root.px
                                    color: own.cropShade
                                }
                            }
                            Rectangle {
                                objectName: "cropLineRight"
                                visible: Page.cropRight > 0
                                x: parent.width - parent.width * Page.cropRight - width
                                y: 0
                                width: 2 * root.px
                                height: parent.height
                                color: Page.cropEdge === 1 ? Tokens.accent : own.cropLineDim
                            }
                            Rectangle {
                                objectName: "cropBorder"
                                anchors.fill: parent
                                radius: 18 * root.px
                                color: "transparent"
                                border.width: root.px
                                border.color: ink.plateEdge
                            }
                        }
                        Line {
                            objectName: "cropCaption"
                            unit: root.px
                            x: 920 * root.px
                            y: 966 * root.px
                            width: 904 * root.px
                            boardLine: true
                            text: Page.rowSentence
                            color: Tokens.muted
                            size: 20
                            weight: Font.Normal
                            elide: Text.ElideRight
                        }
                    }
                    HelpLine {
                        id: helpLine
                        unit: root.px
                        controller: frame.controller
                        model: root.helpEntries
                        ink: ink.help
                        // The foot's middle: 40 above the board's foot, its tallest glyph 38 (EclipseGame:86-90).
                        middle: 1141
                        // Right side: shoulders' Section on the main page, or position count on deeper pages.
                        Item {
                            width: root.inSubpage ? (packagePlace.visible ? packagePlace.width : 0) : (sectionsHelp.visible ? sectionsHelp.width : 0)
                            height: root.inSubpage ? (packagePlace.visible ? packagePlace.height : 0) : (sectionsHelp.visible ? sectionsHelp.height : 0)
                            SectionsHelp {
                                id: sectionsHelp
                                visible: !root.inSubpage && Page.boardHelp >= 0
                                unit: root.px
                                controller: frame.controller
                                model: Help.entries(Help.PageSections)
                                color: ink.glass
                                borderColor: "#38F6E7D8"
                                borderWidth: 1
                                pad: 18
                                tall: 46
                                wordsColor: ink.help
                                placeWords: Page.rowPlace.length > 0 ? Page.rowPlace : (Page.tab === Page.States && cards.activeFocus && cards.count > 0 && Page.note.length === 0 ? Words.place(cards.currentIndex, cards.count) : "")
                            }
                            Line {
                                id: packagePlace
                                objectName: "packagePlace"
                                visible: Page.rowPlace.length > 0 && ((root.inPackage && rowsBox.activeFocus) || (root.inEdit && editBox.activeFocus) || (root.inCover && coverBox.activeFocus) || (root.inScrape && scrapeBox.activeFocus) || (root.inCrop && cropBox.activeFocus) || root.inViewer)
                                unit: root.px
                                boardLine: true
                                text: Page.rowPlace
                                color: Tokens.muted
                                size: 21
                            }
                        }
                    }
                    // The never-played board draws Start in the shoulders' box rather than Home's capsule (EclipseGameNoState:78).
                    Binding {
                        target: helpLine.bar
                        property: "glyph"
                        value: gameGlyph
                    }
                    Component {
                        id: gameGlyph
                        Glyph {
                            unit: root.px
                            side: helpLine.glyphSide
                            ink: ink.help
                            boxMenu: true
                            keyPairs: true
                        }
                    }
                }
            }
            // A button of the board's: Resume or Start, filled with its pad button's glyph (parts/Pill.qml), else outlined, with the heart for Favourite (parts/WordPill.qml).
            component PillSlot: Item {
                id: slot
                required property int index
                required property string label
                required property int button
                required property bool filled
                required property bool heart
                required property bool on
                required property int act
                required property int row
                readonly property Item walk: slot.filled ? lead : word
                width: slot.walk.width
                height: slot.walk.height
                Component.onCompleted: if (slot.index === 0 && !root.inSubpage && (Page.tab === Page.Overview || (Page.tab === Page.Media && Page.mediaNote.length > 0))) slot.walk.forceActiveFocus()
                Pill {
                    id: lead
                    objectName: "pill." + slot.index
                    visible: slot.filled
                    enabled: slot.filled
                    unit: root.px
                    controller: frame.controller
                    place: slot.index
                    pixelScale: canvas.unit
                    button: slot.button
                    label: slot.label
                    filled: lead.activeFocus
                    glows: true
                    playMark: !lead.activeFocus
                    fill: Tokens.surfaceRaised
                    borderWidth: 1.5 * root.px
                    outline: "#8CF6E7D8"
                    leadGlyph: lead.activeFocus
                    labelWeight: Font.DemiBold
                    glyphSide: 46
                    leadLetterSize: 22
                    labelSize: 26
                    lead: lead.activeFocus ? 14 : 26
                    trail: lead.activeFocus ? 34 : 28
                    gap: lead.activeFocus ? 16 : 12
                    // Resume uses RESUME_ACTION from placed_pills so every tab launches the resume point (R-20261123-35).
                    onActivated: Actions.pageAct(slot.act)
                    onActiveFocusChanged: if (lead.activeFocus) Actions.pageFocus(slot.act, 0)
                    Keys.onLeftPressed: (event) => { (pills.itemAt(slot.index - 1) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onRightPressed: (event) => { (pills.itemAt(slot.index + 1) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onUpPressed: (event) => { (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onDownPressed: (event) => {
                        if (Page.tab === Page.States && cards.count > 0) {
                            cards.positionViewAtIndex(0, ListView.Contain)
                            cards.currentIndex = 0
                            const first = (cards.currentItem as CardSlot)?.walk
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        } else if (Page.tab === Page.Media && mediaList.count > 0) {
                            mediaList.positionViewAtIndex(0, ListView.Contain)
                            mediaList.currentIndex = 0
                            const first = (mediaList.currentItem as MediaSlot)?.walk
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        } else if (rowsBox.visible && boardRows.count > 0) {
                            boardRows.positionViewAtIndex(0, ListView.Contain)
                            boardRows.currentIndex = 0
                            const first = (boardRows.currentItem as PageRowSlot)?.rowItem
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                }
                WordPill {
                    id: word
                    objectName: "pill." + slot.index
                    visible: !slot.filled
                    enabled: !slot.filled
                    unit: root.px
                    controller: frame.controller
                    place: slot.index
                    pixelScale: canvas.unit
                    label: slot.label
                    heart: slot.heart
                    on: slot.on
                    outline: ink.pillLine
                    onColour: ink.rose
                    onActivated: Actions.pageAct(slot.act)
                    onActiveFocusChanged: if (word.activeFocus) Actions.pageFocus(slot.act, 0)
                    Keys.onLeftPressed: (event) => { (pills.itemAt(slot.index - 1) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onRightPressed: (event) => { (pills.itemAt(slot.index + 1) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onUpPressed: (event) => { (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onDownPressed: (event) => {
                        if (Page.tab === Page.States && cards.count > 0) {
                            cards.positionViewAtIndex(0, ListView.Contain)
                            cards.currentIndex = 0
                            const first = (cards.currentItem as CardSlot)?.walk
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        } else if (Page.tab === Page.Media && mediaList.count > 0) {
                            mediaList.positionViewAtIndex(0, ListView.Contain)
                            mediaList.currentIndex = 0
                            const first = (mediaList.currentItem as MediaSlot)?.walk
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        } else if (rowsBox.visible && boardRows.count > 0) {
                            boardRows.positionViewAtIndex(0, ListView.Contain)
                            boardRows.currentIndex = 0
                            const first = (boardRows.currentItem as PageRowSlot)?.rowItem
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                }
            }
            // A card of the States tab: a state's (parts/StateCard.qml), reached and loaded, or the empty slot's, shown only (parts/EmptyCard.qml).
            component CardSlot: FocusScope {
                id: card
                required property int index
                required property string label
                required property string line
                required property string picture
                required property bool marked
                required property string badge
                required property int row
                readonly property Item walk: card.row >= 0 ? state : null
                readonly property bool resting: Page.note.length > 0
                // The focus starts on the first state's card, else, with no state, on the first button, as the cards are made for a game.
                Component.onCompleted: if (card.index === 0 && !root.inSubpage && Page.tab === Page.States && !cards.activeFocus && cards.currentIndex <= 0) { cards.currentIndex = 0; (card.row >= 0 ? state : (pills.itemAt(0) as PillSlot)?.walk)?.forceActiveFocus() }
                // The empty slot is never reached: a focus the row hands it (a state deleted beside it) goes to the card before it, else to the first button.
                onActiveFocusChanged: {
                    if (!card.activeFocus || card.row >= 0)
                        return
                    if (card.index > 0) {
                        cards.currentIndex = card.index - 1
                        cards.positionViewAtIndex(card.index - 1, ListView.Contain)
                        const prior = (cards.currentItem as CardSlot)?.walk
                        prior?.forceActiveFocus()
                    } else {
                        const first = (pills.itemAt(0) as PillSlot)?.walk
                        first?.forceActiveFocus()
                    }
                }
                width: card.row >= 0 ? state.width : empty.width
                height: card.row >= 0 ? state.height : empty.height
                // Whole inside the row's window, or not drawn, as Home's Recent row leaves a tile out.
                visible: card.x - cards.contentX >= -0.5 && card.x - cards.contentX + card.width <= cards.width + 0.5
                StateCard {
                    id: state
                    objectName: "card." + card.index
                    visible: card.row >= 0
                    enabled: card.row >= 0
                    unit: root.px
                    pixelScale: canvas.unit
                    controller: frame.controller
                    place: card.index
                    label: card.label
                    line: card.line
                    picture: card.picture
                    marked: card.marked
                    badge: card.badge
                    shadow: own.cardShadow
                    ringGlow: ink.ringGlow
                    onActivated: Actions.pageChoose(card.row)
                    onActiveFocusChanged: {
                        if (!state.activeFocus)
                            return
                        cards.currentIndex = card.index
                        cards.positionViewAtIndex(card.index, ListView.Contain)
                        Actions.pageFocus(-1, card.row)
                    }
                    focus: true
                    Keys.onLeftPressed: (event) => {
                        if (card.index > 0) {
                            cards.currentIndex = card.index - 1
                            cards.positionViewAtIndex(card.index - 1, ListView.Contain)
                            const prev = (cards.currentItem as CardSlot)?.walk
                            prev?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        if (card.index + 1 < cards.count) {
                            const prior = cards.currentIndex
                            cards.currentIndex = card.index + 1
                            const next = (cards.currentItem as CardSlot)?.walk
                            if (next) {
                                cards.positionViewAtIndex(card.index + 1, ListView.Contain)
                                next.forceActiveFocus(Qt.TabFocusReason)
                            } else {
                                cards.currentIndex = prior
                            }
                        }
                        event.accepted = true
                    }
                    Keys.onUpPressed: (event) => { (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onDownPressed: (event) => event.accepted = true
                }
                EmptyCard {
                    id: empty
                    objectName: "card." + card.index
                    visible: card.row < 0
                    unit: root.px
                    label: card.label
                    line: card.resting ? own.restLine : ink.pillLine
                    ink: card.resting ? own.restWords : ink.help
                }
            }
            // A tile of the Media tab (parts/MediaCard.qml): Left/Right walk sideways, A opens the viewer, Up goes to pills.
            component MediaSlot: FocusScope {
                id: mediaSlot
                required property int index
                required property string key
                required property string label
                required property string picture
                required property int row
                readonly property Item walk: mediaTile
                Component.onCompleted: if (mediaSlot.index === 0 && !root.inSubpage && Page.tab === Page.Media && !mediaList.activeFocus && mediaList.currentIndex <= 0) { mediaList.currentIndex = 0; mediaTile.forceActiveFocus() }
                width: mediaTile.width
                height: mediaTile.height
                visible: mediaSlot.x - mediaList.contentX >= -0.5 && mediaSlot.x - mediaList.contentX + mediaSlot.width <= mediaList.width + 0.5
                MediaCard {
                    id: mediaTile
                    objectName: "media." + mediaSlot.index
                    unit: root.px
                    pixelScale: canvas.unit
                    controller: frame.controller
                    place: mediaSlot.index
                    label: mediaSlot.label
                    picture: mediaSlot.picture
                    shadow: own.cardShadow
                    ringGlow: ink.ringGlow
                    onActivated: Actions.pageChoose(mediaSlot.row)
                    onActiveFocusChanged: {
                        if (!mediaTile.activeFocus)
                            return
                        mediaList.currentIndex = mediaSlot.index
                        mediaList.positionViewAtIndex(mediaSlot.index, ListView.Contain)
                        Actions.pageFocus(-1, mediaSlot.row)
                    }
                    focus: true
                    Keys.onLeftPressed: (event) => {
                        if (mediaSlot.index > 0) {
                            mediaList.positionViewAtIndex(mediaSlot.index - 1, ListView.Contain)
                            mediaList.currentIndex = mediaSlot.index - 1
                            const prev = (mediaList.currentItem as MediaSlot)?.walk
                            prev?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        if (mediaSlot.index + 1 < mediaList.count) {
                            mediaList.positionViewAtIndex(mediaSlot.index + 1, ListView.Contain)
                            mediaList.currentIndex = mediaSlot.index + 1
                            const next = (mediaList.currentItem as MediaSlot)?.walk
                            next?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onUpPressed: (event) => { (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason); event.accepted = true }
                    Keys.onDownPressed: (event) => event.accepted = true
                }
            }
            // A row of Details, Options, Cheats or Package (parts/PageRow.qml): scrolls in its window without clipping glow.
            component PageRowSlot: FocusScope {
                id: rowSlot
                required property int index
                required property string key
                required property string label
                required property string value
                required property bool steps
                required property bool opens
                required property bool isSwitch
                required property bool switchOn
                required property string sentence
                required property int row
                readonly property Item rowItem: pageRow
                width: boardRows.width
                height: pageRow.height
                visible: rowSlot.y - boardRows.contentY >= -0.5 && rowSlot.y - boardRows.contentY + rowSlot.height <= boardRows.height + 0.5
                Component.onCompleted: if (rowSlot.index === (Page.subpage === Page.Package ? Page.focusedContent : 0) && rowsBox.visible && !boardRows.activeFocus) { boardRows.currentIndex = rowSlot.index; pageRow.forceActiveFocus() }
                PageRow {
                    id: pageRow
                    focus: true
                    unit: root.px
                    controller: frame.controller
                    pixelScale: canvas.unit
                    width: parent.width
                    index: rowSlot.index
                    label: rowSlot.label
                    value: rowSlot.value
                    steps: rowSlot.steps
                    opens: rowSlot.opens
                    isSwitch: rowSlot.isSwitch
                    switchOn: rowSlot.switchOn
                    onActivated: rowSlot.steps ? Actions.pageStep(rowSlot.row, 1) : Actions.pageChoose(rowSlot.row)
                    onStepped: (by) => {
                        pageRow.forceActiveFocus()
                        Actions.pageStep(rowSlot.row, by)
                    }
                    onActiveFocusChanged: {
                        if (!pageRow.activeFocus)
                            return
                        boardRows.currentIndex = rowSlot.index
                        boardRows.positionViewAtIndex(rowSlot.index, ListView.Contain)
                        Actions.pageFocus(-1, rowSlot.row)
                    }
                    Keys.onUpPressed: (event) => {
                        if (rowSlot.index > 0) {
                            boardRows.positionViewAtIndex(rowSlot.index - 1, ListView.Contain)
                            boardRows.currentIndex = rowSlot.index - 1
                            const prev = (boardRows.currentItem as PageRowSlot)?.rowItem
                            prev?.forceActiveFocus(Qt.TabFocusReason)
                        } else if (root.inPackage) {
                            (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason)
                        } else {
                            (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        if (rowSlot.index + 1 < boardRows.count) {
                            boardRows.positionViewAtIndex(rowSlot.index + 1, ListView.Contain)
                            boardRows.currentIndex = rowSlot.index + 1
                            const next = (boardRows.currentItem as PageRowSlot)?.rowItem
                            next?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onLeftPressed: (event) => {
                        if (rowSlot.steps || rowSlot.isSwitch)
                            Actions.pageStep(rowSlot.row, -1)
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        if (rowSlot.steps || rowSlot.isSwitch)
                            Actions.pageStep(rowSlot.row, 1)
                        event.accepted = true
                    }
                }
            }
            // One row of Edit Details' two 9-row columns (EclipseGameEdit:43-68, R-20261116-32).
            component EditRowSlot: FocusScope {
                id: editSlot
                required property int index
                required property string key
                required property string label
                required property string value
                required property bool steps
                required property bool opens
                required property bool isSwitch
                required property bool switchOn
                required property bool editable
                required property bool separated
                required property string sentence
                required property int row
                readonly property Item rowItem: editRow
                x: (editSlot.index < 9 ? 0 : 888) * root.px
                y: (editSlot.index % 9) * 72 * root.px
                width: 840 * root.px
                height: editRow.height
                Component.onCompleted: if (editSlot.index === Math.min(Math.max(Page.focusedContent, 0), 17) && editBox.visible && !editBox.activeFocus) editRow.forceActiveFocus()
                PageRow {
                    id: editRow
                    focus: true
                    unit: root.px
                    controller: frame.controller
                    pixelScale: canvas.unit
                    width: parent.width
                    index: editSlot.index
                    label: editSlot.label
                    value: editSlot.value
                    steps: editSlot.steps
                    opens: editSlot.opens
                    isSwitch: editSlot.isSwitch
                    switchOn: editSlot.switchOn
                    editable: editSlot.editable
                    onActivated: Actions.pageChoose(editSlot.row)
                    onStepped: (by) => {
                        editRow.forceActiveFocus()
                        Actions.pageStep(editSlot.row, by)
                    }
                    onActiveFocusChanged: if (editRow.activeFocus) Actions.pageFocus(-1, editSlot.row)
                    Keys.onUpPressed: (event) => {
                        if (editSlot.index > 0) {
                            (editRows.itemAt(editSlot.index - 1) as EditRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
                        } else {
                            (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        if (editSlot.index + 1 < editRows.count)
                            (editRows.itemAt(editSlot.index + 1) as EditRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
                        event.accepted = true
                    }
                    Keys.onLeftPressed: (event) => {
                        if (editSlot.steps) {
                            Actions.pageStep(editSlot.row, -1)
                        } else if (editSlot.index >= 9) {
                            (editRows.itemAt(editSlot.index - 9) as EditRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        if (editSlot.steps) {
                            Actions.pageStep(editSlot.row, 1)
                        } else if (editSlot.index + 9 < editRows.count) {
                            (editRows.itemAt(editSlot.index + 9) as EditRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                }
            }
            // One tile of Cover Art's 4-column grid (EclipseGameCover:43-52).
            component CoverSlot: FocusScope {
                id: coverSlot
                required property int index
                required property string key
                required property string label
                required property string line
                required property string picture
                required property bool marked
                readonly property Item walk: coverCard
                width: coverGrid.cellWidth
                height: coverGrid.cellHeight
                visible: coverSlot.y - coverGrid.contentY >= -0.5 && coverSlot.y - coverGrid.contentY + coverCard.height <= coverGrid.height + 0.5
                Component.onCompleted: if (coverSlot.index === Math.min(Math.max(Page.focusedContent, 0), Math.max(coverGrid.count - 1, 0)) && coverBox.visible && !coverGrid.activeFocus) { coverGrid.currentIndex = coverSlot.index; coverCard.forceActiveFocus() }
                CoverCard {
                    id: coverCard
                    focus: true
                    unit: root.px
                    controller: frame.controller
                    pixelScale: canvas.unit
                    index: coverSlot.index
                    label: coverSlot.label
                    picture: coverSlot.picture
                    marked: coverSlot.marked
                    badge: coverSlot.line
                    shadow: own.cardShadow
                    ringGlow: ink.ringGlow
                    onActivated: Actions.pageChoose(coverSlot.index)
                    onActiveFocusChanged: {
                        if (!coverCard.activeFocus)
                            return
                        coverGrid.currentIndex = coverSlot.index
                        coverGrid.positionViewAtIndex(coverSlot.index, GridView.Contain)
                        Actions.pageFocus(-1, coverSlot.index)
                    }
                    Keys.onLeftPressed: (event) => {
                        if (coverSlot.index > 0) {
                            coverGrid.positionViewAtIndex(coverSlot.index - 1, GridView.Contain)
                            coverGrid.currentIndex = coverSlot.index - 1
                            const prev = (coverGrid.currentItem as CoverSlot)?.walk
                            prev?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        if (coverSlot.index + 1 < coverGrid.count) {
                            coverGrid.positionViewAtIndex(coverSlot.index + 1, GridView.Contain)
                            coverGrid.currentIndex = coverSlot.index + 1
                            const next = (coverGrid.currentItem as CoverSlot)?.walk
                            next?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onUpPressed: (event) => {
                        if (coverSlot.index >= 4) {
                            coverGrid.positionViewAtIndex(coverSlot.index - 4, GridView.Contain)
                            coverGrid.currentIndex = coverSlot.index - 4
                            const above = (coverGrid.currentItem as CoverSlot)?.walk
                            above?.forceActiveFocus(Qt.TabFocusReason)
                        } else {
                            (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        const target = coverSlot.index + 4
                        if (target < coverGrid.count) {
                            coverGrid.positionViewAtIndex(target, GridView.Contain)
                            coverGrid.currentIndex = target
                            const below = (coverGrid.currentItem as CoverSlot)?.walk
                            below?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                }
            }
            // One match or action row of Scrape This Game's dialog card (EclipseGameScrape:46-52).
            component ScrapeSlot: FocusScope {
                id: scrapeSlot
                required property int index
                required property string key
                required property string label
                required property bool separated
                required property int row
                readonly property Item rowItem: scrapeRow
                readonly property real topGap: scrapeSlot.separated ? 18 * root.px : 0
                width: scrapeList.width
                height: scrapeRow.height + scrapeSlot.topGap
                visible: scrapeSlot.y - scrapeList.contentY >= -0.5 && scrapeSlot.y - scrapeList.contentY + scrapeSlot.height <= scrapeList.height + 0.5
                Component.onCompleted: if (scrapeSlot.index === 0 && scrapeBox.visible && !scrapeList.activeFocus) { scrapeList.currentIndex = 0; scrapeRow.forceActiveFocus() }
                ScrapeRow {
                    id: scrapeRow
                    focus: true
                    y: scrapeSlot.topGap
                    unit: root.px
                    controller: frame.controller
                    pixelScale: canvas.unit
                    index: scrapeSlot.index
                    label: scrapeSlot.label
                    ringColor: own.matchRing
                    glowColor: own.matchGlow
                    onActivated: Actions.pageChoose(scrapeSlot.row)
                    onActiveFocusChanged: {
                        if (!scrapeRow.activeFocus)
                            return
                        scrapeList.currentIndex = scrapeSlot.index
                        scrapeList.positionViewAtIndex(scrapeSlot.index, ListView.Contain)
                        Actions.pageFocus(-1, scrapeSlot.row)
                    }
                    Keys.onUpPressed: (event) => {
                        if (scrapeSlot.index > 0) {
                            scrapeList.positionViewAtIndex(scrapeSlot.index - 1, ListView.Contain)
                            scrapeList.currentIndex = scrapeSlot.index - 1
                            const prev = (scrapeList.currentItem as ScrapeSlot)?.rowItem
                            prev?.forceActiveFocus(Qt.TabFocusReason)
                        } else {
                            (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        if (scrapeSlot.index + 1 < scrapeList.count) {
                            scrapeList.positionViewAtIndex(scrapeSlot.index + 1, ListView.Contain)
                            scrapeList.currentIndex = scrapeSlot.index + 1
                            const next = (scrapeList.currentItem as ScrapeSlot)?.rowItem
                            next?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onLeftPressed: (event) => event.accepted = true
                    Keys.onRightPressed: (event) => event.accepted = true
                }
            }
            // One row of This game's crop (EclipseGameCrop:44-54, R-20261116-33).
            component CropRowSlot: FocusScope {
                id: cropSlot
                required property int index
                required property string key
                required property string label
                required property string value
                required property bool steps
                required property bool opens
                required property bool isSwitch
                required property bool switchOn
                required property int row
                readonly property Item rowItem: cropRow
                width: cropRows.width
                height: cropRow.height
                Component.onCompleted: if (cropSlot.index === Math.min(Math.max(Page.focusedContent, 0), 4) && cropBox.visible && !cropRows.activeFocus) { cropRows.currentIndex = cropSlot.index; cropRow.forceActiveFocus() }
                PageRow {
                    id: cropRow
                    focus: true
                    unit: root.px
                    controller: frame.controller
                    pixelScale: canvas.unit
                    width: parent.width
                    index: cropSlot.index
                    label: cropSlot.label
                    value: cropSlot.value
                    steps: cropSlot.steps
                    opens: cropSlot.opens
                    isSwitch: cropSlot.isSwitch
                    switchOn: cropSlot.switchOn
                    onActivated: Actions.pageChoose(cropSlot.row)
                    onStepped: (by) => {
                        cropRow.forceActiveFocus()
                        Actions.pageStep(cropSlot.row, by)
                    }
                    onActiveFocusChanged: {
                        if (!cropRow.activeFocus)
                            return
                        cropRows.currentIndex = cropSlot.index
                        cropRows.positionViewAtIndex(cropSlot.index, ListView.Contain)
                        Actions.pageFocus(-1, cropSlot.row)
                    }
                    Keys.onUpPressed: (event) => {
                        if (cropSlot.index > 0) {
                            cropRows.positionViewAtIndex(cropSlot.index - 1, ListView.Contain)
                            cropRows.currentIndex = cropSlot.index - 1
                            const prev = (cropRows.currentItem as CropRowSlot)?.rowItem
                            prev?.forceActiveFocus(Qt.TabFocusReason)
                        } else {
                            (trail.links.itemAt(trail.links.count - 2) as Row)?.children[0]?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        if (cropSlot.index + 1 < cropRows.count) {
                            cropRows.positionViewAtIndex(cropSlot.index + 1, ListView.Contain)
                            cropRows.currentIndex = cropSlot.index + 1
                            const next = (cropRows.currentItem as CropRowSlot)?.rowItem
                            next?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onLeftPressed: (event) => {
                        Actions.pageStep(cropSlot.row, -1)
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        Actions.pageStep(cropSlot.row, 1)
                        event.accepted = true
                    }
                }
            }
        }
    }
}
