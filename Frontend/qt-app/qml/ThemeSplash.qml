pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.App

// A theme's splash that enters Home (EmuSen_Qt.md §3.5.2): the window's clock for it, held at its first frame until the start has read what Home shows and the sound is ready, the skip, and the sound.
Item {
    id: lead
    // The chosen theme's host, whose splash leads the start.
    property ThemeHost host: null
    // The window shows a splash (AppSettings.StartupSplash) and this one leads it.
    property bool shown: false
    // A read the start makes is still out: the library's, the look's, the settings'.
    property bool reading: true
    property string platformLibrary
    // AppSettings.NavigationSounds, and the Sound volume once the settings are read.
    property bool sounds: true
    property real volume: 1
    readonly property var screen: lead.host?.splashScreen ?? null
    // The screen's length, whether it is drawn still, and its sound, kept once read: the screen goes with the splash.
    property real length: 0
    property bool still: false
    property url source
    // Everything the splash waits for is in: the clock runs from here, and the sound with it.
    readonly property bool set: lead.shown && lead.length > 0 && !lead.reading && (sound.settled || lead.source.toString() === "")
    property real clock: 0
    property bool skipped: false
    readonly property bool ended: lead.length > 0 && lead.clock >= lead.length
    // The moment drawn: the clock's, the last frame under reduced motion (the screen's `still`).
    readonly property real time: lead.still ? lead.length : lead.clock
    // Ended, and the press that skipped it let go, so it is no press of Home's; once so, it stays so.
    readonly property bool finished: lead.ended && lead.set && !Kit.PadRouter.pending
    property bool done: false
    readonly property alias sound: sound
    focus: lead.shown && !lead.done
    visible: lead.shown && !lead.done
    // A press or a tap: to the last frame, the sound let go softly; one before the clock runs is kept for when it does.
    signal skipAsked()
    onSkipAsked: {
        lead.skipped = true
        if (lead.set && !lead.ended) {
            lead.clock = lead.length
            sound.end(Kit.Tokens.fade)
        }
    }
    onSetChanged: {
        if (!lead.set)
            return
        if (lead.skipped)
            lead.clock = lead.length
        else
            sound.start()
    }
    onFinishedChanged: if (lead.finished) lead.done = true
    // Read from the screen while it is placed, and kept after.
    Binding {
        target: lead
        property: "length"
        value: lead.screen?.length ?? 0
        when: lead.screen !== null
        restoreMode: Binding.RestoreNone
    }
    Binding {
        target: lead
        property: "still"
        value: lead.screen?.still ?? false
        when: lead.screen !== null
        restoreMode: Binding.RestoreNone
    }
    Binding {
        target: lead
        property: "source"
        // As the theme writes it, from its screen's own file (Qt 6, "Qt.resolvedUrl(url, context)").
        value: lead.screen ? Qt.resolvedUrl(lead.screen.sound.source, lead.screen) : ""
        when: lead.screen !== null
        restoreMode: Binding.RestoreNone
    }
    // The clock on Qt's animation clock, which a test's harness holds and steps (EmuSen_Kit.md §7.36).
    NumberAnimation {
        target: lead
        property: "clock"
        from: 0
        to: lead.length
        duration: lead.length
        running: lead.set && !lead.skipped
    }
    SoundFile {
        id: file
        objectName: "splashSoundFile"
        source: lead.source
        rate: sound.rate
        channels: sound.channels
        onReadChanged: if (file.read) sound.keep()
    }
    SplashSound {
        id: sound
        objectName: "splashSound"
        platformLibrary: lead.platformLibrary
        folder: lead.host?.folder ?? ""
        source: file.source
        file: file
        on: lead.sounds
        volume: lead.volume
    }
    // Every key is kept from the screens behind; one the kit's map makes a press skips, through the router as the pad's do.
    Keys.onPressed: (event) => {
        event.accepted = true
        if (Kit.PadRouter.pushKey(event.key, event.modifiers))
            lead.skipAsked()
    }
    TapHandler {
        acceptedButtons: Qt.AllButtons
        onTapped: lead.skipAsked()
    }
    Connections {
        target: Kit.PadRouter
        enabled: lead.activeFocus
        function onPressed(button) { lead.skipAsked() }
    }
    Timer {
        interval: 8
        repeat: true
        running: lead.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
        onTriggered: Kit.PadRouter.pump()
    }
}
