pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// The Home card over Home, opened at Power by its place or at Menu/Home by Start (EmuSen_Interface.md §2.3; board #5).
Kit.NormalizedCanvas {
    id: homeCardCanvas
    objectName: "homeCardCanvas"
    required property Screens screens
    required property LibraryModel library
    required property SessionModel session
    required property Clock clock
    required property bool gameMode
    required property int padFamily
    property string inHand
    property int battery: -1
    readonly property bool eclipse: Kit.Tokens.coronaRim > 0
    readonly property alias slide: homeCard.slide
    visible: homeCard.open || homeCard.leaving
    CardModel {
        id: homeCardModel
        objectName: "homeCardModel"
        card: CardModel.Home
        eclipse: homeCardCanvas.eclipse
        start: homeCardCanvas.screens.page
        shown: homeCardCanvas.screens.shown === Screens.HomeCard
        facts: homeCardCanvas.eclipse
            ? homeCardCanvas.library.eclipseHomeCardFacts(homeCardCanvas.session.running ? homeCardCanvas.session.game : "", homeCardCanvas.session.running ? homeCardCanvas.session.console : "", homeCardCanvas.clock.paused, homeCardCanvas.gameMode, homeCardCanvas.library.startable, homeCardCanvas.library.toolSummaries, homeCardCanvas.inHand)
            : homeCardCanvas.library.homeCardFacts(homeCardCanvas.session.running ? homeCardCanvas.session.game : "", homeCardCanvas.session.running ? homeCardCanvas.session.console : "", homeCardCanvas.clock.paused, homeCardCanvas.gameMode, homeCardCanvas.library.startable, homeCardCanvas.library.toolSummaries)
        onActed: (key) => homeCardCanvas.screens.act(key)
        onClosed: homeCardCanvas.screens.close()
    }
    MenuCard {
        id: homeCard
        anchors.fill: parent
        open: homeCardCanvas.screens.shown === Screens.HomeCard
        eclipse: homeCardCanvas.eclipse
        focus: open
        padFamily: homeCardCanvas.padFamily
        pixelScale: homeCardCanvas.unit
        battery: homeCardCanvas.battery
        clockText: homeCardCanvas.clock.shortTime
        rows: homeCardModel
        title: homeCardModel.title
        breadcrumb: homeCardModel.breadcrumb
        footer: homeCardModel.footer
        current: homeCardModel.current
        paneLines: homeCardModel.paneLines
        paneMuted: homeCardModel.paneMuted
        panePicture: homeCardModel.panePicture
        paneCaption: homeCardModel.paneCaption
        paneCore: homeCardModel.paneCore
        panePlanet: homeCardModel.panePlanet
        paneStatus: homeCardModel.paneStatus
        paneStatusOk: homeCardModel.paneStatusOk
        menu: homeCardModel.menu
        hasContinue: homeCardModel.hasContinue
        continueTitle: homeCardModel.continueTitle
        continueSubtitle: homeCardModel.continueSubtitle
        continuePicture: homeCardModel.continuePicture
        positionLine: homeCardModel.positionLine
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
