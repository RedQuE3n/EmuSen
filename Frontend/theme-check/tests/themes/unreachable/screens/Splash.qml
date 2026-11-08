import QtQuick
import EmuSen.Theme 1.0

// A splash that breathes its wordmark while motion is allowed and stands still under reduced motion.
ThemeScreen {
    objectName: "Splash"
    role: ThemeScreen.Splash
    length: 3000
    NormalizedCanvas {
        Text {
            objectName: "wordmark"
            anchors.centerIn: parent
            text: Words.title
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 64
            NumberAnimation on opacity {
                from: 0.6
                to: 1
                duration: Tokens.reel
                loops: Animation.Infinite
                running: !Tokens.reducedMotion
            }
        }
    }
}
