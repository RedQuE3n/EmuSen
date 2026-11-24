pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's game page as its board draws it (EclipseGame.dc.html, EclipseGameNoState.dc.html): the disc and the horizon, the trail, the game's words and buttons, the tabs and the States tab's cards, every measure the board's in canvas units (EmuSen_Qt.md §3.4.12).
ThemeScreen {
    id: screen
    objectName: "gameScreen"
    role: ThemeScreen.Game
    // The tabs this screen draws; the program draws Media, Cheats, every deeper page and every question (1.15).
    pageSections: (1 << Page.Overview) | (1 << Page.States) | (1 << Page.Details) | (1 << Page.Options)
    // Where the focus starts: the tab's content (cards or rows), else the first button.
    focusItems: [
        Page.tab === Page.States
            ? ((cards.currentItem as CardSlot)?.walk ?? (cards.itemAtIndex(0) as CardSlot)?.walk ?? (pills.itemAt(0) as PillSlot)?.walk ?? null)
            : (Page.tab === Page.Details || Page.tab === Page.Options)
                ? ((boardRows.itemAt(0) as PageRowSlot)?.rowItem ?? (pills.itemAt(0) as PillSlot)?.walk ?? null)
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
            // The help in the boards' words where they draw it, else the program's for the focus.
            readonly property int helpKind: Page.boardHelp >= 0 ? Page.boardHelp : Page.help
            readonly property var helpEntries: root.helpKind === Help.PageOption ? Help.entries(root.helpKind, { defaultable: Page.defaultable }) : Help.entries(root.helpKind)
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
                anchors.fill: parent
                ground: ink.ground
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: 980 * root.px; y: -160 * root.px; width: 980 * root.px; height: 800 * root.px; color: field0.color; blur: 90 * root.px; opacity: 0.85 },
                    BackdropField { x: 1340 * root.px; y: 260 * root.px; width: 760 * root.px; height: 700 * root.px; color: field1.color; blur: 110 * root.px; opacity: 0.9 },
                    BackdropField { x: 1180 * root.px; y: 210 * root.px; width: 640 * root.px; height: 46 * root.px; corner: 40 * root.px; color: field3.color; blur: 22 * root.px; opacity: 0.55; rotation: -18 }
                ]
            }
            CoronaDisc {
                id: disc
                objectName: "gameDisc"
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
                    // The header: the mark, the trail, and the status on its glass (EclipseGame:26-40).
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
                        onChosen: (index) => index === 0 ? Actions.openSection(Page.trailSection) : Actions.openConsole(Page.console)
                        onDownward: (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason)
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
                    // The game's column: its kicker, its name, its line and the board's buttons (EclipseGame:42-53), 22 apart.
                    Line {
                        id: kicker
                        objectName: "gameKicker"
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
                        x: root.margin
                        y: kicker.y + kicker.height + 22 * root.px
                        width: 900 * root.px
                        height: title.lineCount * title.lineHeight * root.px
                    }
                    Line {
                        id: title
                        objectName: "gameTitle"
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
                        unit: root.px
                        controller: frame.controller
                        tabs: Page.tabs
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
                        visible: Page.tab === Page.Overview
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
                        visible: Page.tab === Page.States
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
                        visible: Page.tab === Page.States && Page.note.length > 0
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
                    // Details and Options rows with their aside hint (EclipseGameDetails:64-81, EclipseGameOptions:64-81).
                    FocusScope {
                        id: rowsBox
                        objectName: "gameRows"
                        visible: Page.tab === Page.Details || Page.tab === Page.Options
                        x: root.margin
                        y: 748 * root.px
                        width: 900 * root.px
                        height: rowsColumn.height
                        onVisibleChanged: if (rowsBox.visible) ((boardRows.itemAt(0) as PageRowSlot)?.rowItem ?? (pills.itemAt(0) as PillSlot)?.walk)?.forceActiveFocus()
                        Column {
                            id: rowsColumn
                            width: parent.width
                            spacing: 10 * root.px
                            Repeater {
                                id: boardRows
                                model: Page.boardRows
                                delegate: PageRowSlot {}
                            }
                        }
                    }
                    Column {
                        objectName: "gameAside"
                        visible: rowsBox.visible && rowsBox.activeFocus && Page.rowPlace.length > 0
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
                        // The shoulders' Section at the right, where the boards draw the line; the program's own line names it among its entries.
                        SectionsHelp {
                            visible: Page.boardHelp >= 0
                            unit: root.px
                            controller: frame.controller
                            model: Help.entries(Help.PageSections)
                            color: "transparent"
                            pad: 0
                            // No glass: the box is the shoulders' own, 34 high (EclipseGame:92-94).
                            tall: 34
                            wordsColor: ink.help
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
                Component.onCompleted: if (slot.index === 0 && Page.tab === Page.Overview) slot.walk.forceActiveFocus()
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
                            cards.currentIndex = 0
                            cards.positionViewAtIndex(0, ListView.Contain)
                            const first = (cards.currentItem as CardSlot)?.walk
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        } else if ((Page.tab === Page.Details || Page.tab === Page.Options) && boardRows.count > 0) {
                            (boardRows.itemAt(0) as PageRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
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
                            cards.currentIndex = 0
                            cards.positionViewAtIndex(0, ListView.Contain)
                            const first = (cards.currentItem as CardSlot)?.walk
                            first?.forceActiveFocus(Qt.TabFocusReason)
                        } else if ((Page.tab === Page.Details || Page.tab === Page.Options) && boardRows.count > 0) {
                            (boardRows.itemAt(0) as PageRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
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
                Component.onCompleted: if (card.index === 0 && Page.tab === Page.States && !cards.activeFocus && cards.currentIndex <= 0) { cards.currentIndex = 0; (card.row >= 0 ? state : (pills.itemAt(0) as PillSlot)?.walk)?.forceActiveFocus() }
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
            // A row of Details or Options (parts/PageRow.qml): Left/Right step, A steps or opens, Up from row 0 goes to the pills.
            component PageRowSlot: Item {
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
                width: rowsColumn.width
                height: pageRow.height
                Component.onCompleted: if (rowSlot.index === 0 && (Page.tab === Page.Details || Page.tab === Page.Options)) pageRow.forceActiveFocus()
                PageRow {
                    id: pageRow
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
                    onActiveFocusChanged: if (pageRow.activeFocus) Actions.pageFocus(-1, rowSlot.row)
                    Keys.onUpPressed: (event) => {
                        if (rowSlot.index > 0) {
                            (boardRows.itemAt(rowSlot.index - 1) as PageRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
                        } else {
                            (pills.itemAt(0) as PillSlot)?.walk?.forceActiveFocus(Qt.TabFocusReason)
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        if (rowSlot.index + 1 < boardRows.count)
                            (boardRows.itemAt(rowSlot.index + 1) as PageRowSlot)?.rowItem?.forceActiveFocus(Qt.TabFocusReason)
                        event.accepted = true
                    }
                    Keys.onLeftPressed: (event) => {
                        if (rowSlot.steps)
                            Actions.pageStep(rowSlot.row, -1)
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        if (rowSlot.steps)
                            Actions.pageStep(rowSlot.row, 1)
                        event.accepted = true
                    }
                }
            }
        }
    }
}
