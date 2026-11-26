pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// The Pause Card, opened by the chord over the game; the game resumes once the card has slid away (EmuSen_Interface.md §20.6).
Kit.NormalizedCanvas {
    id: cardCanvas
    objectName: "cardCanvas"
    required property Screens screens
    required property LibraryModel library
    required property SessionModel session
    required property SettingsModel settings
    required property int padFamily
    required property bool pauseShown
    required property bool gameShown
    property bool bigScreen: true
    property Clock clock: null
    property int battery: -1
    readonly property alias slide: card.slide
    visible: cardCanvas.bigScreen && (card.open || card.leaving)
    PauseCard {
        id: card
        anchors.fill: parent
        library: cardCanvas.library
        session: cardCanvas.session
        settings: cardCanvas.settings
        padFamily: cardCanvas.padFamily
        pixelScale: cardCanvas.unit
        battery: cardCanvas.battery
        clockText: cardCanvas.clock ? cardCanvas.clock.shortTime : ""
        open: cardCanvas.bigScreen && cardCanvas.pauseShown
        focus: cardCanvas.bigScreen && cardCanvas.pauseShown
        onResumed: cardCanvas.screens.play()
        onBackToLibrary: cardCanvas.screens.library()
        onCheatsToolAsked: cardCanvas.screens.openCheats()
        onQuitGame: cardCanvas.session.close()
        onQuitAll: cardCanvas.screens.quit()
        onGone: if (cardCanvas.gameShown) cardCanvas.session.resume()
    }
}
