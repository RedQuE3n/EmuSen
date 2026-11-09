import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and beside them a view flicked from the start along a row too long to cross, so it never comes to rest.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Flickable {
            id: restless
            objectName: "restless"
            x: 640
            y: 64
            width: 400
            height: 200
            clip: true
            contentWidth: 10000000
            contentHeight: 200
            flickDeceleration: 1
        }
        Timer {
            running: true
            repeat: true
            triggeredOnStart: true
            interval: 200
            onTriggered: restless.flick(-4000, 0)
        }
    }
}
