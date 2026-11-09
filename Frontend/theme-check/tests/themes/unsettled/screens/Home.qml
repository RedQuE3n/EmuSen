import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and beside them a view flicked again before it can come to rest, so the audit can never settle it.
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
            contentWidth: 4000
            contentHeight: 200
            flickableDeceleration: 1
            Component.onCompleted: restless.flick(-4000, 0)
            onMovementEnded: restless.flick(restless.atXEnd ? 4000 : -4000, 0)
        }
    }
}
