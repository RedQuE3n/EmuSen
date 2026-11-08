import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// A game on screen: its session, its picture at its display shape, and the pad given to the game while it runs unpaused with nothing over it; the menu chord is the host's to answer (EmuSen_Qt.md §1.8.5, §7.1).
Item {
    id: root
    property double feed: 0
    property string platformLibrary
    // The game named, the session's own, so a game the session lets go (B on the Resume question) is named again afresh.
    property alias game: session.game
    property string cores
    property string engine
    // The pad reader's hand-over to the game (PadReader.gamePads).
    property double pads: 0
    // Whether the game is in front: a host clears it while a screen of its own is over the game.
    property bool inFront: true
    // Guide held with Up, Down or the right shoulder saves, loads or takes a screenshot over the game (EmuSen_Kit.md §7.26).
    property bool guideChords: false
    readonly property alias session: session
    readonly property alias picture: picture
    readonly property bool ownsPad: session.running && !session.paused && root.inFront
    signal menu()

    // While the game owns the pad only the chord leaves the router, and it reaches the host.
    Binding {
        target: Kit.PadRouter
        property: "chordOnly"
        value: root.ownsPad
    }
    Binding {
        target: Kit.PadRouter
        property: "guideChords"
        value: root.guideChords
    }
    Connections {
        target: Kit.PadRouter
        enabled: root.inFront
        function onMenuChord() { root.menu() }
        function onGuideChord(button) { session.guideChord(button) }
    }
    // The router's clock while the game is in front, where no frame of the kit's runs it.
    Timer {
        interval: 8
        repeat: true
        running: root.inFront && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
        onTriggered: Kit.PadRouter.pump()
    }

    SessionModel {
        id: session
        objectName: "session"
        platformLibrary: root.platformLibrary
        feed: root.feed
        cores: root.cores
        engine: root.engine
        pads: root.pads
    }

    // The bars are the game's and always black, whatever the theme: nothing white or themed ever shows beside the picture.
    Rectangle {
        objectName: "bars"
        anchors.fill: parent
        color: "#000000"
    }

    // The picture at its display shape, the session's, centred with bars where the window's shape differs, as Mistress letterboxes it.
    Kit.GamePicture {
        id: picture
        objectName: "picture"
        readonly property real shape: session.pictureAspect > 0 ? session.pictureAspect : root.width / Math.max(1, root.height)
        anchors.centerIn: parent
        width: Math.min(root.width, root.height * picture.shape)
        height: Math.min(root.height, root.width / picture.shape)
        feed: root.feed
        // Redrawn at each swap only while the game runs; paused or with no game its last picture stands still (EmuSen_Qt.md §9.3.2).
        live: root.feed !== 0 && session.running && !session.paused
    }
}
