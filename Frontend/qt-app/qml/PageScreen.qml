pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models

// A game's page with its own model, from X on Home and the grid, a theme's openPage and the Home card's Selected game
// (EmuSen_Qt.md §7.8); its media viewer, scrape and Edit Details are its own (§7.8.2).
GamePage {
    id: pageScreen
    required property Screens screens
    required property LibraryModel library
    required property string platformLibrary
    // The game in hand, whose page is shown.
    required property string inHand
    required property bool splashShown
    // The recorded ScreenScraper a test names for the scraper; empty for ScreenScraper itself.
    property alias scraperTest: pageModel.scraperTest
    // A game played, from its start or from a saved state's file; Open Folder; what failed.
    signal play(string path)
    signal playFrom(string path, string state)
    signal openFolder(string path)
    signal said(string words)
    visible: pageScreen.screens.shown === Screens.Page
    page: pageModel
    inputs: visible && !pageScreen.splashShown
    onBacked: pageScreen.screens.back()
    onActed: (key) => pageScreen.screens.act(key)
    GamePageModel {
        id: pageModel
        objectName: "pageModel"
        platformLibrary: pageScreen.platformLibrary
        cores: pageScreen.library.cores
        // The game a door opened the page for (Settings' Cheats: the game last played), else the game in hand.
        game: pageScreen.screens.shown === Screens.Page ? pageScreen.library.gameFile(pageScreen.screens.pageGame || pageScreen.inHand) : ""
        opensAt: pageScreen.screens.pageAt
        onPlayed: (path, state) => state.length > 0 ? pageScreen.playFrom(path, state) : pageScreen.play(path)
        onHidden: pageScreen.screens.back()
        onOpenFolder: (path) => pageScreen.openFolder(path)
    }
    Connections {
        target: pageModel
        function onHidden() { pageScreen.library.refresh() }
    }
    // A name edited on the page is the grid's too (EmuSen_Qt.md §7.8.2).
    Connections {
        target: pageModel
        function onEdited() { pageScreen.library.refresh() }
    }
    Connections {
        target: pageModel
        function onProblemChanged() { pageScreen.said(pageModel.problem) }
    }
}
