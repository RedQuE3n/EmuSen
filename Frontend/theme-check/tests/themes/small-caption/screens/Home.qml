import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and words under the capitals' floor by the one name the floor's table of exceptions holds: an
// exemption of a class a ruling accepts, so listed and not a finding.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Text {
            objectName: "moonriseCaption"
            x: 672
            y: 400
            text: "Moonrise"
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 12
        }
    }
}
