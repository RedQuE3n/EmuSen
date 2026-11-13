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
    // The lines over the game the settings turn on: the messages, the frame rate, and the bar behind them.
    property bool showStatusText: false
    property bool showFps: false
    property bool showStatusBar: false
    readonly property alias session: session
    readonly property alias picture: picture
    readonly property bool ownsPad: session.running && !session.paused && root.inFront
    signal menu()
    // The hotkeys whose actions are the window's: Exit to Library and Fullscreen.
    signal exitAsked()
    signal fullScreenAsked()

    // The keyboard over the game: the console's keys to the game, the hotkeys to their actions (EmuSen_Qt.md §7.7.H6).
    Keys.onPressed: (event) => event.accepted = session.running && session.key(event.key, event.modifiers, true, event.isAutoRepeat)
    Keys.onReleased: (event) => event.accepted = session.running && session.key(event.key, event.modifiers, false, event.isAutoRepeat)

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
        onExitAsked: root.exitAsked()
        onFullScreenAsked: root.fullScreenAsked()
        platformLibrary: root.platformLibrary
        feed: root.feed
        cores: root.cores
        engine: root.engine
        pads: root.pads
        // A still picture is drawn again through what the settings now ask for.
        onPictureRead: picture.update()
        // Another window in front of the game's, which Pause in background acts on (the window's own active state).
        background: !root.Window.active
    }

    // The bars are the game's and always black, whatever the theme: nothing white or themed ever shows beside the picture.
    Rectangle {
        objectName: "bars"
        anchors.fill: parent
        color: "#000000"
    }

    // The picture at its display shape, the session's, centred with bars where the window's shape differs, as Mistress
    // letterboxes it; a crop the filter does not make itself is the part of the picture this item clips to (EmuSen_Qt.md §1.8.10).
    Item {
        id: shown
        objectName: "shown"
        readonly property real shape: session.pictureAspect > 0 ? session.pictureAspect : root.width / Math.max(1, root.height)
        readonly property rect crop: session.pictureCrop
        anchors.centerIn: parent
        width: Math.min(root.width, root.height * shown.shape)
        height: Math.min(root.height, root.width / shown.shape)
        clip: shown.crop.width < 1 || shown.crop.height < 1

        Kit.GamePicture {
            id: picture
            objectName: "picture"
            width: shown.width / Math.max(0.01, shown.crop.width)
            height: shown.height / Math.max(0.01, shown.crop.height)
            x: -shown.crop.x * picture.width
            y: -shown.crop.y * picture.height
            feed: root.feed
            // Redrawn at each swap only while the game runs; paused or with no game its last picture stands still (EmuSen_Qt.md §9.3.2).
            live: root.feed !== 0 && session.running && !session.paused
        }
    }

    // The status line over the game in front: the messages at the bottom left, the frame rate while it runs at the bottom
    // right, on a black bar where Status bar is on; in design units, as the notices are.
    Kit.NormalizedCanvas {
        id: lines
        readonly property bool showing: root.inFront && session.running
        Rectangle {
            objectName: "statusBar"
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: Kit.Tokens.fontCaption + 2 * Kit.Tokens.base * 2
            color: "#000000"
            opacity: 0.6
            visible: root.showStatusBar && (statusLine.visible || fpsLine.visible)
        }
        Text {
            id: statusLine
            objectName: "statusLine"
            anchors.left: parent.left
            anchors.leftMargin: Kit.Tokens.base * 3
            anchors.right: fpsLine.left
            anchors.bottom: parent.bottom
            anchors.bottomMargin: Kit.Tokens.base * 2
            text: session.status
            elide: Text.ElideRight
            color: "#ffffff"
            font.pixelSize: Kit.Tokens.fontCaption
            visible: lines.showing && root.showStatusText && text.length > 0
        }
        Text {
            id: fpsLine
            objectName: "fpsLine"
            anchors.right: parent.right
            anchors.rightMargin: Kit.Tokens.base * 3
            anchors.bottom: parent.bottom
            anchors.bottomMargin: Kit.Tokens.base * 2
            text: session.fpsWords
            color: "#ffffff"
            font.pixelSize: Kit.Tokens.fontCaption
            visible: lines.showing && root.showFps && !session.paused
        }
    }
}
