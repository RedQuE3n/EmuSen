pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
import EmuSen.Models

// A row's games under the night sky, the Grid canvas in canvas units: the heading with the console's planet, the cards in five columns, the focused game's words beneath (EmuSen_Interface.md §24.2; EmuSen_Qt.md §7.1).
FocusScope {
    id: root
    objectName: "grid"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    property string key
    // Canvas units to pixels, for the covers' decoded size.
    property real pixelScale: 1
    // The row the grid opened, a tile of Choose a console.
    property var row
    readonly property string name: root.row ? root.row.name : ""
    readonly property string heading: root.row ? root.row.heading : ""
    readonly property real hue: root.row ? root.row.hue : 0
    readonly property string world: root.row ? root.row.world : ""
    readonly property string countWords: root.row ? root.row.countWords : ""
    // The rows beside it on Choose a console, named by the shoulders.
    property var previousRow
    property var nextRow
    // The List card over the grid, and the search with the keyboard docked (EmuSen_Interface.md §18.1).
    property bool listOpen: false
    property bool searching: false
    // While searching, whether the docked keyboard holds the focus rather than the covers.
    property bool typing: false
    readonly property alias games: games
    signal closed()
    signal played(string path)
    signal acted(string key)
    signal cardClosed()
    signal searchClosed()
    // The focus taken by the keys or the covers otherwise than by the shoulders (the pointer).
    signal focusMoved(bool keys)

    // The search's words: the keyboard types them, the grid's model narrows by them and is their one owner.
    Kit.KeyboardModel {
        id: typist
        objectName: "typist"
        onTextChanged: games.search = typist.text
        onClosed: root.searchClosed()
    }
    Connections {
        target: games
        function onSearchChanged() { typist.setText(games.search, games.search.length) }
    }
    onSearchingChanged: typist.setText(games.search, games.search.length)

    Kit.Sky {
        objectName: "gridSky"
        anchors.fill: parent
        glowHue: games.count > 0 && cells.currentIndex >= 0 ? games.hueAt(cells.currentIndex) : root.hue
        glowX: 0.5
        glowY: 0.1
    }
    Kit.Frame {
        id: frame
        anchors.fill: parent
        surfaceDrawn: false
        padFamily: root.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        focus: !root.listOpen
        // Under the List card the grid is drawn dimmed and nothing on it is reached: a backdrop to the gate.
        property bool backdrop: root.listOpen
        enabled: !root.listOpen
        controller.navigationShown: false
        help: HelpModel {
            screen: root.typing ? HelpModel.Search : games.search.length > 0 ? HelpModel.GridSearched : HelpModel.Grid
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { games.back() }
            function onActionInvoked(key) { root.acted(key) }
        }
        // The keyboard's own actions, Y Space and X Done, while it holds the focus; its close is the screen's way out.
        Connections {
            target: frame.controller
            enabled: root.typing
            function onActionInvoked(key) { typist.act(key) }
        }
        Item {
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            Row {
                x: 64
                y: 40.8
                spacing: 16
                Text {
                    objectName: "gridPath"
                    y: 7.2
                    text: Words.gamesPath
                    font.letterSpacing: 2
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    lineHeight: 24
                    lineHeightMode: Text.FixedHeight
                }
                Kit.Body {
                    objectName: "gridPlanet"
                    y: 2.2
                    width: 34
                    height: 34
                    world: root.world
                    visible: root.world.length > 0
                }
                Text {
                    objectName: "gridTitle"
                    text: root.heading
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensed
                    font.weight: Font.Medium
                    font.pixelSize: 32
                    font.letterSpacing: 3
                    font.capitalization: Font.AllUppercase
                    lineHeight: 38.4
                    lineHeightMode: Text.FixedHeight
                }
                Text {
                    objectName: "gridCount"
                    y: 7.2
                    text: games.countLine.length > 0 ? games.countLine : root.countWords
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    lineHeight: 24
                    lineHeightMode: Text.FixedHeight
                }
            }
            Row {
                objectName: "gridNeighbours"
                x: root.width - 64 - width
                y: 34
                height: 52
                spacing: 12
                Kit.PadGlyph {
                    objectName: "gridPreviousShoulder"
                    anchors.verticalCenter: parent.verticalCenter
                    width: 30
                    height: 30
                    family: root.padFamily
                    button: Kit.Pad.PageUp
                }
                Text {
                    objectName: "gridPrevious"
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.previousRow ? root.previousRow.name : ""
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 18
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.previousRow !== null && root.nextRow !== null
                    text: Words.between
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 18
                }
                Text {
                    objectName: "gridNext"
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.nextRow ? root.nextRow.name : ""
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 18
                }
                Kit.PadGlyph {
                    objectName: "gridNextShoulder"
                    anchors.verticalCenter: parent.verticalCenter
                    width: 30
                    height: 30
                    family: root.padFamily
                    button: Kit.Pad.PageDown
                }
            }
            // The search's field over the one row of covers, never focused: the keyboard types into it (LunaP's GallerySearch).
            TextField {
                id: field
                objectName: "GallerySearch"
                x: 64
                y: 100
                width: 1152
                height: 44
                visible: root.searching
                focusPolicy: Qt.NoFocus
                activeFocusOnTab: false
                readOnly: true
                leftPadding: 16
                rightPadding: 16
                text: typist.text
                cursorPosition: typist.caret
                color: Kit.Tokens.textStrong
                font.family: faces.regular
                font.pixelSize: 22
                Accessible.name: Words.search
            }
            // Held by the screen's state even after the pointer moved the focus, which breaks a plain focus binding.
            Binding {
                keyboard.focus: root.typing
                cells.focus: !root.typing
            }
            Kit.Keyboard {
                id: keyboard
                objectName: "GalleryKeyboard"
                model: typist
                y: 398
                width: root.width
                visible: root.searching
                labelFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontBody })
                onActiveFocusChanged: if (root.searching && keyboard.activeFocus !== root.typing) root.focusMoved(keyboard.activeFocus)
            }
            GridView {
                id: cells
                objectName: "gridCells"
                // Sixteen units of margin about the gallery, so a lifted cover's ring is drawn whole.
                x: 142 - 16
                y: (root.searching ? 156 : 108) - 16
                width: 5 * 204 + 32
                height: (root.searching ? 1 : 2) * 264 - 24 + 32
                leftMargin: 16
                topMargin: 16
                cellWidth: 204
                cellHeight: 264
                clip: true
                keyNavigationWraps: false
                currentIndex: games.count > 0 ? 0 : -1
                highlightFollowsCurrentItem: false
                // The two rows shown move only when the focus leaves them, a whole row at a time and at once, as Mistress's gallery moves.
                property int firstRow: 0
                readonly property int focusRow: Math.floor(Math.max(0, cells.currentIndex) / 5)
                readonly property real firstRowY: cells.originY - cells.topMargin + cells.cellHeight * cells.firstRow
                onFocusRowChanged: cells.firstRow = Math.max(0, Math.min(cells.firstRow, cells.focusRow), cells.focusRow - 1)
                onFirstRowYChanged: cells.contentY = cells.firstRowY
                model: GameModel {
                    id: games
                    objectName: "gridGames"
                    games: root.library.loading ? "" : root.library.gamesOf(root.key)
                    section: root.key
                    sectionName: root.heading
                    focused: cells.currentIndex
                    startConsole: root.library.startsEmpty
                    onPassedBack: root.closed()
                }
                delegate: Kit.Walkable {
                    id: cell
                    required property int index
                    required property string path
                    required property string title
                    required property string cover
                    required property real hue
                    required property bool favourite
                    readonly property bool lit: cell.GridView.isCurrentItem && cells.activeFocus
                    controller: frame.controller
                    objectName: "gridCell." + cell.index
                    Accessible.name: cell.title
                    width: 180
                    height: 240
                    z: cell.lit ? 1 : 0
                    onActivated: root.played(cell.path)
                    onActiveFocusChanged: if (cell.activeFocus) cells.currentIndex = cell.index
                    Cover {
                        objectName: "gridCover." + cell.index
                        anchors.fill: parent
                        source: cell.cover
                        title: cell.title
                        hue: cell.hue
                        favourite: cell.favourite
                        lit: cell.lit
                        lifts: !root.searching
                        pixelScale: root.pixelScale
                    }
                }
            }
            Text {
                objectName: "gridFocusTitle"
                // Under the docked keyboard while a search is typed, as Mistress's keyboard covers them.
                visible: !root.searching
                x: 64
                y: 628
                width: root.width - 128
                text: games.count > 0 && cells.currentIndex >= 0 ? games.titleAt(cells.currentIndex) : ""
                elide: Text.ElideRight
                color: Kit.Tokens.textStrong
                font.family: faces.condensed
                font.weight: Font.Medium
                font.pixelSize: 32
                lineHeight: 38.4
                lineHeightMode: Text.FixedHeight
            }
            Text {
                objectName: "gridFocusFacts"
                // Under the docked keyboard while a search is typed, as Mistress's keyboard covers them.
                visible: !root.searching
                x: 64
                y: 670
                width: root.width - 128
                text: games.count > 0 && cells.currentIndex >= 0 ? games.factsAt(cells.currentIndex) : ""
                elide: Text.ElideRight
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: 22
                lineHeight: 26.4
                lineHeightMode: Text.FixedHeight
            }
        }
    }
    CardModel {
        id: listCardModel
        objectName: "listCardModel"
        card: CardModel.List
        shown: root.listOpen
        facts: games.listFacts
        onActed: (key) => cells.currentIndex = games.listAct(key)
        onStayed: (key) => cells.currentIndex = games.listAct(key)
        onClosed: root.cardClosed()
    }
    Connections {
        target: listCardModel
        function onActed(key) { root.acted(key) }
    }
    // The List card's own canvas name, so a test finds its slide among the program's cards.
    Item {
        objectName: "listCardCanvas"
        readonly property alias slide: listCard.slide
    }
    MenuCard {
        id: listCard
        anchors.fill: parent
        visible: listCard.open || listCard.leaving
        open: root.listOpen
        focus: root.listOpen
        padFamily: root.padFamily
        rows: listCardModel
        title: listCardModel.title
        breadcrumb: listCardModel.breadcrumb
        footer: listCardModel.footer
        current: listCardModel.current
        paneLines: listCardModel.paneLines
        paneMuted: listCardModel.paneMuted
        panePicture: listCardModel.panePicture
        paneCaption: listCardModel.paneCaption
        help: HelpModel {
            screen: HelpModel.Card
        }
        onChosen: (index) => listCardModel.choose(index)
        onStepped: (index, by, held) => listCardModel.step(index, by)
        onFocused: (index) => listCardModel.select(index)
        onBacked: listCardModel.back()
        onActed: (key) => root.acted(key)
    }
    Faces {
        id: faces
    }
}
