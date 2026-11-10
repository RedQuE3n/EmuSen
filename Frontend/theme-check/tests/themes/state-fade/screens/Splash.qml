import QtQuick
import EmuSen.Theme 1.0

// A fixture of theme-audit's states: a caption that fades to rest at 0.3 between 0.5 s and 1.5 s, and a title that fades in to full between 4.5 s and 5.5 s; only the caption, at rest, is too faint.
ThemeScreen {
    id: screen
    objectName: "Splash"
    role: ThemeScreen.Splash
    length: 6000
    still: Tokens.reducedMotion
    Rectangle {
        anchors.fill: parent
        color: Tokens.surface
    }
    NormalizedCanvas {
        id: canvas
        property real time: screen.still ? screen.length : 0
        NumberAnimation on time {
            to: screen.length
            duration: screen.length
            running: !screen.still
        }
        Text {
            objectName: "restsFaint"
            x: 64
            y: 200
            opacity: 0.3 * Math.min(1, Math.max(0, (canvas.time - 500) / 1000))
            text: Words.title
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 20
        }
        Text {
            objectName: "fadesIn"
            x: 64
            y: 400
            opacity: Math.min(1, Math.max(0, (canvas.time - 4500) / 1000))
            text: Words.title
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 20
        }
    }
}
