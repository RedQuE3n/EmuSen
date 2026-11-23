pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's game page as its board draws it (EclipseGame.dc.html, EclipseGameNoState.dc.html): the disc and the horizon, the trail, the game's words and buttons, the tabs and the States tab's cards, every measure the board's in canvas units (EmuSen_Qt.md §3.4.11).
ThemeScreen {
    id: screen
    objectName: "gameScreen"
    role: ThemeScreen.Game
    // The tabs this screen draws; the program draws the others, every deeper page and every question (1.11).
    pageSections: ["states"]
    // Where the focus starts: the first state's card, else the first button.
    focusItems: [
        root.firstCard ?? root.firstPill
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
            readonly property Item firstPill: pills.count > 0 ? (pills.itemAt(0) as PillSlot)?.walk ?? null : null
            // The cards lead with the states; with none the first card is the empty one, which is never reached.
            readonly property Item firstCard: cards.count > 0 && (cards.itemAtIndex(0) as CardSlot)?.row >= 0 ? (cards.itemAtIndex(0) as CardSlot).walk : null
            // The help in the boards' words where they draw it, else the program's for the focus.
            readonly property int helpKind: Page.boardHelp >= 0 ? Page.boardHelp : Page.help
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
                help: Help.entries(root.helpKind)
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
                        below: root.firstPill
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
                    // The cards at the board's size, the row following the focus sideways as far as the right margin; a card the row has carried past either end is not drawn (R-20261123-33).
                    ListView {
                        id: cards
                        objectName: "gameCards"
                        x: root.margin
                        y: 748 * root.px
                        width: root.width - root.margin
                        height: 265 * root.px
                        orientation: ListView.Horizontal
                        spacing: 24 * root.px
                        interactive: false
                        keyNavigationEnabled: false
                        highlightRangeMode: ListView.ApplyRange
                        preferredHighlightBegin: 0
                        preferredHighlightEnd: cards.width - root.margin
                        highlightMoveDuration: 0
                        // The focus starts on the first state's card, else on the first button.
                        focus: Page.cards.count > 0 && Page.note.length === 0
                        model: Page.cards
                        delegate: CardSlot {}
                    }
                    // The words beside the empty card where the game has no state (EclipseGameNoState:69): 60 down, the row's gap and its own 24 after the card.
                    Line {
                        objectName: "gameNote"
                        unit: root.px
                        visible: Page.note.length > 0
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
                    HelpLine {
                        unit: root.px
                        controller: frame.controller
                        model: Help.entries(root.helpKind)
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
                Pill {
                    id: lead
                    objectName: slot.filled ? "pill." + slot.index : ""
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
                    focus: slot.index === 0 && !cards.focus && slot.filled
                    glyphSide: 46
                    leadLetterSize: 22
                    labelSize: 26
                    lead: 14
                    trail: 34
                    gap: 16
                    // Resume chooses the resume point's card; the others are their actions (R-20261123-35).
                    onActivated: slot.row >= 0 ? Actions.pageChoose(slot.row) : Actions.pageAct(slot.act)
                    onActiveFocusChanged: if (lead.activeFocus) Actions.pageFocus(slot.act, 0)
                    KeyNavigation.right: (pills.itemAt(slot.index + 1) as PillSlot)?.walk ?? null
                    KeyNavigation.up: trail.last
                    KeyNavigation.down: root.firstCard
                }
                WordPill {
                    id: word
                    objectName: slot.filled ? "" : "pill." + slot.index
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
                    focus: slot.index === 0 && !cards.focus && !slot.filled
                    onActivated: slot.row >= 0 ? Actions.pageChoose(slot.row) : Actions.pageAct(slot.act)
                    onActiveFocusChanged: if (word.activeFocus) Actions.pageFocus(slot.act, 0)
                    KeyNavigation.left: (pills.itemAt(slot.index - 1) as PillSlot)?.walk ?? null
                    KeyNavigation.right: (pills.itemAt(slot.index + 1) as PillSlot)?.walk ?? null
                    KeyNavigation.up: trail.last
                    KeyNavigation.down: root.firstCard
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
                // The first card holds the row's focus whatever the view's current index after the cards are told again.
                focus: card.index === 0
                width: card.row >= 0 ? state.width : empty.width
                height: card.row >= 0 ? state.height : empty.height
                // Whole inside the row's window, or not drawn, as Home's Recent row leaves a tile out.
                visible: card.x - cards.contentX >= -0.5 && card.x - cards.contentX + card.width <= cards.width - root.margin + 0.5
                StateCard {
                    id: state
                    objectName: card.row >= 0 ? "card." + card.index : ""
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
                        Actions.pageFocus(-1, card.row)
                    }
                    focus: true
                    KeyNavigation.left: (cards.itemAtIndex(card.index - 1) as CardSlot)?.walk ?? null
                    KeyNavigation.right: (cards.itemAtIndex(card.index + 1) as CardSlot)?.walk ?? null
                    KeyNavigation.up: root.firstPill
                }
                EmptyCard {
                    id: empty
                    objectName: card.row < 0 ? "card." + card.index : ""
                    visible: card.row < 0
                    unit: root.px
                    label: card.label
                    line: card.resting ? own.restLine : ink.pillLine
                    ink: card.resting ? own.restWords : ink.help
                }
            }
        }
    }
}
