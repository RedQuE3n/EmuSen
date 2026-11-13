pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// The Home card over Home, opened at Power by its place (EmuSen_Interface.md §2.3; EmuSen_BigPicture.md §51.2).
Kit.NormalizedCanvas {
    id: homeCardCanvas
    objectName: "homeCardCanvas"
    required property Screens screens
    required property LibraryModel library
    required property SessionModel session
    required property Clock clock
    required property bool gameMode
    required property int padFamily
    readonly property alias slide: homeCard.slide
    visible: homeCard.open || homeCard.leaving
    CardModel {
        id: homeCardModel
        objectName: "homeCardModel"
        card: CardModel.Home
        start: homeCardCanvas.screens.page
        shown: homeCardCanvas.screens.shown === Screens.HomeCard
        // Every fact is an argument, so the card follows the library's consoles and tools as they are read.
        facts: homeCardCanvas.library.homeCardFacts(homeCardCanvas.session.running ? homeCardCanvas.session.game : "", homeCardCanvas.session.running ? homeCardCanvas.session.console : "", homeCardCanvas.clock.paused, homeCardCanvas.gameMode, homeCardCanvas.library.startable, homeCardCanvas.library.toolSummaries)
        onActed: (key) => homeCardCanvas.screens.act(key)
        onClosed: homeCardCanvas.screens.close()
    }
    MenuCard {
        id: homeCard
        anchors.fill: parent
        open: homeCardCanvas.screens.shown === Screens.HomeCard
        focus: open
        padFamily: homeCardCanvas.padFamily
        rows: homeCardModel
        title: homeCardModel.title
        breadcrumb: homeCardModel.breadcrumb
        footer: homeCardModel.footer
        current: homeCardModel.current
        paneLines: homeCardModel.paneLines
        paneMuted: homeCardModel.paneMuted
        panePicture: homeCardModel.panePicture
        paneCaption: homeCardModel.paneCaption
        help: HelpModel {
            screen: homeCardModel.help
        }
        onChosen: (index) => homeCardModel.choose(index)
        onStepped: (index, by, held) => homeCardModel.step(index, by)
        onFocused: (index) => homeCardModel.select(index)
        onBacked: homeCardModel.back()
        onActed: (key) => homeCardCanvas.screens.act(key)
    }
}
