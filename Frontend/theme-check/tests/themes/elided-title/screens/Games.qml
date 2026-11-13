import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// The section's games as rows, each title cut with an ellipsis in a box too narrow for it: data, so an exemption of the
// class R-20261111-44 accepts, listed and not a finding.
ThemeScreen {
    objectName: "Games"
    role: ThemeScreen.Games
    Rectangle {
        anchors.fill: parent
        color: Tokens.surface
    }
    Rows {
        rows: Browse.games
        role: "title"
    }
}
