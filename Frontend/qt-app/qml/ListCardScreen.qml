pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Esde as Esde

// The List card over the grid, carrying Sort, Filter and Start Console (EmuSen_Interface.md §18.1; boards #44-#46).
Kit.NormalizedCanvas {
    id: listCardCanvas
    objectName: "listCardCanvas"
    required property Screens screens
    required property LibraryModel library
    required property GameModel games
    required property int padFamily
    required property bool esdeLook
    required property Esde.View esdeView
    property Clock clock: null
    property int battery: -1
    readonly property bool eclipse: Kit.Tokens.coronaRim > 0
    readonly property alias slide: listCard.slide
    visible: listCard.open || listCard.leaving
    CardModel {
        id: listCardModel
        objectName: "listCardModel"
        card: CardModel.List
        eclipse: listCardCanvas.eclipse
        start: listCardCanvas.screens.listPage
        shown: listCardCanvas.screens.shown === Screens.List
        facts: listCardCanvas.esdeLook ? listCardCanvas.library.esdeListFacts(listCardCanvas.esdeView?.scene.shelf ?? "", listCardCanvas.esdeView?.scene.selected ?? 0) : listCardCanvas.games.listFacts
        onActed: (key) => { if (!listCardCanvas.esdeLook) listCardCanvas.games.focused = listCardCanvas.games.listAct(key) }
        onStayed: (key) => { if (!listCardCanvas.esdeLook) listCardCanvas.games.focused = listCardCanvas.games.listAct(key) }
        onClosed: listCardCanvas.screens.close()
    }
    Connections {
        target: listCardModel
        function onActed(key) { listCardCanvas.screens.act(key) }
    }
    Connections {
        target: listCardModel
        enabled: listCardCanvas.esdeLook
        function onActed(key) { listCardCanvas.esdeView.scene.select(listCardCanvas.library.esdeListAct(listCardCanvas.esdeView.scene.shelf, key, listCardCanvas.esdeView.scene.selected)) }
        function onStayed(key) { listCardCanvas.esdeView.scene.select(listCardCanvas.library.esdeListAct(listCardCanvas.esdeView.scene.shelf, key, listCardCanvas.esdeView.scene.selected)) }
    }
    MenuCard {
        id: listCard
        anchors.fill: parent
        open: listCardCanvas.screens.shown === Screens.List
        eclipse: listCardCanvas.eclipse
        focus: open
        padFamily: listCardCanvas.padFamily
        pixelScale: listCardCanvas.unit
        battery: listCardCanvas.battery
        clockText: listCardCanvas.clock ? listCardCanvas.clock.shortTime : ""
        rows: listCardModel
        title: listCardModel.title
        breadcrumb: listCardModel.breadcrumb
        footer: listCardModel.footer
        current: listCardModel.current
        paneLines: listCardModel.paneLines
        paneMuted: listCardModel.paneMuted
        panePicture: listCardModel.panePicture
        paneCaption: listCardModel.paneCaption
        paneCore: listCardModel.paneCore
        panePlanet: listCardModel.panePlanet
        paneStatus: listCardModel.paneStatus
        paneStatusOk: listCardModel.paneStatusOk
        positionLine: listCardModel.positionLine
        help: HelpModel {
            screen: listCardModel.help
        }
        onChosen: (index) => listCardModel.choose(index)
        onStepped: (index, by, held) => listCardModel.step(index, by)
        onFocused: (index) => listCardModel.select(index)
        onBacked: listCardModel.back()
        onActed: (key) => listCardCanvas.screens.act(key)
    }
}
