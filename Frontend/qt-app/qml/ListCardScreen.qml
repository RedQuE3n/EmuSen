pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Esde as Esde

// The List card over the grid (EmuSen_Interface.md §18.1): its actions are the grid's model's, and its jumps move the
// focus the theme's Games screen follows.
Kit.NormalizedCanvas {
    id: listCardCanvas
    objectName: "listCardCanvas"
    required property Screens screens
    required property LibraryModel library
    required property GameModel games
    required property int padFamily
    // The ES-DE theme's look and its view, whose gamelist the card acts on in place of the own grid.
    required property bool esdeLook
    required property Esde.View esdeView
    readonly property alias slide: listCard.slide
    visible: listCard.open || listCard.leaving
    CardModel {
        id: listCardModel
        objectName: "listCardModel"
        card: CardModel.List
        shown: listCardCanvas.screens.shown === Screens.List
        facts: listCardCanvas.esdeLook ? listCardCanvas.library.esdeListFacts(listCardCanvas.esdeView?.scene.shelf ?? "", listCardCanvas.esdeView?.scene.selected ?? 0) : listCardCanvas.games.listFacts
        // Under an ES-DE theme the own grid is hidden and acted on by nothing; the theme's gamelist is acted on below.
        onActed: (key) => { if (!listCardCanvas.esdeLook) listCardCanvas.games.focused = listCardCanvas.games.listAct(key) }
        onStayed: (key) => { if (!listCardCanvas.esdeLook) listCardCanvas.games.focused = listCardCanvas.games.listAct(key) }
        onClosed: listCardCanvas.screens.close()
    }
    Connections {
        target: listCardModel
        function onActed(key) { listCardCanvas.screens.act(key) }
    }
    // Under an ES-DE theme the card acts on the shown gamelist, and the view selects the entry it answers.
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
        focus: open
        padFamily: listCardCanvas.padFamily
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
            screen: listCardModel.help
        }
        onChosen: (index) => listCardModel.choose(index)
        onStepped: (index, by, held) => listCardModel.step(index, by)
        onFocused: (index) => listCardModel.select(index)
        onBacked: listCardModel.back()
        onActed: (key) => listCardCanvas.screens.act(key)
    }
}
