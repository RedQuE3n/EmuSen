pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// The Pause Card over the paused game: the card form drawing PauseModel's pages, the context pane its own drawing in the form's pane box, and what the card asks of the session carried to it (EmuSen_Interface.md §2.8, §18.5, §20; EmuSen_Qt.md §7.1 step 7, §7.3).
MenuCard {
    id: root
    required property LibraryModel library
    required property SessionModel session
    readonly property alias model: pause
    readonly property int coverRow: games.revision >= 0 ? games.rowOf(root.session.game) : -1
    signal resumed()
    signal backToLibrary()
    // Quit Game answered Yes, and Quit EmuSen.
    signal quitGame()
    signal quitAll()
    rows: pause
    title: pause.title
    breadcrumb: pause.breadcrumb
    current: pause.current
    paneDrawn: pane.model.shown
    cover: root.coverRow >= 0 ? games.coverAt(root.coverRow) : ""
    help: HelpModel {
        screen: pause.helpScreen
    }
    onOpenChanged: if (root.open) pause.open()
    onChosen: (index) => pause.choose(index)
    onStepped: (index, by, held) => pause.step(index, by, held)
    onFocused: (index) => pause.focusRow(index)
    onBacked: pause.back()
    onActed: (key) => pause.act(key)
    PauseModel {
        id: pause
        objectName: "pause"
        game: root.library.titleOf(root.session.game)
        facts: root.session.cardFacts
        onResumed: root.resumed()
        onLibrary: root.backToLibrary()
        onSlotChosen: (slot) => root.session.chooseSlot(slot)
        onSaveAsked: (slot) => root.session.saveState(slot)
        onLoadAsked: (slot) => root.session.loadState(slot)
        onRewindAsked: (frame) => root.session.rewindTo(frame)
        onRestartAsked: root.session.restart()
        onQuitGameAsked: root.quitGame()
        onQuitAsked: root.quitAll()
    }
    GameModel {
        id: games
        games: root.library.recent
    }
    CardPane {
        id: pane
        anchors.fill: parent
        pane: pause.pane
    }
}
