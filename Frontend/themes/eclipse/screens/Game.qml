pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's game page and Package inner page as their boards draw them (EclipseGame*.dc.html; EmuSen_Qt.md §3.4.12).
ThemeScreen {
    id: screen
    objectName: "gameScreen"
    role: ThemeScreen.Game
    // All six tabs and the Package deeper page; Edit Details, Cover Art, Display crop, viewer and questions stay the program's (1.16).
    pageSections: (1 << Page.Overview) | (1 << Page.States) | (1 << Page.Media) | (1 << Page.Details) | (1 << Page.Options) | (1 << Page.Cheats)
    pageSubpages: 1 << Page.Package
    // Where the focus starts: the deeper page's focused row, the tab's content (cards, media or rows), else the first button.
    focusItems: [
        Page.subpage === Page.Package
            ? ((boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(Page.focusedContent) as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem ?? null)
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
            readonly property bool inPackage: Page.subpage === Page.Package
            readonly property int shownTab: Page.tab
            // The help in the boards' words where they draw it, else the program's for the focus.
            readonly property int helpKind: Page.boardHelp >= 0 ? Page.boardHelp : Page.help
            readonly property var helpEntries: root.helpKind === Help.PageOption
                ? Help.entries(root.helpKind, { defaultable: Page.defaultable })
                : root.helpKind === Help.PageMedia
                    ? Help.entries(root.helpKind, { defaultable: Page.coverable })
                    : Help.entries(root.helpKind)
            onInPackageChanged: {
                if (root.inPackage) {
                    const at = Math.min(Math.max(Page.focusedContent, 0), Math.max(boardRows.count - 1, 0))
                    boardRows.positionViewAtIndex(at, ListView.Contain)
                    boardRows.currentIndex = at
                    const item = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(at) as PageRowSlot)?.rowItem
                    item?.forceActiveFocus()
                } else if (rowsBox.visible) {
                    boardRows.positionViewAtIndex(0, ListView.Contain)
                    boardRows.currentIndex = 0
                    const item = (boardRows.currentItem as PageRowSlot)?.rowItem ?? (boardRows.itemAtIndex(0) as PageRowSlot)?.rowItem
                    item?.forceActiveFocus()
                }
            }
            onShownTabChanged: {
                if (root.inPackage)
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
                visible: !root.inPackage
                anchors.fill: parent
                ground: ink.ground
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: 980 * root.px; y: -160 * root.px; width: 980 * root.px; height: 800 * root.px; color: field0.color; blur: 90 * root.px; opacity: 0.85 },
                    BackdropField { x: 1340 * root.px; y: 260 * root.px; width: 760 * root.px; height: 700 * root.px; color: field1.color; blur: 110 * root.px; opacity: 0.9 },
                    BackdropField { x: 1180 * root.px; y: 210 * root.px; width: 640 * root.px; height: 46 * root.px; corner: 40 * root.px; color: field3.color; blur: 22 * root.px; opacity: 0.55; rotation: -18 }
                ]
            }
            // The Package deeper page's dark ground and two top-right fields (EclipseGamePackage:17-19).
            Backdrop {
                objectName: "packageBackdrop"
                visible: root.inPackage
                anchors.fill: parent
                ground: Tokens.surface
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: 1240 * root.px; y: -420 * root.px; width: 900 * root.px; height: 720 * root.px; color: field0.color; blur: 110 * root.px; opacity: 0.5 },
                    BackdropField { x: 1500 * root.px; y: -200 * root.px; width: 700 * root.px; height: 600 * root.px; color: field1.color; blur: 120 * root.px; opacity: 0.55 }
                ]
            }
            CoronaDisc {
                id: disc
                objectName: "gameDisc"
                visible: !root.inPackage
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
                visible: !root.inPackage
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
                        unit: root.px
                        named: false
                        x: root.margin
                        y: 80 * root.px - height / 2
                    }
                    Trail {
                        id: trail
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
                            } else {
                                (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason)
                            }
                        }
                    }
                    Status {
                        objectName: "gameStatus"
                        unit: root.px
                        controller: frame.controller
                        searchable: false
                        battery: true
                        glassColor: ink.glass
                        pixelScale: canvas.unit
                        x: root.width - root.margin - width
                        y: 80 * root.px - height / 2
                    }
                    // The Package deeper page's heading and subtitle (EclipseGamePackage:37-40).
                    Item {
                        id: pkgTitleLine
                        objectName: "packageTitleLine"
                        visible: root.inPackage
                        x: root.margin
                        y: 150 * root.px
                        width: 1100 * root.px
                        height: pkgTitle.lineCount * pkgTitle.lineHeight * root.px
                    }
                    Line {
                        id: pkgTitle
                        objectName: "packageTitle"
                        visible: root.inPackage
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
                        visible: root.inPackage
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
                        visible: !root.inPackage
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
                        visible: !root.inPackage
                        x: root.margin
                        y: kicker.y + kicker.height + 22 * root.px
                        width: 900 * root.px
                        height: title.lineCount * title.lineHeight * root.px
                    }
                    Line {
                        id: title
                        objectName: "gameTitle"
                        visible: !root.inPackage
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
                        visible: !root.inPackage
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
                        visible: !root.inPackage
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
                    Tabs {
                        visible: !root.inPackage
                        unit: root.px
                        controller: frame.controller
                        tabs: root.inPackage ? [] : Page.tabs
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
                        visible: !root.inPackage && Page.tab === Page.Overview
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
                        visible: !root.inPackage && Page.tab === Page.States
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
                        visible: !root.inPackage && Page.tab === Page.States && Page.note.length > 0
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
                        visible: !root.inPackage && Page.tab === Page.Media && Page.mediaNote.length === 0
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
                        visible: !root.inPackage && Page.tab === Page.Media && Page.mediaNote.length > 0
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
                        visible: root.inPackage || Page.tab === Page.Details || Page.tab === Page.Options || Page.tab === Page.Cheats
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
                            model: Page.boardRows
                            delegate: PageRowSlot {}
                        }
                    }
                    Column {
                        objectName: "gameAside"
                        visible: !root.inPackage && rowsBox.visible && rowsBox.activeFocus && Page.rowPlace.length > 0
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
                    HelpLine {
                        id: helpLine
                        unit: root.px
                        controller: frame.controller
                        model: root.helpEntries
                        ink: ink.help
                        // The foot's middle: 40 above the board's foot, its tallest glyph 38 (EclipseGame:86-90).
                        middle: 1141
                        // Right side: shoulders' Section on the main page, or row place on the Package deeper page.
                        Item {
                            width: root.inPackage ? (packagePlace.visible ? packagePlace.width : 0) : (sectionsHelp.visible ? sectionsHelp.width : 0)
                            height: root.inPackage ? (packagePlace.visible ? packagePlace.height : 0) : (sectionsHelp.visible ? sectionsHelp.height : 0)
                            SectionsHelp {
                                id: sectionsHelp
                                visible: !root.inPackage && Page.boardHelp >= 0
                                unit: root.px
                                controller: frame.controller
                                model: Help.entries(Help.PageSections)
                                color: "transparent"
                                pad: 0
                                // No glass: the box is the shoulders' own, 34 high (EclipseGame:92-94).
                                tall: 34
                                wordsColor: ink.help
                            }
                            Line {
                                id: packagePlace
                                objectName: "packagePlace"
                                visible: root.inPackage && rowsBox.activeFocus && Page.rowPlace.length > 0
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
                Component.onCompleted: if (slot.index === 0 && !root.inPackage && (Page.tab === Page.Overview || (Page.tab === Page.Media && Page.mediaNote.length > 0))) slot.walk.forceActiveFocus()
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
                    filled: true
                    glows: false
                    glyphSide: 46
                    leadLetterSize: 22
                    labelSize: 26
                    lead: 14
                    trail: 34
                    gap: 16
                    // Resume chooses the resume point's card; the others are their actions (R-20261123-35).
                    onActivated: slot.row >= 0 ? Actions.pageChoose(slot.row) : Actions.pageAct(slot.act)
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
                    onActivated: slot.row >= 0 ? Actions.pageChoose(slot.row) : Actions.pageAct(slot.act)
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
                Component.onCompleted: if (card.index === 0 && !root.inPackage && Page.tab === Page.States && !cards.activeFocus && cards.currentIndex <= 0) { cards.currentIndex = 0; (card.row >= 0 ? state : (pills.itemAt(0) as PillSlot)?.walk)?.forceActiveFocus() }
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
                Component.onCompleted: if (mediaSlot.index === 0 && !root.inPackage && Page.tab === Page.Media && !mediaList.activeFocus && mediaList.currentIndex <= 0) { mediaList.currentIndex = 0; mediaTile.forceActiveFocus() }
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
        }
    }
}
