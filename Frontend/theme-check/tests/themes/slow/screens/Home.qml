import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and a veil that fades in over 1.5 s each time the focus moves, longer than any of the kit's transitions.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Item {
            property Item held: Window.activeFocusItem
            onHeldChanged: fade.restart()
            Rectangle {
                id: veil
                objectName: "veil"
                x: 700
                y: 300
                width: 400
                height: 300
                color: Tokens.accent
            }
            NumberAnimation {
                id: fade
                target: veil
                property: "opacity"
                from: 0
                to: 1
                duration: 1500
            }
        }
    }
}
