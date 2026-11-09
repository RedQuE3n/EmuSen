import QtQuick
import EmuSen.Theme 1.0

// A fixture of theme-audit's states: a splash whose wordmark stands past the right edge from 2.5 s to 3.5 s and in place otherwise, still at its end under reduced motion.
ThemeScreen {
    id: screen
    objectName: "Splash"
    role: ThemeScreen.Splash
    length: 6000
    still: Tokens.reducedMotion
    NormalizedCanvas {
        id: canvas
        property real time: screen.still ? screen.length : 0
        NumberAnimation on time {
            to: screen.length
            duration: screen.length
            running: !screen.still
        }
        Text {
            objectName: "wordmark"
            x: canvas.time >= 2500 && canvas.time < 3500 ? 1200 : 64
            y: 200
            text: Words.title
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 64
        }
    }
}
